#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Repo {
    None,
    Clean(Sync),
    Dirty(Sync),
    Detached,
    Pending,
    Error,
}

// #[derive(Debug, Copy, Clone, Eq, PartialEq)]
// pub enum State {
//     Bisect,
//     Cherry,
//     Merge,
//     Rebase,
//     Revert,
//     Mailbox,
// }
//
// impl From<git2::RepositoryState> for State {
//     fn from(state: git2::RepositoryState) -> Self {
//         use git2::RepositoryState;
//
//         match state {
//             RepositoryState::Merge => Self::Merge,
//             RepositoryState::Revert | RepositoryState::RevertSequence => Self::Revert,
//             RepositoryState::CherryPick | RepositoryState::CherryPickSequence => Self::Cherry,
//             RepositoryState::Bisect => Self::Bisect,
//             RepositoryState::Rebase
//             | RepositoryState::RebaseInteractive
//             | RepositoryState::RebaseMerge => Self::Rebase,
//             RepositoryState::ApplyMailbox | RepositoryState::ApplyMailboxOrRebase => Self::Mailbox,
//             RepositoryState::Clean => {}
//         }
//     }
// }

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Sync {
    Behind,
    Ahead,
    Diverged,
    UpToDate,
    Local,
}

pub fn prompt(path: &std::path::PathBuf) -> Repo {
    let repo = match git2::Repository::open(path).ok() {
        Some(repo) => repo,
        None => return Repo::None,
    };

    if repo.state() != git2::RepositoryState::Clean {
        return Repo::Pending;
    }

    let head = match repo.head() {
        Ok(head) => head,
        Err(_) => return Repo::Error,
    };

    if !head.is_branch() {
        return Repo::Detached;
    }

    let head = match head.name() {
        Some(name) => name,
        None => return Repo::Error,
    };

    let sync = match repo.branch_upstream_name(head) {
        Ok(buf) => {
            match buf.as_str() {
                Some(upstream) => {
                    match repo.revparse(format!("{head}..{upstream}").as_str()) {
                        Ok(a) => {
                            println!("to: {:?}, from: {:?}", a.to(), a.from());
                            Sync::Behind
                        }
                        Err(e) => {
                            println!("{e}");
                            Sync::UpToDate
                        }
                    }
                    // let r = repo.revwalk().unwrap();
                    // r.take_while(Result::is_ok).map(Result::unwrap).map(|e| e
                    // r.reset()
                }
                None => Sync::Diverged,
            }
        }
        Err(_) => Sync::Local,
    };

    // let remote = repo
    //     .branch_upstream_name(head.as_str())
    //     .map(|b| b.as_str().map(String::from));

    let status = match repo.statuses(Some(
        git2::StatusOptions::new()
            .include_ignored(false)
            .include_untracked(true),
    )) {
        Ok(status) => status,
        Err(_) => return Repo::Error,
    };

    if status.iter().next().is_some() {
        Repo::Dirty(sync)
    } else {
        Repo::Clean(sync)
    }
}
