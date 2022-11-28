#![deny(warnings, rust_2018_idioms, clippy::pedantic)]

mod git;

macro_rules! style {
    (reset) => {
        "[m"
    };

    (fg = $color: expr $(, $($param: expr)*)?) => {
        concat!("[3", $color, "m" $(, $($param),*)?)
    };

    (bg = $color: expr $(, $($param: expr)*)?) => {
        concat!("[4", $color, "m" $(, $($param),*)?)
    };

    (fg = $fg: expr, bg = $bg: expr $(, $($param: expr)*)?) => {
        concat!("[3", $fg, "m", "[4", $bg, "m" $(, $($param),*)?)
    };
}

macro_rules! color {
    (black) => {
        "0"
    };
    (red) => {
        "1"
    };
    (green) => {
        "2"
    };
    (yellow) => {
        "3"
    };
    (blue) => {
        "4"
    };
    (magenta) => {
        "5"
    };
    (cyan) => {
        "6"
    };
    (white) => {
        "7"
    };
    ([$param: literal]) => {
        concat!("8;5;", $param)
    };
    ([$r: literal, $g: literal, $b: literal]) => {
        concat!("8;2;", $r, ";", $g, ";", $b)
    };
    (reset) => {
        "9"
    };
}

macro_rules! symbol {
    (error) => {
        "✘"
    };
    (jobs) => {
        ""
    };
    (python) => {
        " " // " "
    };
    (new) => {
        ""
    };
    (branch) => {
        ""
    };
    (ref) => {
        "➦"
    };
    (merge) => {
        ""
    };
    (bisect) => {
        ""
    };
    (rebase) => {
        ""
    };
    (cherry) => {
        ""
    };
    (revert) => {
        ""
    };
    (mailbox) => {
        ""
    };
    (ahead) => {
        ""
    };
    (behind) => {
        ""
    };
    (local) => {
        ""
    };
    (gone) => {
        ""
    };
    (warn) => {
        ""
    };
    (div) => {
        ""
    };
    (div thin) => {
        ""
    };
}

fn left(args: impl Iterator<Item = String>) {
    let (host, error, jobs, long) = args.fold((None, false, false, false), |acc, curr| {
        if curr.is_empty() {
            acc
        } else if curr == "-e" {
            (acc.0, true, acc.2, acc.3)
        } else if curr == "-j" {
            (acc.0, acc.1, true, acc.3)
        } else if curr == "-l" {
            (acc.0, acc.1, acc.2, true)
        } else {
            (Some(curr), acc.1, acc.2, acc.3)
        }
    });

    if long {
        long::prompt(host, error, jobs);
    } else {
        short::prompt(host, error, jobs);
    }
}

mod short {
    pub fn prompt(host: Option<String>, error: bool, jobs: bool) {
        use std::io::Write;

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

        let venv = if std::env::var("VIRTUAL_ENV").is_ok() {
            style!(fg = color!(green), symbol!(python))
        } else {
            ""
        };

        let (host, host_padding) = host.map_or_else(|| (String::new(), ""), |host| (host, " "));

        let pwd = std::env::current_dir()
            .or_else(|_| std::env::var("PWD").map(std::path::PathBuf::from))
            .ok();

        let pwd_string = if let Some(ref pwd) = pwd {
            pwd_string(pwd)
        } else {
            String::new()
        };

        let git_string = if let Some(ref pwd) = pwd {
            git_string(pwd)
        } else {
            style!(fg = color!(black), bg = color!(reset), symbol!(div))
        };

        let mut stdout = std::io::stdout().lock();
        drop(
            write!(
                stdout,
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
            )
            .and_then(|_| stdout.flush()),
        );
    }

    fn git_string(path: &std::path::PathBuf) -> &'static str {
        use crate::git::short as git;

        macro_rules! prompt {
            (default $state: expr) => {
                concat!(symbol!(branch), prompt!($state))
            };
            (warn $state: expr) => {
                concat!(symbol!(warn), prompt!($state))
            };
            ($sync: expr, $state: expr) => {
                style!(fg = $sync, symbol!(branch) prompt!($state))
            };
            ($state: expr) => {
                style!(fg = color!(black), bg = $state, symbol!(div) style!(fg = $state, bg = color!(reset), symbol!(div)))
            };
        }

