use crate::Result;

pub fn prompt(out: impl std::io::Write, host: Option<String>, error: bool, jobs: bool) -> Result {
    prompt_inner(out, host, error, jobs, &SysEnv)
}

fn prompt_inner(
    mut out: impl std::io::Write,
    host: Option<String>,
    error: bool,
    jobs: bool,
    enver: &impl EnvFetcher,
) -> Result {
    let mut last = None;

    if error {
        out.div(&mut last, color!(black), color!(red))?;
        write!(out, symbol!(error))?;
    };

    if jobs {
        out.div(&mut last, color!(black), color!(cyan))?;
        write!(out, symbol!(jobs))?;
    }

    if let Some(host) = host {
        out.div(&mut last, color!(black), color!(reset))?;
        write!(out, "{host}")?;
        write!(out, style!(reset))?;
        write!(out, style!(bg = color!(black)))?;
    };

    if let Some(venv) = enver.venv() {
        out.div(&mut last, color!(cyan), color!(black))?;
        if let Some(venv) = venv.rsplit(std::path::MAIN_SEPARATOR).next() {
            write!(out, "{venv}")?;
        } else {
            write!(out, "{venv}")?;
        }
    };

    let pwd = enver.pwd();

    out.div(&mut last, color!(blue), color!(black))?;
    if let Some(ref pwd) = pwd {
        if let Some(pwd) = pwd.to_str() {
            if let Some(pwd) = enver.home().and_then(|home| pwd.strip_prefix(&home)) {
                write!(out, "~{pwd}")?;
            } else {
                write!(out, "{pwd}")?;
            }
        }
    }

    if let Some(ref pwd) = pwd {
        match git::prompt(pwd) {
            git::Repo::None => {}
            git::Repo::Error => {
                out.div(&mut last, color!(red), color!(black))?;
                write!(out, "!")?;
            }
            git::Repo::Regular(head, sync, changes) => {
                if changes.clean() {
                    out.render_sync(&mut last, sync)?;
                    out.div(&mut last, color!(green), color!(black))?;
                    write!(out, concat!(symbol!(branch), "{head}"), head = head)?;
                } else {
                    out.render_changes(&mut last, changes)?;
                    if !matches!(
                        sync,
                        git::Sync::Tracked {
                            ahead: 0,
                            behind: 0
                        }
                    ) {
                        out.div(&mut last, color!(black), color!(reset))?;
                        write!(out, symbol!(div thin))?;
                        out.render_sync(&mut last, sync)?;
                    }
                    out.div(&mut last, color!(yellow), color!(black))?;
                    write!(out, concat!(symbol!(branch), "{head}"), head = head)?;
                }
            }
            git::Repo::Detached(head, changes) => {
                out.render_changes(&mut last, changes)?;
                out.div(&mut last, color!(magenta), color!(black))?;
                write!(out, concat!(symbol!(ref), "{head}"), head = head)?;
            }
            git::Repo::Pending(head, pending, changes) => {
                out.render_changes(&mut last, changes)?;
                out.div(&mut last, color!(cyan), color!(black))?;
                write!(
                    out,
                    concat!(symbol!(branch), "{head} {pending}"),
                    head = head,
                    pending = pending_symbol(pending),
                )?;
            }
            git::Repo::New(changes) => {
                out.render_changes(&mut last, changes)?;
                out.div(&mut last, color!(cyan), color!(black))?;
                write!(out, symbol!(new))?;
            }
        }
    };
    out.div(&mut last, color!(reset), color!(reset))?;
    out.flush()
}

trait Writer {
    fn div(
        &mut self,
        last: &mut Option<&'static str>,
        to: &'static str,
        fg: &'static str,
    ) -> Result;

