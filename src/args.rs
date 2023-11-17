use crate::command;

pub fn parse() -> command::Command {
    let mut args = std::env::args();
    let bin = args.next();
    let command = args.next();

    match command.as_deref() {
        Some("r") => command::Command::Right(parse_right(args)),
        Some("l") => command::Command::Left(parse_left(args)),
        Some("t") => {
            if let Some(tmux) = args.next().map(|pwd| command::Tmux { pwd }) {
                command::Command::Tmux(tmux)
            } else {
                command::Command::Help(command::Help { bin })
            }
        }
        Some("v") => command::Command::Version,
        _ => command::Command::Help(command::Help { bin }),
    }
}

fn parse_right(args: impl Iterator<Item = String>) -> command::Right {
    let this = command::Right {
        compat: command::Compat::None,
    };

    args.filter(|s| !s.is_empty()).fold(this, |mut acc, curr| {
        if curr == "-z" {
            acc.compat = command::Compat::Zsh;
        } else if let Some(sub) = curr.strip_prefix("-w").filter(|s| !s.is_empty()) {
            acc.compat = command::Compat::Win(String::from(sub));
        }
        acc
    })
}

fn parse_left(args: impl Iterator<Item = String>) -> command::Left {
    let this = command::Left {
        host: None,
        error: false,
        jobs: false,
        long: false,
        compat: command::Compat::None,
    };

    args.filter(|s| !s.is_empty()).fold(this, |mut acc, curr| {
        if curr == "-e" {
            acc.error = true;
        } else if curr == "-j" {
            acc.jobs = true;
        } else if curr == "-l" {
            acc.long = true;
        } else if curr == "-z" {
            acc.compat = command::Compat::Zsh;
        } else if let Some(sub) = curr.strip_prefix("-w").filter(|s| !s.is_empty()) {
            acc.compat = command::Compat::Win(String::from(sub));
        } else {
            acc.host = Some(curr);
        }
        acc
    })
}

#[cfg(test)]
mod tests {
    use crate::command;

    #[test]
    fn parse_right_empty() {
        assert_eq!(
            command::Right {
                compat: command::Compat::None,
            },
            super::parse_right(std::iter::empty())
        );
    }

    #[test]
    fn parse_right_no_match() {
        assert_eq!(
            command::Right {
                compat: command::Compat::None,
            },
            super::parse_right(["bla", "-w", "ble"].map(String::from).into_iter())
        );
    }

    #[test]
    fn parse_right_zsh() {
        assert_eq!(
            command::Right {
                compat: command::Compat::Zsh,
            },
            super::parse_right(["-z"].map(String::from).into_iter())
        );
    }

    #[test]
    fn parse_right_win() {
        assert_eq!(
            command::Right {
                compat: command::Compat::Win(String::from("2")),
            },
            super::parse_right(["-w2"].map(String::from).into_iter())
        );
    }

    #[test]
    fn parse_right_take_last() {
        assert_eq!(
            command::Right {
                compat: command::Compat::Zsh,
            },
            super::parse_right(["-w12", "-z"].map(String::from).into_iter())
        );
    }

    #[test]
    fn parse_left_empty() {
        assert_eq!(
            command::Left {
                host: None,
                error: false,
                jobs: false,
                long: false,
                compat: command::Compat::None,
            },
            super::parse_left(std::iter::empty())
        );
    }

    #[test]
    fn parse_left_win() {
        assert_eq!(
            command::Left {
                host: None,
                error: false,
                jobs: false,
                long: false,
                compat: command::Compat::Win(String::from("yo")),
            },
            super::parse_left(["-wyo"].map(String::from).into_iter())
        );
    }

    #[test]
    fn parse_left_invalid_win() {
        assert_eq!(
            command::Left {
                host: Some(String::from("-w")),
                error: false,
                jobs: false,
                long: false,
                compat: command::Compat::None,
            },
            super::parse_left(["-w"].map(String::from).into_iter())
        );
    }

    #[test]
    fn parse_left_take_last() {
        assert_eq!(
            command::Left {
                host: Some(String::from("last")),
                error: false,
                jobs: false,
                long: false,
                compat: command::Compat::None,
            },
            super::parse_left(["first", "second", "last"].map(String::from).into_iter())
        );
    }

    #[test]
    fn parse_left_all_options() {
        assert_eq!(
            command::Left {
                host: Some(String::from("last")),
                error: true,
                jobs: true,
                long: true,
                compat: command::Compat::Zsh,
            },
            super::parse_left(
                ["first", "", "-3", "-e", "second", "-j", "last", "-l", "-e", "-j", "-z", ""]
                    .map(String::from)
                    .into_iter()
            )
        );
    }
}
