use crate::Result;

pub fn render<Out>(mut out: Out) -> Result
where
    Out: std::io::Write,
{
    writeln!(out, env!("CARGO_PKG_VERSION"))
}
