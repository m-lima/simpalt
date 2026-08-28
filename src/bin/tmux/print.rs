use crate::Result;
use simpalt::{
    git::long as git,
    print::{Color, Div, Printer, Symbol},
};

pub fn render<P>(printer: P, pwd: String) -> Result
where
    P: Printer,
{
    render_inner(printer, git::parse(&std::path::PathBuf::from(pwd)))
}

pub fn render_inner<P>(mut printer: P, repo: git::Repo) -> Result
where
    P: Printer,
{
    fn add_div<P>(printer: &mut P) -> Result<&mut P>
    where
        P: Printer,
    {
        Ok(printer
            .fg(Color::Vga(237))
            .txt(Div::SlantTopRight)?
            .bg(Color::Vga(237)))
    }

    match repo {
        git::Repo::None | git::Repo::Error => return Ok(()),
        git::Repo::Regular(head, sync, changes) => {
            add_div(&mut printer)?
                .fg(Color::Magenta)
                .gap()?
                .txt(Symbol::Branch)?
                .fg(Color::Vga(246))
                .txt(head)?
                .gap()?;
            let has_changes = render_changes(&mut printer, changes)?;
            render_sync(&mut printer, sync, has_changes)?;
        }
        git::Repo::Detached(head, changes) => {
            add_div(&mut printer)?
                .fg(Color::Red)
                .gap()?
                .txt(Symbol::Warn)?
                .fg(Color::Vga(246))
                .txt(head)?
                .gap()?;
            render_changes(&mut printer, changes)?;
        }
        git::Repo::Pending(head, pending, changes) => {
            add_div(&mut printer)?
                .fg(Color::Magenta)
                .gap()?
                .txt(Symbol::Branch)?
                .fg(Color::Vga(246))
                .txt(head)?
                .gap()?
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
        printer.div(Div::SlantTopRight, Color::Vga(236))?;
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
            printer.fg(Color::Vga(246)).txt(Symbol::SlantTop)
        } else {
            printer.div(Div::SlantTopRight, Color::Vga(236))
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

#[cfg(test)]
mod tests {
    use super::*;

    pub fn expect<'a, I: IntoIterator<Item = &'a str>>(result: &str, expected: I) -> String {
        let expected = String::from_iter(expected);
        println!("{result}");
        println!("{expected}");
        expected
    }

    pub fn test(repo: git::Repo) -> String {
        {
            let mut buffer = String::new();
            let printer = unsafe { simpalt::print::Ansi::new(buffer.as_mut_vec()) };
            render_inner(printer, repo.clone()).unwrap();
            println!("{buffer}[m");
        }
        let mut buffer = String::new();
        let printer = unsafe { simpalt::print::Tmux::new(buffer.as_mut_vec()) };
        render_inner(printer, repo).unwrap();
        buffer
    }

    #[test]
    fn none() {
        let result = test(git::Repo::None);
        let expected = expect(&result, []);
        assert_eq!(result, expected);
    }

    #[test]
    fn error() {
        let result = test(git::Repo::Error);
        let expected = expect(&result, []);
        assert_eq!(result, expected);
    }

    mod regular {
        use super::*;

        #[test]
        fn local() {
            let result = test(git::Repo::Regular(
                String::from("bloink"),
                git::Sync::Local,
                git::Changes::default(),
            ));
            let expected = expect(
                &result,
                [
                    "#[fg=colour237]",
                    Div::SlantTopRight.str(),
                    "#[fg=magenta,bg=colour237]",
                    " ",
                    Symbol::Branch.str(),
                    "#[fg=colour246]",
                    "bloink",
                    " ",
                    "#[fg=colour236]",
                    Div::SlantTopRight.str(),
                    "#[fg=cyan,bg=colour236]",
                    " ",
                    Symbol::Local.str(),
                    " ",
                    "local",
                    " ",
                ],
            );
            assert_eq!(result, expected);
        }

        #[test]
        fn gone() {
            let result = test(git::Repo::Regular(
                String::from("bloink"),
                git::Sync::Gone,
                git::Changes {
                    added: 1,
                    ..Default::default()
                },
            ));
            let expected = expect(
                &result,
                [
                    "#[fg=colour237]",
                    Div::SlantTopRight.str(),
                    "#[fg=magenta,bg=colour237]",
                    " ",
                    Symbol::Branch.str(),
                    "#[fg=colour246]",
                    "bloink",
                    " ",
                    "#[fg=colour236]",
                    Div::SlantTopRight.str(),
                    "#[fg=green,bg=colour236]",
                    " ",
                    "+1",
                    " ",
                    "#[fg=colour246]",
                    Symbol::SlantTop.str(),
                    "#[fg=magenta]",
                    " ",
                    Symbol::Gone.str(),
                    " ",
                    "gone",
                    " ",
                ],
            );
            assert_eq!(result, expected);
        }

        #[test]
        fn tracked_in_sync() {
            let result = test(git::Repo::Regular(
                String::from("bloink"),
                git::Sync::Tracked {
                    ahead: 0,
                    behind: 0,
                },
                git::Changes {
                    added: 1,
                    modified: 2,
                    removed: 4,
                    conflicted: 8,
                },
            ));
            let expected = expect(
                &result,
                [
                    "#[fg=colour237]",
                    Div::SlantTopRight.str(),
                    "#[fg=magenta,bg=colour237]",
                    " ",
                    Symbol::Branch.str(),
                    "#[fg=colour246]",
                    "bloink",
                    " ",
                    "#[fg=colour236]",
                    Div::SlantTopRight.str(),
                    "#[fg=green,bg=colour236]",
                    " ",
                    "+1",
                    " ",
                    "#[fg=red]",
                    "-4",
                    " ",
                    "#[fg=blue]",
                    "~2",
                    " ",
                    "#[fg=magenta]",
                    "!8",
                    " ",
                ],
            );
            assert_eq!(result, expected);
        }

        #[test]
        fn tracked_diverged() {
            let result = test(git::Repo::Regular(
                String::from("bloink"),
                git::Sync::Tracked {
                    ahead: 1,
                    behind: 2,
                },
                git::Changes::default(),
            ));
            let expected = expect(
                &result,
                [
                    "#[fg=colour237]",
                    Div::SlantTopRight.str(),
                    "#[fg=magenta,bg=colour237]",
                    " ",
                    Symbol::Branch.str(),
                    "#[fg=colour246]",
                    "bloink",
                    " ",
                    "#[fg=colour236]",
                    Div::SlantTopRight.str(),
                    "#[fg=yellow,bg=colour236]",
                    " ",
                    Symbol::Ahead.str(),
                    " ",
                    "1",
                    " ",
                    "#[fg=red]",
                    Symbol::Behind.str(),
                    " ",
                    "2",
                    " ",
                ],
            );
            assert_eq!(result, expected);
        }

        #[test]
        fn tracked_diverged_out_of_sync() {
            let result = test(git::Repo::Regular(
                String::from("bloink"),
                git::Sync::Tracked {
                    ahead: 1,
                    behind: 2,
                },
                git::Changes {
                    added: 1,
                    modified: 2,
                    removed: 4,
                    conflicted: 8,
                },
            ));
            let expected = expect(
                &result,
                [
                    "#[fg=colour237]",
                    Div::SlantTopRight.str(),
                    "#[fg=magenta,bg=colour237]",
                    " ",
                    Symbol::Branch.str(),
                    "#[fg=colour246]",
                    "bloink",
                    " ",
                    "#[fg=colour236]",
                    Div::SlantTopRight.str(),
                    "#[fg=green,bg=colour236]",
                    " ",
                    "+1",
                    " ",
                    "#[fg=red]",
                    "-4",
                    " ",
                    "#[fg=blue]",
                    "~2",
                    " ",
                    "#[fg=magenta]",
                    "!8",
                    " ",
                    "#[fg=colour246]",
                    Symbol::SlantTop.str(),
                    "#[fg=yellow]",
                    " ",
                    Symbol::Ahead.str(),
                    " ",
                    "1",
                    " ",
                    "#[fg=red]",
                    Symbol::Behind.str(),
                    " ",
                    "2",
                    " ",
                ],
            );
            assert_eq!(result, expected);
        }
    }

    mod detached {
        use super::*;

        #[test]
        fn clean() {
            let result = test(git::Repo::Detached(
                String::from("bloink"),
                git::Changes::default(),
            ));
            let expected = expect(
                &result,
                [
                    "#[fg=colour237]",
                    Div::SlantTopRight.str(),
                    "#[fg=red,bg=colour237]",
                    " ",
                    Symbol::Warn.str(),
                    "#[fg=colour246]",
                    "bloink",
                    " ",
                ],
            );
            assert_eq!(result, expected);
        }

        #[test]
        fn one() {
            let result = test(git::Repo::Detached(
                String::from("bloink"),
                git::Changes {
                    added: 1,
                    ..Default::default()
                },
            ));
            let expected = expect(
                &result,
                [
                    "#[fg=colour237]",
                    Div::SlantTopRight.str(),
                    "#[fg=red,bg=colour237]",
                    " ",
                    Symbol::Warn.str(),
                    "#[fg=colour246]",
                    "bloink",
                    " ",
                    "#[fg=colour236]",
                    Div::SlantTopRight.str(),
                    "#[fg=green,bg=colour236]",
                    " ",
                    "+1",
                    " ",
                ],
            );
            assert_eq!(result, expected);
        }

        #[test]
        fn all() {
            let result = test(git::Repo::Detached(
                String::from("bloink"),
                git::Changes {
                    added: 1,
                    modified: 2,
                    removed: 4,
                    conflicted: 8,
                },
            ));
            let expected = expect(
                &result,
                [
                    "#[fg=colour237]",
                    Div::SlantTopRight.str(),
                    "#[fg=red,bg=colour237]",
                    " ",
                    Symbol::Warn.str(),
                    "#[fg=colour246]",
                    "bloink",
                    " ",
                    "#[fg=colour236]",
                    Div::SlantTopRight.str(),
                    "#[fg=green,bg=colour236]",
                    " ",
                    "+1",
                    " ",
                    "#[fg=red]",
                    "-4",
                    " ",
                    "#[fg=blue]",
                    "~2",
                    " ",
                    "#[fg=magenta]",
                    "!8",
                    " ",
                ],
            );
            assert_eq!(result, expected);
        }
    }

    mod pending {
        use super::*;

        #[test]
        fn merge_clean() {
            let result = test(git::Repo::Pending(
                String::from("bloink"),
                git::Pending::Merge,
                git::Changes::default(),
            ));
            let expected = expect(
                &result,
                [
                    "#[fg=colour237]",
                    Div::SlantTopRight.str(),
                    "#[fg=magenta,bg=colour237]",
                    " ",
                    Symbol::Branch.str(),
                    "#[fg=colour246]",
                    "bloink",
                    " ",
                    Symbol::Merge.str(),
                    " ",
                ],
            );
            assert_eq!(result, expected);
        }

        #[test]
        fn rebase_one() {
            let result = test(git::Repo::Pending(
                String::from("bloink"),
                git::Pending::Rebase,
                git::Changes {
                    added: 1,
                    ..Default::default()
                },
            ));
            let expected = expect(
                &result,
                [
                    "#[fg=colour237]",
                    Div::SlantTopRight.str(),
                    "#[fg=magenta,bg=colour237]",
                    " ",
                    Symbol::Branch.str(),
                    "#[fg=colour246]",
                    "bloink",
                    " ",
                    Symbol::Rebase.str(),
                    " ",
                    "#[fg=colour236]",
                    Div::SlantTopRight.str(),
                    "#[fg=green,bg=colour236]",
                    " ",
                    "+1",
                    " ",
                ],
            );
            assert_eq!(result, expected);
        }

        #[test]
        fn bisect_all() {
            let result = test(git::Repo::Pending(
                String::from("bloink"),
                git::Pending::Bisect,
                git::Changes {
                    added: 1,
                    modified: 2,
                    removed: 4,
                    conflicted: 8,
                },
            ));
            let expected = expect(
                &result,
                [
                    "#[fg=colour237]",
                    Div::SlantTopRight.str(),
                    "#[fg=magenta,bg=colour237]",
                    " ",
                    Symbol::Branch.str(),
                    "#[fg=colour246]",
                    "bloink",
                    " ",
                    Symbol::Bisect.str(),
                    " ",
                    "#[fg=colour236]",
                    Div::SlantTopRight.str(),
                    "#[fg=green,bg=colour236]",
                    " ",
                    "+1",
                    " ",
                    "#[fg=red]",
                    "-4",
                    " ",
                    "#[fg=blue]",
                    "~2",
                    " ",
                    "#[fg=magenta]",
                    "!8",
                    " ",
                ],
            );
            assert_eq!(result, expected);
        }
    }

    mod new {
        use super::*;

        #[test]
        fn clean() {
            let result = test(git::Repo::New(git::Changes::default()));
            let expected = expect(&result, []);
            assert_eq!(result, expected);
        }

        #[test]
        fn one() {
            let result = test(git::Repo::New(git::Changes {
                added: 1,
                ..Default::default()
            }));
            let expected = expect(
                &result,
                [
                    "#[fg=colour236]",
                    Div::SlantTopRight.str(),
                    "#[fg=green,bg=colour236]",
                    " ",
                    "+1",
                    " ",
                ],
            );
            assert_eq!(result, expected);
        }

        #[test]
        fn all() {
            let result = test(git::Repo::New(git::Changes {
                added: 1,
                modified: 2,
                removed: 4,
                conflicted: 8,
            }));
            let expected = expect(
                &result,
                [
                    "#[fg=colour236]",
                    Div::SlantTopRight.str(),
                    "#[fg=green,bg=colour236]",
                    " ",
                    "+1",
                    " ",
                    "#[fg=red]",
                    "-4",
                    " ",
                    "#[fg=blue]",
                    "~2",
                    " ",
                    "#[fg=magenta]",
                    "!8",
                    " ",
                ],
            );
            assert_eq!(result, expected);
        }
    }
}
