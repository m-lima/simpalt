use crate::Result;

#[derive(Default)]
pub struct Args {
    pub show_version: bool,
    pub mode: Option<Mode>,
}

pub fn parse(mut args: std::env::Args) -> Result<Option<Args>> {
    let mut state = Args::default();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "h" => return Ok(None),
            "v" => {
                if state.show_version {
                    return Err(std::io::Error::other("Version specified twice"));
                }
                state.show_version = true;
            }
            "-m" => {
                if state.mode.is_some() {
                    return Err(std::io::Error::other("Mode specified twice"));
                }
                state.mode = Some(set_mode(&mut args)?);
            }
            _ => return Err(std::io::Error::other("Invalid argument")),
        }
    }

    Ok(Some(state))
}

fn set_mode(args: &mut std::env::Args) -> Result<Mode> {
    match args.next().as_deref() {
        Some("a") => Ok(Mode::Ansi),
        Some("z") => Ok(Mode::Zsh),
        Some("w") => match args.next() {
            Some(sub) => Ok(Mode::Win(sub)),
            _ => Err(std::io::Error::other("Invalid substitution")),
        },
        _ => Err(std::io::Error::other("Invalid mode")),
    }
}

#[derive(Clone, Eq, PartialEq)]
pub enum Mode {
    Ansi,
    Zsh,
    Win(String),
}
