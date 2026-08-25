use super::{Color, Div, Printer};

pub struct Zsh<Out> {
    out: Out,
    fg: Color,
    bg: Color,
    last_fg: Option<Color>,
    last_bg: Option<Color>,
    has_gap: bool,
}

impl<Out> Printer for Zsh<Out>
where
    Out: std::io::Write,
{
    fn fg(&mut self, color: Color) -> &mut Self {
        self.fg = color;
        self
    }

    fn bg(&mut self, color: Color) -> &mut Self {
        self.bg = color;
        self
    }

    fn div(&mut self, div: Div, into: Color) -> std::io::Result<&mut Self> {
        self.fg = self.bg;
        self.bg = into;
        self.txt(div)
    }

    fn gap(&mut self) -> std::io::Result<&mut Self> {
        if !self.has_gap {
            self.txt(" ")?;
            self.has_gap = true;
        }
        Ok(self)
    }

    fn txt<S: std::fmt::Display>(&mut self, txt: S) -> std::io::Result<&mut Self> {
        match (self.fg == self.last_fg, self.bg == self.last_bg) {
            (true, true) => write!(self.out, "{txt}")?,
            (false, true) => {
                self.out.write_all(b"%{[3")?;
                self.color(true)?;
                write!(self.out, "m%}}{txt}")?;
                self.last_fg = Some(self.fg);
            }
            (true, false) => {
                self.out.write_all(b"%{[4")?;
                self.color(false)?;
                write!(self.out, "m%}}{txt}")?;
                self.last_bg = Some(self.bg);
            }
            (false, false) => {
                match (self.fg == Color::Reset, self.bg == Color::Reset) {
                    (true, true) => write!(self.out, "%{{[;m%}}{txt}")?,
                    (false, true) => {
                        self.out.write_all(b"%{[;3")?;
                        self.color(true)?;
                        write!(self.out, "m%}}{txt}")?;
                    }
                    (true, false) => {
                        self.out.write_all(b"%{[;4")?;
                        self.color(false)?;
                        write!(self.out, "m%}}{txt}")?;
                    }
                    (false, false) => {
                        self.out.write_all(b"%{[3")?;
                        self.color(true)?;
                        self.out.write_all(b";4")?;
                        self.color(false)?;
                        write!(self.out, "m%}}{txt}")?;
                    }
                }
                self.last_fg = Some(self.fg);
                self.last_bg = Some(self.bg);
            }
        }
        self.has_gap = false;
        Ok(self)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.out.flush()
    }
}

impl<Out> Zsh<Out>
where
    Out: std::io::Write,
{
    pub fn new(out: Out) -> Self {
        Self {
            out,
            fg: Color::Reset,
            bg: Color::Reset,
            last_fg: None,
            last_bg: None,
            has_gap: false,
        }
    }

    fn color(&mut self, fg: bool) -> std::io::Result<()> {
        match if fg { self.fg } else { self.bg } {
            Color::Black => self.out.write_all(b"0"),
            Color::Red => self.out.write_all(b"1"),
            Color::Green => self.out.write_all(b"2"),
            Color::Yellow => self.out.write_all(b"3"),
            Color::Blue => self.out.write_all(b"4"),
            Color::Magenta => self.out.write_all(b"5"),
            Color::Cyan => self.out.write_all(b"6"),
            Color::White => self.out.write_all(b"7"),
            Color::Vga(c) => write!(self.out, "8;5;{c}"),
            Color::Rgb { r, g, b } => write!(self.out, "8;2;{r};{g};{b}"),
            Color::Reset => self.out.write_all(b"9"),
        }
    }
}
