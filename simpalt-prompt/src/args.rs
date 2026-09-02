use crate::Result;

#[derive(Clone)]
pub enum Action {
    Help,
    Version,
    Left(LeftOptions),
    Right(RightOptions),
}

#[derive(Default, Clone)]
pub struct LeftOptions {
    pub long: bool,
    pub error: bool,
    pub jobs: bool,
    pub symbol: Option<String>,
    pub mode: Option<Mode>,
}

#[derive(Default, Clone)]
pub struct RightOptions {
    pub mode: Option<Mode>,
}

pub fn parse(mut args: std::env::Args) -> Result<Action> {
    let Some(action) = args.next() else {
        return Err(std::io::Error::other("No action provided"));
    };

    match action.as_str() {
        "h" | "-h" => Ok(Action::Help),
        "v" => {
            if args.next().is_none() {
                Ok(Action::Version)
            } else {
                Err(std::io::Error::other("Action 'v' takes no options"))
            }
        }
        "l" => Ok(Action::Left(parse_left(&mut args)?)),
        "r" => Ok(Action::Right(parse_right(&mut args)?)),
        a => Err(std::io::Error::other(format!("Unknown action '{a}'"))),
    }
}

fn parse_left(args: &mut std::env::Args) -> Result<LeftOptions> {
    let mut options = LeftOptions::default();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-m" => {
                if options.mode.is_some() {
                    return Err(std::io::Error::other("Mode specified twice"));
                }
                options.mode = Some(set_mode(args)?);
            }
            "-l" => {
                if options.long {
                    return Err(std::io::Error::other("Long flag specified twice"));
                }
                options.long = true;
            }
            "-s" => {
                if options.symbol.is_some() {
                    return Err(std::io::Error::other("Symbol specified twice"));
                }
                if let Some(arg) = args.next() {
                    options.symbol = Some(arg);
                } else {
                    return Err(std::io::Error::other("Invalid symbol argument"));
                }
            }
            "-e" => {
                if options.error {
                    return Err(std::io::Error::other("Error flag specified twice"));
                }
                options.error = true;
            }
            "-j" => {
                if options.jobs {
                    return Err(std::io::Error::other("Jobs flag specified twice"));
                }
                options.jobs = true;
            }
            _ => return Err(std::io::Error::other("Invalid option")),
        }
    }

    Ok(options)
}

fn parse_right(args: &mut std::env::Args) -> Result<RightOptions> {
    let mut options = RightOptions::default();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-m" => {
                if options.mode.is_some() {
                    return Err(std::io::Error::other("Mode specified twice"));
                }
                options.mode = Some(set_mode(args)?);
            }
            _ => return Err(std::io::Error::other("Invalid option")),
        }
    }

    Ok(options)
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
