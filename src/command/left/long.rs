use crate::Result;
use crate::git::long as git;

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
    let mut last = None;

    if error {
        out.div(&mut last, color!(black), color!(red))?;
        write!(out, symbol!(error))?;
    }

    if jobs {
        out.div(&mut last, color!(black), color!(cyan))?;
        write!(out, symbol!(jobs))?;
    }

    if let Some(host) = host {
        out.div(&mut last, color!(black), color!(reset))?;
        write!(out, "{host}")?;
        write!(out, style!(reset to bg = color!(black)))?;
    }

    if let Some(venv) = enver.venv() {
        out.div(&mut last, color!(cyan), color!(black))?;
        if let Some(venv) = venv.rsplit(std::path::MAIN_SEPARATOR).next() {
            write!(out, "{venv}")?;
        } else {
            write!(out, "{venv}")?;
        }
    }

    if let Some((direnv, active)) = enver.direnv() {
        if active {
            out.div(&mut last, color!(green), color!(black))?;
        } else {
            out.div(&mut last, color!(magenta), color!(black))?;
        }
        if let Some(direnv) = direnv.rsplit(std::path::MAIN_SEPARATOR).next() {
            write!(out, "{direnv}")?;
        } else {
            write!(out, "{direnv}")?;
        }
    }

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
        match git::parse(pwd) {
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
    }
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
    fn direnv(&self) -> Option<(String, bool)>;
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
    }

    #[test]
    fn all_empty() {
        let result = test(|s| render_inner(s, None, false, false, &MockEnv::default()));
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
                style!(reset to bg = color!(black)),
                " ",
                style!(fg = color!(black), bg = color!(cyan), symbol!(div)),
                style!(fg = color!(black)),
                " py ",
                style!(fg = color!(cyan), bg = color!(magenta), symbol!(div)),
                style!(fg = color!(black)),
                " direnv ",
                style!(fg = color!(magenta), bg = color!(blue), symbol!(div)),
                style!(fg = color!(black)),
                " ~/further/on ",
                style!(fg = color!(blue), bg = color!(reset), symbol!(div)),
                style!(fg = color!(reset)),
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
                    pwd: Some(std::path::PathBuf::from("/some/home/path/further/on")),
                    home: Some(String::from("/some/home/path")),
                    venv: None,
                    direnv: Some((String::from("/some/direnv"), false)),
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
                style!(reset to bg = color!(black)),
                " ",
                style!(fg = color!(black), bg = color!(magenta), symbol!(div)),
                style!(fg = color!(black)),
                " direnv ",
                style!(fg = color!(magenta), bg = color!(blue), symbol!(div)),
                style!(fg = color!(black)),
                " ~/further/on ",
                style!(fg = color!(blue), bg = color!(reset), symbol!(div)),
                style!(fg = color!(reset)),
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
                    pwd: Some(std::path::PathBuf::from("/some/home/path/further/on")),
                    home: Some(String::from("/some/home/path")),
                    venv: None,
                    direnv: Some((String::from("/some/direnv"), true)),
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
                style!(reset to bg = color!(black)),
                " ",
                style!(fg = color!(black), bg = color!(green), symbol!(div)),
                style!(fg = color!(black)),
                " direnv ",
                style!(fg = color!(green), bg = color!(blue), symbol!(div)),
                style!(fg = color!(black)),
                " ~/further/on ",
                style!(fg = color!(blue), bg = color!(reset), symbol!(div)),
                style!(fg = color!(reset)),
                " "
            )
        );
    }
}
