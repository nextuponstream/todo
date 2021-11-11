//! Create Todo list in active Todo context inside configuration
use super::{todo_path, Context, TodoList};
use clap::{crate_authors, App, Arg, ArgMatches};
use dialoguer::Confirm;
use log::{debug, trace};
use std::fs::read_to_string;
use std::path::Path;
use std::process::exit;

/// Returns Todo create command
pub fn create_command() -> App<'static, 'static> {
    App::new("create")
        .about("Create a new todo list within Todo context")
        .author(crate_authors!())
        .arg(
            Arg::with_name("label")
                .short("l")
                .long("label")
                .value_name("LABEL")
                .help("Filter by label")
                .value_delimiter(",")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("title")
                .short("t")
                .long("title")
                .value_name("TITLE")
                .help("Sets title of todo")
                .takes_value(true)
                .empty_values(false)
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("content")
                .short("c")
                .long("content")
                .value_name("CONTENT")
                .help("Sets content of todo")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("item")
                .short("i")
                .long("item")
                .multiple(true)
                .value_name("ITEM")
                .help("An item of your todo list")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("motives")
                .short("m")
                .long("motives")
                .multiple(true)
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
    debug!("todo to create:\n{}", todo);

    // Individual files allow for manual editing without the pain of scrolling through
    // all other todo's.
    let filepath = todo_path(ctx.folder_location.as_str(), todo.title.as_str());

    prompt_for_todo_folder_if_not_exists(ctx, &todo)?;

    match read_to_string(&filepath) {
        Ok(_) => {
            trace!("Potential overwrite detected");
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

/// Prompts user for Todo folder creation if it does not exists. Exits if user answer is negative.
fn prompt_for_todo_folder_if_not_exists(
    ctx: &Context,
    todo_list: &TodoList,
) -> Result<(), std::io::Error> {
    Ok(if !Path::exists(Path::new(ctx.folder_location.as_str())) {
        if Confirm::new()
            .with_prompt(format!(
                "Todo folder location for this context does not exists. {} cannot be created. Create {} ?",
                todo_list.title,
                ctx.folder_location
            ))
            .interact()?
        {
            std::fs::create_dir(ctx.folder_location.as_str())?;
        } else {
            exit(0);
        }
    })
}
