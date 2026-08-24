use super::{Color, Div, Printer};

pub struct Tmux<Out> {
    out: Out,
    fg: Color,
    bg: Color,
    last_fg: Color,
    last_bg: Color,
}

impl<Out> Printer for Tmux<Out>
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

    fn txt<S: std::fmt::Display>(&mut self, txt: S) -> std::io::Result<&mut Self> {
        match (self.fg == self.last_fg, self.bg == self.last_bg) {
            (true, true) => write!(self.out, "{txt}")?,
            (false, true) => {
                self.out.write_all(b"#[fg=")?;
                self.color(true)?;
                write!(self.out, "]{txt}")?;
                self.last_fg = self.fg;
            }
            (true, false) => {
                self.out.write_all(b"#[bg=")?;
                self.color(false)?;
                write!(self.out, "]{txt}")?;
                self.last_bg = self.bg;
            }
            (false, false) => {
                if self.fg == Color::Reset && self.bg == Color::Reset {
                    write!(self.out, "#[none]{txt}")?;
                } else {
                    self.out.write_all(b"#[fg=")?;
                    self.color(true)?;
                    self.out.write_all(b",bg=")?;
                    self.color(false)?;
                    write!(self.out, "]{txt}")?;
                }
                self.last_fg = self.fg;
                self.last_bg = self.bg;
            }
        }
        Ok(self)
    }
}

impl<Out> Tmux<Out>
where
    Out: std::io::Write,
{
    fn color(&mut self, fg: bool) -> std::io::Result<()> {
        match if fg { self.fg } else { self.bg } {
            Color::Black => self.out.write_all(b"black"),
            Color::Red => self.out.write_all(b"red"),
            Color::Green => self.out.write_all(b"green"),
            Color::Yellow => self.out.write_all(b"yellow"),
            Color::Blue => self.out.write_all(b"blue"),
            Color::Magenta => self.out.write_all(b"magenta"),
            Color::Cyan => self.out.write_all(b"cyan"),
            Color::White => self.out.write_all(b"white"),
            Color::Vga(c) => write!(self.out, "colour{c}"),
            Color::Rgb { r, g, b } => write!(self.out, "#{r:02x}{g:02x}{b:02x}"),
            Color::Reset => self.out.write_all(b"default"),
        }
    }
}
