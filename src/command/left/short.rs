use crate::Result;
use crate::git::short as git;
use crate::print::{Color, Printer, Symbol};

macro_rules! chevron {
    ($printer: ident, $color: ident) => {
        $printer
            .div(crate::print::Div::ChevronLeft, crate::print::Color::$color)?
            .div(crate::print::Div::ChevronLeft, crate::print::Color::Reset)?
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
        printer
            .fg(Color::Reset)
            .txt(&host)?
            .fg(Color::Reset)
            .bg(Color::Black)
            .gap()?;
    }

    let pwd = enver.pwd();

    if let Some(ref pwd) = pwd {
        printer.txt_gap(pwd_string(pwd, enver))?;
        render_git(&mut printer, git::parse(pwd))?;
    } else {
        chevron!(printer, Blue);
    }

    printer.fg(Color::Reset).bg(Color::Reset).gap()?.flush()
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
            printer.txt(Symbol::Warn)?;
            chevron!(printer, Cyan)
        }
        git::Repo::Pending => {
            printer.txt(Symbol::Branch)?;
            chevron!(printer, Cyan)
        }
        git::Repo::Untracked => {
            printer.txt(Symbol::Branch)?;
            chevron!(printer, Magenta)
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
    use crate::test;

    macro_rules! chevron {
        ($color: expr) => {
            concat!(
                style!(fg = color!(black), bg = $color, symbol!(div)),
                style!(reset to fg = $color, symbol!(div)),
            )
        };
    }

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
        let expected = concat!(
            style!(reset to bg = color!(black)),
            " ",
            // Missing statuses
            // Missing HOST
            // Missing PWD
            chevron!(color!(blue)),
            style!(reset),
            " "
        );
        println!("{result}");
        println!("{expected}");
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
        let expected = concat!(
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
        );
        println!("{result}");
        println!("{expected}");
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
        let expected = concat!(
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
        );
        println!("{result}");
        println!("{expected}");
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
        let expected = concat!(
            style!(reset to bg = color!(black)),
            " ",
            // Missing statuses
            // Missing HOST
            "~",
            " ",
            chevron!(color!(blue)),
            style!(reset),
            " "
        );
        println!("{result}");
        println!("{expected}");
        assert_eq!(result, expected);
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
                    nixshell: Some(true),
                },
            )
        });
        let expected = concat!(
            style!(reset to bg = color!(black)),
            " ",
            style!(fg = color!(red), symbol!(error)),
            " ",
            style!(fg = color!(cyan), symbol!(jobs)),
            " ",
            style!(fg = color!(yellow), symbol!(pkg)),
            " ",
            style!(fg = color!(red), symbol!(direnv)),
            " ",
            style!(fg = color!(green), symbol!(python)),
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
        );
        println!("{result}");
        println!("{expected}");
        assert_eq!(result, expected);
    }

    #[test]
    fn direnv_inactive() {
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
                    nixshell: None,
                },
            )
        });
        let expected = concat!(
            style!(reset to bg = color!(black)),
            " ",
            style!(fg = color!(red), symbol!(error)),
            " ",
            style!(fg = color!(cyan), symbol!(jobs)),
            " ",
            style!(fg = color!(red), symbol!(direnv)),
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
        );
        println!("{result}");
        println!("{expected}");
        assert_eq!(result, expected);
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
                    nixshell: Some(false),
                },
            )
        });
        let expected = concat!(
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
        );
        println!("{result}");
        println!("{expected}");
        assert_eq!(result, expected);
    }

    #[test]
    fn nixshell_generic() {
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
                    direnv: None,
                    nixshell: Some(false),
                },
            )
        });
        let expected = concat!(
            style!(reset to bg = color!(black)),
            " ",
            style!(fg = color!(red), symbol!(error)),
            " ",
            style!(fg = color!(cyan), symbol!(jobs)),
            " ",
            style!(fg = color!(yellow), symbol!(pkg)),
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
        );
        println!("{result}");
        println!("{expected}");
        assert_eq!(result, expected);
    }

    #[test]
    fn nixshell_package() {
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
                    direnv: None,
                    nixshell: Some(true),
                },
            )
        });
        let expected = concat!(
            style!(reset to bg = color!(black)),
            " ",
            style!(fg = color!(red), symbol!(error)),
            " ",
            style!(fg = color!(cyan), symbol!(jobs)),
            " ",
            style!(fg = color!(yellow), symbol!(pkg)),
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
        );
        println!("{result}");
        println!("{expected}");
        assert_eq!(result, expected);
    }

    #[test]
    fn nixshell_generic_direnv_active() {
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
                    nixshell: Some(false),
                },
            )
        });
        let expected = concat!(
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
        );
        println!("{result}");
        println!("{expected}");
        assert_eq!(result, expected);
    }

    #[test]
    fn nixshell_generic_direnv_inactive() {
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
                    nixshell: Some(false),
                },
            )
        });
        let expected = concat!(
            style!(reset to bg = color!(black)),
            " ",
            style!(fg = color!(red), symbol!(error)),
            " ",
            style!(fg = color!(cyan), symbol!(jobs)),
            " ",
            style!(fg = color!(yellow), symbol!(pkg)),
            " ",
            style!(fg = color!(red), symbol!(direnv)),
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
        );
        println!("{result}");
        println!("{expected}");
        assert_eq!(result, expected);
    }

    #[test]
    fn nixshell_package_direnv_active() {
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
                    nixshell: Some(true),
                },
            )
        });
        let expected = concat!(
            style!(reset to bg = color!(black)),
            " ",
            style!(fg = color!(red), symbol!(error)),
            " ",
            style!(fg = color!(cyan), symbol!(jobs)),
            " ",
            style!(fg = color!(yellow), symbol!(pkg)),
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
        );
        println!("{result}");
        println!("{expected}");
        assert_eq!(result, expected);
    }

    #[test]
    fn nixshell_package_direnv_inactive() {
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
                    nixshell: Some(true),
                },
            )
        });
        let expected = concat!(
            style!(reset to bg = color!(black)),
            " ",
            style!(fg = color!(red), symbol!(error)),
            " ",
            style!(fg = color!(cyan), symbol!(jobs)),
            " ",
            style!(fg = color!(yellow), symbol!(pkg)),
            " ",
            style!(fg = color!(red), symbol!(direnv)),
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
        );
        println!("{result}");
        println!("{expected}");
        assert_eq!(result, expected);
    }

    #[test]
    fn git_sync_clean() {
        assert_eq!(
            test(|mut s| render_git(&mut s, git::Repo::Clean(git::Sync::Behind))),
            concat!(branch!(color!(red)), chevron!(color!(green)))
        );
        assert_eq!(
            test(|mut s| render_git(&mut s, git::Repo::Clean(git::Sync::Ahead))),
            concat!(branch!(color!(yellow)), chevron!(color!(green)))
        );
        assert_eq!(
            test(|mut s| render_git(&mut s, git::Repo::Clean(git::Sync::Diverged))),
            concat!(branch!(color!(magenta)), chevron!(color!(green)))
        );
        assert_eq!(
            test(|mut s| render_git(&mut s, git::Repo::Clean(git::Sync::UpToDate))),
            concat!(branch!(), chevron!(color!(green)))
        );
        assert_eq!(
            test(|mut s| render_git(&mut s, git::Repo::Clean(git::Sync::Local))),
            concat!(branch!(color!(blue)), chevron!(color!(green)))
        );
    }

    #[test]
    fn git_sync_dirty() {
        assert_eq!(
            test(|mut s| render_git(&mut s, git::Repo::Dirty(git::Sync::Behind))),
            concat!(branch!(color!(red)), chevron!(color!(yellow)))
        );
        assert_eq!(
            test(|mut s| render_git(&mut s, git::Repo::Dirty(git::Sync::Ahead))),
            concat!(branch!(color!(yellow)), chevron!(color!(yellow)))
        );
        assert_eq!(
            test(|mut s| render_git(&mut s, git::Repo::Dirty(git::Sync::Diverged))),
            concat!(branch!(color!(magenta)), chevron!(color!(yellow)))
        );
        assert_eq!(
            test(|mut s| render_git(&mut s, git::Repo::Dirty(git::Sync::UpToDate))),
            concat!(branch!(), chevron!(color!(yellow)))
        );
        assert_eq!(
            test(|mut s| render_git(&mut s, git::Repo::Dirty(git::Sync::Local))),
            concat!(branch!(color!(blue)), chevron!(color!(yellow)))
        );
    }

    #[test]
    fn git_status() {
        assert_eq!(
            test(|mut s| render_git(&mut s, git::Repo::None)),
            chevron!(color!(blue))
        );
        assert_eq!(
            test(|mut s| render_git(&mut s, git::Repo::Clean(git::Sync::UpToDate))),
            concat!(branch!(), chevron!(color!(green)))
        );
        assert_eq!(
            test(|mut s| render_git(&mut s, git::Repo::Dirty(git::Sync::UpToDate))),
            concat!(branch!(), chevron!(color!(yellow)))
        );
        assert_eq!(
            test(|mut s| render_git(&mut s, git::Repo::Detached)),
            concat!(branch!(), chevron!(color!(magenta)))
        );
        assert_eq!(
            test(|mut s| render_git(&mut s, git::Repo::Pending)),
            concat!(symbol!(warn), chevron!(color!(cyan)))
        );
        assert_eq!(
            test(|mut s| render_git(&mut s, git::Repo::Untracked)),
            concat!(branch!(), chevron!(color!(cyan)))
        );
        assert_eq!(
            test(|mut s| render_git(&mut s, git::Repo::Error)),
            chevron!(color!(red))
        );
    }
}
