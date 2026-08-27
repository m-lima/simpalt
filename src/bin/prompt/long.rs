use crate::Result;
use simpalt::{
    git::long as git,
    print::{Color, Div, Printer, Symbol},
};

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
    if error {
        printer
            .bg(Color::Black)
            .fg(Color::Red)
            .txt_gap(Symbol::Error)?;
    }

    if jobs {
        printer
            .bg(Color::Black)
            .fg(Color::Magenta)
            .txt_gap(Symbol::Jobs)?;
    }

    let direnv = enver.direnv();
    let nixshell = enver.nixshell();

    if nixshell == Nixshell::Generic && !matches!(direnv, Some((_, true))) {
        printer
            .bg(Color::Black)
            .fg(Color::Cyan)
            .txt_gap(Symbol::Flake)?;
    }

    if let Some(host) = host {
        printer
            .bg(Color::Black)
            .fg(Color::Reset)
            .gap()?
            .txt(&host)?
            .invalidate()
            .fg(Color::Reset)
            .bg(Color::Black)
            .gap()?;
    }

    if let Nixshell::Package(pkg) = nixshell {
        printer
            .fg(Color::Black)
            .div(Div::ChevronLeft, Color::Yellow)?
            .txt_gap(&pkg)?;
    }

    if let Some((direnv, active)) = direnv {
        if active {
            printer.fg(Color::Green);
        } else {
            printer.fg(Color::Red);
        }

        printer.div(Div::ChevronLeft, Color::Black)?;

        if let Some(inner) = direnv.rsplit(std::path::MAIN_SEPARATOR).next() {
            printer.txt_gap(inner)?;
        } else {
            printer.txt_gap(&direnv)?;
        }
    }

    if let Some(venv) = enver.venv() {
        printer
            .fg(Color::Black)
            .div(Div::ChevronLeft, Color::Cyan)?;

        if let Some(inner) = venv.rsplit(std::path::MAIN_SEPARATOR).next() {
            printer.txt_gap(inner)?;
        } else {
            printer.txt_gap(&venv)?;
        }
    }

    let pwd = enver.pwd();

    if let Some(ref pwd) = pwd {
        if let Some(pwd) = pwd.to_str() {
            printer
                .fg(Color::Black)
                .div(Div::ChevronLeft, Color::Blue)?;

            if let Some(pwd) = enver.home().and_then(|home| pwd.strip_prefix(&home)) {
                printer.gap()?.txt("~")?.txt(pwd)?.gap()?;
            } else {
                printer.txt_gap(pwd)?;
            }
        }
        render_git(&mut printer, git::parse(pwd))?;
    } else {
        printer.bg(Color::Blue).gap()?;
    }

    printer
        .fg(Color::Reset)
        .div(Div::ChevronLeft, Color::Reset)?
        .gap()?
        .flush()
}

fn render_git<P>(printer: &mut P, repo: git::Repo) -> Result
where
    P: Printer,
{
    match repo {
        git::Repo::None => {}
        git::Repo::Error => {
            printer
                .fg(Color::Black)
                .div(Div::ChevronLeft, Color::Red)?
                .txt(Symbol::Warn)?;
        }
        git::Repo::Regular(head, sync, changes) => {
            if changes.clean() {
                render_sync(printer, sync)?;
                printer
                    .fg(Color::Black)
                    .div(Div::ChevronLeft, Color::Green)?
                    .gap()?
                    .txt(Symbol::Branch)?
                    .txt(&head)?
                    .gap()?;
            } else {
                render_changes(printer, changes)?;
                if !matches!(
                    sync,
                    git::Sync::Tracked {
                        ahead: 0,
                        behind: 0
                    }
                ) {
                    printer
                        .fg(Color::Reset)
                        .div(Div::ChevronLeft, Color::Black)?
                        .txt_gap(Symbol::SlantTop)?;
                    render_sync(printer, sync)?;
                }
                printer
                    .fg(Color::Black)
                    .div(Div::ChevronLeft, Color::Yellow)?
                    .gap()?
                    .txt(Symbol::Branch)?
                    .txt(&head)?
                    .gap()?;
            }
        }
        git::Repo::Detached(head, changes) => {
            render_changes(printer, changes)?;
            printer
                .fg(Color::Black)
                .div(Div::ChevronLeft, Color::Magenta)?
                .gap()?
                .txt(Symbol::Ref)?
                .txt(&head)?
                .gap()?;
        }
        git::Repo::Pending(head, pending, changes) => {
            render_changes(printer, changes)?;
            printer
                .fg(Color::Black)
                .div(Div::ChevronLeft, Color::Cyan)?
                .gap()?
                .txt(Symbol::Branch)?
                .txt(&head)?
                .txt_gap(pending_symbol(pending))?;
        }
        git::Repo::New(changes) => {
            render_changes(printer, changes)?;
            printer
                .fg(Color::Black)
                .div(Div::ChevronLeft, Color::Cyan)?
                .txt_gap(Symbol::New)?;
        }
    }
    Ok(())
}