    fn render_changes(&mut self, last: &mut Option<&'static str>, changes: git::Changes) -> Result;

    fn render_sync(&mut self, last: &mut Option<&'static str>, sync: git::Sync) -> Result;
}

impl<W: std::io::Write> Writer for W {
    fn div(
        &mut self,
        last: &mut Option<&'static str>,
        to: &'static str,
        fg: &'static str,
    ) -> Result {
        if let Some(last) = last {
            if &to == last {
                write!(self, " [3{fg}m")?;
            } else {
                write!(
                    self,
                    concat!(" [3{last}m[4{to}m", symbol!(div), "[3{fg}m "),
                    last = last,
                    to = to,
                    fg = fg,
                )?;
            }
        } else {
            write!(self, "[3{fg}m[4{to}m ")?;
        }
        *last = Some(to);
        Ok(())
    }

    fn render_changes(&mut self, last: &mut Option<&'static str>, changes: git::Changes) -> Result {
        if changes.added > 0 {
            self.div(last, color!(black), color!(green))?;
            write!(self, "+{added}", added = changes.added)?;
        }

        if changes.removed > 0 {
            self.div(last, color!(black), color!(red))?;
            write!(self, "-{removed}", removed = changes.removed)?;
        }

        if changes.modified > 0 {
            self.div(last, color!(black), color!(blue))?;
            write!(self, "~{modified}", modified = changes.modified)?;
        }

        if changes.conflicted > 0 {
            self.div(last, color!(black), color!(magenta))?;
            write!(self, "!{conflicted}", conflicted = changes.conflicted)?;
        }
        Ok(())
    }

    fn render_sync(&mut self, last: &mut Option<&'static str>, sync: git::Sync) -> Result {
        match sync {
            git::Sync::Local => {
                self.div(last, color!(black), color!(cyan))?;
                write!(self, concat!(symbol!(local), " local"))
            }
            git::Sync::Gone => {
                self.div(last, color!(black), color!(magenta))?;
                write!(self, concat!(symbol!(gone), " gone"))
            }
            git::Sync::Tracked { ahead, behind } => {
                if ahead > 0 {
                    self.div(last, color!(black), color!(yellow))?;
                    write!(self, concat!(symbol!(ahead), "{ahead}"), ahead = ahead)?;
                }
                if behind > 0 {
                    self.div(last, color!(black), color!(red))?;
                    write!(self, concat!(symbol!(behind), "{behind}"), behind = behind)?;
                }
                Ok(())
            }
        }
    }
}

const fn pending_symbol(pending: git::Pending) -> &'static str {
    match pending {
        git::Pending::Merge => symbol!(merge),
        git::Pending::Revert => symbol!(revert),
        git::Pending::Cherry => symbol!(cherry),
        git::Pending::Bisect => symbol!(bisect),
        git::Pending::Rebase => symbol!(rebase),
        git::Pending::Mailbox => symbol!(mailbox),
    }
}

trait EnvFetcher {
    fn pwd(&self) -> Option<std::path::PathBuf>;
    fn home(&self) -> Option<String>;
    fn venv(&self) -> Option<String>;
}

#[derive(Copy, Clone)]
struct SysEnv;

impl EnvFetcher for SysEnv {
    fn pwd(&self) -> Option<std::path::PathBuf> {
        std::env::current_dir()
            .or_else(|_| std::env::var("PWD").map(std::path::PathBuf::from))
            .ok()
    }

    fn home(&self) -> Option<String> {
        std::env::var("HOME").ok()
    }

    fn venv(&self) -> Option<String> {
        std::env::var("VIRTUAL_ENV").ok()
    }
}

mod git {
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

