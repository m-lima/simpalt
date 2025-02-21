use crate::Result;
use crate::git::long as git;

macro_rules! style {
    (reset $(, $($param: expr),*)?) => {
        concat!("#[none]", $(, $($param),*)?)
    };

    (fg = $fg: expr, bg = $bg: expr $(, $($param: expr),*)?) => {
        concat!("#[fg=", $fg, ",bg=", $bg, "]" $(, $($param),*)?)
    };

    (fg = $color: expr $(, $($param: expr),*)?) => {
        concat!("#[fg=", $color, "]" $(, $($param),*)?)
    };

    (bg = $color: expr $(, $($param: expr),*)?) => {
        concat!("#[bg=", $color, "]" $(, $($param),*)?)
    };
}

macro_rules! color {
    (black) => {
        "black"
    };
    (red) => {
        "red"
    };
    (green) => {
        "green"
    };
    (yellow) => {
        "yellow"
    };
    (blue) => {
        "blue"
    };
    (magenta) => {
        "magenta"
    };
    (cyan) => {
        "cyan"
    };
    (white) => {
        "white"
    };
    (gray) => {
        color!(246)
    };
    (dark gray) => {
        color!(236)
    };
    ($param: literal) => {
        concat!("colour", $param)
    };
    ($r: literal, $g: literal, $b: literal) => {
        concat!("#", $r, $g, $b)
    };
    (reset) => {
        "default"
    };
}

#[derive(Debug, Eq, PartialEq)]
pub struct Args {
    pub pwd: String,
}

pub fn render<Out>(out: Out, args: Args) -> Result
where
    Out: std::io::Write,
{
    render_git(out, git::parse(&std::path::PathBuf::from(args.pwd)))
}

fn render_git<Out>(mut out: Out, repo: git::Repo) -> Result
where
    Out: std::io::Write,
{
    match repo {
        git::Repo::None | git::Repo::Error => Ok(()),
        git::Repo::Regular(head, sync, changes) => {
            write!(out, style!(fg = color!(237), symbol!(slant)))?;
            write!(
                out,
                style!(fg = color!(magenta), bg = color!(237), " ", symbol!(branch))
            )?;
            write!(out, style!(fg = color!(gray), "{head} "), head = head)?;
            let changed_bg = render_changes(&mut out, changes)?;
            render_sync(&mut out, sync, changed_bg)?;
            out.flush()
        }
        git::Repo::Detached(head, changes) => {
            write!(out, style!(fg = color!(237), symbol!(slant)))?;
            write!(
                out,
                style!(fg = color!(magenta), bg = color!(237), " ", symbol!(branch))
            )?;
            write!(out, style!(fg = color!(gray), "{head} "), head = head)?;
            render_changes(&mut out, changes)?;
            out.flush()
        }
        git::Repo::Pending(head, pending, changes) => {
            write!(out, style!(fg = color!(237), symbol!(slant)))?;
            write!(
                out,
                style!(fg = color!(magenta), bg = color!(237), " ", symbol!(branch))
            )?;
            write!(
                out,
                style!(fg = color!(gray), "{head} {pending}"),
                head = head,
                pending = pending_symbol(pending),
            )?;
            render_changes(&mut out, changes)?;
            out.flush()
        }
        git::Repo::New(changes) => {
            render_changes(&mut out, changes)?;
            out.flush()
        }
    }
}

fn render_changes<Out>(out: &mut Out, changes: git::Changes) -> Result<bool>
where
    Out: std::io::Write,
{
    let mut changed_bg = false;
    if changes.added > 0 {
        write!(out, style!(fg = color!(dark gray), symbol!(slant)))?;
        write!(
            out,
            style!(fg = color!(green), bg = color!(dark gray), " +{added}"),
            added = changes.added
        )?;
        changed_bg = true;
    }

    if changes.removed > 0 {
        if changed_bg {
            write!(
                out,
                style!(fg = color!(red), " -{removed}"),
                removed = changes.removed
            )?;
        } else {
            write!(out, style!(fg = color!(dark gray), symbol!(slant)))?;
            write!(
                out,
                style!(fg = color!(red), bg = color!(dark gray), " -{removed}"),
                removed = changes.removed
            )?;
            changed_bg = true;
        }
    }

    if changes.modified > 0 {
        if changed_bg {
            write!(
                out,
                style!(fg = color!(blue), " ~{modified}"),
                modified = changes.modified
            )?;
        } else {
            write!(out, style!(fg = color!(dark gray), symbol!(slant)))?;
            write!(
                out,
                style!(fg = color!(blue), bg = color!(dark gray), " ~{modified}"),
                modified = changes.modified
            )?;
            changed_bg = true;
        }
    }

    if changes.conflicted > 0 {
        if changed_bg {
            write!(
                out,
                style!(fg = color!(magenta), " !{conflicted}"),
                conflicted = changes.conflicted
            )?;
        } else {
            write!(out, style!(fg = color!(dark gray), symbol!(slant)))?;
            write!(
                out,
                style!(
                    fg = color!(magenta),
                    bg = color!(dark gray),
                    " !{conflicted}"
                ),
                conflicted = changes.conflicted
            )?;
            changed_bg = true;
        }
    }

    if changed_bg {
        write!(out, " ")?;
    }

    Ok(changed_bg)
}

fn render_sync<Out>(out: &mut Out, sync: git::Sync, changed_bg: bool) -> Result
where
    Out: std::io::Write,
{
    fn add_slant<Out>(out: &mut Out, changed_bg: bool) -> Result
    where
        Out: std::io::Write,
    {
        if changed_bg {
            write!(
                out,
                style!(fg = color!(gray), " ", symbol!(slant thin), " ")
            )
        } else {
            write!(out, style!(fg = color!(gray), symbol!(slant thin)))?;
            write!(out, style!(bg = color!(dark gray), " "))
        }
    }

    match sync {
        git::Sync::Local => {
            add_slant(out, changed_bg)?;
            write!(out, style!(fg = color!(cyan), symbol!(local), " local "))
        }
        git::Sync::Gone => {
            add_slant(out, changed_bg)?;
            write!(out, style!(fg = color!(magenta), symbol!(gone), " gone "))
        }
        git::Sync::Tracked { ahead, behind } => {
            let has_ahead = ahead > 0;
            if has_ahead {
                add_slant(out, changed_bg)?;
                write!(
                    out,
                    style!(fg = color!(yellow), symbol!(ahead), "{ahead} "),
                    ahead = ahead
                )?;
            }

            if behind > 0 {
                if !has_ahead {
                    add_slant(out, changed_bg)?;
                }

                write!(
                    out,
                    style!(fg = color!(red), symbol!(behind), "{behind} "),
                    behind = behind
                )?;
            }

            Ok(())
        }
    }
}

const fn pending_symbol(pending: git::Pending) -> &'static str {
    match pending {
        git::Pending::Merge => symbol!(merge),
        git::Pending::Revert => symbol!(revert),
        git::Pending::Cherry => symbol!(cherry),
        git::Pending::Bisect => symbol!(bisect),
        git::Pending::Rebase => symbol!(rebase),
        git::Pending::Mailbox => symbol!(mailbox),
    }
}
