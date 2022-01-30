//! Move Todo list in specified Todo context
use core::fmt;

use crate::{prompt_for_todo_folder_if_not_exists, todo_path};

use super::Configuration;
use clap::{crate_authors, App, Arg, ArgMatches};

/// Errors for move command
#[derive(Debug)]
pub enum Error {
    /// The specified Context could not be found among Contexts.
    ///
    /// First argument indicates if old path is malformed. Otherwise new path is malformed.
    /// Second argument is the name of the context.
    /// Third argument is the name of available context.
    UnknownContext(bool, String, Vec<String>),
    PromptingUserForContextFolderCreation,
    Renaming,
    // The file could not be moved because it is does not exists.
    //
    // First argument is the name of the file to move
    // Second argument is the path to the file to move
    NothingToMove(String, String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnknownContext(is_old_path, ctx_name, ctxs_available_names) => {
                if *is_old_path {
                    writeln!(f, "Old path is unknown!")?;
                } else {
                    writeln!(f, "New path is unknown!")?;
                }
                writeln!(f, "\"{ctx_name}\" does not match any available context.")?;
                writeln!(f, "Please select a name among:")?;
                for ctx_name in ctxs_available_names {
                    writeln!(f, "- {ctx_name}")?;
                }
            },
            Error::PromptingUserForContextFolderCreation => {
                writeln!(f, "Something went wrong while asking user to create Todo Context folder to move Todo list into.")?
            },
            Error::Renaming => {
                writeln!(f, "Error while renaming the file to another location.")?
            }
            Error::NothingToMove(file, filepath) => {
                writeln!(f, "File \"{file}\" could not be moved because there is nothing at \"{filepath}\"")?
            }
        }

        Ok(())
    }
}

/// Returns the Edit Todo command
pub fn move_command() -> App<'static, 'static> {
    App::new("move")
        .about("Move todo list into other Todo context")
        .author(crate_authors!())
        .arg(
            Arg::with_name("title")
                .short("t")
                .long("title")
                .value_name("TITLE")
                .index(1)
                .help("Title of Todo list to move")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("context name")
                .short("c")
                .long("ctx")
                .value_name("CONTEXT_NAME")
                .index(2)
                .help("Name of todo context to move to")
                .takes_value(true)
                .required(true),
        )
}

/// Move Todo list from active Todo to specified context
pub fn move_command_process(args: &ArgMatches, config: &Configuration) -> Result<(), Error> {
    let title = args.value_of("title").unwrap();
    let ctx_name = args.value_of("context name").unwrap();

    let (old_path, new_path) = match paths_for_moving_todo_list(title, ctx_name, config) {
        Ok(vs) => (vs.0, vs.1),
        Err(e) => {
            eprintln!("{e}");
            return Err(e);
        }
    };

    let new_ctx = match config.ctxs.iter().find(|&ctx| ctx.name == ctx_name) {
        Some(ctx) => ctx,
        None => {
            // Note this should be unreachable considering this same bit of code used in
            // paths_for_moving_todo_list
            let e = Error::UnknownContext(
                false,
                ctx_name.to_string(),
                config.ctxs.iter().map(|ctx| ctx.name.to_string()).collect(),
            );
            eprintln!("{e}");
            return Err(e);
        }
    };

    // Note: std::fs::rename does not indicate why the renaming fails. However
    // we can assume rename will fail if there is no file to copy from hence why
    // we test if filepath leads to a file.
    if !std::path::Path::new(&old_path).is_file() {
        return Err(Error::NothingToMove(title.to_string(), old_path));
    }

    if let Err(e) = prompt_for_todo_folder_if_not_exists(new_ctx) {
        eprintln!("Error: {e}");
        return Err(Error::PromptingUserForContextFolderCreation);
    }

    if std::fs::rename(&old_path, &new_path).is_err() {
        eprintln!("Error: file could not be moved from {old_path} to {new_path}.");
        return Err(Error::Renaming);
    }

    Ok(())
}

/// Returns the path of the Todo list and the new path to move the Todo list
fn paths_for_moving_todo_list(
    title: &str,
    ctx_name: &str,
    config: &Configuration,
) -> Result<(String, String), Error> {
    let current_folder_location_of_todo_list = match config
        .ctxs
        .iter()
        .find(|&ctx| ctx.name == config.active_ctx_name)
    {
        Some(ctx) => ctx.folder_location.as_str(),
        None => {
            eprintln!("Error: No matching context was found for {ctx_name}");
            return Err(Error::UnknownContext(
                true,
                ctx_name.to_string(),
                config.ctxs.iter().map(|ctx| ctx.name.to_string()).collect(),
            ));
        }
    };
    let new_folder_location_of_todo_list =
        match config.ctxs.iter().find(|&ctx| ctx.name == ctx_name) {
            Some(ctx) => ctx.folder_location.as_str(),
            None => {
                eprintln!("Error: No matching context was found for {ctx_name}");
                return Err(Error::UnknownContext(
                    false,
                    ctx_name.to_string(),
                    config.ctxs.iter().map(|ctx| ctx.name.to_string()).collect(),
                ));
            }
        };

    let old_path = todo_path(current_folder_location_of_todo_list, title);
    let new_path = todo_path(new_folder_location_of_todo_list, title);

    Ok((old_path, new_path))
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::Context;
    // use simplelog::*;

    // // TODO wait for before/after_test macro
    // // https://github.com/rust-lang/rfcs/issues/1664
    // fn init() {
    //     let _ = TermLogger::init(
    //         LevelFilter::Debug,
    //         Config::default(),
    //         TerminalMode::Mixed,
    //         ColorChoice::Auto,
    //     );
    // }

    #[test]
    fn well_formed_paths() {
        let config = Configuration {
            active_ctx_name: "ctx1".to_string(),
            ctxs: vec![
                Context {
                    ide: "".to_string(),
                    name: "ctx1".to_string(),
                    timezone: "".to_string(),
                    folder_location: "/path/to/folder1".to_string(),
                },
                Context {
                    ide: "".to_string(),
                    name: "ctx2".to_string(),
                    timezone: "".to_string(),
                    folder_location: "/path/to/folder2".to_string(),
                },
            ],
        };
        let (old_path, new_path) = paths_for_moving_todo_list("file", "ctx2", &config).unwrap();
        // Note: abstract the file extension to not make the test brittle
        let expected_old_path = "/path/to/folder1/file.";
        let expected_new_path = "/path/to/folder2/file.";
        assert!(old_path.starts_with(expected_old_path));
        assert!(new_path.starts_with(expected_new_path));
    }

    #[test]
    fn unknown_context_throws_error() {
        let config = Configuration {
            active_ctx_name: "ctx1".to_string(),
            ctxs: vec![
                Context {
                    ide: "".to_string(),
                    name: "ctx1".to_string(),
                    timezone: "".to_string(),
                    folder_location: "/path/to/folder1".to_string(),
                },
                Context {
                    ide: "".to_string(),
                    name: "ctx2".to_string(),
                    timezone: "".to_string(),
                    folder_location: "/path/to/folder2".to_string(),
                },
            ],
        };
        let paths = paths_for_moving_todo_list("file", "unknown", &config);
        assert!(paths.is_err());
        match paths.unwrap_err() {
            Error::UnknownContext(is_old_path, unknown_ctx, available_ctxs) => {
                assert!(!is_old_path);
                assert_eq!(unknown_ctx, "unknown");
                assert_eq!(available_ctxs, vec!["ctx1".to_string(), "ctx2".to_string()])
            }

            _ => assert!(false),
        }
    }
}
