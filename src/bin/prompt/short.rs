use crate::Result;
use simpalt::{
    git::short as git,
    print::{Color, Div, Printer, Symbol},
};

macro_rules! chevron {
    ($printer: ident, $color: ident) => {
        $printer
            .div(Div::ChevronLeft, Color::$color)?
            .div(Div::ChevronLeft, Color::Reset)?
    };
}

pub fn render<P>(printer: P, host: Option<String>, error: bool, jobs: bool) -> Result
where
    P: Printer,
{
    render_inner(printer, host, error, jobs, &SysEnv)
}

fn render_inner<P, Env>(
    mut printer: P,
    host: Option<String>,
    error: bool,
    jobs: bool,
    enver: &Env,
) -> Result
where
    P: Printer,
    Env: EnvFetcher,
{
    printer.fg(Color::Reset).bg(Color::Black);

    if error {
        printer.fg(Color::Red).txt_gap(Symbol::Error)?;
    }

    if jobs {
        printer.fg(Color::Magenta).txt_gap(Symbol::Jobs)?;
    }

    let direnv = enver.direnv();
    if let Some(nixshell) = enver.nixshell()
        && (nixshell || !matches!(direnv, Some(true)))
    {
        printer.fg(Color::Yellow).txt_gap(Symbol::Package)?;
    }

    if let Some(active) = direnv {
        if active {
            printer.fg(Color::Green)
        } else {
            printer.fg(Color::Red)
        }
        .txt_gap(Symbol::Direnv)?;
    }

    if enver.venv() {
        printer.fg(Color::Green).txt_gap(Symbol::Python)?;
    }

    if let Some(host) = host {
        printer.fg(Color::Reset).gap()?.txt(&host)?.invalidate();
    }

    printer.fg(Color::Reset).bg(Color::Black);

    let pwd = enver.pwd();

    if let Some(ref pwd) = pwd {
        printer.txt_gap(pwd_string(pwd, enver))?;
        render_git(&mut printer, git::parse(pwd))?;
    } else {
        printer.gap()?;
        chevron!(printer, Blue);
    }

    printer.fg(Color::Reset).gap()?.flush()
}

