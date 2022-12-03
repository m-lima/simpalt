use crate::Result;

pub fn zsh(mut output: impl std::io::Write, mut input: impl std::io::Read) -> Result {
    let mut buffer = {
        let buffer = std::mem::MaybeUninit::<[u8; 1024]>::uninit();
        unsafe { buffer.assume_init() }
    };

    let mut bytes = input.read(&mut buffer)?;
    while bytes > 0 {
        let mut cursor = 0;
        let mut escaped = None;

        for (i, byte) in buffer[..bytes].iter().copied().enumerate() {
            match escaped {
                Some(true) => {
                    if byte == b'' {
                        escaped = Some(false);
                    } else {
                        if cursor < i {
                            output.write_all(&buffer[cursor..i])?;
                            cursor = i;
                        }
                        output.write_all(b"%}")?;
                        escaped = None;
                    }
                }
                Some(false) => {
                    escaped = Some(byte == b'm');
                }
                None => {
                    if byte == b'' {
                        if cursor < i {
                            output.write_all(&buffer[cursor..i])?;
                            cursor = i;
                        }
                        output.write_all(b"%{")?;
                        escaped = Some(false);
                    }
                }
            }
        }

        if cursor < bytes {
            output.write_all(&buffer[cursor..bytes])?;
        }

        bytes = input.read(&mut buffer)?;
    }

    Ok(())
}

pub fn win(
    mut output: impl std::io::Write,
    mut input: impl std::io::Read,
    replacement: &str,
) -> Result {
    let mut buffer = {
        let buffer = std::mem::MaybeUninit::<[u8; 1024]>::uninit();
        unsafe { buffer.assume_init() }
    };

    let mut bytes = input.read(&mut buffer)?;
    while bytes > 0 {
        let mut cursor = 0;
        let mut state = None;

        for (i, byte) in buffer[..bytes].iter().copied().enumerate() {
            if byte == b'm' {
                state = None;
            } else {
                match state {
                    Some((true, true)) => {
                        if byte == b'0' {
                            if cursor < i {
                                output.write_all(&buffer[cursor..i])?;
                                cursor = i + 1;
                            }
                            output.write_all(replacement.as_bytes())?;
                        }
                        state = Some((byte == b';' || byte == b'[', false));
                    }
                    Some((true, false)) => {
                        if byte == b'4' {
                            state = Some((true, true));
                        }
                    }
                    Some((false, _)) => {
                        if byte == b'[' || byte == b';' {
                            state = Some((true, false));
                        }
                    }
                    None => {
                        if byte == b'' {
                            state = Some((false, false));
                        }
                    }
                }
            }
        }

        if cursor < bytes {
            output.write_all(&buffer[cursor..bytes])?;
        }

        bytes = input.read(&mut buffer)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::test;

    #[test]
    fn zsh() {
        use super::zsh;

        let input = String::from("abc[31mdef");
        let output = String::from("abc%{[31m%}def");
        assert_eq!(test(|s| zsh(s, input.as_bytes())), output);

        let input = String::from("[38;5;2mabc[mdef");
        let output = String::from("%{[38;5;2m%}abc%{[m%}def");
        assert_eq!(test(|s| zsh(s, input.as_bytes())), output);

        let input = String::from("[38;5;2m[41mabc[49mdef");
        let output = String::from("%{[38;5;2m[41m%}abc%{[49m%}def");
        assert_eq!(test(|s| zsh(s, input.as_bytes())), output);
    }

    #[test]
    fn win() {
        use super::win;

        let input = String::from("abc[31mdef");
        let output = String::from("abc[31mdef");
        assert_eq!(test(|s| win(s, input.as_bytes(), "@")), output);

        let input = String::from("[40mabc[mdef");
        let output = String::from("[4@mabc[mdef");
        assert_eq!(test(|s| win(s, input.as_bytes(), "@")), output);

        let input = String::from("[41mabc[mdef");
        let output = String::from("[41mabc[mdef");
        assert_eq!(test(|s| win(s, input.as_bytes(), "@")), output);

        let input = String::from("[40m;40[40abc[mdef");
        let output = String::from("[4@m;40[40abc[mdef");
        assert_eq!(test(|s| win(s, input.as_bytes(), "@")), output);

        let input = String::from("[31;40mabc[mdef");
        let output = String::from("[31;4@mabc[mdef");
        assert_eq!(test(|s| win(s, input.as_bytes(), "@")), output);

        let input = String::from("[31m[40mabc[mdef");
        let output = String::from("[31m[4@mabc[mdef");
        assert_eq!(test(|s| win(s, input.as_bytes(), "@")), output);

        let input = String::from("[4m[40mabc[mdef");
        let output = String::from("[4m[4@mabc[mdef");
        assert_eq!(test(|s| win(s, input.as_bytes(), "@")), output);

        let input = String::from("[40mabc[31;40mdef");
        let output = String::from("[4@mabc[31;4@mdef");
        assert_eq!(test(|s| win(s, input.as_bytes(), "@")), output);

        let input = String::from("[34;40mabc");
        let output = String::from("[34;4@mabc");
        assert_eq!(test(|s| win(s, input.as_bytes(), "@")), output);
    }
}
