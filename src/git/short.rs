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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path();
        assert_eq!(Repo::None, parse(path));
    }

    mod sync {
        use super::super::*;

        #[test]
        fn behind() {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path();
            let repo = git2::Repository::init(path).unwrap();
            repo.remote("origin", "file:///dev/null").unwrap();
            let sig = repo.signature().unwrap();

            let mut idx = repo.index().unwrap();
            let id = idx.write_tree().unwrap();
            let tree = repo.find_tree(id).unwrap();
            let oid = repo
                .commit(Some("HEAD"), &sig, &sig, "initial commit", &tree, &[])
                .unwrap();

            repo.reference("refs/remotes/origin/master", oid, true, "remote")
                .unwrap();
            let mut local_branch = repo.find_branch("master", git2::BranchType::Local).unwrap();
            local_branch.set_upstream(Some("origin/master")).unwrap();

            let commit_oid = repo.find_commit(oid).unwrap();
            let new_oid = repo
                .commit(
                    Some("HEAD"),
                    &sig,
                    &sig,
                    "second commit",
                    &tree,
                    &[&commit_oid],
                )
                .unwrap();

            let oid_obj = repo.find_object(oid, None).unwrap();
            repo.reset(&oid_obj, git2::ResetType::Hard, None).unwrap();
            repo.reference("refs/remotes/origin/master", new_oid, true, "remote")
                .unwrap();

            assert_eq!(Repo::Clean(Sync::Behind), parse(path));
            std::fs::write(path.join("yo"), b"yo").unwrap();
            assert_eq!(Repo::Dirty(Sync::Behind), parse(path));
        }

        #[test]
        fn ahead() {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path();
            let repo = git2::Repository::init(path).unwrap();
            repo.remote("origin", "file:///dev/null").unwrap();
            let sig = repo.signature().unwrap();

            let mut idx = repo.index().unwrap();
            let id = idx.write_tree().unwrap();
            let tree = repo.find_tree(id).unwrap();
            let oid = repo
                .commit(Some("HEAD"), &sig, &sig, "initial commit", &tree, &[])
                .unwrap();

            repo.reference("refs/remotes/origin/master", oid, true, "remote")
                .unwrap();
            let mut local_branch = repo.find_branch("master", git2::BranchType::Local).unwrap();
            local_branch.set_upstream(Some("origin/master")).unwrap();

            let commit_oid = repo.find_commit(oid).unwrap();
            repo.commit(
                Some("HEAD"),
                &sig,
                &sig,
                "second commit",
                &tree,
                &[&commit_oid],
            )
            .unwrap();

            assert_eq!(Repo::Clean(Sync::Ahead), parse(path));
            std::fs::write(path.join("yo"), b"yo").unwrap();
            assert_eq!(Repo::Dirty(Sync::Ahead), parse(path));
        }

        #[test]
        fn diverged() {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path();
            let repo = git2::Repository::init(path).unwrap();
            repo.remote("origin", "file:///dev/null").unwrap();
            let sig = repo.signature().unwrap();

            let mut idx = repo.index().unwrap();
            let id = idx.write_tree().unwrap();
            let tree = repo.find_tree(id).unwrap();
            let oid = repo
                .commit(Some("HEAD"), &sig, &sig, "initial commit", &tree, &[])
                .unwrap();

            repo.reference("refs/remotes/origin/master", oid, true, "remote")
                .unwrap();
            let mut local_branch = repo.find_branch("master", git2::BranchType::Local).unwrap();
            local_branch.set_upstream(Some("origin/master")).unwrap();

            let commit_oid = repo.find_commit(oid).unwrap();
            let new_oid = repo
                .commit(
                    Some("HEAD"),
                    &sig,
                    &sig,
                    "second commit",
                    &tree,
                    &[&commit_oid],
                )
                .unwrap();

            let oid_obj = repo.find_object(oid, None).unwrap();
            repo.reset(&oid_obj, git2::ResetType::Hard, None).unwrap();
            repo.reference("refs/remotes/origin/master", new_oid, true, "remote")
                .unwrap();

            repo.commit(
                Some("HEAD"),
                &sig,
                &sig,
                "third commit",
                &tree,
                &[&commit_oid],
            )
            .unwrap();

            assert_eq!(Repo::Clean(Sync::Diverged), parse(path));
            std::fs::write(path.join("yo"), b"yo").unwrap();
            assert_eq!(Repo::Dirty(Sync::Diverged), parse(path));
        }

        #[test]
        fn up_to_date() {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path();
            let repo = git2::Repository::init(path).unwrap();
            repo.remote("origin", "file:///dev/null").unwrap();
            let sig = repo.signature().unwrap();

            let mut idx = repo.index().unwrap();
            let id = idx.write_tree().unwrap();
            let tree = repo.find_tree(id).unwrap();
            let oid = repo
                .commit(Some("HEAD"), &sig, &sig, "initial commit", &tree, &[])
                .unwrap();

            repo.reference("refs/remotes/origin/master", oid, true, "remote")
                .unwrap();
            let mut local_branch = repo.find_branch("master", git2::BranchType::Local).unwrap();
            local_branch.set_upstream(Some("origin/master")).unwrap();

            assert_eq!(Repo::Clean(Sync::UpToDate), parse(path));
            std::fs::write(path.join("yo"), b"yo").unwrap();
            assert_eq!(Repo::Dirty(Sync::UpToDate), parse(path));
        }

        #[test]
        fn local() {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path();
            let repo = git2::Repository::init(path).unwrap();
            let sig = repo.signature().unwrap();

            let mut idx = repo.index().unwrap();
            let id = idx.write_tree().unwrap();
            let tree = repo.find_tree(id).unwrap();
            repo.commit(Some("HEAD"), &sig, &sig, "initial commit", &tree, &[])
                .unwrap();

            assert_eq!(Repo::Clean(Sync::Local), parse(path));
            std::fs::write(path.join("yo"), b"yo").unwrap();
            assert_eq!(Repo::Dirty(Sync::Local), parse(path));
        }
    }

    #[test]
    fn pending() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path();
        let repo = git2::Repository::init(path).unwrap();
        let sig = repo.signature().unwrap();

        let mut idx = repo.index().unwrap();
        let id = idx.write_tree().unwrap();
        let tree = repo.find_tree(id).unwrap();
        let oid = repo
            .commit(Some("HEAD"), &sig, &sig, "initial commit", &tree, &[])
            .unwrap();
        let commit = repo.find_commit(oid).unwrap();

        repo.cherrypick(&commit, None).unwrap();

        assert_eq!(Repo::Pending, parse(path));
    }

    #[test]
    fn detached() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path();
        let repo = git2::Repository::init(path).unwrap();
        let sig = repo.signature().unwrap();

        let mut idx = repo.index().unwrap();
        let id = idx.write_tree().unwrap();
        let tree = repo.find_tree(id).unwrap();
        let oid = repo
            .commit(Some("HEAD"), &sig, &sig, "initial commit", &tree, &[])
            .unwrap();
        let oid = repo.find_commit(oid).unwrap();

        repo.commit(Some("HEAD"), &sig, &sig, "second commit", &tree, &[&oid])
            .unwrap();

        let target = repo.revparse_single("HEAD~1").unwrap();
        repo.checkout_tree(&target, None).unwrap();
        repo.set_head_detached(target.id()).unwrap();
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
