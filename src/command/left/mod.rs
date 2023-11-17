mod long;
mod short;

use super::Compat;
use crate::{compat, Result};

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
        Compat::None => render_inner(out, args.long, args.host, args.error, args.jobs),
        Compat::Zsh => render_inner(
            compat::Zsh::new(out),
            args.long,
            args.host,
            args.error,
            args.jobs,
        ),
        Compat::Win(sub) => render_inner(
            compat::Win::new(out, sub),
            args.long,
            args.host,
            args.error,
            args.jobs,
        ),
    }
}

fn render_inner<Out>(out: Out, long: bool, host: Option<String>, error: bool, jobs: bool) -> Result
where
    Out: std::io::Write,
{
    if long {
        long::render(out, host, error, jobs)
    } else {
        short::render(out, host, error, jobs)
    }
}
