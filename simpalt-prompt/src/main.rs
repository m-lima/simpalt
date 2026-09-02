#![deny(warnings, rust_2018_idioms, clippy::pedantic)]

mod args;
mod direnv;
mod help;
mod long;
mod short;
mod time;
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
    let action = args::parse(args)?;

    match action {
        args::Action::Help => help::render(std::io::stdout().lock(), bin),
        args::Action::Version => version::render(std::io::stdout().lock()),
        args::Action::Left(options) => match options.mode {
            Some(args::Mode::Ansi) | None => {
                if options.long {
                    long::render(
                        simpalt::print::Ansi::new(std::io::stdout().lock()),
                        options.symbol,
                        options.error,
                        options.jobs,
                    )
                } else {
                    short::render(
                        simpalt::print::Ansi::new(std::io::stdout().lock()),
                        options.symbol,
                        options.error,
                        options.jobs,
                    )
                }
            }
            Some(args::Mode::Zsh) => {
                if options.long {
                    long::render(
                        simpalt::print::Zsh::new(std::io::stdout().lock()),
                        options.symbol,
                        options.error,
                        options.jobs,
                    )
                } else {
                    short::render(
                        simpalt::print::Zsh::new(std::io::stdout().lock()),
                        options.symbol,
                        options.error,
                        options.jobs,
                    )
                }
            }
            Some(args::Mode::Win(sub)) => {
                if options.long {
                    long::render(
                        simpalt::print::Win::new(std::io::stdout().lock(), sub),
                        options.symbol,
                        options.error,
                        options.jobs,
                    )
                } else {
                    short::render(
                        simpalt::print::Win::new(std::io::stdout().lock(), sub),
                        options.symbol,
                        options.error,
                        options.jobs,
                    )
                }
            }
        },
        args::Action::Right(options) => match options.mode {
            Some(args::Mode::Ansi) | None => {
                time::render(simpalt::print::Ansi::new(std::io::stdout().lock()))
            }
            Some(args::Mode::Zsh) => {
                time::render(simpalt::print::Zsh::new(std::io::stdout().lock()))
            }
            Some(args::Mode::Win(sub)) => {
                time::render(simpalt::print::Win::new(std::io::stdout().lock(), sub))
            }
        },
    }
}

#[cfg(test)]
mod tests {
    pub fn expect<'a, I: IntoIterator<Item = &'a str>>(result: &str, expected: I) -> String {
        let expected = String::from_iter(expected);
        println!("{result}");
        println!("{expected}");
        expected
    }

    pub fn test<F>(t: F) -> String
    where
        F: FnOnce(simpalt::print::Ansi<&mut Vec<u8>>) -> std::io::Result<()>,
    {
        let mut buffer = String::new();
        let printer = unsafe { simpalt::print::Ansi::new(buffer.as_mut_vec()) };
        t(printer).unwrap();
        buffer
    }

    #[cfg(test)]
    pub fn test_from<F>(fg: simpalt::print::Color, bg: simpalt::print::Color, t: F) -> String
    where
        F: FnOnce(simpalt::print::Ansi<&mut Vec<u8>>) -> std::io::Result<()>,
    {
        use simpalt::print::Printer;

        let mut buffer = String::new();
        let mut printer = unsafe { simpalt::print::Ansi::new(buffer.as_mut_vec()) };
        printer.fg(fg).bg(bg).txt("").unwrap();
        t(printer).unwrap();
        let (len, _) = buffer.char_indices().find(|(_, c)| *c == 'm').unwrap();
        buffer.drain(..=len);
        buffer
    }
}
