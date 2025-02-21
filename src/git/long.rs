#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Repo {
    None,
    Regular(String, Sync, Changes),
    Detached(String, Changes),
    Pending(String, Pending, Changes),
    New(Changes),
    Error,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Sync {
    Local,
    Gone,
    Tracked { ahead: usize, behind: usize },
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Pending {
    Merge,
    Revert,
    Cherry,
    Bisect,
    Rebase,
    Mailbox,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct Changes {
    pub added: usize,
    pub modified: usize,
    pub removed: usize,
    pub conflicted: usize,
}

impl Changes {
    pub fn clean(&self) -> bool {
        self.added == 0 && self.modified == 0 && self.removed == 0 && self.conflicted == 0
    }
}

pub fn parse(path: &std::path::PathBuf) -> Repo {
    fn short_id(oid: git2::Oid) -> Option<String> {
        let mut oid = oid.as_bytes().iter();
        match (oid.next(), oid.next(), oid.next(), oid.next()) {
            (Some(a), None, None, None) => Some(format!("{a:02x}")),
            (Some(a), Some(b), None, None) => Some(format!("{a:02x}{b:02x}")),
            (Some(a), Some(b), Some(c), None) => Some(format!("{a:02x}{b:02x}{c:02x}")),
            (Some(a), Some(b), Some(c), Some(d)) => Some(format!("{a:02x}{b:02x}{c:02x}{d:02x}")),
            _ => None,
        }
    }

    let Some(repo) = git2::Repository::discover(path).ok() else {
        return Repo::None;
    };

    let Some(changes) = get_changes(&repo) else {
        return Repo::Error;
    };

    let Ok(head) = repo.head() else {
        return Repo::New(changes);
    };

    let head = head.shorthand().map_or_else(
        || String::from("??"),
        |short| {
            short
                .eq("HEAD")
                .then(|| head.target())
                .flatten()
                .and_then(short_id)
                .unwrap_or_else(|| String::from(short))
        },
    );

    match repo.state() {
        git2::RepositoryState::Merge => return Repo::Pending(head, Pending::Merge, changes),
        git2::RepositoryState::Revert | git2::RepositoryState::RevertSequence => {
            return Repo::Pending(head, Pending::Revert, changes);
        }
        git2::RepositoryState::CherryPick | git2::RepositoryState::CherryPickSequence => {
            return Repo::Pending(head, Pending::Cherry, changes);
        }
        git2::RepositoryState::Bisect => return Repo::Pending(head, Pending::Bisect, changes),
        git2::RepositoryState::Rebase
        | git2::RepositoryState::RebaseInteractive
        | git2::RepositoryState::RebaseMerge => {
            return Repo::Pending(head, Pending::Rebase, changes);
        }
        git2::RepositoryState::ApplyMailbox | git2::RepositoryState::ApplyMailboxOrRebase => {
            return Repo::Pending(head, Pending::Mailbox, changes);
        }
        git2::RepositoryState::Clean => {}
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
                git2::ErrorClass::Reference => Sync::Gone,
                _ => return Repo::Error,
            },
            git2::ErrorCode::InvalidSpec => return Repo::Detached(head, changes),
            _ => return Repo::Error,
        },
    };

    Repo::Regular(head, sync, changes)
}

fn walk(walker: &mut git2::Revwalk<'_>, rev: &git2::Revspec<'_>) -> Option<usize> {
    let to = rev.to()?;
    let from = rev.from()?;
    walker.hide(from.id()).ok()?;
    walker.push(to.id()).ok()?;

    Some(walker.take_while(Result::is_ok).count())
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

    Some(Sync::Tracked { ahead, behind })
}

fn get_changes(repo: &git2::Repository) -> Option<Changes> {
    repo.statuses(Some(
        git2::StatusOptions::new()
            .include_ignored(false)
            .include_untracked(true),
    ))
    .ok()
    .map(|status| {
        status
            .iter()
            .map(|s| s.status())
            .fold(Changes::default(), |acc, curr| match curr {
                git2::Status::INDEX_NEW | git2::Status::WT_NEW => Changes {
                    added: acc.added + 1,
                    ..acc
                },
                git2::Status::INDEX_DELETED | git2::Status::WT_DELETED => Changes {
                    removed: acc.removed + 1,
                    ..acc
                },
                git2::Status::INDEX_TYPECHANGE
                | git2::Status::WT_TYPECHANGE
                | git2::Status::INDEX_RENAMED
                | git2::Status::WT_RENAMED
                | git2::Status::INDEX_MODIFIED
                | git2::Status::WT_MODIFIED => Changes {
                    modified: acc.modified + 1,
                    ..acc
                },
                git2::Status::CONFLICTED => Changes {
                    conflicted: acc.conflicted + 1,
                    ..acc
                },
                _ => acc,
            })
    })
}
