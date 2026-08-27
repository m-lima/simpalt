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
