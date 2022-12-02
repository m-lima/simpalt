use crate::Result;

macro_rules! chevron {
    ($color: expr) => {
        concat!(
            style!(fg = color!(black), bg = $color, symbol!(div)),
            style!(fg = $color, bg = color!(reset), symbol!(div)),
        )
    };
}

trait EnvFetcher {
    fn pwd(&self) -> Option<std::path::PathBuf>;
    fn home(&self) -> Option<std::path::PathBuf>;
    fn venv(&self) -> bool;
}

#[derive(Copy, Clone)]
struct SysEnv;

impl EnvFetcher for SysEnv {
    fn pwd(&self) -> Option<std::path::PathBuf> {
        std::env::current_dir()
            .or_else(|_| std::env::var("PWD").map(std::path::PathBuf::from))
            .ok()
    }

    fn home(&self) -> Option<std::path::PathBuf> {
        std::env::var("HOME").map(std::path::PathBuf::from).ok()
    }

    fn venv(&self) -> bool {
        std::env::var("VIRTUAL_ENV").is_ok()
    }
}

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
    let error = if error {
        style!(fg = color!(red), symbol!(error) " ")
    } else {
        ""
    };

    let jobs = if jobs {
        style!(fg = color!(cyan), symbol!(jobs) " ")
    } else {
        ""
    };

    let venv = if enver.venv() {
        style!(fg = color!(green), symbol!(python))
    } else {
        ""
    };

    let (host, host_padding) = host.map_or_else(|| (String::new(), ""), |host| (host, " "));

    let pwd = enver.pwd();

    let pwd_string = if let Some(ref pwd) = pwd {
        pwd_string(pwd, enver)
    } else {
        String::new()
    };

    let git_string = if let Some(ref pwd) = pwd {
        git_string(git::prompt(pwd))
    } else {
        chevron!(color!(blue))
    };

    write!(
        out,
        concat!(
            style!(bg = color!(black), " {error}{jobs}{venv}"),
            style!(fg = color!(reset), "{host}"),
            style!(reset),
            style!(bg = color!(black), "{host_padding}{pwd_string} "),
            "{git_string}",
            style!(reset),
            " "
        ),
        error = error,
        jobs = jobs,
        venv = venv,
        host_padding = host_padding,
        host = host,
        pwd_string = pwd_string,
        git_string = git_string,
    )?;
    out.flush()
}

fn git_string(repo: git::Repo) -> &'static str {
    macro_rules! branch {
        ($branch: expr, $color: expr) => {
            concat!(style!(fg = $branch, symbol!(branch)), chevron!($color))
        };
        ($color: expr) => {
            concat!(symbol!(branch), chevron!($color))
        };
    }

    match repo {
        git::Repo::None => chevron!(color!(blue)),
        git::Repo::Clean(sync) => match sync {
            git::Sync::UpToDate => branch!(color!(green)),
            git::Sync::Behind => branch!(color!(red), color!(green)),
            git::Sync::Ahead => branch!(color!(yellow), color!(green)),
            git::Sync::Diverged => branch!(color!(magenta), color!(green)),
            git::Sync::Local => branch!(color!(blue), color!(green)),
        },
        git::Repo::Dirty(sync) => match sync {
            git::Sync::UpToDate => branch!(color!(yellow)),
            git::Sync::Behind => branch!(color!(red), color!(yellow)),
            git::Sync::Ahead => branch!(color!(yellow), color!(yellow)),
            git::Sync::Diverged => branch!(color!(magenta), color!(yellow)),
            git::Sync::Local => branch!(color!(blue), color!(yellow)),
        },
        git::Repo::Pending => concat!(symbol!(warn), chevron!(color!(cyan))),
        git::Repo::Untracked => branch!(color!(cyan)),
        git::Repo::Detached => branch!(color!(magenta)),
        git::Repo::Error => chevron!(color!(red)),
    }
}

fn pwd_string(path: &std::path::PathBuf, enver: &impl EnvFetcher) -> String {
    if let Some(home) = enver.home() {
        if home.eq(path) {
            return String::from("~");
        }
    }

    let (prefix, components) =
        path.components()
            .fold((None, vec![]), |(prefix, mut list), curr| match curr {
                std::path::Component::Prefix(prefix) => (Some(prefix), list),
                std::path::Component::RootDir | std::path::Component::Normal(_) => {
                    list.push(curr);
                    (prefix, list)
                }
                std::path::Component::ParentDir => {
                    list.pop();
                    (prefix, list)
                }
                std::path::Component::CurDir => (prefix, list),
            });

    if let Some(std::path::Component::Normal(path)) = components.last() {
        String::from(path.to_string_lossy())
    } else if let Some(prefix) = prefix {
        String::from(prefix.as_os_str().to_string_lossy())
    } else {
        String::from(std::path::MAIN_SEPARATOR)
    }
}

mod git {
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

