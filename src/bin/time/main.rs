#![deny(warnings, rust_2018_idioms, clippy::pedantic)]

mod args;
mod help;
mod print;
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
        if args.mode.is_some() {
            Err(std::io::Error::other("Version does not take arguments"))
        } else {
            version::render(std::io::stdout().lock())
        }
    } else {
        match args.mode {
            Some(args::Mode::Ansi) | None => {
                print::render(simpalt::print::Ansi::new(std::io::stdout().lock()))
            }
            Some(args::Mode::Zsh) => {
                print::render(simpalt::print::Zsh::new(std::io::stdout().lock()))
            }
            Some(args::Mode::Win(sub)) => {
                print::render(simpalt::print::Win::new(std::io::stdout().lock(), sub))
            }
        }
    }
}
