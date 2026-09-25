use crate::Result;

pub fn render<P>(mut printer: P, pwd: &str) -> Result
where
    P: simpalt::print::Printer,
{
    simpalt_status::git::render(&mut printer, pwd)?;
    #[cfg(feature = "media")]
    simpalt_status::media::render(&mut printer)?;
    simpalt_status::time::render(&mut printer)?;

    printer.flush()
}
