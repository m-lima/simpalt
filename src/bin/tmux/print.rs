use crate::Result;
use simpalt::{
    git::long as git,
    print::{Color, Div, Printer, Symbol},
};

pub fn render<P>(mut printer: P, pwd: String) -> Result
where
    P: Printer,
{
    fn add_branch<P>(printer: &mut P, head: String) -> Result<&mut P>
    where
        P: Printer,
    {
        printer
            .fg(Color::Vga(237))
            .txt(Div::SlantTopRight)?
            .gap()?
            .fg(Color::Magenta)
            .bg(Color::Vga(237))
            .txt(Symbol::Branch)?
            .fg(Color::Vga(246))
            .txt(head)?
            .gap()
    }

    match git::parse(&std::path::PathBuf::from(pwd)) {
        git::Repo::None | git::Repo::Error => return Ok(()),
        git::Repo::Regular(head, sync, changes) => {
            add_branch(&mut printer, head)?;
            let has_changes = render_changes(&mut printer, changes)?;
            render_sync(&mut printer, sync, has_changes)?;
        }
        git::Repo::Detached(head, changes) => {
            add_branch(&mut printer, head)?;
            render_changes(&mut printer, changes)?;
        }
        git::Repo::Pending(head, pending, changes) => {
            add_branch(&mut printer, head)?
                .fg(Color::Vga(246))
                .txt_gap(Symbol::from(pending))?;
            render_changes(&mut printer, changes)?;
        }
        git::Repo::New(changes) => {
            render_changes(&mut printer, changes)?;
        }
    }

    printer.flush()
}

fn render_changes<P>(printer: &mut P, changes: git::Changes) -> Result<bool>
where
    P: Printer,
{
    let has_changes = if changes.added > 0
        || changes.removed > 0
        || changes.modified > 0
        || changes.conflicted > 0
    {
        printer
            .gap()?
            .div(Div::SlantTopRight, Color::Vga(236))?
            .gap()?;
        true
    } else {
        false
    };

    if changes.added > 0 {
        printer
            .fg(Color::Green)
            .gap()?
            .txt("+")?
            .txt(changes.added)?
            .gap()?;
    }

    if changes.removed > 0 {
        printer
            .fg(Color::Red)
            .gap()?
            .txt("-")?
            .txt(changes.removed)?
            .gap()?;
    }

    if changes.modified > 0 {
        printer
            .fg(Color::Blue)
            .gap()?
            .txt("~")?
            .txt(changes.modified)?
            .gap()?;
    }

    if changes.conflicted > 0 {
        printer
            .fg(Color::Magenta)
            .gap()?
            .txt("!")?
            .txt(changes.conflicted)?
            .gap()?;
    }

    Ok(has_changes)
}

fn render_sync<P>(printer: &mut P, sync: git::Sync, has_changes: bool) -> Result
where
    P: Printer,
{
    fn add_slant<P>(printer: &mut P, has_changes: bool) -> Result<&mut P>
    where
        P: Printer,
    {
        if has_changes {
            printer.fg(Color::Vga(246)).txt_gap(Symbol::SlantTop)
        } else {
            printer
                .gap()?
                .div(Div::SlantTopRight, Color::Vga(236))?
                .gap()
        }
    }

    match sync {
        git::Sync::Local => add_slant(printer, has_changes)?
            .fg(Color::Cyan)
            .txt_gap(Symbol::Local)?
            .txt_gap("local")
            .map(|_| ()),
        git::Sync::Gone => add_slant(printer, has_changes)?
            .fg(Color::Magenta)
            .txt_gap(Symbol::Gone)?
            .txt_gap("gone")
            .map(|_| ()),
        git::Sync::Tracked { ahead, behind } => {
            let has_ahead = ahead > 0;
            if has_ahead {
                add_slant(printer, has_changes)?
                    .fg(Color::Yellow)
                    .txt_gap(Symbol::Ahead)?
                    .txt_gap(ahead)?;
            }

            if behind > 0 {
                if !has_ahead {
                    add_slant(printer, has_changes)?;
                }

                printer
                    .fg(Color::Red)
                    .gap()?
                    .txt_gap(Symbol::Behind)?
                    .txt_gap(behind)?;
            }

            Ok(())
        }
    }
}