    pub fn prompt(path: &std::path::PathBuf) -> Repo {
        let repo = match git2::Repository::open(path).ok() {
            Some(repo) => repo,
            None => return Repo::None,
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
}

#[cfg(test)]
mod tests {
    use super::{git, git_string, prompt_inner, EnvFetcher};
    use crate::test;

    macro_rules! branch {
        () => {
            symbol!(branch)
        };
        ($color: expr) => {
            style!(fg = $color, symbol!(branch))
        };
    }

    #[derive(Default)]
    struct MockEnv {
        pwd: Option<std::path::PathBuf>,
        home: Option<std::path::PathBuf>,
        venv: bool,
    }

    impl EnvFetcher for MockEnv {
        fn pwd(&self) -> Option<std::path::PathBuf> {
            self.pwd.clone()
        }

        fn home(&self) -> Option<std::path::PathBuf> {
            self.home.clone()
        }

        fn venv(&self) -> bool {
            self.venv
        }
    }

    #[test]
    fn all_empty() {
        let result = test(|s| prompt_inner(s, None, false, false, &MockEnv::default()));
        assert_eq!(
            result,
            concat!(
                style!(bg = color!(black)),
                " ",
                // Missing statuses
                style!(fg = color!(reset)),
                // Missing HOST
                style!(reset),
                style!(bg = color!(black)),
                // Missing PWD
                " ",
                chevron!(color!(blue)),
                style!(reset),
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
                style!(bg = color!(black)),
                " ",
                // Missing statuses
                style!(fg = color!(reset)),
                // Missing HOST
                style!(reset),
                style!(bg = color!(black)),
                "/",
                " ",
                chevron!(color!(blue)),
                style!(reset),
                " "
            )
        );
    }

    #[test]
    fn last_path() {
        let result = test(|s| {
            prompt_inner(
                s,
                None,
                false,
                false,
                &MockEnv {
                    pwd: Some(std::path::PathBuf::from("/some/home/path/")),
                    home: Some(std::path::PathBuf::from("/some/other/path")),
                    ..MockEnv::default()
                },
            )
        });
        assert_eq!(
            result,
            concat!(
                style!(bg = color!(black)),
                " ",
                // Missing statuses
                style!(fg = color!(reset)),
                // Missing HOST
                style!(reset),
                style!(bg = color!(black)),
                "path",
                " ",
                chevron!(color!(blue)),
                style!(reset),
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
                    pwd: Some(std::path::PathBuf::from("/some/home/path/")),
                    home: Some(std::path::PathBuf::from("/some/home/path")),
                    ..MockEnv::default()
                },
            )
        });
        assert_eq!(
            result,
            concat!(
                style!(bg = color!(black)),
                " ",
                // Missing statuses
                style!(fg = color!(reset)),
                // Missing HOST
                style!(reset),
                style!(bg = color!(black)),
                "~",
                " ",
                chevron!(color!(blue)),
                style!(reset),
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
                    pwd: Some(std::path::PathBuf::from("/some/home/path/")),
                    home: Some(std::path::PathBuf::from("/some/home/path")),
                    venv: true,
                },
            )
        });
        assert_eq!(
            result,
            concat!(
                style!(bg = color!(black)),
                " ",
                style!(fg = color!(red), symbol!(error)),
                " ",
                style!(fg = color!(cyan), symbol!(jobs)),
                " ",
                style!(fg = color!(green), symbol!(python)), // already contains a space
                style!(fg = color!(reset)),
                style!(fg = color!(red), "H"),
                style!(reset),
                style!(bg = color!(black)),
                " ",
                "~",
                " ",
                chevron!(color!(blue)),
                style!(reset),
                " "
            )
        );
    }

    #[test]
    fn git_sync_clean() {
        assert_eq!(
            git_string(git::Repo::Clean(git::Sync::Behind)),
            concat!(branch!(color!(red)), chevron!(color!(green)))
        );
        assert_eq!(
            git_string(git::Repo::Clean(git::Sync::Ahead)),
            concat!(branch!(color!(yellow)), chevron!(color!(green)))
        );
        assert_eq!(
            git_string(git::Repo::Clean(git::Sync::Diverged)),
            concat!(branch!(color!(magenta)), chevron!(color!(green)))
        );
        assert_eq!(
            git_string(git::Repo::Clean(git::Sync::UpToDate)),
            concat!(branch!(), chevron!(color!(green)))
        );
        assert_eq!(
            git_string(git::Repo::Clean(git::Sync::Local)),
            concat!(branch!(color!(blue)), chevron!(color!(green)))
        );
    }

    #[test]
    fn git_sync_dirty() {
        assert_eq!(
            git_string(git::Repo::Dirty(git::Sync::Behind)),
            concat!(branch!(color!(red)), chevron!(color!(yellow)))
        );
        assert_eq!(
            git_string(git::Repo::Dirty(git::Sync::Ahead)),
            concat!(branch!(color!(yellow)), chevron!(color!(yellow)))
        );
        assert_eq!(
            git_string(git::Repo::Dirty(git::Sync::Diverged)),
            concat!(branch!(color!(magenta)), chevron!(color!(yellow)))
        );
        assert_eq!(
            git_string(git::Repo::Dirty(git::Sync::UpToDate)),
            concat!(branch!(), chevron!(color!(yellow)))
        );
        assert_eq!(
            git_string(git::Repo::Dirty(git::Sync::Local)),
            concat!(branch!(color!(blue)), chevron!(color!(yellow)))
        );
    }

    #[test]
    fn git_status() {
        assert_eq!(git_string(git::Repo::None), chevron!(color!(blue)));
        assert_eq!(
            git_string(git::Repo::Clean(git::Sync::UpToDate)),
            concat!(branch!(), chevron!(color!(green)))
        );
        assert_eq!(
            git_string(git::Repo::Dirty(git::Sync::UpToDate)),
            concat!(branch!(), chevron!(color!(yellow)))
        );
        assert_eq!(
            git_string(git::Repo::Detached),
            concat!(branch!(), chevron!(color!(magenta)))
        );
        assert_eq!(
            git_string(git::Repo::Pending),
            concat!(symbol!(warn), chevron!(color!(cyan)))
        );
        assert_eq!(
            git_string(git::Repo::Untracked),
            concat!(branch!(), chevron!(color!(cyan)))
        );
        assert_eq!(git_string(git::Repo::Error), chevron!(color!(red)));
    }
}
