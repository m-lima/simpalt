use super::{
    Color, Result,
    printer::{State, sealed},
};

pub struct Ansi<Out> {
    out: Out,
    state: State,
}

impl<Out> Ansi<Out>
where
    Out: std::io::Write,
{
    pub fn new(out: Out) -> Self {
        Self {
            out,
            state: State::default(),
        }
    }

    fn color(&mut self, fg: bool) -> std::io::Result<()> {
        match if fg { self.state.fg } else { self.state.bg } {
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

impl<Out> sealed::Printer for Ansi<Out>
where
    Out: std::io::Write,
{
    fn flush(&mut self) -> Result {
        self.out.flush()
    }

    fn state(&mut self) -> &mut super::printer::State {
        &mut self.state
    }

    fn write_plain<S: std::fmt::Display>(&mut self, txt: S) -> Result {
        write!(self.out, "{txt}")
    }

    fn write_reset<S: std::fmt::Display>(&mut self, txt: S) -> Result {
        write!(self.out, "[m{txt}")
    }

    fn write_fg<S: std::fmt::Display>(&mut self, txt: S) -> Result {
        self.out.write_all(b"[3")?;
        self.color(true)?;
        write!(self.out, "m{txt}")
    }

    fn write_bg<S: std::fmt::Display>(&mut self, txt: S) -> Result {
        self.out.write_all(b"[4")?;
        self.color(false)?;
        write!(self.out, "m{txt}")
    }

    fn write_reset_and_fg<S: std::fmt::Display>(&mut self, txt: S) -> Result {
        self.out.write_all(b"[;3")?;
        self.color(true)?;
        write!(self.out, "m{txt}")
    }

    fn write_reset_and_bg<S: std::fmt::Display>(&mut self, txt: S) -> Result {
        self.out.write_all(b"[;4")?;
        self.color(false)?;
        write!(self.out, "m{txt}")
    }

    fn write<S: std::fmt::Display>(&mut self, txt: S) -> Result {
        self.out.write_all(b"[3")?;
        self.color(true)?;
        self.out.write_all(b";4")?;
        self.color(false)?;
        write!(self.out, "m{txt}")
    }
}
