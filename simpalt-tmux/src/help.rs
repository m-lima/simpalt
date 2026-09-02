pub fn render<Out>(mut out: Out, bin: Option<&String>) -> crate::Result
where
    Out: std::io::Write,
{
    let bin = bin
        .map(std::path::PathBuf::from)
        .and_then(|p| {
            p.file_name()
                .and_then(std::ffi::OsStr::to_str)
                .map(String::from)
        })
        .unwrap_or_else(|| String::from(env!("CARGO_BIN_NAME")));

    writeln!(out, "Usage: {bin} <ACTION> [OPTIONS...]")?;
    writeln!(out)?;
    writeln!(out, "Actions:")?;
    writeln!(out, "   h | -h   Print this help message")?;
    writeln!(out, "   v        Print the version")?;
    writeln!(out, "   s <PATH> Print the status for the given path")
}
