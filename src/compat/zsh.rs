use crate::Result;

#[derive(Eq, PartialEq)]
enum Escape {
    Normal,
    Escaped,
    MaybeDone,
}

pub struct Zsh<Out>
where
    Out: std::io::Write,
{
    out: Out,
    escape: Escape,
}

impl<Out> Zsh<Out>
where
    Out: std::io::Write,
{
    pub fn new(out: Out) -> Self {
        Self {
            out,
            escape: Escape::Normal,
        }
    }
}

impl<Out> std::io::Write for Zsh<Out>
where
    Out: std::io::Write,
{
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let mut cursor = 0;

        for (i, byte) in buf.iter().copied().enumerate() {
            match self.escape {
                Escape::Normal => {
                    if byte == b'' {
                        if cursor < i {
                            self.out.write_all(&buf[cursor..i])?;
                            cursor = i;
                        }
                        self.out.write_all(b"%{")?;
                        self.escape = Escape::Escaped;
                    }
                }
                Escape::Escaped => {
                    if byte == b'm' {
                        self.escape = Escape::MaybeDone;
                    }
                }
                Escape::MaybeDone => {
                    if byte == b'' {
                        self.escape = Escape::Escaped;
                    } else {
                        if cursor < i {
                            self.out.write_all(&buf[cursor..i])?;
                            cursor = i;
                        }
                        self.out.write_all(b"%}")?;
                        self.escape = Escape::Normal;
                    }
                }
            }
        }

        if cursor < buf.len() {
            self.out.write_all(&buf[cursor..])?;
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> Result {
        self.out.flush()
    }
}

impl<Out> Drop for Zsh<Out>
where
    Out: std::io::Write,
{
    fn drop(&mut self) {
        if self.escape != Escape::Normal {
            drop(self.out.write_all(b"%}"));
            drop(self.out.flush());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn simple() {
        let input = String::from("abc[31mdef");
        let expected = String::from("abc%{[31m%}def");
        let mut output = Vec::new();
        Zsh::new(&mut output).write_all(input.as_bytes()).unwrap();
        let output = String::from_utf8(output).unwrap();
        assert_eq!(output, expected);
    }

    #[test]
    fn double_m() {
        let input = String::from("abc[31mmdef");
        let expected = String::from("abc%{[31m%}mdef");
        let mut output = Vec::new();
        Zsh::new(&mut output).write_all(input.as_bytes()).unwrap();
        let output = String::from_utf8(output).unwrap();
        assert_eq!(output, expected);
    }

    #[test]
    fn with_end() {
        let input = String::from("[31mYo[m");
        let expected = String::from("%{[31m%}Yo%{[m%}");
        let mut output = Vec::new();
        Zsh::new(&mut output).write_all(input.as_bytes()).unwrap();
        let output = String::from_utf8(output).unwrap();
        assert_eq!(output, expected);
    }

    #[test]
    fn long_escape() {
        let input = String::from("[38;5;2mabc[mdef");
        let expected = String::from("%{[38;5;2m%}abc%{[m%}def");
        let mut output = Vec::new();
        Zsh::new(&mut output).write_all(input.as_bytes()).unwrap();
        let output = String::from_utf8(output).unwrap();
        assert_eq!(output, expected);
    }

    #[test]
    fn long_escape_with_end() {
        let input = String::from("[38;5;2m[41mabc[49mdef");
        let expected = String::from("%{[38;5;2m[41m%}abc%{[49m%}def");
        let mut output = Vec::new();
        Zsh::new(&mut output).write_all(input.as_bytes()).unwrap();
        let output = String::from_utf8(output).unwrap();
        assert_eq!(output, expected);
    }
}
