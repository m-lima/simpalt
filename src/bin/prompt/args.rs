use crate::Result;

#[derive(Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct Args {
    pub show_version: bool,
    pub long: bool,
    pub error: bool,
    pub jobs: bool,
    pub symbol: Option<String>,
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
            "-l" => {
                if state.long {
                    return Err(std::io::Error::other("Long flag specified twice"));
                }
                state.long = true;
            }
            "-s" => {
                if state.symbol.is_some() {
                    return Err(std::io::Error::other("Symbol specified twice"));
                }
                if let Some(arg) = args.next() {
                    state.symbol = Some(arg);
                } else {
                    return Err(std::io::Error::other("Invalid symbol argument"));
                }
            }
            "-e" => {
                if state.error {
                    return Err(std::io::Error::other("Error flag specified twice"));
                }
                state.error = true;
            }
            "-j" => {
                if state.jobs {
                    return Err(std::io::Error::other("Jobs flag specified twice"));
                }
                state.jobs = true;
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
