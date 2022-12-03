use crate::Result;

// TODO: Can ZSH avoid regex and just stream the output with the added chars?
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
}
