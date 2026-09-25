use crate::Result;

pub fn render(buffer: &mut Vec<u8>) -> Result {
    let mut printer = simpalt::print::Ansi::new(buffer);
    simpalt::print::Printer::txt(&mut printer, "Status")?;
    simpalt::print::Printer::flush(&mut printer)
}
