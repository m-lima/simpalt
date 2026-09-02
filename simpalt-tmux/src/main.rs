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
    let action = args::parse(args)?;

    match action {
        args::Action::Help => help::render(std::io::stdout().lock(), bin),
        args::Action::Version => version::render(std::io::stdout().lock()),
        args::Action::Status(path) => {
            print::render(simpalt::print::Tmux::new(std::io::stdout().lock()), path)
        }
    }
}
