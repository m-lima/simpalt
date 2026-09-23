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

#[must_use]
pub fn parse(path: &std::path::Path) -> Repo {
    let Some(repo) = gix::discover(path).ok() else {
        return Repo::None;
    };

    if repo.state().is_some() {
        return Repo::Pending;
    }

    let mut branch = match repo.head() {
        Ok(head) => match head.kind {
            gix::head::Kind::Symbolic(r) => gix::prelude::ReferenceExt::attach(r, &repo),
            gix::head::Kind::Unborn(_) => return Repo::Untracked,
            gix::head::Kind::Detached { .. } => return Repo::Detached,
        },
        Err(e) => {
            use gix::reference::find::existing::Error;
            match e {
                Error::NotFound { .. } => return Repo::Untracked,
                Error::Find(_) => return Repo::Error,
            }
        }
    };

    let Ok(head_oid) = branch.peel_to_id() else {
        return Repo::Untracked;
    };

    let Some(remote_oid) = branch
        .remote_ref_name(gix::remote::Direction::Fetch)
        .and_then(Result::ok)
        .and_then(|n| repo.find_reference(&n).ok())
        .and_then(|mut r| r.peel_to_id().ok())
    else {
        return Repo::Untracked;
    };

    let sync = {
        if head_oid == remote_oid {
            Sync::UpToDate
        } else if let Ok(base_oid) = repo.merge_base(head_oid, remote_oid) {
            if base_oid == remote_oid {
                Sync::Ahead
            } else if base_oid == head_oid {
                Sync::Behind
            } else {
                Sync::Diverged
            }
        } else {
            return Repo::Error;
        }
    };

    repo_state(&repo, sync)
}

fn repo_state(repo: &gix::Repository, sync: Sync) -> Repo {
    match repo.is_dirty() {
        Ok(dirty) => {
            if dirty {
                Repo::Dirty(sync)
            } else {
                Repo::Clean(sync)
            }
        }
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
