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

    writeln!(out, "Usage: {bin} [ACTION] [FLAG]")?;
    writeln!(out)?;
    writeln!(out, "Action:")?;
    writeln!(out, "  h         Print this help message")?;
    writeln!(out, "  v         Print the version")?;
    writeln!(out)?;
    writeln!(out, "Flags:")?;
    writeln!(out, "  -m <MODE> Select in which mode to print")?;
    writeln!(out)?;
    writeln!(out, "Modes:")?;
    writeln!(out, "  a         Print in ANSI mode (default)")?;
    writeln!(out, "  z         Print in ZSH mode")?;
    writeln!(out, "  w <SUB>   Print in Windows mode")?;
    writeln!(out, "            SUB will replace the black background")
}
