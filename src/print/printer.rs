use super::{Color, Div, Result};

#[derive(Copy, Clone)]
pub struct State {
    pub fg: Color,
    pub bg: Color,
    pub last_fg: Option<Color>,
    pub last_bg: Option<Color>,
    pub has_gap: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            fg: Color::Reset,
            bg: Color::Reset,
            last_fg: None,
            last_bg: None,
            has_gap: false,
        }
    }
}

#[allow(private_bounds, clippy::missing_errors_doc)]
pub trait Printer: sealed::Printer {
    #[inline]
    fn flush(&mut self) -> Result {
        sealed::Printer::flush(self)
    }

    #[inline]
    fn fg(&mut self, color: Color) -> &mut Self {
        self.state().fg = color;
        self
    }

    #[inline]
    fn bg(&mut self, color: Color) -> &mut Self {
        self.state().bg = color;
        self
    }

    #[inline]
    fn invalidate(&mut self) -> &mut Self {
        let state = self.state();
        state.last_fg = None;
        state.last_bg = None;
        self
    }

    fn div(&mut self, div: Div, into: Color) -> Result<&mut Self> {
        if into == self.state().last_bg {
            Ok(self)
        } else {
            self.state().bg = into;
            if let Some(last_bg) = self.state().last_bg {
                let old_fg = self.state().fg;
                self.state().fg = last_bg;
                self.txt(div)?;
                self.state().fg = old_fg;
                Ok(self)
            } else {
                self.txt("")
            }
        }
    }

    #[inline]
    fn gap(&mut self) -> Result<&mut Self> {
        if !self.state().has_gap {
            self.txt(" ")?;
            self.state().has_gap = true;
        }
        Ok(self)
    }

    #[inline]
    fn txt_gap<S: std::fmt::Display>(&mut self, txt: S) -> Result<&mut Self> {
        self.gap()?;
        self.txt(txt)?;
        self.gap()
    }

    fn txt<S: std::fmt::Display>(&mut self, txt: S) -> Result<&mut Self> {
        let mut state = *self.state();

        match (state.fg == state.last_fg, state.bg == state.last_bg) {
            (true, true) => {
                self.write_plain(txt)?;
            }
            (false, true) => {
                if state.fg == Color::Reset && state.bg == Color::Reset {
                    self.write_reset(txt)?;
                } else {
                    self.write_fg(txt)?;
                }
                state.last_fg = Some(state.fg);
            }
            (true, false) => {
                if state.fg == Color::Reset && state.bg == Color::Reset {
                    self.write_reset(txt)?;
                } else {
                    self.write_bg(txt)?;
                }
                state.last_bg = Some(state.bg);
            }
            (false, false) => {
                match (state.fg == Color::Reset, state.bg == Color::Reset) {
                    (true, true) => {
                        self.write_reset(txt)?;
                    }
                    (false, true) => {
                        self.write_reset_and_fg(txt)?;
                    }
                    (true, false) => {
                        self.write_reset_and_bg(txt)?;
                    }
                    (false, false) => {
                        self.write(txt)?;
                    }
                }
                state.last_fg = Some(state.fg);
                state.last_bg = Some(state.bg);
            }
        }
        state.has_gap = false;

        *self.state() = state;
        Ok(self)
    }
}

pub mod sealed {
    use super::{Result, State};

    pub(crate) trait Printer {
        fn flush(&mut self) -> Result;
        fn state(&mut self) -> &mut State;
        fn write_plain<S: std::fmt::Display>(&mut self, txt: S) -> Result;
        fn write_reset<S: std::fmt::Display>(&mut self, txt: S) -> Result;
        fn write_fg<S: std::fmt::Display>(&mut self, txt: S) -> Result;
        fn write_bg<S: std::fmt::Display>(&mut self, txt: S) -> Result;
        fn write_reset_and_fg<S: std::fmt::Display>(&mut self, txt: S) -> Result;
        fn write_reset_and_bg<S: std::fmt::Display>(&mut self, txt: S) -> Result;
        fn write<S: std::fmt::Display>(&mut self, txt: S) -> Result;
    }

    impl<P> super::Printer for P where P: Printer {}
}
