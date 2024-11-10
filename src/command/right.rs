use super::Compat;
use crate::{compat, Result};
use chrono::Timelike;

#[derive(Debug, Eq, PartialEq)]
pub struct Args {
    pub compat: Compat,
}

pub fn render<Out>(out: Out, args: Args) -> Result
where
    Out: std::io::Write,
{
    match args.compat {
        Compat::None => render_inner(out),
        Compat::Zsh => render_inner(compat::Zsh::new(out)),
        Compat::Win(sub) => render_inner(compat::Win::new(out, sub)),
    }
}

fn render_inner<Out>(mut out: Out) -> Result
where
    Out: std::io::Write,
{
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test;

    #[test]
    fn right() {
        let result = test(|s| render_inner(s));

        let regex =
            regex::Regex::new("^\\[38;5;23m[0-2][0-9]:[0-5][0-9]:[0-5][0-9]\\[m$").unwrap();
        assert!(regex.is_match(&result));
    }
}
