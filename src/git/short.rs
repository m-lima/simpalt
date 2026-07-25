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

pub fn parse(path: &std::path::Path) -> Repo {
    let Some(repo) = git2::Repository::discover(path).ok() else {
        return Repo::None;
    };

    if repo.state() != git2::RepositoryState::Clean {
        return Repo::Pending;
    }

    let head = match repo.head() {
        Ok(head) => head,
        Err(e) => {
            return match e.code() {
                git2::ErrorCode::UnbornBranch => Repo::Untracked,
                _ => Repo::Error,
            };
        }
    };

    let Some(head_oid) = head.target() else {
        return Repo::Error;
    };

    let upstream = match git2::Branch::wrap(head).upstream() {
        Ok(upstream) => upstream,
        Err(e) => {
            return match (e.code(), e.class()) {
                (git2::ErrorCode::NotFound, _) => repo_state(&repo, Sync::Local),
                (git2::ErrorCode::GenericError, git2::ErrorClass::Invalid) => Repo::Detached,
                _ => Repo::Error,
            };
        }
    };

    let Some(upstream_oid) = upstream.get().target() else {
        return Repo::Error;
    };

    let sync = match repo.graph_ahead_behind(head_oid, upstream_oid) {
        Ok((1.., 1..)) => Sync::Diverged,
        Ok((1.., _)) => Sync::Ahead,
        Ok((_, 1..)) => Sync::Behind,
        Ok(_) => Sync::UpToDate,
        Err(_) => return Repo::Error,
    };

    repo_state(&repo, sync)
}

fn repo_state(repo: &git2::Repository, sync: Sync) -> Repo {
    match repo.statuses(Some(
        git2::StatusOptions::new()
            .include_ignored(false)
            .include_untracked(true),
    )) {
        Ok(status) if status.is_empty() => Repo::Clean(sync),
        Ok(_) => Repo::Dirty(sync),
        Err(_) => Repo::Error,
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::TestRepo;
    use super::*;

    #[test]
    fn none() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path();
        assert_eq!(Repo::None, parse(path));
    }

    #[test]
    fn sync() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path();

        let repo = TestRepo::new_with_remote(path);
        let (commit, coid) = repo.commit(&[]);
        repo.set_upstream(coid);

        let coid_a = repo.commit_headless(&[&commit]);
        let coid_b = repo.commit_headless(&[&commit]);
        println!("{coid_a} {coid_b}");

        assert_eq!(Repo::Clean(Sync::UpToDate), parse(path));

        repo.set_pointers(coid_a, coid);
        assert_eq!(Repo::Clean(Sync::Ahead), parse(path));

        repo.set_pointers(coid, coid_a);
        assert_eq!(Repo::Clean(Sync::Behind), parse(path));

        repo.set_pointers(coid_b, coid_a);
        assert_eq!(Repo::Clean(Sync::Diverged), parse(path));

        std::fs::write(path.join("yo"), b"yo").unwrap();
        repo.set_pointers(coid, coid);
        assert_eq!(Repo::Dirty(Sync::UpToDate), parse(path));

        repo.set_pointers(coid_a, coid);
        assert_eq!(Repo::Dirty(Sync::Ahead), parse(path));

        repo.set_pointers(coid, coid_a);
        assert_eq!(Repo::Dirty(Sync::Behind), parse(path));

        repo.set_pointers(coid_b, coid_a);
        assert_eq!(Repo::Dirty(Sync::Diverged), parse(path));
    }

    #[test]
    fn pending() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path();

        let repo = TestRepo::new(path);
        let (commit, _) = repo.commit(&[]);

        repo.repo.cherrypick(&commit, None).unwrap();

        assert_eq!(Repo::Pending, parse(path));
    }

    #[test]
    fn detached() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path();

        let repo = TestRepo::new(path);

        let (commit, _) = repo.commit(&[]);
        repo.commit(&[&commit]);

        let target = repo.repo.revparse_single("HEAD~1").unwrap();
        repo.repo.checkout_tree(&target, None).unwrap();
        repo.repo.set_head_detached(target.id()).unwrap();
        assert_eq!(Repo::Detached, parse(path));
    }

    #[test]
    fn untracked() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path();
        git2::Repository::init(path).unwrap();
        assert_eq!(Repo::Untracked, parse(path));
    }
}
