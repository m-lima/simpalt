mod direnv;
mod long;
mod short;

use super::Compat;
use crate::{Result, print};

#[derive(Debug, Eq, PartialEq)]
pub struct Args {
    pub host: Option<String>,
    pub error: bool,
    pub jobs: bool,
    pub long: bool,
    pub compat: Compat,
}

pub fn render<Out>(out: Out, args: Args) -> Result
where
    Out: std::io::Write,
{
    match args.compat {
        Compat::None => render_inner(
            print::Ansi::new(out),
            args.long,
            args.host,
            args.error,
            args.jobs,
        ),
        Compat::Zsh => render_inner(
            print::Zsh::new(out),
            args.long,
            args.host,
            args.error,
            args.jobs,
        ),
        Compat::Win(sub) => render_inner(
            print::Win::new(out, sub),
            args.long,
            args.host,
            args.error,
            args.jobs,
        ),
    }
}

fn render_inner<P>(printer: P, long: bool, host: Option<String>, error: bool, jobs: bool) -> Result
where
    P: print::Printer,
{
    if long {
        long::render(printer, host, error, jobs)
    } else {
        short::render(printer, host, error, jobs)
    }
}
