use super::{
    Color, Result,
    printer::{State, sealed},
};

pub struct Zsh<Out> {
    out: Out,
    state: State,
}

impl<Out> Zsh<Out>
where
    Out: std::io::Write,
{
    pub fn new(out: Out) -> Self {
        Self {
            out,
            state: State::default(),
        }
    }
}

impl<Out> sealed::Printer for Zsh<Out>
where
    Out: std::io::Write,
{
    fn flush(&mut self) -> Result {
        self.out.flush()
    }

    fn state(&mut self) -> &mut State {
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
            (Some(fg), None) => write!(self.out, "%{{[3{fg}m%}}{txt}"),
            (None, Some(bg)) => write!(self.out, "%{{[4{bg}m%}}{txt}"),
            (Some(Color::Reset), Some(Color::Reset)) => write!(self.out, "%{{[m%}}{txt}"),
            (Some(fg), Some(Color::Reset)) => write!(self.out, "%{{[;3{fg}m%}}{txt}"),
            (Some(Color::Reset), Some(bg)) => write!(self.out, "%{{[;4{bg}m%}}{txt}"),
            (Some(fg), Some(bg)) => write!(self.out, "%{{[3{fg};4{bg}m%}}{txt}"),
        }
    }
}
