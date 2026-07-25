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

pub fn parse(path: &std::path::Path) -> Repo {
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
        |_| String::from("??"),
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
            .fold(Changes::default(), |mut acc, curr| {
                if curr.contains(git2::Status::CONFLICTED) {
                    acc.conflicted += 1;
                } else if curr.intersects(git2::Status::INDEX_NEW | git2::Status::WT_NEW) {
                    acc.added += 1;
                } else if curr.intersects(git2::Status::INDEX_DELETED | git2::Status::WT_DELETED) {
                    acc.removed += 1;
                } else if curr.intersects(
                    git2::Status::INDEX_MODIFIED
                        | git2::Status::WT_MODIFIED
                        | git2::Status::INDEX_RENAMED
                        | git2::Status::WT_RENAMED
                        | git2::Status::INDEX_TYPECHANGE
                        | git2::Status::WT_TYPECHANGE,
                ) {
                    acc.modified += 1;
                }
                acc
            })
    })
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

    mod regular {
        use super::*;

        #[test]
        fn local() {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path();

            let repo = TestRepo::new(path);
            repo.commit(&[]);

            assert_eq!(
                Repo::Regular(String::from("master"), Sync::Local, Changes::default()),
                parse(path)
            );
        }

        #[test]
        fn gone() {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path();

            let repo = TestRepo::new_with_remote(path);
            let (_, coid) = repo.commit(&[]);
            repo.set_upstream(coid);

            let mut remote_ref = repo
                .repo
                .find_reference("refs/remotes/origin/master")
                .unwrap();
            remote_ref.delete().unwrap();

            assert_eq!(
                Repo::Regular(String::from("master"), Sync::Gone, Changes::default()),
                parse(path)
            );
        }

        #[test]
        fn tracked() {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path();

            let repo = TestRepo::new_with_remote(path);
            let (commit, coid) = repo.commit(&[]);
            repo.set_upstream(coid);

            let coid_a = repo.commit_headless(&[&commit]);
            let coid_b = repo.commit_headless(&[&commit]);

            assert_eq!(
                Repo::Regular(
                    String::from("master"),
                    Sync::Tracked {
                        ahead: 0,
                        behind: 0
                    },
                    Changes::default()
                ),
                parse(path)
            );

            repo.set_pointers(coid_a, coid);
            assert_eq!(
                Repo::Regular(
                    String::from("master"),
                    Sync::Tracked {
                        ahead: 1,
                        behind: 0
                    },
                    Changes::default()
                ),
                parse(path)
            );

            repo.set_pointers(coid, coid_a);
            assert_eq!(
                Repo::Regular(
                    String::from("master"),
                    Sync::Tracked {
                        ahead: 0,
                        behind: 1
                    },
                    Changes::default()
                ),
                parse(path)
            );

            repo.set_pointers(coid_b, coid_a);
            assert_eq!(
                Repo::Regular(
                    String::from("master"),
                    Sync::Tracked {
                        ahead: 1,
                        behind: 1
                    },
                    Changes::default()
                ),
                parse(path)
            );
        }
    }

    #[test]
    fn detached() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path();

        let repo = TestRepo::new(path);
        let (commit, coid) = repo.commit(&[]);
        repo.commit(&[&commit]);

        let target = repo.repo.revparse_single("HEAD~1").unwrap();
        repo.repo.checkout_tree(&target, None).unwrap();
        repo.repo.set_head_detached(target.id()).unwrap();

        assert_eq!(
            Repo::Detached(short_id(coid).unwrap(), Changes::default()),
            parse(path)
        );
    }

    #[test]
    fn pending() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path();

        let repo = TestRepo::new(path);
        let (commit, _) = repo.commit(&[]);

        repo.repo.cherrypick(&commit, None).unwrap();

        assert_eq!(
            Repo::Pending(String::from("master"), Pending::Cherry, Changes::default()),
            parse(path)
        );
    }

    #[test]
    fn new() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path();
        git2::Repository::init(path).unwrap();
        assert_eq!(Repo::New(Changes::default()), parse(path));
    }

    #[test]
    fn changes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path();

        let repo = TestRepo::new(path);

        assert_eq!(Repo::New(Changes::default()), parse(path));

        let new_file = "yo";
        std::fs::write(path.join(new_file), b"yo").unwrap();

        assert_eq!(
            Repo::New(Changes {
                added: 1,
                ..Changes::default()
            }),
            parse(path)
        );

        repo.add_path(new_file);
        let (commit, _) = repo.commit(&[]);
        assert_eq!(
            Repo::Regular(String::from("master"), Sync::Local, Changes::default()),
            parse(path)
        );

        std::fs::write(path.join(new_file), b"yo2").unwrap();
        assert_eq!(
            Repo::Regular(
                String::from("master"),
                Sync::Local,
                Changes {
                    modified: 1,
                    ..Changes::default()
                }
            ),
            parse(path)
        );

        std::fs::remove_file(path.join(new_file)).unwrap();
        assert_eq!(
            Repo::Regular(
                String::from("master"),
                Sync::Local,
                Changes {
                    removed: 1,
                    ..Changes::default()
                }
            ),
            parse(path)
        );

        repo.repo.branch("branch_a", &commit, false).unwrap();
        std::fs::write(path.join(new_file), b"yo3").unwrap();
        repo.add_path(new_file);
        repo.commit(&[&commit]);

        repo.repo.set_head("refs/heads/branch_a").unwrap();
        repo.repo
            .checkout_head(Some(git2::build::CheckoutBuilder::new().force()))
            .unwrap();

        std::fs::write(path.join(new_file), b"yo4").unwrap();
        repo.add_path(new_file);
        repo.commit(&[&commit]);

        repo.repo.set_head("refs/heads/master").unwrap();
        repo.repo
            .checkout_head(Some(git2::build::CheckoutBuilder::new().force()))
            .unwrap();

        let branch_a_ref = repo
            .repo
            .find_branch("branch_a", git2::BranchType::Local)
            .unwrap();
        let branch_a_annotated = repo
            .repo
            .reference_to_annotated_commit(branch_a_ref.get())
            .unwrap();

        repo.repo.merge(&[&branch_a_annotated], None, None).unwrap();
        assert_eq!(
            Repo::Pending(
                String::from("master"),
                Pending::Merge,
                Changes {
                    conflicted: 1,
                    ..Changes::default()
                }
            ),
            parse(path)
        );
    }
}
