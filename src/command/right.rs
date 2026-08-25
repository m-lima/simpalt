use super::Compat;
use crate::{Result, print};
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
        Compat::None => render_inner(print::Ansi::new(out)),
        Compat::Zsh => render_inner(print::Zsh::new(out)),
        Compat::Win(sub) => render_inner(print::Win::new(out, sub)),
    }
}

fn render_inner<P>(mut printer: P) -> Result
where
    P: print::Printer,
{
    let time = chrono::DateTime::<chrono::Local>::from(std::time::SystemTime::now());

    printer
        .fg(print::Color::Vga(23))
        .bg(print::Color::Reset)
        .txt(format!(
            "{h:02}:{m:02}:{s:02}",
            h = time.hour(),
            m = time.minute(),
            s = time.second(),
        ))?
        .flush()
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
