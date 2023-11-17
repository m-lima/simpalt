#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Repo {
    None,
    Clean(Sync),
    Dirty(Sync),
    Detached,
    Pending,
    Untracked,
    Error,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Sync {
    Behind,
    Ahead,
    Diverged,
    UpToDate,
    Local,
}

pub fn parse(path: &std::path::PathBuf) -> Repo {
    let Some(repo) = git2::Repository::discover(path).ok() else {
        return Repo::None;
    };

    if repo.state() != git2::RepositoryState::Clean {
        return Repo::Pending;
    }

    let sync = match repo.revparse("HEAD..@{upstream}").and_then(|behind| {
        repo.revparse("@{upstream}..HEAD")
            .map(|ahead| get_sync(&repo, &behind, &ahead))
    }) {
        Ok(Some(sync)) => sync,
        Ok(None) => return Repo::Error,
        Err(e) => match e.code() {
            git2::ErrorCode::NotFound => match e.class() {
                git2::ErrorClass::Config => Sync::Local,
                git2::ErrorClass::Reference => return Repo::Untracked,
                _ => return Repo::Error,
            },
            git2::ErrorCode::InvalidSpec => return Repo::Detached,
            _ => return Repo::Error,
        },
    };

    let Ok(status) = repo.statuses(Some(
        git2::StatusOptions::new()
            .include_ignored(false)
            .include_untracked(true),
    )) else {
        return Repo::Error;
    };

    if status.iter().next().is_some() {
        Repo::Dirty(sync)
    } else {
        Repo::Clean(sync)
    }
}

fn walk(walker: &mut git2::Revwalk<'_>, rev: &git2::Revspec<'_>) -> Option<bool> {
    let to = rev.to()?;
    let from = rev.from()?;
    walker.hide(from.id()).ok()?;
    walker.push(to.id()).ok()?;

    Some(walker.take_while(Result::is_ok).next().is_some())
}

fn get_sync(
    repo: &git2::Repository,
    behind: &git2::Revspec<'_>,
    ahead: &git2::Revspec<'_>,
) -> Option<Sync> {
    let mut walker = repo.revwalk().ok()?;

    let behind = walk(&mut walker, behind)?;
    walker.reset().ok()?;
    let ahead = walk(&mut walker, ahead)?;

    Some(match (behind, ahead) {
        (false, false) => Sync::UpToDate,
        (true, false) => Sync::Behind,
        (false, true) => Sync::Ahead,
        (true, true) => Sync::Diverged,
    })
}
