use crate::Result;

#[derive(Default)]
pub struct Args {
    pub show_version: bool,
    pub pwd: Option<String>,
}

pub fn parse(args: std::env::Args) -> Result<Option<Args>> {
    let mut state = Args::default();

    for arg in args {
        if arg == "-h" {
            return Ok(None);
        }

        if arg == "-v" {
            if state.show_version {
                return Err(std::io::Error::other("Version specified twice"));
            }
            state.show_version = true;
        } else {
            if state.pwd.is_some() {
                return Err(std::io::Error::other("Path specified twice"));
            }
            state.pwd = Some(arg);
        }
    }

    Ok(Some(state))
}
