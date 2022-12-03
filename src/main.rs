#![deny(warnings, rust_2018_idioms, clippy::pedantic)]

macro_rules! style {
    (reset $(, $($param: expr),*)?) => {
        concat!("[m" $(, $($param),*)?)
    };

    (fg = $fg: expr, bg = $bg: expr $(, $($param: expr),*)?) => {
        concat!("[3", $fg, "m", "[4", $bg, "m" $(, $($param),*)?)
    };

    (fg = $color: expr $(, $($param: expr),*)?) => {
        concat!("[3", $color, "m" $(, $($param),*)?)
    };

    (bg = $color: expr $(, $($param: expr),*)?) => {
        concat!("[4", $color, "m" $(, $($param),*)?)
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
        "" // " "
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

mod compatibility;
mod long;
mod short;

type Result = std::io::Result<()>;

fn main() {
    let mut args = std::env::args();
    let bin = args.next();
    let command = args.next();

    let out = std::io::stdout().lock();
    drop(match command.as_deref() {
        Some("l") => left(out, args),
        Some("r") => right(out),
        Some("c") => match args.next().as_deref() {
            Some("zsh") => compatibility::zsh(out, std::io::stdin().lock()),
            Some("posh") => compatibility::posh(out, std::io::stdin().lock()),
            _ => help(out, bin),
        },
        _ => help(out, bin),
    });
}

fn left(out: impl std::io::Write, args: impl Iterator<Item = String>) -> Result {
    let (host, error, jobs, long) = parse_params(args);

    if long {
        long::prompt(out, host, error, jobs)
    } else {
        short::prompt(out, host, error, jobs)
    }
}

fn parse_params(args: impl Iterator<Item = String>) -> (Option<String>, bool, bool, bool) {
    args.filter(|s| !s.is_empty())
        .fold((None, false, false, false), |acc, curr| {
            if curr == "-e" {
                (acc.0, true, acc.2, acc.3)
            } else if curr == "-j" {
                (acc.0, acc.1, true, acc.3)
            } else if curr == "-l" {
                (acc.0, acc.1, acc.2, true)
            } else {
                (Some(curr), acc.1, acc.2, acc.3)
            }
        })
}

fn right(mut out: impl std::io::Write) -> Result {
    use chrono::Timelike;
    let time = chrono::DateTime::<chrono::Local>::from(std::time::SystemTime::now());

    write!(
        out,
        style!(fg = color!([23]), "{h:02}:{m:02}:{s:02}", style!(reset)),
        h = time.hour(),
        m = time.minute(),
        s = time.second(),
    )?;
    out.flush()
}

fn help(mut out: impl std::io::Write, bin: Option<String>) -> Result {
    let bin = bin
        .map(std::path::PathBuf::from)
        .and_then(|p| {
            p.file_name()
                .and_then(std::ffi::OsStr::to_str)
                .map(String::from)
        })
        .unwrap_or_else(|| String::from(env!("CARGO_BIN_NAME")));

    writeln!(out, "Usage: {bin} <COMMAND> [ARGUMENTS]*",)?;
    writeln!(out)?;
    writeln!(out, "Commands:")?;
    writeln!(out, "  c  Compatibility layer")?;
    writeln!(out, "  r  Generate right side prompt")?;
    writeln!(out, "  l  Generate left side prompt")?;
    writeln!(out, "  h  Show this help message")?;
    writeln!(out)?;
    writeln!(out, "Arguments (only for left side prompt):")?;
    writeln!(
        out,
        "  HOST   Symbol to be used as host (can contain ansi escape codes)"
    )?;
    writeln!(out, "  -e     Last command was an error")?;
    writeln!(out, "  -j     There are background processes running")?;
    writeln!(out, "  -l     Use the long format")?;
    writeln!(out)?;
    writeln!(out, "Arguments (only for compatibility layer):")?;
    writeln!(out, "  SHELL  The compatibility requested [zsh|posh]")?;
    out.flush()
}

#[cfg(test)]
fn test<F>(testing: F) -> String
where
    F: FnOnce(&mut Vec<u8>) -> Result,
{
    let mut buffer = String::new();
    unsafe { testing(buffer.as_mut_vec()).unwrap() };
    buffer
}

#[cfg(test)]
mod tests {
    use super::test;

    #[test]
    fn right() {
        use super::right;
        let result = test(|s| right(s));

        let regex = regex::Regex::new("^\\[38;5;23m[0-2][0-9]:[0-5][0-9]:[0-5][0-9]\\[m$").unwrap();
        assert!(regex.is_match(&result));
    }

    #[test]
    fn parse_params() {
        use super::parse_params;
        assert_eq!(
            (None, false, false, false),
            parse_params(std::iter::empty())
        );

        assert_eq!(
            (Some(String::from("last")), false, false, false),
            parse_params(
                vec![
                    String::from("first"),
                    String::from("second"),
                    String::from("last")
                ]
                .into_iter()
            )
        );

        assert_eq!(
            (Some(String::from("last")), true, true, true),
            parse_params(
                vec![
                    String::from("first"),
                    String::new(),
                    String::from("-3"),
                    String::from("-e"),
                    String::from("second"),
                    String::from("-j"),
                    String::from("last"),
                    String::from("-l"),
                    String::from("-e"),
                    String::from("-j"),
                    String::new(),
                ]
                .into_iter()
            )
        );
    }
}
