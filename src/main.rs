macro_rules! color {
    (reset) => {
        "[m"
    };

    (fg black) => {
        "[30m"
    };
    (fg red) => {
        "[31m"
    };
    (fg green) => {
        "[32m"
    };
    (fg yellow) => {
        "[33m"
    };
    (fg blue) => {
        "[34m"
    };
    (fg magenta) => {
        "[35m"
    };
    (fg cyan) => {
        "[36m"
    };
    (fg white) => {
        "[37m"
    };
    (fg [$param: literal]) => {
        concat!("[38;5;", $param, "m")
    };
    (fg [$r: literal, $g: literal, $b: literal]) => {
        concat!("[38;2;", $r, ";", $g, ";", $b, "m")
    };
    (fg reset) => {
        "[39m"
    };

    (bg black) => {
        "[40m"
    };
    (bg red) => {
        "[41m"
    };
    (bg green) => {
        "[42m"
    };
    (bg yellow) => {
        "[43m"
    };
    (bg blue) => {
        "[44m"
    };
    (bg magenta) => {
        "[45m"
    };
    (bg cyan) => {
        "[46m"
    };
    (bg white) => {
        "[47m"
    };
    (bg [$param: literal]) => {
        concat!("[48;5;", $param, "m")
    };
    (bg [$r: literal, $g: literal, $b: literal]) => {
        concat!("[48;2;", $r, ";", $g, ";", $b, "m")
    };
    (bg reset) => {
        "[49m"
    };
}

macro_rules! symbol {
    (error) => {
        "✘"
    };
    (jobs) => {
        "" // "⚙"
    };
    (lock) => {
        "" // 
    };
    (venv) => {
        "☢" //     
    };
    (root) => {
        "☢" //  ⚡
    };
    (div thin) => {
        ""
    };
    (div) => {
        ""
    };
}

fn left(host: impl std::fmt::Display, args: impl Iterator<Item = String>) {
    let (error, jobs) = args.fold((false, false), |acc, curr| {
        if curr == "e" {
            (true, acc.1)
        } else if curr == "j" {
            (acc.0, true)
        } else {
            acc
        }
    });

    let error = if error {
        concat!(color!(fg red), symbol!(error), " ")
    } else {
        ""
    };

    let jobs = if jobs {
        concat!(color!(fg cyan), symbol!(jobs), " ")
    } else {
        ""
    };

    let venv = if std::env::var("VIRTUAL_ENV").is_ok() {
        concat!(color!(fg green), " ") // ")
    } else {
        ""
    };

    let pwd = std::env::var("PWD").map_or_else(
        |_| {
            println!(concat!(
                color!(fg red),
                "`PWD` environment variable not available",
                color!(reset)
            ));
            String::new()
        },
        |pwd| {
            if let Ok(home) = std::env::var("HOME") {
                if pwd == home {
                    String::from("~")
                } else {
                    let mut parts = pwd.split(std::path::MAIN_SEPARATOR);
                    let head = parts.next();
                    let tail = parts.last().filter(|t| !t.is_empty());
                    tail.or_else(|| head.filter(|h| !h.is_empty()))
                        .map_or_else(|| String::from(std::path::MAIN_SEPARATOR), String::from)
                }
            } else {
                println!(concat!(
                    color!(fg red),
                    "`HOME` environment variable not available",
                    color!(reset)
                ));
                pwd
            }
        },
    );

    print!(
        concat!(
            color!(bg black),
            " {error}{jobs}{venv}",
            color!(reset),
            color!(bg black),
            "{host}",
            color!(reset),
            color!(bg black),
            " {pwd} ",
            color!(fg black),
            color!(bg blue),
            symbol!(div),
            color!(fg blue),
            color!(bg reset),
            symbol!(div),
        ),
        error = error,
        jobs = jobs,
        venv = venv,
        host = host,
        pwd = pwd
    );
}

fn right() {
    use chrono::Timelike;

    let time = chrono::DateTime::<chrono::Local>::from(std::time::SystemTime::now());
    print!(
        concat!(color!(fg[23]), "{h:02}:{m:02}:{s:02}", color!(reset)),
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

    println!("Usage: {bin} <COMMAND> [HOST [e] [j]]",);
    println!();
    println!("Commands:");
    println!("  r  Generate right side prompt");
    println!("  l  Generate left side prompt");
    println!("  h  Show this help message");
    println!();
    println!("Arguments (only for left side prompt):");
    println!("  HOST  Symbol to be used as host (can contain ansii escape codes)");
    println!("  e     Last command was an error");
    println!("  j     There are background processes running");
}

fn main() {
    let mut args = std::env::args();
    let bin = args.next();
    let command = args.next();
    let host = args.next();

    match (command.as_deref(), host) {
        (Some("r"), _) => right(),
        (Some("l"), Some(host)) => left(host, args),
        _ => help(bin),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pwd_parsing() {}
}
