mod help;
mod left;
mod right;
mod tmux;
mod version;

pub use help::Args as Help;
pub use left::Args as Left;
pub use right::Args as Right;
pub use tmux::Args as Tmux;

#[derive(Debug, Eq, PartialEq)]
pub enum Command {
    Right(Right),
    Left(Left),
    Tmux(Tmux),
    Version,
    Help(Help),
}

impl Command {
    pub fn run<Out>(self, out: Out) -> crate::Result
    where
        Out: std::io::Write,
    {
        match self {
            Self::Right(args) => right::render(out, args),
            Self::Left(args) => left::render(out, args),
            Self::Tmux(args) => tmux::render(out, args),
            Self::Version => version::render(out),
            Self::Help(args) => help::render(out, args),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Compat {
    None,
    Zsh,
    Win(String),
}
