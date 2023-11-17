use crate::Result;

#[derive(Eq, PartialEq)]
enum State {
    Normal,
    InEscape,
    InColor,
    InBackground,
}

pub struct Win<Out>
where
    Out: std::io::Write,
{
    out: Out,
    sub: String,
    state: State,
}

impl<Out> Win<Out>
where
    Out: std::io::Write,
{
    pub fn new<S>(out: Out, sub: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            out,
            sub: sub.into(),
            state: State::Normal,
        }
    }
}

impl<Out> std::io::Write for Win<Out>
where
    Out: std::io::Write,
{
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let mut cursor = 0;

        for (i, byte) in buf.iter().copied().enumerate() {
            if byte == b'm' {
                self.state = State::Normal;
            } else {
                match self.state {
                    State::Normal => {
                        if byte == b'' {
                            self.state = State::InEscape;
                        }
                    }
                    State::InEscape => {
                        if byte == b'[' || byte == b';' {
                            self.state = State::InColor;
                        }
                    }
                    State::InColor => {
                        if byte == b'4' {
                            self.state = State::InBackground;
                        }
                    }
                    State::InBackground => {
                        if byte == b'0' {
                            if cursor < i {
                                self.out.write_all(&buf[cursor..i])?;
                                cursor = i + 1;
                            }
                            self.out.write_all(self.sub.as_bytes())?;
                        } else if byte == b';' || byte == b'[' {
                            self.state = State::InColor;
                        } else {
                            self.state = State::InEscape;
                        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn no_match() {
        let input = String::from("abc[31mdef");
        let expected = String::from("abc[31mdef");
        let mut output = Vec::new();
        Win::new(&mut output, "@")
            .write_all(input.as_bytes())
            .unwrap();
        let output = String::from_utf8(output).unwrap();
        assert_eq!(output, expected);
    }

    #[test]
    fn simple_match() {
        let input = String::from("[40mabc[mdef");
        let expected = String::from("[4@mabc[mdef");
        let mut output = Vec::new();
        Win::new(&mut output, "@")
            .write_all(input.as_bytes())
            .unwrap();
        let output = String::from_utf8(output).unwrap();
        assert_eq!(output, expected);
    }

    #[test]
    fn almost_match() {
        let input = String::from("[41mabc[mdef");
        let expected = String::from("[41mabc[mdef");
        let mut output = Vec::new();
        Win::new(&mut output, "@")
            .write_all(input.as_bytes())
            .unwrap();
        let output = String::from_utf8(output).unwrap();
        assert_eq!(output, expected);
    }

    #[test]
    fn match_then_fake_match() {
        let input = String::from("[40m;40[40abc[mdef");
        let expected = String::from("[4@m;40[40abc[mdef");
        let mut output = Vec::new();
        Win::new(&mut output, "@")
            .write_all(input.as_bytes())
            .unwrap();
        let output = String::from_utf8(output).unwrap();
        assert_eq!(output, expected);
    }

    #[test]
    fn following_semicolon() {
        let input = String::from("[31;40mabc[mdef");
        let expected = String::from("[31;4@mabc[mdef");
        let mut output = Vec::new();
        Win::new(&mut output, "@")
            .write_all(input.as_bytes())
            .unwrap();
        let output = String::from_utf8(output).unwrap();
        assert_eq!(output, expected);
    }

    #[test]
    fn after_closing() {
        let input = String::from("[31m[40mabc[mdef");
        let expected = String::from("[31m[4@mabc[mdef");
        let mut output = Vec::new();
        Win::new(&mut output, "@")
            .write_all(input.as_bytes())
            .unwrap();
        let output = String::from_utf8(output).unwrap();
        assert_eq!(output, expected);
    }

    #[test]
    fn incomplete_background() {
        let input = String::from("[4m[40mabc[mdef");
        let expected = String::from("[4m[4@mabc[mdef");
        let mut output = Vec::new();
        Win::new(&mut output, "@")
            .write_all(input.as_bytes())
            .unwrap();
        let output = String::from_utf8(output).unwrap();
        assert_eq!(output, expected);
    }

    #[test]
    fn at_end() {
        let input = String::from("[40mabc[31;40mdef");
        let expected = String::from("[4@mabc[31;4@mdef");
        let mut output = Vec::new();
        Win::new(&mut output, "@")
            .write_all(input.as_bytes())
            .unwrap();
        let output = String::from_utf8(output).unwrap();
        assert_eq!(output, expected);
    }

    #[test]
    fn color_with_4() {
        let input = String::from("[34;40mabc");
        let expected = String::from("[34;4@mabc");
        let mut output = Vec::new();
        Win::new(&mut output, "@")
            .write_all(input.as_bytes())
            .unwrap();
        let output = String::from_utf8(output).unwrap();
        assert_eq!(output, expected);
    }
}
