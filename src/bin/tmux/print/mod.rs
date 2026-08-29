use crate::Result;

mod git;
mod time;

pub fn render<P>(mut printer: P, pwd: String) -> Result
where
    P: simpalt::print::Printer,
{
    git::render(&mut printer, pwd)?;
    time::render(&mut printer)?;

    printer.flush()
}

#[cfg(test)]
mod tests {
    pub fn expect<'a, I: IntoIterator<Item = &'a str>>(result: &str, expected: I) -> String {
        let expected = String::from_iter(expected);
        println!("{result}");
        println!("{expected}");
        expected
    }
}