    pub fn prompt(path: &std::path::PathBuf) -> Repo {
        fn short_id(oid: git2::Oid) -> Option<String> {
            let mut oid = oid.as_bytes().iter();
            match (oid.next(), oid.next(), oid.next(), oid.next()) {
                (Some(a), None, None, None) => Some(format!("{a:02x}")),
                (Some(a), Some(b), None, None) => Some(format!("{a:02x}{b:02x}")),
                (Some(a), Some(b), Some(c), None) => Some(format!("{a:02x}{b:02x}{c:02x}")),
                (Some(a), Some(b), Some(c), Some(d)) => {
                    Some(format!("{a:02x}{b:02x}{c:02x}{d:02x}"))
                }
                _ => None,
            }
        }

        let repo = match git2::Repository::open(path).ok() {
            Some(repo) => repo,
            None => return Repo::None,
        };

        let changes = match get_changes(&repo) {
            Some(changes) => changes,
            None => return Repo::Error,
        };

        let head = match repo.head() {
            Ok(head) => head,
            Err(_) => return Repo::New(changes),
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
                return Repo::Pending(head, Pending::Revert, changes)
            }
            git2::RepositoryState::CherryPick | git2::RepositoryState::CherryPickSequence => {
                return Repo::Pending(head, Pending::Cherry, changes)
            }
            git2::RepositoryState::Bisect => return Repo::Pending(head, Pending::Bisect, changes),
            git2::RepositoryState::Rebase
            | git2::RepositoryState::RebaseInteractive
            | git2::RepositoryState::RebaseMerge => {
                return Repo::Pending(head, Pending::Rebase, changes)
            }
            git2::RepositoryState::ApplyMailbox | git2::RepositoryState::ApplyMailboxOrRebase => {
                return Repo::Pending(head, Pending::Mailbox, changes)
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
}

#[cfg(test)]
mod tests {
    use super::{prompt_inner, EnvFetcher};
    use crate::test;

    #[derive(Default)]
    struct MockEnv {
        pwd: Option<std::path::PathBuf>,
        home: Option<String>,
        venv: Option<String>,
    }

    impl EnvFetcher for MockEnv {
        fn pwd(&self) -> Option<std::path::PathBuf> {
            self.pwd.clone()
        }

        fn home(&self) -> Option<String> {
            self.home.clone()
        }

        fn venv(&self) -> Option<String> {
            self.venv.clone()
        }
    }

    #[test]
    fn all_empty() {
        let result = test(|s| prompt_inner(s, None, false, false, &MockEnv::default()));
        assert_eq!(
            result,
            concat!(
                // Missing error
                // Missing jobs
                // Missing venv
                // Missing HOST
                style!(fg = color!(black), bg = color!(blue)),
                " ",
                // Missing PWD
                " ",
                style!(fg = color!(blue), bg = color!(reset), symbol!(div)),
                style!(fg = color!(reset)),
                " "
            )
        );
    }

    #[test]
    fn just_pwd() {
        let result = test(|s| {
            prompt_inner(
                s,
                None,
                false,
                false,
                &MockEnv {
                    pwd: Some(std::path::PathBuf::from("/")),
                    ..MockEnv::default()
                },
            )
        });
        assert_eq!(
            result,
            concat!(
                // Missing error
                // Missing jobs
                // Missing venv
                // Missing HOST
                style!(fg = color!(black), bg = color!(blue)),
                " / ",
                style!(fg = color!(blue), bg = color!(reset), symbol!(div)),
                style!(fg = color!(reset)),
                " "
            )
        );
    }

    #[test]
    fn home_match() {
        let result = test(|s| {
            prompt_inner(
                s,
                None,
                false,
                false,
                &MockEnv {
                    pwd: Some(std::path::PathBuf::from("/some/home/path/further/on")),
                    home: Some(String::from("/some/home/path")),
                    ..MockEnv::default()
                },
            )
        });
        assert_eq!(
            result,
            concat!(
                // Missing error
                // Missing jobs
                // Missing venv
                // Missing HOST
                style!(fg = color!(black), bg = color!(blue)),
                " ~/further/on ",
                style!(fg = color!(blue), bg = color!(reset), symbol!(div)),
                style!(fg = color!(reset)),
                " "
            )
        );
    }

    #[test]
    fn all_tags() {
        let result = test(|s| {
            prompt_inner(
                s,
                Some(String::from("[31mH")),
                true,
                true,
                &MockEnv {
                    pwd: Some(std::path::PathBuf::from("/some/home/path/further/on")),
                    home: Some(String::from("/some/home/path")),
                    venv: Some(String::from("py")),
                },
            )
        });
        assert_eq!(
            result,
            concat!(
                style!(fg = color!(red), bg = color!(black)),
                " ",
                symbol!(error),
                " ",
                style!(fg = color!(cyan), symbol!(jobs)),
                " ",
                style!(fg = color!(reset), style!(fg = color!(red), "H")),
                style!(reset),
                style!(bg = color!(black)),
                " ",
                style!(fg = color!(black), bg = color!(cyan), symbol!(div)),
                style!(fg = color!(black)),
                " py ",
                style!(fg = color!(cyan), bg = color!(blue), symbol!(div)),
                style!(fg = color!(black)),
                " ~/further/on ",
                style!(fg = color!(blue), bg = color!(reset), symbol!(div)),
                style!(fg = color!(reset)),
                " "
            )
        );
    }
}