fn pwd_string(path: &std::path::PathBuf, enver: &impl EnvFetcher) -> String {
    if let Some(home) = enver.home()
        && home.eq(path)
    {
        return String::from("~");
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

fn render_git<P>(printer: &mut P, repo: git::Repo) -> Result
where
    P: Printer,
{
    match repo {
        git::Repo::None => chevron!(printer, Blue),
        git::Repo::Clean(sync) => {
            match sync {
                git::Sync::UpToDate => printer.fg(Color::Reset),
                git::Sync::Behind => printer.fg(Color::Red),
                git::Sync::Ahead => printer.fg(Color::Yellow),
                git::Sync::Diverged => printer.fg(Color::Magenta),
                git::Sync::Local => printer.fg(Color::Blue),
            };
            printer.txt(Symbol::Branch)?;
            chevron!(printer, Green)
        }
        git::Repo::Dirty(sync) => {
            match sync {
                git::Sync::UpToDate => printer.fg(Color::Reset),
                git::Sync::Behind => printer.fg(Color::Red),
                git::Sync::Ahead => printer.fg(Color::Yellow),
                git::Sync::Diverged => printer.fg(Color::Magenta),
                git::Sync::Local => printer.fg(Color::Blue),
            };
            printer.txt(Symbol::Branch)?;
            chevron!(printer, Yellow)
        }
        git::Repo::Detached => {
            printer.txt(Symbol::Branch)?;
            chevron!(printer, Magenta)
        }
        git::Repo::Pending => {
            printer.txt(Symbol::Warn)?;
            chevron!(printer, Cyan)
        }
        git::Repo::Untracked => {
            printer.txt(Symbol::Branch)?;
            chevron!(printer, Cyan)
        }
        git::Repo::Error => chevron!(printer, Red),
    };
    Ok(())
}

trait EnvFetcher {
    fn pwd(&self) -> Option<std::path::PathBuf>;
    fn home(&self) -> Option<std::path::PathBuf>;
    fn venv(&self) -> bool;
    fn direnv(&self) -> Option<bool>;
    fn nixshell(&self) -> Option<bool>;
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

    fn nixshell(&self) -> Option<bool> {
        std::env::var("IN_NIX_SHELL")
            .ok()
            .map(|_| std::env::var("NIX_SHELL").is_ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{expect, test, test_from};

    macro_rules! tip {
        ($color: literal) => {{
            let mut tip = String::with_capacity(25);
            tip.push_str(concat!("[30;4", $color, "m"));
            tip.push_str(Div::ChevronLeft.str());
            tip.push_str(concat!("[;3", $color, "m"));
            tip.push_str(Div::ChevronLeft.str());
            tip.push_str("[m");
            tip.push_str(" ");
            tip
        }
        .as_str()};
        (red) => {
            tip!(1)
        };
        (green) => {
            tip!(2)
        };
        (yellow) => {
            tip!(3)
        };
        (blue) => {
            tip!(4)
        };
        (magenta) => {
            tip!(5)
        };
        (cyan) => {
            tip!(6)
        };
    }

    const HOST: &str = "[31mH";

    #[derive(Default)]
    struct MockEnv {
        pwd: Option<std::path::PathBuf>,
        home: Option<std::path::PathBuf>,
        venv: bool,
        direnv: Option<bool>,
        nixshell: Option<bool>,
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

        fn nixshell(&self) -> Option<bool> {
            self.nixshell
        }
    }

    #[test]
    fn all_empty() {
        let result = test(|s| render_inner(s, None, false, false, &MockEnv::default()));
        let expected = expect(
            &result,
            [
                "[;40m",
                " ",
                // Missing statuses
                // Missing HOST
                // Missing PWD
                tip!(blue),
            ],
        );

        assert_eq!(result, expected);
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
        let expected = expect(
            &result,
            [
                "[31;40m",
                " ",
                Symbol::Error.str(),
                " ",
                "[35m",
                Symbol::Jobs.str(),
                " ",
                // Missing HOST
                "[39m",
                "/",
                " ",
                tip!(blue),
            ],
        );
        assert_eq!(result, expected);
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
        let expected = expect(
            &result,
            [
                "[31;40m",
                " ",
                Symbol::Error.str(),
                " ",
                // Missing jobs
                // Missing HOST
                "[39m",
                "path",
                " ",
                tip!(blue),
            ],
        );
        assert_eq!(result, expected);
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
        let expected = expect(
            &result,
            [
                "[;40m",
                " ",
                // Missing statuses
                // Missing HOST
                "~",
                " ",
                tip!(blue),
            ],
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn all_tags() {
        let result = test(|s| {
            render_inner(
                s,
                Some(String::from(HOST)),
                true,
                true,
                &MockEnv {
                    pwd: Some(std::path::PathBuf::from("/some/home/path/")),
                    home: Some(std::path::PathBuf::from("/some/home/path")),
                    venv: true,
                    direnv: Some(false),
                    nixshell: Some(true),
                },
            )
        });
        let expected = expect(
            &result,
            [
                "[31;40m",
                " ",
                Symbol::Error.str(),
                " ",
                "[35m",
                Symbol::Jobs.str(),
                " ",
                "[33m",
                Symbol::Package.str(),
                " ",
                "[31m",
                Symbol::Direnv.str(),
                " ",
                "[32m",
                Symbol::Python.str(),
                " ",
                "[39m",
                HOST,
                "[;40m",
                " ",
                "~",
                " ",
                tip!(blue),
            ],
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn direnv_inactive() {
        let result = test(|s| {
            render_inner(
                s,
                Some(String::from(HOST)),
                true,
                true,
                &MockEnv {
                    pwd: Some(std::path::PathBuf::from("/some/home/path/")),
                    home: Some(std::path::PathBuf::from("/some/home/path")),
                    venv: false,
                    direnv: Some(false),
                    nixshell: None,
                },
            )
        });
        let expected = expect(
            &result,
            [
                "[31;40m",
                " ",
                Symbol::Error.str(),
                " ",
                "[35m",
                Symbol::Jobs.str(),
                " ",
                "[31m",
                Symbol::Direnv.str(),
                " ",
                "[39m",
                HOST,
                "[;40m",
                " ",
                "~",
                " ",
                tip!(blue),
            ],
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn direnv_active() {
        let result = test(|s| {
            render_inner(
                s,
                Some(String::from(HOST)),
                true,
                true,
                &MockEnv {
                    pwd: Some(std::path::PathBuf::from("/some/home/path/")),
                    home: Some(std::path::PathBuf::from("/some/home/path")),
                    venv: false,
                    direnv: Some(true),
                    nixshell: Some(false),
                },
            )
        });
        let expected = expect(
            &result,
            [
                "[31;40m",
                " ",
                Symbol::Error.str(),
                " ",
                "[35m",
                Symbol::Jobs.str(),
                " ",
                "[32m",
                Symbol::Direnv.str(),
                " ",
                "[39m",
                HOST,
                "[;40m",
                " ",
                "~",
                " ",
                tip!(blue),
            ],
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn nixshell_generic() {
        let result = test(|s| {
            render_inner(
                s,
                Some(String::from(HOST)),
                true,
                true,
                &MockEnv {
                    pwd: Some(std::path::PathBuf::from("/some/home/path/")),
                    home: Some(std::path::PathBuf::from("/some/home/path")),
                    venv: false,
                    direnv: None,
                    nixshell: Some(false),
                },
            )
        });
        let expected = expect(
            &result,
            [
                "[31;40m",
                " ",
                Symbol::Error.str(),
                " ",
                "[35m",
                Symbol::Jobs.str(),
                " ",
                "[33m",
                Symbol::Package.str(),
                " ",
                "[39m",
                HOST,
                "[;40m",
                " ",
                "~",
                " ",
                tip!(blue),
            ],
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn nixshell_package() {
        let result = test(|s| {
            render_inner(
                s,
                Some(String::from(HOST)),
                true,
                true,
                &MockEnv {
                    pwd: Some(std::path::PathBuf::from("/some/home/path/")),
                    home: Some(std::path::PathBuf::from("/some/home/path")),
                    venv: false,
                    direnv: None,
                    nixshell: Some(true),
                },
            )
        });
        let expected = expect(
            &result,
            [
                "[31;40m",
                " ",
                Symbol::Error.str(),
                " ",
                "[35m",
                Symbol::Jobs.str(),
                " ",
                "[33m",
                Symbol::Package.str(),
                " ",
                "[39m",
                HOST,
                "[;40m",
                " ",
                "~",
                " ",
                tip!(blue),
            ],
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn nixshell_generic_direnv_active() {
        let result = test(|s| {
            render_inner(
                s,
                Some(String::from(HOST)),
                true,
                true,
                &MockEnv {
                    pwd: Some(std::path::PathBuf::from("/some/home/path/")),
                    home: Some(std::path::PathBuf::from("/some/home/path")),
                    venv: false,
                    direnv: Some(true),
                    nixshell: Some(false),
                },
            )
        });
        let expected = expect(
            &result,
            [
                "[31;40m",
                " ",
                Symbol::Error.str(),
                " ",
                "[35m",
                Symbol::Jobs.str(),
                " ",
                "[32m",
                Symbol::Direnv.str(),
                " ",
                "[39m",
                HOST,
                "[;40m",
                " ",
                "~",
                " ",
                tip!(blue),
            ],
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn nixshell_generic_direnv_inactive() {
        let result = test(|s| {
            render_inner(
                s,
                Some(String::from(HOST)),
                true,
                true,
                &MockEnv {
                    pwd: Some(std::path::PathBuf::from("/some/home/path/")),
                    home: Some(std::path::PathBuf::from("/some/home/path")),
                    venv: false,
                    direnv: Some(false),
                    nixshell: Some(false),
                },
            )
        });
        let expected = expect(
            &result,
            [
                "[31;40m",
                " ",
                Symbol::Error.str(),
                " ",
                "[35m",
                Symbol::Jobs.str(),
                " ",
                "[33m",
                Symbol::Package.str(),
                " ",
                "[31m",
                Symbol::Direnv.str(),
                " ",
                "[39m",
                HOST,
                "[;40m",
                " ",
                "~",
                " ",
                tip!(blue),
            ],
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn nixshell_package_direnv_active() {
        let result = test(|s| {
            render_inner(
                s,
                Some(String::from(HOST)),
                true,
                true,
                &MockEnv {
                    pwd: Some(std::path::PathBuf::from("/some/home/path/")),
                    home: Some(std::path::PathBuf::from("/some/home/path")),
                    venv: false,
                    direnv: Some(true),
                    nixshell: Some(true),
                },
            )
        });
        let expected = expect(
            &result,
            [
                "[31;40m",
                " ",
                Symbol::Error.str(),
                " ",
                "[35m",
                Symbol::Jobs.str(),
                " ",
                "[33m",
                Symbol::Package.str(),
                " ",
                "[32m",
                Symbol::Direnv.str(),
                " ",
                "[39m",
                HOST,
                "[;40m",
                " ",
                "~",
                " ",
                tip!(blue),
            ],
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn nixshell_package_direnv_inactive() {
        let result = test(|s| {
            render_inner(
                s,
                Some(String::from(HOST)),
                true,
                true,
                &MockEnv {
                    pwd: Some(std::path::PathBuf::from("/some/home/path/")),
                    home: Some(std::path::PathBuf::from("/some/home/path")),
                    venv: false,
                    direnv: Some(false),
                    nixshell: Some(true),
                },
            )
        });
        let expected = expect(
            &result,
            [
                "[31;40m",
                " ",
                Symbol::Error.str(),
                " ",
                "[35m",
                Symbol::Jobs.str(),
                " ",
                "[33m",
                Symbol::Package.str(),
                " ",
                "[31m",
                Symbol::Direnv.str(),
                " ",
                "[39m",
                HOST,
                "[;40m",
                " ",
                "~",
                " ",
                tip!(blue),
            ],
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn git_sync_clean() {
        let result = test_from(Color::Reset, Color::Black, |mut s| {
            render_git(&mut s, git::Repo::Clean(git::Sync::Behind))
        });
        let expected = expect(
            &result,
            [
                "[31m",
                Symbol::Branch.str(),
                tip!(green).strip_suffix("[m ").unwrap(),
            ],
        );
        assert_eq!(result, expected);
        let result = test_from(Color::Reset, Color::Black, |mut s| {
            render_git(&mut s, git::Repo::Clean(git::Sync::Ahead))
        });
        let expected = expect(
            &result,
            [
                "[33m",
                Symbol::Branch.str(),
                tip!(green).strip_suffix("[m ").unwrap(),
            ],
        );
        assert_eq!(result, expected);
        let result = test_from(Color::Reset, Color::Black, |mut s| {
            render_git(&mut s, git::Repo::Clean(git::Sync::Diverged))
        });
        let expected = expect(
            &result,
            [
                "[35m",
                Symbol::Branch.str(),
                tip!(green).strip_suffix("[m ").unwrap(),
            ],
        );
        assert_eq!(result, expected);
        let result = test_from(Color::Reset, Color::Black, |mut s| {
            render_git(&mut s, git::Repo::Clean(git::Sync::UpToDate))
        });
        let expected = expect(
            &result,
            [
                Symbol::Branch.str(),
                tip!(green).strip_suffix("[m ").unwrap(),
            ],
        );
        assert_eq!(result, expected);
        let result = test_from(Color::Reset, Color::Black, |mut s| {
            render_git(&mut s, git::Repo::Clean(git::Sync::Local))
        });
        let expected = expect(
            &result,
            [
                "[34m",
                Symbol::Branch.str(),
                tip!(green).strip_suffix("[m ").unwrap(),
            ],
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn git_sync_dirty() {
        let result = test_from(Color::Reset, Color::Black, |mut s| {
            render_git(&mut s, git::Repo::Dirty(git::Sync::Behind))
        });
        let expected = expect(
            &result,
            [
                "[31m",
                Symbol::Branch.str(),
                tip!(yellow).strip_suffix("[m ").unwrap(),
            ],
        );
        assert_eq!(result, expected);
        let result = test_from(Color::Reset, Color::Black, |mut s| {
            render_git(&mut s, git::Repo::Dirty(git::Sync::Ahead))
        });
        let expected = expect(
            &result,
            [
                "[33m",
                Symbol::Branch.str(),
                tip!(yellow).strip_suffix("[m ").unwrap(),
            ],
        );
        assert_eq!(result, expected);
        let result = test_from(Color::Reset, Color::Black, |mut s| {
            render_git(&mut s, git::Repo::Dirty(git::Sync::Diverged))
        });
        let expected = expect(
            &result,
            [
                "[35m",
                Symbol::Branch.str(),
                tip!(yellow).strip_suffix("[m ").unwrap(),
            ],
        );
        assert_eq!(result, expected);
        let result = test_from(Color::Reset, Color::Black, |mut s| {
            render_git(&mut s, git::Repo::Dirty(git::Sync::UpToDate))
        });
        let expected = expect(
            &result,
            [
                Symbol::Branch.str(),
                tip!(yellow).strip_suffix("[m ").unwrap(),
            ],
        );
        assert_eq!(result, expected);
        let result = test_from(Color::Reset, Color::Black, |mut s| {
            render_git(&mut s, git::Repo::Dirty(git::Sync::Local))
        });
        let expected = expect(
            &result,
            [
                "[34m",
                Symbol::Branch.str(),
                tip!(yellow).strip_suffix("[m ").unwrap(),
            ],
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn git_status() {
        let result = test_from(Color::Reset, Color::Black, |mut s| {
            render_git(&mut s, git::Repo::None)
        });
        let expected = expect(&result, [tip!(blue).strip_suffix("[m ").unwrap()]);
        assert_eq!(result, expected);
        let result = test_from(Color::Reset, Color::Black, |mut s| {
            render_git(&mut s, git::Repo::Clean(git::Sync::UpToDate))
        });
        let expected = expect(
            &result,
            [
                Symbol::Branch.str(),
                tip!(green).strip_suffix("[m ").unwrap(),
            ],
        );
        assert_eq!(result, expected);
        let result = test_from(Color::Reset, Color::Black, |mut s| {
            render_git(&mut s, git::Repo::Dirty(git::Sync::UpToDate))
        });
        let expected = expect(
            &result,
            [
                Symbol::Branch.str(),
                tip!(yellow).strip_suffix("[m ").unwrap(),
            ],
        );
        assert_eq!(result, expected);
        let result = test_from(Color::Reset, Color::Black, |mut s| {
            render_git(&mut s, git::Repo::Detached)
        });
        let expected = expect(
            &result,
            [
                Symbol::Branch.str(),
                tip!(magenta).strip_suffix("[m ").unwrap(),
            ],
        );
        assert_eq!(result, expected);
        let result = test_from(Color::Reset, Color::Black, |mut s| {
            render_git(&mut s, git::Repo::Pending)
        });
        let expected = expect(
            &result,
            [Symbol::Warn.str(), tip!(cyan).strip_suffix("[m ").unwrap()],
        );
        assert_eq!(result, expected);
        let result = test_from(Color::Reset, Color::Black, |mut s| {
            render_git(&mut s, git::Repo::Untracked)
        });
        let expected = expect(
            &result,
            [
                Symbol::Branch.str(),
                tip!(cyan).strip_suffix("[m ").unwrap(),
            ],
        );
        assert_eq!(result, expected);
        let result = test_from(Color::Reset, Color::Black, |mut s| {
            render_git(&mut s, git::Repo::Error)
        });
        let expected = expect(&result, [tip!(red).strip_suffix("[m ").unwrap()]);
        assert_eq!(result, expected);
    }
}
