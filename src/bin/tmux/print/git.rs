use crate::Result;

use simpalt::{
    git::long as git,
    print::{Color, Div, Printer, Symbol},
};

pub fn render<P>(printer: &mut P, pwd: String) -> Result
where
    P: Printer,
{
    render_inner(printer, git::parse(&std::path::PathBuf::from(pwd)))
}

fn render_inner<P>(printer: &mut P, repo: git::Repo) -> Result
where
    P: Printer,
{
    fn add_div<P>(printer: &mut P) -> Result<&mut P>
    where
        P: Printer,
    {
        printer.div(Div::SlantTopRight, Color::Vga(237))
    }

    match repo {
        git::Repo::None | git::Repo::Error => {}
        git::Repo::Regular(head, sync, changes) => {
            add_div(printer)?
                .fg(Color::Magenta)
                .gap()?
                .txt(Symbol::Branch)?
                .fg(Color::Vga(246))
                .txt(head)?
                .gap()?;
            let has_changes = render_changes(printer, changes)?;
            render_sync(printer, sync, has_changes)?;
        }
        git::Repo::Detached(head, changes) => {
            add_div(printer)?
                .fg(Color::Red)
                .gap()?
                .txt(Symbol::Warn)?
                .fg(Color::Vga(246))
                .txt(head)?
                .gap()?;
            render_changes(printer, changes)?;
        }
        git::Repo::Pending(head, pending, changes) => {
            add_div(printer)?
                .fg(Color::Magenta)
                .gap()?
                .txt(Symbol::Branch)?
                .fg(Color::Vga(246))
                .txt(head)?
                .gap()?
                .fg(Color::Vga(246))
                .txt_gap(Symbol::from(pending))?;
            render_changes(printer, changes)?;
        }
        git::Repo::New(changes) => {
            render_changes(printer, changes)?;
        }
    }

    Ok(())
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
            .txt_gap(format_args!("+{}", changes.added))?;
    }

    if changes.removed > 0 {
        printer
            .fg(Color::Red)
            .txt_gap(format_args!("-{}", changes.removed))?;
    }

    if changes.modified > 0 {
        printer
            .fg(Color::Blue)
            .txt_gap(format_args!("~{}", changes.modified))?;
    }

    if changes.conflicted > 0 {
        printer
            .fg(Color::Magenta)
            .txt_gap(format_args!("!{}", changes.conflicted))?;
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
            .txt_gap(format_args!("{} local", Symbol::Local))
            .map(|_| ()),
        git::Sync::Gone => add_slant(printer, has_changes)?
            .fg(Color::Magenta)
            .txt_gap(format_args!("{} gone", Symbol::Gone))
            .map(|_| ()),
        git::Sync::Tracked { ahead, behind } => {
            let has_ahead = ahead > 0;
            if has_ahead {
                add_slant(printer, has_changes)?
                    .fg(Color::Yellow)
                    .txt_gap(format_args!("{} {ahead}", Symbol::Ahead))?;
            }

            if behind > 0 {
                if !has_ahead {
                    add_slant(printer, has_changes)?;
                }

                printer
                    .fg(Color::Red)
                    .txt_gap(format_args!("{} {behind}", Symbol::Behind))?;
            }

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::expect;
    use super::*;

    fn test(repo: git::Repo) -> String {
        {
            let mut buffer = String::new();
            let mut printer = unsafe { simpalt::print::Ansi::new(buffer.as_mut_vec()) };
            render_inner(&mut printer, repo.clone()).unwrap();
            println!("{buffer}[m");
        }
        let mut buffer = String::new();
        let mut printer = unsafe { simpalt::print::Tmux::new(buffer.as_mut_vec()) };
        render_inner(&mut printer, repo).unwrap();
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
