use chrono::Timelike;
use simpalt::print::{Color, Printer};

pub fn render<P>(mut printer: P) -> crate::Result
where
    P: Printer,
{
    let time = chrono::DateTime::<chrono::Local>::from(std::time::SystemTime::now());

    printer
        .fg(Color::Vga(23))
        .bg(Color::Reset)
        .txt(format_args!(
            "{h:02}:{m:02}:{s:02}",
            h = time.hour(),
            m = time.minute(),
            s = time.second(),
        ))?
        .fg(Color::Reset)
        .txt("")?
        .flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use simpalt::print;

    fn test<F>(test: F) -> String
    where
        F: FnOnce(&mut Vec<u8>) -> crate::Result,
    {
        let mut buffer = String::new();
        test(unsafe { buffer.as_mut_vec() }).unwrap();
        buffer
    }

    #[test]
    fn ansi() {
        let result = test(|v| render(print::Ansi::new(v)));

        let regex =
            regex::Regex::new("^\\[;38;5;23m[0-2][0-9]:[0-5][0-9]:[0-5][0-9]\\[m$").unwrap();
        assert!(
            regex.is_match(&result),
            "{raw}",
            raw = result.replace('', "^[")
        );
    }

    #[test]
    fn zsh() {
        let result = test(|v| render(print::Zsh::new(v)));

        let regex = regex::Regex::new(
            "^%\\{\\[;38;5;23m%\\}[0-2][0-9]:[0-5][0-9]:[0-5][0-9]%\\{\\[m%\\}$",
        )
        .unwrap();
        assert!(
            regex.is_match(&result),
            "{raw}",
            raw = result.replace('', "^[")
        );
    }

    #[test]
    fn win() {
        let result = test(|v| render(print::Win::new(v, String::from("@"))));

        let regex =
            regex::Regex::new("^\\[;38;5;23m[0-2][0-9]:[0-5][0-9]:[0-5][0-9]\\[m$").unwrap();
        assert!(
            regex.is_match(&result),
            "{raw}",
            raw = result.replace('', "^[")
        );
    }
}
