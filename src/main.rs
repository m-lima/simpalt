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
    (div) => {
        ""
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

mod long {
    use crate::git::long as git;

    pub fn prompt(host: Option<String>, error: bool, jobs: bool) {
        let mut last = None;

        if error {
            div(&mut last, color!(black), color!(red));
            print!(symbol!(error));
        };

        if jobs {
            div(&mut last, color!(black), color!(cyan));
            print!(symbol!(jobs));
        }

        if let Some(host) = host {
            div(&mut last, color!(black), color!(reset));
            print!("{host}");
        };

        if let Ok(venv) = std::env::var("VIRTUAL_ENV") {
            div(&mut last, color!(cyan), color!(black));
            if let Some(venv) = venv.rsplit(std::path::MAIN_SEPARATOR).next() {
                print!("{venv}");
            } else {
                print!("{venv}");
            }
        };

        let pwd = std::env::current_dir()
            .or_else(|_| std::env::var("PWD").map(std::path::PathBuf::from))
            .ok();

        if let Some(ref pwd) = pwd {
            if let Some(pwd) = pwd.to_str() {
                div(&mut last, color!(blue), color!(black));
                if let Some(pwd) = std::env::var("HOME")
                    .ok()
                    .and_then(|home| pwd.strip_prefix(&home))
                {
                    print!("~{pwd}");
                } else {
                    print!("{pwd}");
                }
            }
        }

        if let Some(ref pwd) = pwd {
            match git::prompt(pwd) {
                git::Repo::None => div(&mut last, color!(reset), color!(reset)),
                git::Repo::Clean(head, _) => {
                    div(&mut last, color!(green), color!(black));
                    print!("{head}");
                    // match sync {
                    //     git::Sync::Behind(_) => todo!(),
                    //     git::Sync::Ahead(_) => todo!(),
                    //     git::Sync::Diverged { behind, ahead } => todo!(),
                    //     git::Sync::UpToDate => todo!(),
                    //     git::Sync::Local => todo!(),
                    // }
                }
                git::Repo::Dirty(head, _) => {
                    div(&mut last, color!(yellow), color!(black));
                    print!("{head}");
                    // match sync {
                    //     git::Sync::Behind(_) => todo!(),
                    //     git::Sync::Ahead(_) => todo!(),
                    //     git::Sync::Diverged { behind, ahead } => todo!(),
                    //     git::Sync::UpToDate => todo!(),
                    //     git::Sync::Local => todo!(),
                    // }
                }
                git::Repo::Detached(head) => {
                    div(&mut last, color!(magenta), color!(black));
                    print!(concat!(symbol!(ref), " {head}"), head = head);
                }
                git::Repo::Pending(head, _) => {
                    div(&mut last, color!(cyan), color!(black));
                    print!("{head}");
                }
                git::Repo::New => todo!(),
                git::Repo::Error => todo!(),
            }
        };
        div(&mut last, color!(reset), color!(reset));
    }

    fn div(last: &mut Option<&'static str>, to: &'static str, fg: &'static str) {
        if let Some(last) = last {
            if &to == last {
                print!(" [3{fg}m");
            } else {
                print!(
                    concat!(" [3{last}m[4{to}m", symbol!(div), "[3{fg}m "),
                    last = last,
                    to = to,
                    fg = fg,
                );
            }
        } else {
            print!("[3{fg}m[4{to}m ");
        }
        *last = Some(to);
    }
}

mod short {
    pub fn prompt(host: Option<String>, error: bool, jobs: bool) {
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

        print!(
            concat!(
                style!(bg = color!(black), " {error}{jobs}{venv}"),
                style!(fg = color!(reset), "{host}"),
                style!(reset),
                style!(bg = color!(black), "{host_padding}{pwd_string} "),
                "{git_string}",
                " "
            ),
            error = error,
            jobs = jobs,
            venv = venv,
            host_padding = host_padding,
            host = host,
            pwd_string = pwd_string,
            git_string = git_string,
        );
    }

    fn git_string(path: &std::path::PathBuf) -> &'static str {
        use crate::git::short as git;

        macro_rules! prompt {
            (default $state: expr) => {
                concat!(symbol!(branch), prompt!($state))
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
            git::Repo::Pending => prompt!(default color!(cyan)),
            git::Repo::New => prompt!(color!(cyan)),
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

fn right() {
    use chrono::Timelike;

    let time = chrono::DateTime::<chrono::Local>::from(std::time::SystemTime::now());
    print!(
        style!(fg = color!([23]), "{h:02}:{m:02}:{s:02}" style!(reset)),
        h = time.hour(),
        m = time.minute(),
        s = time.second(),
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
    println!("  HOST  Symbol to be used as host (can contain ansii escape codes)");
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
