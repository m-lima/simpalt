pub mod long;
pub mod short;

#[cfg(test)]
mod tests {
    pub struct TestRepo<const REMOTE: bool> {
        pub repo: git2::Repository,
        pub sig: git2::Signature<'static>,
        pub index: std::sync::Mutex<git2::Index>,
        commits: std::sync::atomic::AtomicU8,
    }

    impl<const REMOTE: bool> TestRepo<REMOTE> {
        fn get_commits(&self) -> u8 {
            self.commits
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        }

        pub fn commit(&self, history: &[&git2::Commit<'_>]) -> (git2::Commit<'_>, git2::Oid) {
            let oid = self.index.lock().unwrap().write_tree().unwrap();
            let tree = self.repo.find_tree(oid).unwrap();
            let message = format!("commit{}", self.get_commits());
            let oid = self
                .repo
                .commit(Some("HEAD"), &self.sig, &self.sig, &message, &tree, history)
                .unwrap();
            let commit = self.repo.find_commit(oid).unwrap();
            self.index.lock().unwrap().write().unwrap();
            (commit, oid)
        }

        pub fn commit_headless(&self, history: &[&git2::Commit<'_>]) -> git2::Oid {
            let oid = self.index.lock().unwrap().write_tree().unwrap();
            let tree = self.repo.find_tree(oid).unwrap();
            let message = format!("commit{}", self.get_commits());
            let oid = self
                .repo
                .commit(None, &self.sig, &self.sig, &message, &tree, history)
                .unwrap();
            self.index.lock().unwrap().write().unwrap();
            oid
        }

        pub fn add_path(&self, path: &str) {
            self.index
                .lock()
                .unwrap()
                .add_path(&std::path::PathBuf::from(path))
                .unwrap();
        }
    }

    impl TestRepo<false> {
        pub fn new(path: &std::path::Path) -> Self {
            let repo = git2::Repository::init(path).unwrap();
            let index = std::sync::Mutex::new(repo.index().unwrap());
            let sig = git2::Signature::now("Test", "test@test.com").unwrap();
            Self {
                repo,
                sig,
                index,
                commits: std::sync::atomic::AtomicU8::new(0),
            }
        }
    }

    impl TestRepo<true> {
        pub fn new_with_remote(path: &std::path::Path) -> Self {
            let TestRepo {
                repo,
                sig,
                index,
                commits,
            } = TestRepo::new(path);
            repo.remote("origin", "file:///dev/null").unwrap();
            Self {
                repo,
                sig,
                index,
                commits,
            }
        }

        pub fn set_upstream(&self, oid: git2::Oid) {
            self.repo
                .reference("refs/remotes/origin/master", oid, true, "remote")
                .unwrap();
            let mut local_branch = self
                .repo
                .find_branch("master", git2::BranchType::Local)
                .unwrap();
            local_branch.set_upstream(Some("origin/master")).unwrap();
        }

        pub fn set_pointers(&self, local: git2::Oid, remote: git2::Oid) {
            self.repo
                .reference("refs/heads/master", local, true, "move local")
                .unwrap();
            self.repo
                .reference("refs/remotes/origin/master", remote, true, "move remote")
                .unwrap();
        }
    }
}
