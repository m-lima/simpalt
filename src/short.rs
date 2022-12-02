use crate::Result;

#[inline]
pub fn prompt(
    mut out: impl std::io::Write,
    host: Option<String>,
    error: bool,
    jobs: bool,
) -> Result {
    let error = if error {
        style!(fg = color!(red), symbol!(error) " ")
    } else {
        ""
    };

    let jobs = if jobs {
        style!(fg = color!(cyan), symbol!(jobs) " ")
    } else {
        ""
    };

    let venv = if std::env::var("VIRTUAL_ENV").is_ok() {
        style!(fg = color!(green), symbol!(python))
    } else {
        ""
    };

    let (host, host_padding) = host.map_or_else(|| (String::new(), ""), |host| (host, " "));

    let pwd = std::env::current_dir()
        .or_else(|_| std::env::var("PWD").map(std::path::PathBuf::from))
        .ok();

    let pwd_string = if let Some(ref pwd) = pwd {
        pwd_string(pwd)
    } else {
        String::new()
    };

    let git_string = if let Some(ref pwd) = pwd {
        git_string(pwd)
    } else {
        style!(fg = color!(black), bg = color!(reset), symbol!(div))
    };

    write!(
        out,
        concat!(
            style!(bg = color!(black), " {error}{jobs}{venv}"),
            style!(fg = color!(reset), "{host}"),
            style!(reset),
            style!(bg = color!(black), "{host_padding}{pwd_string} "),
            "{git_string}",
            style!(reset),
            " "
        ),
        error = error,
        jobs = jobs,
        venv = venv,
        host_padding = host_padding,
        host = host,
        pwd_string = pwd_string,
        git_string = git_string,
    )?;
    out.flush()
}

fn git_string(path: &std::path::PathBuf) -> &'static str {
    use crate::git::short as git;

    macro_rules! prompt {
            (default $state: expr) => {
                concat!(symbol!(branch), prompt!($state))
            };
            (warn $state: expr) => {
                concat!(symbol!(warn), prompt!($state))
            };
            ($sync: expr, $state: expr) => {
                style!(fg = $sync, symbol!(branch) prompt!($state))
            };
            ($state: expr) => {
                style!(fg = color!(black), bg = $state, symbol!(div) style!(fg = $state, bg = color!(reset), symbol!(div)))
            };
        }

    match git::prompt(path) {
        git::Repo::None => prompt!(color!(blue)),
        git::Repo::Clean(sync) => match sync {
            git::Sync::UpToDate => prompt!(default color!(green)),
            git::Sync::Behind => prompt!(color!(red), color!(green)),
            git::Sync::Ahead => prompt!(color!(yellow), color!(green)),
            git::Sync::Diverged => prompt!(color!(magenta), color!(green)),
            git::Sync::Local => prompt!(color!(blue), color!(green)),
        },
        git::Repo::Dirty(sync) => match sync {
            git::Sync::UpToDate => prompt!(default color!(yellow)),
            git::Sync::Behind => prompt!(color!(red), color!(yellow)),
            git::Sync::Ahead => prompt!(color!(yellow), color!(yellow)),
            git::Sync::Diverged => prompt!(color!(magenta), color!(yellow)),
            git::Sync::Local => prompt!(color!(blue), color!(yellow)),
        },
        git::Repo::Pending => prompt!(warn color!(cyan)),
        git::Repo::Untracked => prompt!(default color!(cyan)),
        git::Repo::Detached => prompt!(default color!(magenta)),
        git::Repo::Error => prompt!(color!(red)),
    }
}

fn pwd_string(path: &std::path::PathBuf) -> String {
    if let Ok(home) = std::env::var("HOME").map(std::path::PathBuf::from) {
        if home.eq(path) {
            return String::from("~");
        }
    }

    let (prefix, components) =
        path.components()
            .fold((None, vec![]), |(prefix, mut list), curr| match curr {
                std::path::Component::Prefix(prefix) => (Some(prefix), list),
                std::path::Component::RootDir | std::path::Component::Normal(_) => {
                    list.push(curr);
                    (prefix, list)
                }
                std::path::Component::ParentDir => {
                    list.pop();
                    (prefix, list)
                }
                std::path::Component::CurDir => (prefix, list),
            });

    if let Some(std::path::Component::Normal(path)) = components.last() {
        String::from(path.to_string_lossy())
    } else if let Some(prefix) = prefix {
        String::from(prefix.as_os_str().to_string_lossy())
    } else {
        String::from(std::path::MAIN_SEPARATOR)
    }
}
