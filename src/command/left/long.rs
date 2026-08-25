use crate::Result;
use crate::git::long as git;
use crate::print::{Color, Div, Printer, Symbol};

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
    printer.fg(Color::Reset).bg(Color::Black).txt(" ")?;

    if error {
        printer.fg(Color::Red).txt(Symbol::Error)?.txt(" ")?;
    }

    if jobs {
        printer.fg(Color::Magenta).txt(Symbol::Jobs)?.txt(" ")?;
    }

    let direnv = enver.direnv();
    let nixshell = enver.nixshell();

    if nixshell == Nixshell::Generic && !matches!(direnv, Some((_, true))) {
        printer.fg(Color::Cyan).txt(Symbol::Flake)?.txt(" ")?;
    }

    if let Some(host) = host {
        printer
            .fg(Color::Reset)
            .txt(&host)?
            .bg(Color::Black)
            .txt(" ")?;
    }

    if let Nixshell::Package(pkg) = nixshell {
        printer
            .fg(Color::Black)
            .div(Div::ChevronLeft, Color::Yellow)?
            .txt(" ")?
            .txt(&pkg)?
            .txt(" ")?;
    }

    if let Some((direnv, active)) = direnv {
        if active {
            printer
                .fg(Color::Black)
                .div(Div::ChevronLeft, Color::Green)?
                .txt(" ")?;
        } else {
            printer
                .fg(Color::Black)
                .div(Div::ChevronLeft, Color::Red)?
                .txt(" ")?;
        }
        if let Some(inner) = direnv.rsplit(std::path::MAIN_SEPARATOR).next() {
            printer.txt(inner)?.txt(" ")?;
        } else {
            printer.txt(&direnv)?.txt(" ")?;
        }
    }

    if let Some(venv) = enver.venv() {
        printer
            .fg(Color::Black)
            .div(Div::ChevronLeft, Color::Cyan)?
            .txt(" ")?;
        if let Some(inner) = venv.rsplit(std::path::MAIN_SEPARATOR).next() {
            printer.txt(inner)?.txt(" ")?;
        } else {
            printer.txt(&venv)?.txt(" ")?;
        }
    }

    let pwd = enver.pwd();

    printer
        .fg(Color::Black)
        .div(Div::ChevronLeft, Color::Blue)?
        .txt(" ")?;
    if let Some(ref pwd) = pwd {
        if let Some(pwd) = pwd.to_str() {
            if let Some(pwd) = enver.home().and_then(|home| pwd.strip_prefix(&home)) {
                printer.txt("~")?.txt(pwd)?.txt(" ")?;
            } else {
                printer.txt(pwd)?.txt(" ")?;
            }
        }
        render_git(&mut printer, git::parse(pwd))?;
    }

    printer
        .fg(Color::Reset)
        .div(Div::ChevronLeft, Color::Reset)?
        .txt(" ")?
        .flush()
}

fn render_git<P>(printer: &mut P, repo: git::Repo) -> Result<Option<&'static str>>
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
                out.div(&mut last, color!(green), color!(black))?;
                write!(out, concat!(symbol!(branch), "{head}"), head = head)?;
            } else {
                render_changes(printer, changes)?;
                if !matches!(
                    sync,
                    git::Sync::Tracked {
                        ahead: 0,
                        behind: 0
                    }
                ) {
                    out.div(&mut last, color!(black), color!(reset))?;
                    write!(out, symbol!(div thin))?;
                    render_sync(printer, sync)?;
                }
                out.div(&mut last, color!(yellow), color!(black))?;
                write!(out, concat!(symbol!(branch), "{head}"), head = head)?;
            }
        }
        git::Repo::Detached(head, changes) => {
            render_changes(printer, changes)?;
            out.div(&mut last, color!(magenta), color!(black))?;
            write!(out, concat!(symbol!(ref), "{head}"), head = head)?;
        }
        git::Repo::Pending(head, pending, changes) => {
            render_changes(printer, changes)?;
            out.div(&mut last, color!(cyan), color!(black))?;
            write!(
                out,
                concat!(symbol!(branch), "{head} {pending}"),
                head = head,
                pending = pending_symbol(pending),
            )?;
        }
        git::Repo::New(changes) => {
            render_changes(printer, changes)?;
            out.div(&mut last, color!(cyan), color!(black))?;
            write!(out, symbol!(new))?;
        }
    }
    Ok(last)
}

