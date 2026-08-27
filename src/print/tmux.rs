use super::{
    Color, Result,
    printer::{State, sealed},
};

pub struct Tmux<Out> {
    out: Out,
    state: State,
}

impl<Out> Tmux<Out>
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

impl<Out> sealed::Printer for Tmux<Out>
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
        write!(self.out, "#[none]{txt}")
    }

    fn write_fg<S: std::fmt::Display>(&mut self, txt: S) -> Result {
        self.out.write_all(b"#[fg=")?;
        self.color(true)?;
        write!(self.out, "]{txt}")
    }

    fn write_bg<S: std::fmt::Display>(&mut self, txt: S) -> Result {
        self.out.write_all(b"#[bg=")?;
        self.color(false)?;
        write!(self.out, "]{txt}")
    }

    #[inline]
    fn write_reset_and_fg<S: std::fmt::Display>(&mut self, txt: S) -> Result {
        self.write(txt)
    }

    #[inline]
    fn write_reset_and_bg<S: std::fmt::Display>(&mut self, txt: S) -> Result {
        self.write(txt)
    }

    fn write<S: std::fmt::Display>(&mut self, txt: S) -> Result {
        self.out.write_all(b"#[fg=")?;
        self.color(true)?;
        self.out.write_all(b",bg=")?;
        self.color(false)?;
        write!(self.out, "]{txt}")
    }
}