fn render_changes<P>(printer: &mut P, changes: git::Changes) -> Result
where
    P: Printer,
{
    if changes.added > 0 {
        printer.div(Div::ChevronLeft, Color::Black)?;
        printer
            .fg(Color::Green)
            .gap()?
            .txt("+")?
            .txt(changes.added)?
            .gap()?;
    }

    if changes.removed > 0 {
        printer.div(Div::ChevronLeft, Color::Black)?;
        printer
            .fg(Color::Red)
            .gap()?
            .txt("-")?
            .txt(changes.removed)?
            .gap()?;
    }

    if changes.modified > 0 {
        printer.div(Div::ChevronLeft, Color::Black)?;
        printer
            .fg(Color::Blue)
            .gap()?
            .txt("~")?
            .txt(changes.modified)?
            .gap()?;
    }

    if changes.conflicted > 0 {
        printer.div(Div::ChevronLeft, Color::Black)?;
        printer
            .fg(Color::Magenta)
            .gap()?
            .txt("!")?
            .txt(changes.modified)?
            .gap()?;
    }
    Ok(())
}

fn render_sync<P>(printer: &mut P, sync: git::Sync) -> Result
where
    P: Printer,
{
    printer.div(Div::ChevronLeft, Color::Black)?;

    match sync {
        git::Sync::Local => {
            printer
                .fg(Color::Cyan)
                .txt_gap(Symbol::Local)?
                .txt_gap("local")?;
        }
        git::Sync::Gone => {
            printer
                .fg(Color::Magenta)
                .txt_gap(Symbol::Gone)?
                .txt_gap("gone")?;
        }
        git::Sync::Tracked { ahead, behind } => {
            if ahead > 0 {
                printer
                    .fg(Color::Yellow)
                    .txt_gap(Symbol::Ahead)?
                    .txt_gap(ahead)?;
            }
            if behind > 0 {
                printer
                    .fg(Color::Red)
                    .txt_gap(Symbol::Behind)?
                    .txt_gap(behind)?;
            }
        }
    }

    Ok(())
}

