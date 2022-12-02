use crate::git::long as git;
use crate::Result;

pub fn prompt(
    mut out: impl std::io::Write,
    host: Option<String>,
    error: bool,
    jobs: bool,
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
    };

    if let Ok(venv) = std::env::var("VIRTUAL_ENV") {
        out.div(&mut last, color!(cyan), color!(black))?;
        if let Some(venv) = venv.rsplit(std::path::MAIN_SEPARATOR).next() {
            write!(out, "{venv}")?;
        } else {
            write!(out, "{venv}")?;
        }
    };

    let pwd = std::env::current_dir()
        .or_else(|_| std::env::var("PWD").map(std::path::PathBuf::from))
        .ok();

    if let Some(ref pwd) = pwd {
        if let Some(pwd) = pwd.to_str() {
            out.div(&mut last, color!(blue), color!(black))?;
            if let Some(pwd) = std::env::var("HOME")
                .ok()
                .and_then(|home| pwd.strip_prefix(&home))
            {
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
