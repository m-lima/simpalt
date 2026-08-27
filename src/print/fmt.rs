use crate::git::long as git;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Vga(u8),
    Rgb { r: u8, g: u8, b: u8 },
    Reset,
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write as _;

        match self {
            Color::Black => f.write_char('0'),
            Color::Red => f.write_char('1'),
            Color::Green => f.write_char('2'),
            Color::Yellow => f.write_char('3'),
            Color::Blue => f.write_char('4'),
            Color::Magenta => f.write_char('5'),
            Color::Cyan => f.write_char('6'),
            Color::White => f.write_char('7'),
            Color::Vga(c) => {
                f.write_str("8;5;")?;
                c.fmt(f)
            }
            Color::Rgb { r, g, b } => {
                f.write_str("8;2;")?;
                r.fmt(f)?;
                f.write_char(';')?;
                g.fmt(f)?;
                f.write_char(';')?;
                b.fmt(f)
            }
            Color::Reset => f.write_char('9'),
        }
    }
}

impl PartialEq<Option<Color>> for Color {
    fn eq(&self, other: &Option<Color>) -> bool {
        if let Some(other) = other {
            self.eq(other)
        } else {
            false
        }
    }
}

#[derive(Copy, Clone)]
pub enum Symbol {
    // Processes
    Error,
    Jobs,

    // Environment
    Package,
    Direnv,
    Flake,
    Python,

    // Media
    Pause,
    Play,

    // Git
    New,
    Branch,
    Ref,
    Merge,
    Bisect,
    Rebase,
    Cherry,
    Revert,
    Mailbox,
    Ahead,
    Behind,
    Local,
    Gone,
    Warn,

    // Separators
    ChevronRight,
    ChevronLeft,
    SlantTop,
    SlantBottom,
}

impl Symbol {
    #[must_use]
    pub const fn str(self) -> &'static str {
        match self {
            Symbol::Error => "✘",
            Symbol::Jobs => "",
            Symbol::Package => "󰏓",
            Symbol::Direnv => "",
            Symbol::Flake => "󱄅",
            Symbol::Python => "󰌠",
            Symbol::Pause => "",
            Symbol::Play => "",
            Symbol::New => "",
            Symbol::Branch => "",
            Symbol::Ref => "➦",
            Symbol::Merge => "",
            Symbol::Bisect => "",
            Symbol::Rebase => "",
            Symbol::Cherry => "",
            Symbol::Revert => "",
            Symbol::Mailbox => "",
            Symbol::Ahead => "󰁝",
            Symbol::Behind => "󰁅",
            Symbol::Local => "󰁂",
            Symbol::Gone => "󰁜",
            Symbol::Warn => "󱈸",
            Symbol::ChevronRight => "",
            Symbol::ChevronLeft => "",
            Symbol::SlantTop => "",
            Symbol::SlantBottom => "╱",
        }
    }
}

impl From<git::Pending> for Symbol {
    fn from(value: git::Pending) -> Self {
        match value {
            git::Pending::Merge => Self::Merge,
            git::Pending::Revert => Self::Revert,
            git::Pending::Cherry => Self::Cherry,
            git::Pending::Bisect => Self::Bisect,
            git::Pending::Rebase => Self::Rebase,
            git::Pending::Mailbox => Self::Mailbox,
        }
    }
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.str())
    }
}

#[derive(Copy, Clone)]
pub enum Div {
    ChevronLeft,
    ChevronRight,
    SlantTopLeft,
    SlantTopRight,
    SlantBottomLeft,
    SlantBottomRight,
}

impl Div {
    #[must_use]
    pub const fn str(self) -> &'static str {
        match self {
            Div::ChevronLeft => "",
            Div::ChevronRight => "",
            Div::SlantTopLeft => "",
            Div::SlantTopRight => "",
            Div::SlantBottomLeft => "",
            Div::SlantBottomRight => "",
        }
    }
}

impl std::fmt::Display for Div {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.str())
    }
}
