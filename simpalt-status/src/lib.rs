#![deny(warnings, rust_2018_idioms, clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]

pub mod git;
#[cfg(feature = "media")]
pub mod media;
pub mod time;

type Result<T = ()> = std::io::Result<T>;

#[cfg(test)]
mod tests {
    pub fn expect<'a, I: IntoIterator<Item = &'a str>>(result: &str, expected: I) -> String {
        let expected = String::from_iter(expected);
        println!("{result}");
        println!("{expected}");
        expected
    }
}
