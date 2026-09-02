use crate::Result;

#[derive(Clone)]
pub enum Action {
    Help,
    Version,
    Status(String),
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
        "s" => match args.next() {
            Some(path) => {
                if args.next().is_none() {
                    Ok(Action::Status(path))
                } else {
                    Err(std::io::Error::other("Action 's' takes no options"))
                }
            }
            None => Err(std::io::Error::other("Missing PATH argument")),
        },
        a => Err(std::io::Error::other(format!("Unknown action '{a}'"))),
    }
}
