//! Delete Todo list from active Todo context inside configuration
use super::todo_path;
use super::Context;
use clap::crate_authors;
use clap::{Arg, ArgMatches, Command};
use log::trace;
use std::fs::remove_file;

/// Returns Delete Todo command
pub fn delete_command() -> Command<'static> {
    Command::new("delete")
        .about("Delete todo list by name within Todo context")
        .author(crate_authors!())
        .arg(
            Arg::new("title")
                .short('t')
                .long("title")
                .value_name("TITLE")
                .index(1)
                .help("Title of todo to delete")
                .takes_value(true)
                .required(true),
        )
}

/// Deletes Todo list from active Todo context
pub fn delete_command_process(args: &ArgMatches, ctx: &Context) -> Result<(), std::io::Error> {
    trace!("delete subcommand");

    let title = args.value_of("title").unwrap();
    match remove_file(todo_path(ctx.folder_location.as_str(), title)) {
        Ok(_) => println!("Successfully removed {}", title),
        Err(_) => eprintln!("Error: File does not exist"),
    }

    Ok(())
}
