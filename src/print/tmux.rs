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

    fn color(&mut self, color: Color) -> std::io::Result<()> {
        match color {
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

    fn write<S: std::fmt::Display>(
        &mut self,
        fg: Option<Color>,
        bg: Option<Color>,
        txt: S,
    ) -> Result {
        match (fg, bg) {
            (None, None) => write!(self.out, "{txt}"),
            (Some(fg), None) => {
                self.out.write_all(b"#[fg=")?;
                self.color(fg)?;
                write!(self.out, "]{txt}")
            }
            (None, Some(bg)) => {
                self.out.write_all(b"#[bg=")?;
                self.color(bg)?;
                write!(self.out, "]{txt}")
            }
            (Some(Color::Reset), Some(Color::Reset)) => write!(self.out, "#[none]{txt}"),
            (Some(fg), Some(bg)) => {
                self.out.write_all(b"#[fg=")?;
                self.color(fg)?;
                self.out.write_all(b",bg=")?;
                self.color(bg)?;
                write!(self.out, "]{txt}")
            }
        }
    }
}
