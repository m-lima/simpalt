// #![deny(warnings, rust_2018_idioms, clippy::pedantic)]

// macro_rules! style {
//     (reset $(, $($param: expr),*)?) => {
//         concat!("[m" $(, $($param),*)?)
//     };
//
//     (reset to fg = $color: expr $(, $($param: expr),*)?) => {
//         concat!("[;3", $color, "m" $(, $($param),*)?)
//     };
//
//     (reset to bg = $color: expr $(, $($param: expr),*)?) => {
//         concat!("[;4", $color, "m" $(, $($param),*)?)
//     };
//
//     (fg = $fg: expr, bg = $bg: expr $(, $($param: expr),*)?) => {
//         concat!("[3", $fg, ";4", $bg, "m" $(, $($param),*)?)
//     };
//
//     (fg = $color: expr $(, $($param: expr),*)?) => {
//         concat!("[3", $color, "m" $(, $($param),*)?)
//     };
//
//     (bg = $color: expr $(, $($param: expr),*)?) => {
//         concat!("[4", $color, "m" $(, $($param),*)?)
//     };
// }
//
// macro_rules! color {
//     (black) => {
//         "0"
//     };
//     (red) => {
//         "1"
//     };
//     (green) => {
//         "2"
//     };
//     (yellow) => {
//         "3"
//     };
//     (blue) => {
//         "4"
//     };
//     (magenta) => {
//         "5"
//     };
//     (cyan) => {
//         "6"
//     };
//     (white) => {
//         "7"
//     };
//     ([$param: literal]) => {
//         concat!("8;5;", $param)
//     };
//     ([$r: literal, $g: literal, $b: literal]) => {
//         concat!("8;2;", $r, ";", $g, ";", $b)
//     };
//     (reset) => {
//         "9"
//     };
// }
//
// macro_rules! symbol {
//     (error) => {
//         "✘"
//     };
//     (jobs) => {
//         ""
//     };
//     (pkg) => {
//         "󰏓"
//     };
//     (direnv) => {
//         ""
//     };
//     (flake) => {
//         "󱄅"
//     };
//     (python) => {
//         "󰌠"
//     };
//     (new) => {
//         ""
//     };
//     (branch) => {
//         ""
//     };
//     (ref) => {
//         "➦"
//     };
//     (merge) => {
//         ""
//     };
//     (bisect) => {
//         ""
//     };
//     (rebase) => {
//         ""
//     };
//     (cherry) => {
//         ""
//     };
//     (revert) => {
//         ""
//     };
//     (mailbox) => {
//         ""
//     };
//     (ahead) => {
//         "󰁝"
//     };
//     (behind) => {
//         "󰁅"
//     };
//     (local) => {
//         "󰁂"
//     };
//     (gone) => {
//         "󰁜"
//     };
//     (warn) => {
//         "󱈸"
//     };
//     (div) => {
//         ""
//     };
//     (div thin) => {
//         ""
//     };
//     (slant) => {
//         ""
//     };
//     (slant thin) => {
//         ""
//     };
// }

mod args;
mod direnv;
mod help;
mod long;
mod short;
mod version;

type Result<T = ()> = std::io::Result<T>;

fn main() {
    let mut args = std::env::args();
    let bin = args.next();

    if let Err(err) = fallible_main(args, bin.as_ref()) {
        use std::io::Write;
        let mut out = std::io::stderr().lock();
        writeln!(out, "Error: {err}").unwrap();
        help::render(out, bin.as_ref()).unwrap();
    }
}

fn fallible_main(args: std::env::Args, bin: Option<&String>) -> Result {
    let args = args::parse(args)?;

    let Some(args) = args else {
        return help::render(std::io::stdout().lock(), bin);
    };

    if args.show_version {
        if args.long || args.error || args.jobs || args.mode.is_some() {
            Err(std::io::Error::other("Version does not take arguments"))
        } else {
            version::render(std::io::stdout().lock())
        }
    } else {
        match args.mode {
            Some(args::Mode::Ansi) | None => {
                if args.long {
                    long::render(
                        simpalt::print::Ansi::new(std::io::stdout().lock()),
                        args.symbol,
                        args.error,
                        args.jobs,
                    )
                } else {
                    short::render(
                        simpalt::print::Ansi::new(std::io::stdout().lock()),
                        args.symbol,
                        args.error,
                        args.jobs,
                    )
                }
            }
            Some(args::Mode::Zsh) => {
                if args.long {
                    long::render(
                        simpalt::print::Zsh::new(std::io::stdout().lock()),
                        args.symbol,
                        args.error,
                        args.jobs,
                    )
                } else {
                    short::render(
                        simpalt::print::Zsh::new(std::io::stdout().lock()),
                        args.symbol,
                        args.error,
                        args.jobs,
                    )
                }
            }
            Some(args::Mode::Win(sub)) => {
                if args.long {
                    long::render(
                        simpalt::print::Win::new(std::io::stdout().lock(), sub),
                        args.symbol,
                        args.error,
                        args.jobs,
                    )
                } else {
                    short::render(
                        simpalt::print::Win::new(std::io::stdout().lock(), sub),
                        args.symbol,
                        args.error,
                        args.jobs,
                    )
                }
            }
        }
    }
}

#[cfg(test)]
pub fn test<F>(testing: F) -> String
where
    F: FnOnce(simpalt::print::Ansi<&mut Vec<u8>>) -> std::io::Result<()>,
{
    let mut buffer = String::new();
    let printer = unsafe { simpalt::print::Ansi::new(buffer.as_mut_vec()) };
    testing(printer).unwrap();
    buffer
}

#[cfg(test)]
pub fn test_from<F>(fg: simpalt::print::Color, bg: simpalt::print::Color, testing: F) -> String
where
    F: FnOnce(simpalt::print::Ansi<&mut Vec<u8>>) -> std::io::Result<()>,
{
    use simpalt::print::Printer;

    let mut buffer = String::new();
    let mut printer = unsafe { simpalt::print::Ansi::new(buffer.as_mut_vec()) };
    printer.fg(fg).bg(bg).txt("").unwrap();
    testing(printer).unwrap();
    let (len, _) = buffer.char_indices().find(|(_, c)| *c == 'm').unwrap();
    buffer.drain(..=len);
    buffer
}
