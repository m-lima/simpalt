use crate::git::long as git;
use crate::Result;

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
    ([$param: literal]) => {
        concat!("colour", $param)
    };
    ([$r: literal, $g: literal, $b: literal]) => {
        concat!("#", $r, $g, $b)
    };
    (reset) => {
        "default"
    };
}

pub fn render(out: impl std::io::Write, pwd: String) -> Result {
    render_git(out, git::prompt(&std::path::PathBuf::from(pwd)))
}

fn render_git(mut out: impl std::io::Write, repo: git::Repo) -> Result {
    match repo {
        git::Repo::None | git::Repo::Error => Ok(()),
        git::Repo::Regular(head, sync, changes) => {
            if changes.clean() {
                write!(out, style!(fg = color!(green), symbol!(slant)))?;
                write!(
                    out,
                    style!(
                        fg = color!(black),
                        bg = color!(green),
                        " ",
                        symbol!(branch),
                        "{head} "
                    ),
                    head = head,
                )?;
            } else {
                write!(out, style!(fg = color!(yellow), symbol!(slant)))?;
                write!(
                    out,
                    style!(
                        fg = color!(black),
                        bg = color!(yellow),
                        " ",
                        symbol!(branch),
                        "{head} "
                    ),
                    head = head,
                )?;
            }
            let changed_bg = render_changes(&mut out, changes)?;
            render_sync(&mut out, sync, changed_bg)?;
            out.flush()
        }
        git::Repo::Detached(head, changes) => {
            write!(out, style!(fg = color!(magenta), symbol!(slant)))?;
            write!(
                out,
                style!(
                    fg = color!(black),
                    bg = color!(magenta),
                    " ",
                    symbol!(branch),
                    "{head} "
                ),
                head = head,
            )?;
            render_changes(&mut out, changes)?;
            out.flush()
        }
        git::Repo::Pending(head, pending, changes) => {
            write!(out, style!(fg = color!(cyan), symbol!(slant)))?;
            write!(
                out,
                style!(
                    fg = color!(black),
                    bg = color!(cyan),
                    " ",
                    symbol!(branch),
                    "{head} {pending} "
                ),
                head = head,
                pending = pending_symbol(pending),
            )?;
            render_changes(&mut out, changes)?;
            out.flush()
        }
        git::Repo::New(changes) => {
            write!(out, style!(fg = color!(cyan), symbol!(slant)))?;
            write!(
                out,
                style!(fg = color!(reset), bg = color!(cyan), symbol!(new))
            )?;
            render_changes(&mut out, changes)?;
            out.flush()
        }
    }
}

fn render_changes(out: &mut impl std::io::Write, changes: git::Changes) -> Result<bool> {
    let mut changed_bg = false;
    if changes.added > 0 {
        write!(out, style!(fg = color!(black), symbol!(slant)))?;
        write!(
            out,
            style!(fg = color!(green), bg = color!(black), " +{added}"),
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
            write!(out, style!(fg = color!(black), symbol!(slant)))?;
            write!(
                out,
                style!(fg = color!(red), bg = color!(black), " -{removed}"),
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
            write!(out, style!(fg = color!(black), symbol!(slant)))?;
            write!(
                out,
                style!(fg = color!(blue), bg = color!(black), " ~{modified}"),
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
            write!(out, style!(fg = color!(black), symbol!(slant)))?;
            write!(
                out,
                style!(fg = color!(magenta), bg = color!(black), " !{conflicted}"),
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

fn render_sync(out: &mut impl std::io::Write, sync: git::Sync, changed_bg: bool) -> Result {
    fn add_slant(out: &mut impl std::io::Write, changed_bg: bool) -> Result {
        if changed_bg {
            write!(
                out,
                style!(fg = color!([248]), " ", symbol!(slant thin), " ")
            )
        } else {
            write!(out, style!(fg = color!([248]), symbol!(slant thin)))?;
            write!(out, style!(bg = color!(black), " "))
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
