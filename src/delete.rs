//! Delete todo list from active Todo context inside configuration
use super::todo_path;
use super::Context;
use clap::crate_authors;
use clap::{App, Arg, ArgMatches};
use log::trace;
use std::fs::remove_file;

/// Returns Delete Todo command
pub fn delete_command() -> App<'static, 'static> {
    App::new("delete")
        .about("Delete todo list by name within Todo context")
        .author(crate_authors!())
        .arg(
            Arg::with_name("title")
                .short("t")
                .long("title")
                .value_name("TITLE")
                .index(1)
                .help("Title of todo to delete")
                .takes_value(true)
                .required(true),
        )
}

/// Processes and deletes Todo by title
///
/// TODO is the title unique to context???
pub fn delete_command_process(args: &ArgMatches, ctx: &Context) -> Result<(), std::io::Error> {
    trace!("delete subcommand");

    let title = args.value_of("title").unwrap();
    match remove_file(todo_path(ctx.todo_folder.as_str(), title)) {
        Ok(_) => println!("Successfully removed {}", title),
        Err(_) => eprintln!("Error: File does not exist"),
    }

    Ok(())
}
