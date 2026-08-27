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
    #[must_use]
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

#[must_use]
pub fn parse(path: &std::path::Path) -> Repo {
    let Some(repo) = git2::Repository::discover(path).ok() else {
        return Repo::None;
    };

    let Some(changes) = get_changes(&repo) else {
        return Repo::Error;
    };

    let head = match repo.head() {
        Ok(head) => head,
        Err(e) => {
            return match e.code() {
                git2::ErrorCode::UnbornBranch | git2::ErrorCode::NotFound => Repo::New(changes),
                _ => Repo::Error,
            };
        }
    };

    let name = head.shorthand().map_or_else(
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

    if !head.is_branch() {
        return Repo::Detached(name, changes);
    }

    match repo.state() {
        git2::RepositoryState::Merge => return Repo::Pending(name, Pending::Merge, changes),
        git2::RepositoryState::Revert | git2::RepositoryState::RevertSequence => {
            return Repo::Pending(name, Pending::Revert, changes);
        }
        git2::RepositoryState::CherryPick | git2::RepositoryState::CherryPickSequence => {
            return Repo::Pending(name, Pending::Cherry, changes);
        }
        git2::RepositoryState::Bisect => return Repo::Pending(name, Pending::Bisect, changes),
        git2::RepositoryState::Rebase
        | git2::RepositoryState::RebaseInteractive
        | git2::RepositoryState::RebaseMerge => {
            return Repo::Pending(name, Pending::Rebase, changes);
        }
        git2::RepositoryState::ApplyMailbox | git2::RepositoryState::ApplyMailboxOrRebase => {
            return Repo::Pending(name, Pending::Mailbox, changes);
        }
        git2::RepositoryState::Clean => {}
    }

    let Some(head_oid) = head.target() else {
        return Repo::Error;
    };

    let head_branch = git2::Branch::wrap(head);
    let upstream = match head_branch.upstream() {
        Ok(upstream) => upstream,
        Err(e) => {
            if head_branch
                .get()
                .name()
                .and_then(|r| repo.branch_upstream_remote(r))
                .is_err()
            {
                return Repo::Regular(name, Sync::Local, changes);
            }
            return match e.code() {
                git2::ErrorCode::NotFound => Repo::Regular(name, Sync::Gone, changes),
                _ => Repo::Error,
            };
        }
    };

    let Some(upstream_oid) = upstream.get().target() else {
        return Repo::Error;
    };

    let Ok((ahead, behind)) = repo.graph_ahead_behind(head_oid, upstream_oid) else {
        return Repo::Error;
    };

    Repo::Regular(name, Sync::Tracked { ahead, behind }, changes)
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