const fn pending_symbol(pending: git::Pending) -> Symbol {
    match pending {
        git::Pending::Merge => Symbol::Merge,
        git::Pending::Revert => Symbol::Revert,
        git::Pending::Cherry => Symbol::Cherry,
        git::Pending::Bisect => Symbol::Bisect,
        git::Pending::Rebase => Symbol::Rebase,
        git::Pending::Mailbox => Symbol::Mailbox,
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
enum Nixshell {
    #[default]
    None,
    Generic,
    Package(String),
}

trait EnvFetcher {
    fn pwd(&self) -> Option<std::path::PathBuf>;
    fn home(&self) -> Option<String>;
    fn venv(&self) -> Option<String>;
    fn direnv(&self) -> Option<(String, bool)>;
    fn nixshell(&self) -> Nixshell;
}

#[derive(Copy, Clone)]
struct SysEnv;

impl EnvFetcher for SysEnv {
    fn pwd(&self) -> Option<std::path::PathBuf> {
        std::env::current_dir()
            .ok()
            .or_else(|| std::env::var_os("PWD").map(std::path::PathBuf::from))
    }

    fn home(&self) -> Option<String> {
        std::env::var("HOME").ok()
    }

    fn venv(&self) -> Option<String> {
        std::env::var("VIRTUAL_ENV").ok()
    }

    fn direnv(&self) -> Option<(String, bool)> {
        std::env::var("DIRENV_DIR")
            .ok()
            .map(|d| (d, super::direnv::is_active().unwrap_or(false)))
    }

    fn nixshell(&self) -> Nixshell {
        match std::env::var("NIX_SHELL") {
            Ok(pkg) => Nixshell::Package(pkg),
            Err(_) => match std::env::var("IN_NIX_SHELL") {
                Ok(_) => Nixshell::Generic,
                Err(_) => Nixshell::None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{expect, test};

    const HOST: &str = "[31mH";

    #[derive(Default)]
    struct MockEnv {
        pwd: Option<std::path::PathBuf>,
        home: Option<String>,
        venv: Option<String>,
        direnv: Option<(String, bool)>,
        nixshell: Nixshell,
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

        fn direnv(&self) -> Option<(String, bool)> {
            self.direnv.clone()
        }

        fn nixshell(&self) -> Nixshell {
            self.nixshell.clone()
        }
    }

    #[test]
    fn all_empty() {
        let result = test(|s| render_inner(s, None, false, false, &MockEnv::default()));
        let expected = expect(
            &result,
            [
                "[;44m",
                " ",
                // Missing error
                // Missing jobs
                // Missing venv
                // Missing HOST
                // Missing PWD
                "[;34m",
                Div::ChevronLeft.str(),
                "[m",
                " ",
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
                false,
                false,
                &MockEnv {
                    pwd: Some(std::path::PathBuf::from("/")),
                    ..MockEnv::default()
                },
            )
        });
        let expected = expect(
            &result,
            [
                "[30;44m",
                // Missing error
                // Missing jobs
                // Missing venv
                // Missing HOST
                " ",
                "/",
                " ",
                "[;34m",
                Div::ChevronLeft.str(),
                "[m",
                " ",
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
                    pwd: Some(std::path::PathBuf::from("/some/home/path/further/on")),
                    home: Some(String::from("/some/home/path")),
                    ..MockEnv::default()
                },
            )
        });
        let expected = expect(
            &result,
            [
                "[30;44m",
                // Missing error
                // Missing jobs
                // Missing venv
                // Missing HOST
                " ",
                "~/further/on",
                " ",
                "[;34m",
                Div::ChevronLeft.str(),
                "[m",
                " ",
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
                    pwd: Some(std::path::PathBuf::from("/some/home/path/further/on")),
                    home: Some(String::from("/some/home/path")),
                    venv: Some(String::from("py")),
                    direnv: Some((String::from("/some/direnv"), false)),
                    nixshell: Nixshell::Package(String::from("pkg1 pkg2")),
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
                "[39m",
                HOST,
                "[;40m",
                " ",
                "[30;43m",
                Div::ChevronLeft.str(),
                " ",
                "pkg1",
                " ",
                "pkg2",
                " ",
                "[33;40m",
                Div::ChevronLeft.str(),
                "[31m",
                " ",
                "direnv",
                " ",
                "[30;46m",
                Div::ChevronLeft.str(),
                " ",
                "py",
                " ",
                "[36;44m",
                Div::ChevronLeft.str(),
                "[30m",
                " ",
                "~/further/on",
                " ",
                "[;34m",
                Div::ChevronLeft.str(),
                "[m",
                " ",
            ],
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn venv() {
        let result = test(|s| {
            render_inner(
                s,
                Some(String::from(HOST)),
                true,
                true,
                &MockEnv {
                    pwd: Some(std::path::PathBuf::from("/some/home/path/further/on")),
                    home: Some(String::from("/some/home/path")),
                    venv: Some(String::from("py")),
                    direnv: None,
                    nixshell: Nixshell::None,
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
                "[39m",
                HOST,
                "[;40m",
                " ",
                "[30;46m",
                Div::ChevronLeft.str(),
                " ",
                "py",
                " ",
                "[36;44m",
                Div::ChevronLeft.str(),
                "[30m",
                " ",
                "~/further/on",
                " ",
                "[;34m",
                Div::ChevronLeft.str(),
                "[m",
                " ",
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
                    pwd: Some(std::path::PathBuf::from("/some/home/path/further/on")),
                    home: Some(String::from("/some/home/path")),
                    venv: None,
                    direnv: Some((String::from("/some/direnv"), false)),
                    nixshell: Nixshell::None,
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
                "[39m",
                HOST,
                "[;40m",
                " ",
                "[31m",
                "direnv",
                " ",
                "[30;44m",
                Div::ChevronLeft.str(),
                " ",
                "~/further/on",
                " ",
                "[;34m",
                Div::ChevronLeft.str(),
                "[m",
                " ",
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
                    pwd: Some(std::path::PathBuf::from("/some/home/path/further/on")),
                    home: Some(String::from("/some/home/path")),
                    venv: None,
                    direnv: Some((String::from("/some/direnv"), true)),
                    nixshell: Nixshell::Generic,
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
                "[39m",
                HOST,
                "[;40m",
                " ",
                "[32m",
                "direnv",
                " ",
                "[30;44m",
                Div::ChevronLeft.str(),
                " ",
                "~/further/on",
                " ",
                "[;34m",
                Div::ChevronLeft.str(),
                "[m",
                " ",
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
                    pwd: Some(std::path::PathBuf::from("/some/home/path/further/on")),
                    home: Some(String::from("/some/home/path")),
                    venv: None,
                    direnv: None,
                    nixshell: Nixshell::Generic,
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
                "[36m",
                Symbol::Flake.str(),
                " ",
                "[39m",
                HOST,
                "[;40m",
                " ",
                "[30;44m",
                Div::ChevronLeft.str(),
                " ",
                "~/further/on",
                " ",
                "[;34m",
                Div::ChevronLeft.str(),
                "[m",
                " ",
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
                    pwd: Some(std::path::PathBuf::from("/some/home/path/further/on")),
                    home: Some(String::from("/some/home/path")),
                    venv: None,
                    direnv: None,
                    nixshell: Nixshell::Package(String::from("pkg1 pkg2")),
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
                "[39m",
                HOST,
                "[;40m",
                " ",
                "[30;43m",
                Div::ChevronLeft.str(),
                " ",
                "pkg1",
                " ",
                "pkg2",
                " ",
                "[33;44m",
                Div::ChevronLeft.str(),
                "[30m",
                " ",
                "~/further/on",
                " ",
                "[;34m",
                Div::ChevronLeft.str(),
                "[m",
                " ",
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
                    pwd: Some(std::path::PathBuf::from("/some/home/path/further/on")),
                    home: Some(String::from("/some/home/path")),
                    venv: None,
                    direnv: Some((String::from("/some/direnv"), true)),
                    nixshell: Nixshell::Generic,
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
                "[39m",
                HOST,
                "[;40m",
                " ",
                "[32m",
                "direnv",
                " ",
                "[30;44m",
                Div::ChevronLeft.str(),
                " ",
                "~/further/on",
                " ",
                "[;34m",
                Div::ChevronLeft.str(),
                "[m",
                " ",
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
                    pwd: Some(std::path::PathBuf::from("/some/home/path/further/on")),
                    home: Some(String::from("/some/home/path")),
                    venv: None,
                    direnv: Some((String::from("/some/direnv"), false)),
                    nixshell: Nixshell::Generic,
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
                "[36m",
                Symbol::Flake.str(),
                " ",
                "[39m",
                HOST,
                "[;40m",
                " ",
                "[31m",
                "direnv",
                " ",
                "[30;44m",
                Div::ChevronLeft.str(),
                " ",
                "~/further/on",
                " ",
                "[;34m",
                Div::ChevronLeft.str(),
                "[m",
                " ",
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
                    pwd: Some(std::path::PathBuf::from("/some/home/path/further/on")),
                    home: Some(String::from("/some/home/path")),
                    venv: None,
                    direnv: Some((String::from("/some/direnv"), true)),
                    nixshell: Nixshell::Package(String::from("pkg1 pkg2")),
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
                "[39m",
                HOST,
                "[;40m",
                " ",
                "[30;43m",
                Div::ChevronLeft.str(),
                " ",
                "pkg1",
                " ",
                "pkg2",
                " ",
                "[33;40m",
                Div::ChevronLeft.str(),
                "[32m",
                " ",
                "direnv",
                " ",
                "[30;44m",
                Div::ChevronLeft.str(),
                " ",
                "~/further/on",
                " ",
                "[;34m",
                Div::ChevronLeft.str(),
                "[m",
                " ",
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
                    pwd: Some(std::path::PathBuf::from("/some/home/path/further/on")),
                    home: Some(String::from("/some/home/path")),
                    venv: None,
                    direnv: Some((String::from("/some/direnv"), false)),
                    nixshell: Nixshell::Package(String::from("pkg1 pkg2")),
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
                "[39m",
                HOST,
                "[;40m",
                " ",
                "[30;43m",
                Div::ChevronLeft.str(),
                " ",
                "pkg1",
                " ",
                "pkg2",
                " ",
                "[33;40m",
                Div::ChevronLeft.str(),
                "[31m",
                " ",
                "direnv",
                " ",
                "[30;44m",
                Div::ChevronLeft.str(),
                " ",
                "~/further/on",
                " ",
                "[;34m",
                Div::ChevronLeft.str(),
                "[m",
                " ",
            ],
        );
        assert_eq!(result, expected);
    }
}
