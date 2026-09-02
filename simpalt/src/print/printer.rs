use super::{Color, Div, Result};

#[derive(Clone, Default)]
pub struct State {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub last_fg: Option<Color>,
    pub last_bg: Option<Color>,
    pub has_gap: bool,
}

#[allow(private_bounds, clippy::missing_errors_doc)]
pub trait Printer: sealed::Printer {
    #[inline]
    fn flush(&mut self) -> Result {
        sealed::Printer::flush(self)
    }

    #[inline]
    fn fg(&mut self, color: Color) -> &mut Self {
        self.state().fg = Some(color);
        self
    }

    #[inline]
    fn bg(&mut self, color: Color) -> &mut Self {
        self.state().bg = Some(color);
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
        let State { fg, last_bg, .. } = *self.state();

        match div {
            Div::ChevronLeft | Div::SlantTopLeft | Div::SlantBottomLeft => {
                if let Some(last_bg) = last_bg {
                    if last_bg != into {
                        self.write(Some(last_bg), Some(into), div)?;
                        let state = self.state();
                        state.has_gap = false;
                        state.last_fg = Some(last_bg);
                        state.last_bg = Some(into);
                        state.bg = Some(into);
                    }
                } else {
                    self.state().bg = Some(into);
                }
            }
            Div::ChevronRight | Div::SlantTopRight | Div::SlantBottomRight => {
                self.write((into != fg).then_some(into), None, div)?;
                let state = self.state();
                state.has_gap = false;
                state.last_fg = Some(into);
                state.bg = Some(into);
            }
        }

        Ok(self)
    }

    #[inline]
    fn gap(&mut self) -> Result<&mut Self> {
        let State {
            fg,
            bg,
            last_fg,
            last_bg,
            has_gap,
            ..
        } = *self.state();
        if !has_gap {
            if Color::Reset == fg && fg == bg {
                self.write(fg, bg, " ")?;
            } else {
                self.write(
                    fg.filter(|fg| *fg != last_fg),
                    bg.filter(|bg| *bg != last_bg),
                    " ",
                )?;
            }

            let state = self.state();

            state.last_fg = fg;
            state.last_bg = bg;
            state.has_gap = true;
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
        let State {
            fg,
            bg,
            last_fg,
            last_bg,
            ..
        } = *self.state();

        if Color::Reset == fg && fg == bg {
            self.write(fg, bg, txt)?;
        } else {
            self.write(
                fg.filter(|fg| *fg != last_fg),
                bg.filter(|bg| *bg != last_bg),
                txt,
            )?;
        }

        let state = self.state();
        state.last_fg = fg;
        state.last_bg = bg;
        state.has_gap = false;

        Ok(self)
    }
}

pub mod sealed {

    use super::{Color, Result, State};

    pub(crate) trait Printer {
        fn flush(&mut self) -> Result;
        fn state(&mut self) -> &mut State;
        fn write<S: std::fmt::Display>(
            &mut self,
            fg: Option<Color>,
            bg: Option<Color>,
            txt: S,
        ) -> Result;
    }

    impl<P> super::Printer for P where P: Printer {}
}
