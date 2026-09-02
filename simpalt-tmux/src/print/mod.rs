use crate::Result;

mod git;
#[cfg(feature = "media")]
mod media;
mod time;

pub fn render<P>(mut printer: P, pwd: String) -> Result
where
    P: simpalt::print::Printer,
{
    git::render(&mut printer, pwd)?;
    #[cfg(feature = "media")]
    media::render(&mut printer)?;
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
