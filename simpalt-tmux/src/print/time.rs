use chrono::{Datelike, Timelike};
use simpalt::print::{Color, Div, Printer};

pub fn render<P>(printer: &mut P) -> crate::Result
where
    P: Printer,
{
    let time = chrono::DateTime::<chrono::Local>::from(std::time::SystemTime::now());

    printer
        .div(Div::SlantTopRight, Color::Vga(239))?
        .fg(Color::Vga(248))
        .txt_gap(format_args!(
            "{day_of_week} {month} {day_of_month:02} {hour:02}:{minute:02}",
            day_of_week = time.weekday(),
            month = to_month(&time),
            day_of_month = time.day(),
            hour = time.hour(),
            minute = time.minute(),
        ))?;

    Ok(())
}

fn to_month(time: &chrono::DateTime<chrono::Local>) -> &'static str {
    match time.month() {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        12 => "Dec",
        _ => "???",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ansi() {
        {
            let mut buffer = String::new();
            let mut printer = unsafe { simpalt::print::Ansi::new(buffer.as_mut_vec()) };
            render(&mut printer).unwrap();
            println!("{buffer}[m");
        }

        let result = {
            let mut buffer = String::new();
            let mut printer = unsafe { simpalt::print::Tmux::new(buffer.as_mut_vec()) };
            render(&mut printer).unwrap();
            buffer
        };

        let result = result
            .strip_prefix("#[fg=colour239]#[fg=colour248,bg=colour239] ")
            .unwrap();

        let regex =
            regex::Regex::new("^(Mon|Tue|Wed|Thu|Fri|Sat|Sun) (Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec) [0-3][0-9] [0-2][0-9]:[0-5][0-9] $").unwrap();
        assert!(
            regex.is_match(result),
            "{raw}",
            raw = result.replace('', "^[")
        );
    }
}
