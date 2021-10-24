//! Edit todo list in active Todo context inside configuration
use super::{todo_path, Context};
use clap::{crate_authors, App, Arg, ArgMatches};
use log::trace;
use std::process::Command;

/// Returns the Edit Todo command
pub fn edit_command() -> App<'static, 'static> {
    App::new("edit")
        .about("Edit todo list within Todo context")
        .author(crate_authors!())
        .arg(
            Arg::with_name("title")
                .short("t")
                .long("title")
                .value_name("TITLE")
                .index(1)
                .help("Title of todo to open")
                .takes_value(true)
                .required(true),
        )
}

/// Processes arguments and edits Todo given a name. Edit command opens IDE as configured in Todo
/// context.
pub fn edit_command_process(args: &ArgMatches, ctx: &Context) -> Result<(), std::io::Error> {
    trace!("edit subcommand");
    println!("Listing all todo's from {}", ctx.todo_folder);

    let title = args.value_of("title").unwrap();

    Command::new(ctx.ide.as_str())
        .arg(todo_path(ctx.todo_folder.as_str(), title))
        .status()
        .expect("IDE error");

    Ok(())
}
