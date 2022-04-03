//! Create Todo list in active Todo context inside configuration
use super::{prompt_for_todo_folder_if_not_exists, todo_path, Context, TodoList};
use clap::{crate_authors, Arg, ArgMatches, Command};
use dialoguer::Confirm;
use log::trace;
use std::fs::read_to_string;

/// Returns Todo create command
pub fn create_command() -> Command<'static> {
    Command::new("create")
        .about("Create a new todo list within Todo context")
        .author(crate_authors!())
        .arg(
            Arg::new("label")
                .short('l')
                .long("label")
                .value_name("LABEL")
                .help("Filter by label")
                .value_delimiter(',')
                .takes_value(true),
        )
        .arg(
            Arg::new("title")
                .value_name("TITLE")
                .help("Sets title of todo")
                .takes_value(true)
                .forbid_empty_values(true)
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("content")
                .short('c')
                .long("content")
                .value_name("CONTENT")
                .help("Sets content of todo")
                .takes_value(true),
        )
        .arg(
            Arg::new("item")
                .short('i')
                .long("item")
                .multiple_values(true)
                .value_name("ITEM")
                .help("An item of your todo list")
                .takes_value(true),
        )
        .arg(
            Arg::new("motives")
                .short('m')
                .long("motives")
                .multiple_values(true)
                .value_name("MOTIVE")
                .help("list of motives that appears in reverse order of the todo")
                .takes_value(true),
        )
}

/// Creates a new Todo list in active Todo context
pub fn create_command_process(args: &ArgMatches, ctx: &Context) -> Result<(), std::io::Error> {
    trace!("create subcommand");
    let todo = TodoList {
        title: args.value_of("title").unwrap().to_string(),
        description: args.value_of("content").unwrap_or("").to_string(),
        // https://stackoverflow.com/a/37547426/16631150
        labels: args
            .values_of("label")
            .unwrap_or_default()
            .map(|s| s.to_string())
            .collect(),
        list_items: args
            .values_of("item")
            .unwrap_or_default()
            .map(|s| s.to_string())
            .collect(),
        motives: args
            .values_of("motives")
            .unwrap_or_default()
            .map(|s| s.to_string())
            .collect(),
    };

    // Individual files allow for manual editing without the pain of scrolling through
    // all other todo's.
    let filepath = todo_path(ctx.folder_location.as_str(), todo.title.as_str());

    if let Err(e) = prompt_for_todo_folder_if_not_exists(ctx) {
        eprintln!("Error: {e}");
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Todo creation error",
        ));
    }

    match read_to_string(&filepath) {
        Ok(_) => {
            if !Confirm::new()
                .with_prompt(format!(
                    "This operation will overwrite todo \"{}\". Continue?",
                    todo.title
                ))
                .interact()?
            {
                return Ok(());
            }
        }
        Err(e) => {
            // in cargo test, file cannot be written to
            trace!("File cannot be open: {}", e);
        }
    }

    std::fs::write(&filepath, format!("{}", todo))?;
    println!("Saved todo \"{}\" ({})", todo.title, ctx.folder_location);

    Ok(())
}
