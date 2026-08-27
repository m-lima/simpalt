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

    writeln!(out, "Usage: {bin} [ACTION] [FLAGS...]")?;
    writeln!(out)?;
    writeln!(out, "Action:")?;
    writeln!(out, "  h         Print this help message")?;
    writeln!(out, "  v         Print the version")?;
    writeln!(out)?;
    writeln!(out, "Flags:")?;
    writeln!(out, "  -m <MODE> Select in which mode to print")?;
    writeln!(out, "  -l        Print using the long format")?;
    writeln!(out, "  -e        Last command was an error")?;
    writeln!(out, "  -j        There are background processess running")?;
    writeln!(out, "  -s <HOST> Host symbol to use")?;
    writeln!(out)?;
    writeln!(out, "Modes:")?;
    writeln!(out, "  a         Print in ANSI mode (default)")?;
    writeln!(out, "  z         Print in ZSH mode")?;
    writeln!(out, "  w <SUB>   Print in Windows mode")?;
    writeln!(out, "            SUB will replace the black background")
}
