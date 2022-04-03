//! Edit Todo list in active Todo context
use super::{todo_path, Configuration, Context};
use clap::{crate_authors, Arg, ArgMatches, Command};
use core::fmt;
use log::trace;

pub enum Error {
    UnknownContext(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Error::UnknownContext(ctx) => writeln!(f, "Unknown context \"{ctx}\" was referrenced."),
        }
    }
}

/// Returns the Edit Todo command
pub fn edit_command() -> Command<'static> {
    Command::new("edit")
        .about("Edit todo list within Todo context")
        .author(crate_authors!())
        .arg(
            Arg::new("title")
                .short('t')
                .long("title")
                .value_name("TITLE")
                .index(1)
                .help("Title of todo list")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::new("context name")
                .short('c')
                .long("ctx")
                .value_name("CONTEXT")
                .index(2)
                .help("Context of todo list")
                .takes_value(true),
        )
}

/// Edits Todo list in active Todo context with configured IDE
pub fn edit_command_process(
    args: &ArgMatches,
    ctx: &Context,
    config: &Configuration,
) -> Result<(), Error> {
    trace!("edit subcommand");
    println!("Listing all todo's from {}", ctx.folder_location);

    let title = args.value_of("title").unwrap();
    let (ctx_ide, ctx_folder) = if let Some(name) = args.value_of("context name") {
        if let Some(ctx) = config.ctxs.iter().find(|ctx| ctx.name == name) {
            (ctx.ide.as_str(), ctx.folder_location.as_str())
        } else {
            return Err(Error::UnknownContext(name.to_string()));
        }
    } else {
        (ctx.ide.as_str(), ctx.folder_location.as_str())
    };

    std::process::Command::new(ctx_ide)
        .arg(todo_path(ctx_folder, title))
        .status()
        .expect("IDE error");

    Ok(())
}
