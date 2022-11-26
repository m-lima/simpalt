mod args;
mod symbol;

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
    (fail) => {
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
        "☢" // 
    };
    (div thin) => {
        ""
    };
    (div) => {
        ""
    };
}

struct Color;

enum Text {
    Static(&'static str),
    Dynamic(String),
}

struct Segment {
    text: Text,
    fg: Option<Color>,
    bg: Option<Color>,
}

const BEGIN: Segment = Segment {
    text: Text::Static(""),
    fg: None,
    bg: None,
};

mod prompt {
    use super::{Color, Segment, Text};

    pub(super) fn aws() -> Option<Segment> {
        if std::env::var("AWS_VAULT").is_ok() {
            Some(Segment {
                text: Text::Static(""),
                fg: None,
                bg: Some(Color),
            })
        } else {
            None
        }
    }
}

fn left(args: args::Args) {
    let status = args.status.map_or(false, |s| s != 0);
    let root = if args.root {
        concat!(color!(fg yellow), symbol!(root), " ")
    } else {
        ""
    };
    let status = if status {
        concat!(color!(fg red), symbol!(fail), " ")
    } else {
        ""
    };
    let jobs = if args.jobs {
        concat!(color!(fg cyan), symbol!(jobs), " ")
    } else {
        ""
    };
    let host = concat!(color!(fg reset), "⏾ ");
    let venv = if std::env::var("VIRTUAL_ENV").is_ok() {
        concat!(
            color!(fg black),
            color!(bg cyan),
            symbol!(div),
            color!(fg cyan),
            color!(bg black),
            symbol!(div),
            color!(fg reset),
            " ",
        )
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
            " {root}{status}{jobs}{venv}{host}{pwd} ",
            color!(fg black),
            color!(bg blue),
            symbol!(div),
            color!(fg blue),
            color!(bg reset),
            symbol!(div),
        ),
        root = root,
        status = status,
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

fn main() {
    let args = args::parse();

    match args {
        args::Mode::Left(args) => left(args),
        args::Mode::Right => right(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pwd_parsing() {}
}
