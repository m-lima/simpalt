use crate::Result;
use crate::git::short as git;

macro_rules! chevron {
    ($color: expr) => {
        concat!(
            style!(fg = color!(black), bg = $color, symbol!(div)),
            style!(reset to fg = $color, symbol!(div)),
        )
    };
}

pub fn render<Out>(out: Out, host: Option<String>, error: bool, jobs: bool) -> Result
where
    Out: std::io::Write,
{
    render_inner(out, host, error, jobs, &SysEnv)
}

fn render_inner<Out, Env>(
    mut out: Out,
    host: Option<String>,
    error: bool,
    jobs: bool,
    enver: &Env,
) -> Result
where
    Out: std::io::Write,
    Env: EnvFetcher,
{
    write!(out, style!(reset to bg = color!(black), " "))?;
    let mut should_recolor = false;

    if error {
        write!(out, style!(fg = color!(red), symbol!(error), " "))?;
        should_recolor = true;
    }

    if jobs {
        write!(out, style!(fg = color!(cyan), symbol!(jobs), " "))?;
        should_recolor = true;
    }

    if enver.venv() {
        write!(out, style!(fg = color!(green), symbol!(python), " "))?;
        should_recolor = true;
    }

    if let Some(active) = enver.direnv() {
        if active {
            write!(out, style!(fg = color!(green), symbol!(direnv), " "))?;
        } else {
            write!(out, style!(fg = color!(blue), symbol!(direnv), " "))?;
        }
        should_recolor = true;
    }

    if let Some(host) = host {
        if should_recolor {
            write!(out, style!(fg = color!(reset), "{host}"), host = host)?;
            should_recolor = false;
        } else {
            write!(out, "{host}")?;
        }
        write!(out, style!(reset to bg = color!(black), " "))?;
    }

    let pwd = enver.pwd();

    if let Some(ref pwd) = pwd {
        if should_recolor {
            write!(
                out,
                style!(fg = color!(reset), "{pwd} "),
                pwd = pwd_string(pwd, enver)
            )?;
        } else {
            write!(out, "{pwd} ", pwd = pwd_string(pwd, enver))?;
        }
    }

    if let Some(ref pwd) = pwd {
        out.git(git::parse(pwd))?;
    } else {
        write!(out, chevron!(color!(blue)))?;
    }

    write!(out, style!(reset, " "))?;
    out.flush()
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

trait Writer {
    fn git(&mut self, repo: git::Repo) -> Result;
}

impl<W: std::io::Write> Writer for W {
    fn git(&mut self, repo: git::Repo) -> Result {
        macro_rules! branch {
            (none $color: expr) => {
                write!(self, chevron!($color))
            };
            (warn $color: expr) => {
                write!(self, concat!(symbol!(warn), chevron!($color)))
            };
            ($branch: expr, $color: expr) => {
                write!(
                    self,
                    style!(fg = $branch, symbol!(branch), chevron!($color))
                )
            };
            ($color: expr) => {
                write!(self, concat!(symbol!(branch), chevron!($color)))
            };
        }

        match repo {
            git::Repo::None => branch!(none color!(blue)),
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
            git::Repo::Pending => branch!(warn color!(cyan)),
            git::Repo::Untracked => branch!(color!(cyan)),
            git::Repo::Detached => branch!(color!(magenta)),
            git::Repo::Error => branch!(none color!(red)),
        }
    }
}

trait EnvFetcher {
    fn pwd(&self) -> Option<std::path::PathBuf>;
    fn home(&self) -> Option<std::path::PathBuf>;
    fn venv(&self) -> bool;
    fn direnv(&self) -> Option<bool>;
}

#[derive(Copy, Clone)]
struct SysEnv;

impl EnvFetcher for SysEnv {
    fn pwd(&self) -> Option<std::path::PathBuf> {
        std::env::current_dir()
            .ok()
            .or_else(|| std::env::var_os("PWD").map(std::path::PathBuf::from))
    }

    fn home(&self) -> Option<std::path::PathBuf> {
        std::env::var_os("HOME").map(std::path::PathBuf::from)
    }

    fn venv(&self) -> bool {
        std::env::var("VIRTUAL_ENV").is_ok()
    }

    fn direnv(&self) -> Option<bool> {
        super::direnv::is_active()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        direnv: Option<bool>,
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

        fn direnv(&self) -> Option<bool> {
            self.direnv
        }
    }

    #[test]
    fn all_empty() {
        let result = test(|s| render_inner(s, None, false, false, &MockEnv::default()));
        assert_eq!(
            result,
            concat!(
                style!(reset to bg = color!(black)),
                " ",
                // Missing statuses
                // Missing HOST
                // Missing PWD
                chevron!(color!(blue)),
                style!(reset),
                " "
            )
        );
    }

    #[test]
    fn just_pwd() {
        let result = test(|s| {
            render_inner(
                s,
                None,
                true,
                true,
                &MockEnv {
                    pwd: Some(std::path::PathBuf::from("/")),
                    ..MockEnv::default()
                },
            )
        });
        assert_eq!(
            result,
            concat!(
                style!(reset to bg = color!(black)),
                " ",
                style!(fg = color!(red)),
                symbol!(error),
                " ",
                style!(fg = color!(cyan)),
                symbol!(jobs),
                " ",
                // Missing HOST
                style!(fg = color!(reset)),
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
            render_inner(
                s,
                None,
                true,
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
                style!(reset to bg = color!(black)),
                " ",
                style!(fg = color!(red)),
                symbol!(error),
                // Missing jobs
                // Missing HOST
                " ",
                style!(fg = color!(reset)),
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
            render_inner(
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
                style!(reset to bg = color!(black)),
                " ",
                // Missing statuses
                // Missing HOST
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
            render_inner(
                s,
                Some(String::from("[31mH")),
                true,
                true,
                &MockEnv {
                    pwd: Some(std::path::PathBuf::from("/some/home/path/")),
                    home: Some(std::path::PathBuf::from("/some/home/path")),
                    venv: true,
                    direnv: Some(false),
                },
            )
        });
        assert_eq!(
            result,
            concat!(
                style!(reset to bg = color!(black)),
                " ",
                style!(fg = color!(red), symbol!(error)),
                " ",
                style!(fg = color!(cyan), symbol!(jobs)),
                " ",
                style!(fg = color!(green), symbol!(python)),
                " ",
                style!(fg = color!(blue), symbol!(direnv)),
                " ",
                style!(fg = color!(reset)),
                style!(fg = color!(red), "H"),
                style!(reset to bg = color!(black)),
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
    fn direnv() {
        let result = test(|s| {
            render_inner(
                s,
                Some(String::from("[31mH")),
                true,
                true,
                &MockEnv {
                    pwd: Some(std::path::PathBuf::from("/some/home/path/")),
                    home: Some(std::path::PathBuf::from("/some/home/path")),
                    venv: false,
                    direnv: Some(false),
                },
            )
        });
        assert_eq!(
            result,
            concat!(
                style!(reset to bg = color!(black)),
                " ",
                style!(fg = color!(red), symbol!(error)),
                " ",
                style!(fg = color!(cyan), symbol!(jobs)),
                " ",
                style!(fg = color!(blue), symbol!(direnv)),
                " ",
                style!(fg = color!(reset)),
                style!(fg = color!(red), "H"),
                style!(reset to bg = color!(black)),
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
    fn direnv_active() {
        let result = test(|s| {
            render_inner(
                s,
                Some(String::from("[31mH")),
                true,
                true,
                &MockEnv {
                    pwd: Some(std::path::PathBuf::from("/some/home/path/")),
                    home: Some(std::path::PathBuf::from("/some/home/path")),
                    venv: false,
                    direnv: Some(true),
                },
            )
        });
        assert_eq!(
            result,
            concat!(
                style!(reset to bg = color!(black)),
                " ",
                style!(fg = color!(red), symbol!(error)),
                " ",
                style!(fg = color!(cyan), symbol!(jobs)),
                " ",
                style!(fg = color!(green), symbol!(direnv)),
                " ",
                style!(fg = color!(reset)),
                style!(fg = color!(red), "H"),
                style!(reset to bg = color!(black)),
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
            test(|s| s.git(git::Repo::Clean(git::Sync::Behind))),
            concat!(branch!(color!(red)), chevron!(color!(green)))
        );
        assert_eq!(
            test(|s| s.git(git::Repo::Clean(git::Sync::Ahead))),
            concat!(branch!(color!(yellow)), chevron!(color!(green)))
        );
        assert_eq!(
            test(|s| s.git(git::Repo::Clean(git::Sync::Diverged))),
            concat!(branch!(color!(magenta)), chevron!(color!(green)))
        );
        assert_eq!(
            test(|s| s.git(git::Repo::Clean(git::Sync::UpToDate))),
            concat!(branch!(), chevron!(color!(green)))
        );
        assert_eq!(
            test(|s| s.git(git::Repo::Clean(git::Sync::Local))),
            concat!(branch!(color!(blue)), chevron!(color!(green)))
        );
    }

    #[test]
    fn git_sync_dirty() {
        assert_eq!(
            test(|s| s.git(git::Repo::Dirty(git::Sync::Behind))),
            concat!(branch!(color!(red)), chevron!(color!(yellow)))
        );
        assert_eq!(
            test(|s| s.git(git::Repo::Dirty(git::Sync::Ahead))),
            concat!(branch!(color!(yellow)), chevron!(color!(yellow)))
        );
        assert_eq!(
            test(|s| s.git(git::Repo::Dirty(git::Sync::Diverged))),
            concat!(branch!(color!(magenta)), chevron!(color!(yellow)))
        );
        assert_eq!(
            test(|s| s.git(git::Repo::Dirty(git::Sync::UpToDate))),
            concat!(branch!(), chevron!(color!(yellow)))
        );
        assert_eq!(
            test(|s| s.git(git::Repo::Dirty(git::Sync::Local))),
            concat!(branch!(color!(blue)), chevron!(color!(yellow)))
        );
    }

    #[test]
    fn git_status() {
        assert_eq!(test(|s| s.git(git::Repo::None)), chevron!(color!(blue)));
        assert_eq!(
            test(|s| s.git(git::Repo::Clean(git::Sync::UpToDate))),
            concat!(branch!(), chevron!(color!(green)))
        );
        assert_eq!(
            test(|s| s.git(git::Repo::Dirty(git::Sync::UpToDate))),
            concat!(branch!(), chevron!(color!(yellow)))
        );
        assert_eq!(
            test(|s| s.git(git::Repo::Detached)),
            concat!(branch!(), chevron!(color!(magenta)))
        );
        assert_eq!(
            test(|s| s.git(git::Repo::Pending)),
            concat!(symbol!(warn), chevron!(color!(cyan)))
        );
        assert_eq!(
            test(|s| s.git(git::Repo::Untracked)),
            concat!(branch!(), chevron!(color!(cyan)))
        );
        assert_eq!(test(|s| s.git(git::Repo::Error)), chevron!(color!(red)));
    }
}
