use crate::Result;

#[derive(Debug, Eq, PartialEq)]
pub struct Args {
    pub bin: Option<String>,
}

pub fn render<Out>(mut out: Out, args: Args) -> Result
where
    Out: std::io::Write,
{
    let bin = args
        .bin
        .map(std::path::PathBuf::from)
        .and_then(|p| {
            p.file_name()
                .and_then(std::ffi::OsStr::to_str)
                .map(String::from)
        })
        .unwrap_or_else(|| String::from(env!("CARGO_BIN_NAME")));

    writeln!(out, "Usage: {bin} <COMMAND>")?;
    writeln!(out)?;
    writeln!(out, "Commands:")?;
    writeln!(out, "  c       Compatibility layer")?;
    writeln!(out, "  r       Generate right side prompt")?;
    writeln!(out, "  l       Generate left side prompt")?;
    writeln!(out, "  t       Generate tmux right side status")?;
    writeln!(out, "  v       Print the current version")?;
    writeln!(out, "  h       Show this help message")?;
    writeln!(out)?;
    writeln!(out, "Arguments for `r` command:")?;
    writeln!(out, "  -z      Print escape codes compatible with zsh")?;
    writeln!(out, "  -w<SUB> Replace black background with SUB")?;
    writeln!(out)?;
    writeln!(out, "Arguments for `l` command:")?;
    writeln!(out, "  HOST    Symbol to be used as host (can be escaped)",)?;
    writeln!(out, "  -e      Last command was an error")?;
    writeln!(out, "  -j      There are background processes running")?;
    writeln!(out, "  -l      Use the long format")?;
    writeln!(out, "  -z      Print escape codes compatible with zsh")?;
    writeln!(out, "  -w<SUB> Replace black background with SUB")?;
    writeln!(out)?;
    writeln!(out, "Arguments for `t` command:")?;
    writeln!(out, "  PWD     Working directory for command",)
}