        match git::prompt(path) {
            git::Repo::None => prompt!(color!(blue)),
            git::Repo::Clean(sync) => match sync {
                git::Sync::UpToDate => prompt!(default color!(green)),
                git::Sync::Behind => prompt!(color!(red), color!(green)),
                git::Sync::Ahead => prompt!(color!(yellow), color!(green)),
                git::Sync::Diverged => prompt!(color!(magenta), color!(green)),
                git::Sync::Local => prompt!(color!(blue), color!(green)),
            },
            git::Repo::Dirty(sync) => match sync {
                git::Sync::UpToDate => prompt!(default color!(yellow)),
                git::Sync::Behind => prompt!(color!(red), color!(yellow)),
                git::Sync::Ahead => prompt!(color!(yellow), color!(yellow)),
                git::Sync::Diverged => prompt!(color!(magenta), color!(yellow)),
                git::Sync::Local => prompt!(color!(blue), color!(yellow)),
            },
            git::Repo::Pending => prompt!(warn color!(cyan)),
            git::Repo::Untracked => prompt!(default color!(cyan)),
            git::Repo::Detached => prompt!(default color!(magenta)),
            git::Repo::Error => prompt!(color!(red)),
        }
    }

    fn pwd_string(path: &std::path::PathBuf) -> String {
        if let Ok(home) = std::env::var("HOME").map(std::path::PathBuf::from) {
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
}

mod long {
    use crate::git::long as git;

    pub fn prompt(host: Option<String>, error: bool, jobs: bool) {
        Prompt(std::io::stdout().lock()).print(host, error, jobs);
    }

    struct Prompt<W>(W);

    impl<W: std::io::Write> Prompt<W> {
        fn print(&mut self, host: Option<String>, error: bool, jobs: bool) {
            let mut last = None;

            if error {
                self.div(&mut last, color!(black), color!(red));
                drop(write!(self.0, symbol!(error)));
            };

            if jobs {
                self.div(&mut last, color!(black), color!(cyan));
                drop(write!(self.0, symbol!(jobs)));
            }

            if let Some(host) = host {
                self.div(&mut last, color!(black), color!(reset));
                drop(write!(self.0, "{host}"));
            };

            if let Ok(venv) = std::env::var("VIRTUAL_ENV") {
                self.div(&mut last, color!(cyan), color!(black));
                if let Some(venv) = venv.rsplit(std::path::MAIN_SEPARATOR).next() {
                    drop(write!(self.0, "{venv}"));
                } else {
                    drop(write!(self.0, "{venv}"));
                }
            };

            let pwd = std::env::current_dir()
                .or_else(|_| std::env::var("PWD").map(std::path::PathBuf::from))
                .ok();

            if let Some(ref pwd) = pwd {
                if let Some(pwd) = pwd.to_str() {
                    self.div(&mut last, color!(blue), color!(black));
                    if let Some(pwd) = std::env::var("HOME")
                        .ok()
                        .and_then(|home| pwd.strip_prefix(&home))
                    {
                        drop(write!(self.0, "~{pwd}"));
                    } else {
                        drop(write!(self.0, "{pwd}"));
                    }
                }
            }

            if let Some(ref pwd) = pwd {
                match git::prompt(pwd) {
                    git::Repo::None => self.div(&mut last, color!(reset), color!(reset)),
                    git::Repo::Error => {
                        self.div(&mut last, color!(red), color!(black));
                        drop(write!(self.0, "!"));
                    }
                    git::Repo::Regular(head, sync, changes) => {
                        if changes.clean() {
                            self.render_sync(&mut last, sync);
                            self.div(&mut last, color!(green), color!(black));
                            drop(write!(
                                self.0,
                                concat!(symbol!(branch), "{head}"),
                                head = head
                            ));
                        } else {
                            self.render_changes(&mut last, changes);
                            if !matches!(
                                sync,
                                git::Sync::Tracked {
                                    ahead: 0,
                                    behind: 0
                                }
                            ) {
                                self.div(&mut last, color!(black), color!(reset));
                                drop(write!(self.0, symbol!(div thin)));
                                self.render_sync(&mut last, sync);
                            }
                            self.div(&mut last, color!(yellow), color!(black));
                            drop(write!(
                                self.0,
                                concat!(symbol!(branch), "{head}"),
                                head = head
                            ));
                        }
                    }
                    git::Repo::Detached(head, changes) => {
                        self.render_changes(&mut last, changes);
                        self.div(&mut last, color!(magenta), color!(black));
                        drop(write!(self.0, concat!(symbol!(ref), "{head}"), head = head));
                    }
                    git::Repo::Pending(head, pending, changes) => {
                        self.render_changes(&mut last, changes);
                        self.div(&mut last, color!(cyan), color!(black));
                        drop(write!(
                            self.0,
                            concat!(symbol!(branch), "{head} {pending}"),
                            head = head,
                            pending = pending_symbol(pending),
                        ));
                    }
                    git::Repo::New(changes) => {
                        self.render_changes(&mut last, changes);
                        self.div(&mut last, color!(cyan), color!(black));
                        drop(write!(self.0, symbol!(new)));
                    }
                }
            };
            self.div(&mut last, color!(reset), color!(reset));
            drop(self.0.flush());
        }

        fn div(&mut self, last: &mut Option<&'static str>, to: &'static str, fg: &'static str) {
            if let Some(last) = last {
                if &to == last {
                    drop(write!(self.0, " [3{fg}m"));
                } else {
                    drop(write!(
                        self.0,
                        concat!(" [3{last}m[4{to}m", symbol!(div), "[3{fg}m "),
                        last = last,
                        to = to,
                        fg = fg,
                    ));
                }
            } else {
                drop(write!(self.0, "[3{fg}m[4{to}m "));
            }
            *last = Some(to);
        }

        fn render_changes(&mut self, last: &mut Option<&'static str>, changes: git::Changes) {
            if changes.added > 0 {
                self.div(last, color!(black), color!(green));
                drop(write!(self.0, "+{added}", added = changes.added));
            }

            if changes.removed > 0 {
                self.div(last, color!(black), color!(red));
                drop(write!(self.0, "-{removed}", removed = changes.removed));
            }

            if changes.modified > 0 {
                self.div(last, color!(black), color!(blue));
                drop(write!(self.0, "~{modified}", modified = changes.modified));
            }

            if changes.conflicted > 0 {
                self.div(last, color!(black), color!(magenta));
                drop(write!(
                    self.0,
                    "!{conflicted}",
                    conflicted = changes.conflicted
                ));
            }
        }

        fn render_sync(&mut self, last: &mut Option<&'static str>, sync: git::Sync) {
            match sync {
                git::Sync::Local => {
                    self.div(last, color!(black), color!(cyan));
                    drop(write!(self.0, concat!(symbol!(local), " local")));
                }
                git::Sync::Gone => {
                    self.div(last, color!(black), color!(magenta));
                    drop(write!(self.0, concat!(symbol!(gone), " gone")));
                }
                git::Sync::Tracked { ahead, behind } => {
                    if ahead > 0 {
                        self.div(last, color!(black), color!(yellow));
                        drop(write!(
                            self.0,
                            concat!(symbol!(ahead), "{ahead}"),
                            ahead = ahead
                        ));
                    }
                    if behind > 0 {
                        self.div(last, color!(black), color!(red));
                        drop(write!(
                            self.0,
                            concat!(symbol!(behind), "{behind}"),
                            behind = behind
                        ));
                    }
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
}

fn right() {
    use chrono::Timelike;
    use std::io::Write;

    let time = chrono::DateTime::<chrono::Local>::from(std::time::SystemTime::now());

    let mut stdout = std::io::stdout().lock();
    drop(
        write!(
            stdout,
            style!(fg = color!([23]), "{h:02}:{m:02}:{s:02}" style!(reset)),
            h = time.hour(),
            m = time.minute(),
            s = time.second(),
        )
        .and_then(|_| stdout.flush()),
    );
}

fn help(bin: Option<String>) {
    let bin = bin
        .map(std::path::PathBuf::from)
        .and_then(|p| {
            p.file_name()
                .and_then(std::ffi::OsStr::to_str)
                .map(String::from)
        })
        .unwrap_or_else(|| String::from(env!("CARGO_BIN_NAME")));

    println!("Usage: {bin} <COMMAND> [HOST|-e|-j|-l]*",);
    println!();
    println!("Commands:");
    println!("  r  Generate right side prompt");
    println!("  l  Generate left side prompt");
    println!("  h  Show this help message");
    println!();
    println!("Arguments (only for left side prompt):");
    println!("  HOST  Symbol to be used as host (can contain ansi escape codes)");
    println!("  -e    Last command was an error");
    println!("  -j    There are background processes running");
    println!("  -l    Use the long format");
}

fn main() {
    let mut args = std::env::args();
    let bin = args.next();
    let command = args.next();

    match command.as_deref() {
        Some("r") => right(),
        Some("l") => left(args),
        _ => help(bin),
    }
}
