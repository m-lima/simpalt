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

#[must_use]
pub fn parse(path: &std::path::Path) -> Repo {
    let Some(repo) = gix::discover(path).ok() else {
        return Repo::None;
    };

    let Some(changes) = get_changes(&repo) else {
        return Repo::Error;
    };

    let (name, head) = match repo.head() {
        Ok(head) => match head.kind {
            gix::head::Kind::Symbolic(r) => (r.name.shorten().to_string(), r),
            gix::head::Kind::Unborn(_) => return Repo::New(changes),
            gix::head::Kind::Detached { target, .. } => {
                return Repo::Detached(target.to_hex_with_len(8).to_string(), changes);
            }
        },
        Err(e) => {
            use gix::reference::find::existing::Error;
            match e {
                Error::NotFound { .. } => return Repo::New(changes),
                Error::Find(_) => return Repo::Error,
            }
        }
    };

    if let Some(state) = repo.state() {
        use gix::state::InProgress;

        let pending = match state {
            InProgress::ApplyMailbox | InProgress::ApplyMailboxRebase => Pending::Mailbox,
            InProgress::Bisect => Pending::Bisect,
            InProgress::CherryPick | InProgress::CherryPickSequence => Pending::Cherry,
            InProgress::Merge => Pending::Merge,
            InProgress::Rebase | InProgress::RebaseInteractive => Pending::Rebase,
            InProgress::Revert | InProgress::RevertSequence => Pending::Revert,
        };

        return Repo::Pending(name, pending, changes);
    }

    let mut branch = gix::prelude::ReferenceExt::attach(head, &repo);
    let Ok(head_oid) = branch.peel_to_id() else {
        return Repo::Error;
    };

    let Some(remote_ref) = branch
        .remote_tracking_ref_name(gix::remote::Direction::Fetch)
        .and_then(Result::ok)
    else {
        return Repo::Regular(name, Sync::Local, changes);
    };

    let Some(remote_oid) = repo
        .find_reference(&remote_ref)
        .ok()
        .and_then(|mut r| r.peel_to_id().ok())
    else {
        return Repo::Regular(name, Sync::Gone, changes);
    };

    let (ahead, behind) = {
        if head_oid == remote_oid {
            (0, 0)
        } else if let Ok(base) = repo.merge_base(head_oid, remote_oid) {
            let ahead = if head_oid == base {
                0
            } else {
                match repo
                    .rev_walk(Some(head_oid))
                    .with_hidden(Some(base))
                    .all()
                    .map(std::iter::Iterator::count)
                {
                    Ok(count) => count,
                    Err(_) => return Repo::Error,
                }
            };
            let behind = if remote_oid == base {
                0
            } else {
                match repo
                    .rev_walk(Some(remote_oid))
                    .with_hidden(Some(base))
                    .all()
                    .map(std::iter::Iterator::count)
                {
                    Ok(count) => count,
                    Err(_) => return Repo::Error,
                }
            };
            (ahead, behind)
        } else {
            return Repo::Error;
        }
    };

    Repo::Regular(name, Sync::Tracked { ahead, behind }, changes)
}

fn get_changes(repo: &gix::Repository) -> Option<Changes> {
    let status = repo.status(gix::progress::Discard).ok()?;
    let mut changes = Changes::default();

    for item in status.into_iter(None).ok()? {
        use gix::{
            diff::index::ChangeRef,
            status::{
                Item,
                index_worktree::Item as Index,
                plumbing::index_as_worktree::{Change, EntryStatus},
            },
        };

        match item.ok()? {
            Item::IndexWorktree(item) => match item {
                Index::Modification { status, .. } => match status {
                    EntryStatus::Conflict { .. } => changes.conflicted += 1,
                    EntryStatus::Change(change) => match change {
                        Change::Removed => changes.removed += 1,
                        Change::Type { .. }
                        | Change::Modification { .. }
                        | Change::SubmoduleModification(_) => changes.modified += 1,
                    },
                    EntryStatus::NeedsUpdate(_) | EntryStatus::IntentToAdd => {}
                },
                Index::DirectoryContents { .. } | Index::Rewrite { .. } => changes.added += 1,
            },
            Item::TreeIndex(change) => match change {
                ChangeRef::Addition { .. } => changes.added += 1,
                ChangeRef::Deletion { .. } => changes.removed += 1,
                ChangeRef::Modification { .. } | ChangeRef::Rewrite { .. } => changes.modified += 1,
            },
        }
    }

    Some(changes)
}

#[cfg(test)]
mod tests {
    use super::super::tests::TestRepo;
    use super::*;

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
