use crate::Result;

pub fn render(buffer: &mut Vec<u8>) -> Result {
    let mut printer = simpalt::print::Ansi::new(buffer);
    simpalt_status::git::render(&mut printer, "/host")?;
    #[cfg(feature = "media")]
    simpalt_status::media::render(&mut printer)?;
    simpalt_status::time::render(&mut printer)?;
    simpalt::print::Printer::flush(&mut printer)
}