trait Writer {
    fn div(
        &mut self,
        last: &mut Option<&'static str>,
        to: &'static str,
        fg: &'static str,
    ) -> Result;
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
                    concat!(" [3{last};4{to}m", symbol!(div), "[3{fg}m "),
                    last = last,
                    to = to,
                    fg = fg,
                )?;
            }
        } else {
            write!(self, "[3{fg};4{to}m ")?;
        }
        *last = Some(to);
        Ok(())
    }
}

fn render_changes<P>(printer: &mut P, changes: git::Changes) -> Result
where
    P: Printer,
{
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

fn render_sync<P>(printer: &mut P, sync: git::Sync) -> Result
where
    P: Printer,
{
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
    use crate::test;

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
        let expected = concat!(
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
                false,
                false,
                &MockEnv {
                    pwd: Some(std::path::PathBuf::from("/")),
                    ..MockEnv::default()
                },
            )
        });
        let expected = concat!(
            // Missing error
            // Missing jobs
            // Missing venv
            // Missing HOST
            style!(fg = color!(black), bg = color!(blue)),
            " / ",
            style!(fg = color!(blue), bg = color!(reset), symbol!(div)),
            style!(fg = color!(reset)),
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
                    pwd: Some(std::path::PathBuf::from("/some/home/path/further/on")),
                    home: Some(String::from("/some/home/path")),
                    ..MockEnv::default()
                },
            )
        });
        let expected = concat!(
            // Missing error
            // Missing jobs
            // Missing venv
            // Missing HOST
            style!(fg = color!(black), bg = color!(blue)),
            " ~/further/on ",
            style!(fg = color!(blue), bg = color!(reset), symbol!(div)),
            style!(fg = color!(reset)),
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
                    pwd: Some(std::path::PathBuf::from("/some/home/path/further/on")),
                    home: Some(String::from("/some/home/path")),
                    venv: Some(String::from("py")),
                    direnv: Some((String::from("/some/direnv"), false)),
                    nixshell: Nixshell::Package(String::from("pkg1 pkg2")),
                },
            )
        });
        let expected = concat!(
            style!(fg = color!(red), bg = color!(black)),
            " ",
            symbol!(error),
            " ",
            style!(fg = color!(magenta), symbol!(jobs)),
            " ",
            style!(fg = color!(reset), style!(fg = color!(red), "H")),
            style!(reset to bg = color!(black)),
            " ",
            style!(fg = color!(black), bg = color!(yellow), symbol!(div)),
            style!(fg = color!(black)),
            " pkg1 pkg2 ",
            style!(fg = color!(yellow), bg = color!(black), symbol!(div)),
            style!(fg = color!(red)),
            " direnv ",
            style!(fg = color!(black), bg = color!(cyan), symbol!(div)),
            style!(fg = color!(black)),
            " py ",
            style!(fg = color!(cyan), bg = color!(blue), symbol!(div)),
            style!(fg = color!(black)),
            " ~/further/on ",
            style!(fg = color!(blue), bg = color!(reset), symbol!(div)),
            style!(fg = color!(reset)),
            " "
        );
        println!("{result}");
        println!("{expected}");
        assert_eq!(result, expected);
    }

    #[test]
    fn venv() {
        let result = test(|s| {
            render_inner(
                s,
                Some(String::from("[31mH")),
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
        let expected = concat!(
            style!(fg = color!(red), bg = color!(black)),
            " ",
            symbol!(error),
            " ",
            style!(fg = color!(magenta), symbol!(jobs)),
            " ",
            style!(fg = color!(reset), style!(fg = color!(red), "H")),
            style!(reset to bg = color!(black)),
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
                    pwd: Some(std::path::PathBuf::from("/some/home/path/further/on")),
                    home: Some(String::from("/some/home/path")),
                    venv: None,
                    direnv: Some((String::from("/some/direnv"), false)),
                    nixshell: Nixshell::None,
                },
            )
        });
        let expected = concat!(
            style!(fg = color!(red), bg = color!(black)),
            " ",
            symbol!(error),
            " ",
            style!(fg = color!(magenta), symbol!(jobs)),
            " ",
            style!(fg = color!(reset), style!(fg = color!(red), "H")),
            style!(reset to bg = color!(black)),
            " ",
            style!(fg = color!(red)),
            "direnv ",
            style!(fg = color!(black), bg = color!(blue), symbol!(div)),
            style!(fg = color!(black)),
            " ~/further/on ",
            style!(fg = color!(blue), bg = color!(reset), symbol!(div)),
            style!(fg = color!(reset)),
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
                    pwd: Some(std::path::PathBuf::from("/some/home/path/further/on")),
                    home: Some(String::from("/some/home/path")),
                    venv: None,
                    direnv: Some((String::from("/some/direnv"), true)),
                    nixshell: Nixshell::Generic,
                },
            )
        });
        let expected = concat!(
            style!(fg = color!(red), bg = color!(black)),
            " ",
            symbol!(error),
            " ",
            style!(fg = color!(magenta), symbol!(jobs)),
            " ",
            style!(fg = color!(reset), style!(fg = color!(red), "H")),
            style!(reset to bg = color!(black)),
            " ",
            style!(fg = color!(green)),
            "direnv ",
            style!(fg = color!(black), bg = color!(blue), symbol!(div)),
            style!(fg = color!(black)),
            " ~/further/on ",
            style!(fg = color!(blue), bg = color!(reset), symbol!(div)),
            style!(fg = color!(reset)),
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
                    pwd: Some(std::path::PathBuf::from("/some/home/path/further/on")),
                    home: Some(String::from("/some/home/path")),
                    venv: None,
                    direnv: None,
                    nixshell: Nixshell::Generic,
                },
            )
        });
        let expected = concat!(
            style!(fg = color!(red), bg = color!(black)),
            " ",
            symbol!(error),
            " ",
            style!(fg = color!(magenta), symbol!(jobs)),
            " ",
            style!(fg = color!(cyan), symbol!(flake)),
            " ",
            style!(fg = color!(reset), style!(fg = color!(red), "H")),
            style!(reset to bg = color!(black)),
            " ",
            style!(fg = color!(black), bg = color!(blue), symbol!(div)),
            style!(fg = color!(black)),
            " ~/further/on ",
            style!(fg = color!(blue), bg = color!(reset), symbol!(div)),
            style!(fg = color!(reset)),
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
                    pwd: Some(std::path::PathBuf::from("/some/home/path/further/on")),
                    home: Some(String::from("/some/home/path")),
                    venv: None,
                    direnv: None,
                    nixshell: Nixshell::Package(String::from("pkg1 pkg2")),
                },
            )
        });
        let expected = concat!(
            style!(fg = color!(red), bg = color!(black)),
            " ",
            symbol!(error),
            " ",
            style!(fg = color!(magenta), symbol!(jobs)),
            " ",
            style!(fg = color!(reset), style!(fg = color!(red), "H")),
            style!(reset to bg = color!(black)),
            " ",
            style!(fg = color!(black), bg = color!(yellow), symbol!(div)),
            style!(fg = color!(black)),
            " pkg1 pkg2 ",
            style!(fg = color!(yellow), bg = color!(blue), symbol!(div)),
            style!(fg = color!(black)),
            " ~/further/on ",
            style!(fg = color!(blue), bg = color!(reset), symbol!(div)),
            style!(fg = color!(reset)),
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
                    pwd: Some(std::path::PathBuf::from("/some/home/path/further/on")),
                    home: Some(String::from("/some/home/path")),
                    venv: None,
                    direnv: Some((String::from("/some/direnv"), true)),
                    nixshell: Nixshell::Generic,
                },
            )
        });
        let expected = concat!(
            style!(fg = color!(red), bg = color!(black)),
            " ",
            symbol!(error),
            " ",
            style!(fg = color!(magenta), symbol!(jobs)),
            " ",
            style!(fg = color!(reset), style!(fg = color!(red), "H")),
            style!(reset to bg = color!(black)),
            " ",
            style!(fg = color!(green)),
            "direnv ",
            style!(fg = color!(black), bg = color!(blue), symbol!(div)),
            style!(fg = color!(black)),
            " ~/further/on ",
            style!(fg = color!(blue), bg = color!(reset), symbol!(div)),
            style!(fg = color!(reset)),
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
                    pwd: Some(std::path::PathBuf::from("/some/home/path/further/on")),
                    home: Some(String::from("/some/home/path")),
                    venv: None,
                    direnv: Some((String::from("/some/direnv"), false)),
                    nixshell: Nixshell::Generic,
                },
            )
        });
        let expected = concat!(
            style!(fg = color!(red), bg = color!(black)),
            " ",
            symbol!(error),
            " ",
            style!(fg = color!(magenta), symbol!(jobs)),
            " ",
            style!(fg = color!(cyan), symbol!(flake)),
            " ",
            style!(fg = color!(reset), style!(fg = color!(red), "H")),
            style!(reset to bg = color!(black)),
            " ",
            style!(fg = color!(red)),
            "direnv ",
            style!(fg = color!(black), bg = color!(blue), symbol!(div)),
            style!(fg = color!(black)),
            " ~/further/on ",
            style!(fg = color!(blue), bg = color!(reset), symbol!(div)),
            style!(fg = color!(reset)),
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
                    pwd: Some(std::path::PathBuf::from("/some/home/path/further/on")),
                    home: Some(String::from("/some/home/path")),
                    venv: None,
                    direnv: Some((String::from("/some/direnv"), true)),
                    nixshell: Nixshell::Package(String::from("pkg1 pkg2")),
                },
            )
        });
        let expected = concat!(
            style!(fg = color!(red), bg = color!(black)),
            " ",
            symbol!(error),
            " ",
            style!(fg = color!(magenta), symbol!(jobs)),
            " ",
            style!(fg = color!(reset), style!(fg = color!(red), "H")),
            style!(reset to bg = color!(black)),
            " ",
            style!(fg = color!(black), bg = color!(yellow), symbol!(div)),
            style!(fg = color!(black)),
            " pkg1 pkg2 ",
            style!(fg = color!(yellow), bg = color!(black), symbol!(div)),
            style!(fg = color!(green)),
            " direnv ",
            style!(fg = color!(black), bg = color!(blue), symbol!(div)),
            style!(fg = color!(black)),
            " ~/further/on ",
            style!(fg = color!(blue), bg = color!(reset), symbol!(div)),
            style!(fg = color!(reset)),
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
                    pwd: Some(std::path::PathBuf::from("/some/home/path/further/on")),
                    home: Some(String::from("/some/home/path")),
                    venv: None,
                    direnv: Some((String::from("/some/direnv"), false)),
                    nixshell: Nixshell::Package(String::from("pkg1 pkg2")),
                },
            )
        });
        let expected = concat!(
            style!(fg = color!(red), bg = color!(black)),
            " ",
            symbol!(error),
            " ",
            style!(fg = color!(magenta), symbol!(jobs)),
            " ",
            style!(fg = color!(reset), style!(fg = color!(red), "H")),
            style!(reset to bg = color!(black)),
            " ",
            style!(fg = color!(black), bg = color!(yellow), symbol!(div)),
            style!(fg = color!(black)),
            " pkg1 pkg2 ",
            style!(fg = color!(yellow), bg = color!(black), symbol!(div)),
            style!(fg = color!(red)),
            " direnv ",
            style!(fg = color!(black), bg = color!(blue), symbol!(div)),
            style!(fg = color!(black)),
            " ~/further/on ",
            style!(fg = color!(blue), bg = color!(reset), symbol!(div)),
            style!(fg = color!(reset)),
            " "
        );
        println!("{result}");
        println!("{expected}");
        assert_eq!(result, expected);
    }
}
