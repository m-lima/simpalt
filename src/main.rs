#![deny(warnings, rust_2018_idioms, clippy::pedantic)]

macro_rules! style {
    (reset $(, $($param: expr),*)?) => {
        concat!("[m" $(, $($param),*)?)
    };

    (reset to fg = $color: expr $(, $($param: expr),*)?) => {
        concat!("[;3", $color, "m" $(, $($param),*)?)
    };

    (reset to bg = $color: expr $(, $($param: expr),*)?) => {
        concat!("[;4", $color, "m" $(, $($param),*)?)
    };

    (fg = $fg: expr, bg = $bg: expr $(, $($param: expr),*)?) => {
        concat!("[3", $fg, ";4", $bg, "m" $(, $($param),*)?)
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
        "󰌠"
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
        ""
    };
    (ahead) => {
        "󰁝"
    };
    (behind) => {
        "󰁅"
    };
    (local) => {
        "󰁂"
    };
    (gone) => {
        "󰁜"
    };
    (warn) => {
        "󱈸"
    };
    (div) => {
        ""
    };
    (div thin) => {
        ""
    };
    (slant) => {
        ""
    };
    (slant thin) => {
        ""
    };
}

mod args;
mod command;
mod compat;
mod git;

type Result<T = ()> = std::io::Result<T>;

fn main() {
    drop(args::parse().run(std::io::stdout().lock()));
}

#[cfg(test)]
fn test<F>(testing: F) -> String
where
    F: FnOnce(&mut Vec<u8>) -> crate::Result,
{
    let mut buffer = String::new();
    unsafe { testing(buffer.as_mut_vec()).unwrap() };
    buffer
}
