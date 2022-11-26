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
        0
    };
    (red) => {
        1
    };
    (green) => {
        2
    };
    (yellow) => {
        3
    };
    (blue) => {
        4
    };
    (magenta) => {
        5
    };
    (cyan) => {
        6
    };
    (white) => {
        7
    };
    ([$param: literal]) => {
        concat!("8;5;", $param)
    };
    ([$r: literal, $g: literal, $b: literal]) => {
        concat!("8;2;", $r, ";", $g, ";", $b)
    };
    (reset) => {
        9
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

fn pwd(path: String) -> String {
    if let Ok(home) = std::env::var("HOME") {
        if path == home {
            String::from("~")
        } else {
            let mut parts = path.split(std::path::MAIN_SEPARATOR);
            let head = parts.next();
            let tail = parts.last().filter(|t| !t.is_empty());
            tail.or_else(|| head.filter(|h| !h.is_empty()))
                .map_or_else(|| String::from(std::path::MAIN_SEPARATOR), String::from)
        }
    } else {
        println!(
            style!(fg = color!(red), "`HOME` environment variable not available" style!(reset))
        );
        path
    }
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
        style!(fg = color!(green), " ") // ")
    } else {
        ""
    };

    print!(
        concat!(
            style!(bg = color!(black), " {error}{jobs}{venv}"),
            style!(fg = color!(reset), "{host}"),
            style!(reset),
            style!(bg = color!(black), " {pwd} "),
            style!(fg = color!(black), bg = color!(blue), symbol!(div)),
            style!(fg = color!(blue), bg = color!(reset), symbol!(div)),
        ),
        error = error,
        jobs = jobs,
        venv = venv,
        host = host,
        pwd = pwd(std::env::var("PWD").unwrap()),
    );
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
    fn pwd_parsing() {
        let tests = [
            ("", "/"),
            ("/", "/"),
            ("a/", "a"),
            ("a/b", "b"),
            ("/a/b", "b"),
            ("C:/a", "a"),
            ("C:/", "C:"),
            ("C:", "C:"),
        ]
        .map(|(a, b)| {
            (
                a.replace('/', String::from(std::path::MAIN_SEPARATOR).as_str()),
                b.replace('/', String::from(std::path::MAIN_SEPARATOR).as_str()),
            )
        });

        for (input, output) in tests {
            assert_eq!(pwd(input), output);
        }
    }
}
