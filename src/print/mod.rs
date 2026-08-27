mod ansi;
mod fmt;
mod printer;
mod tmux;
mod win;
mod zsh;

pub use fmt::{Color, Div, Symbol};
pub use printer::Printer;

pub use ansi::Ansi;
pub use tmux::Tmux;
pub use win::Win;
pub use zsh::Zsh;

type Result<T = ()> = std::io::Result<T>;
