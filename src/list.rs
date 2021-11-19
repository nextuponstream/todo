//! List all Todo lists in active Todo context
use crate::{
    parse::{parse_todo_list, ParsedTodoList},
    Configuration,
};
use clap::{crate_authors, App, Arg, ArgMatches};
use log::{debug, trace};
use std::fs::read_to_string;
use walkdir::WalkDir;

/// Returns Todo list command
pub fn list_command() -> App<'static, 'static> {
    App::new("list")
        .about("List all todo list within Todo context with tasks remaining")
        .author(crate_authors!())
        .arg(
            Arg::with_name("label")
                .short("l")
                .long("label")
                .value_name("LABEL")
                .help("Filters by label")
                .value_delimiter(",")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("short")
                .short("s")
                .long("short")
                .help("Displays one line summary"),
        )
        .arg(
            Arg::with_name("all")
                .short("a")
                .long("all")
                .help("Shows all Todo lists"),
        )
        .arg(
            Arg::with_name("done")
                .short("d")
                .long("done")
                .help("Shows only fully completed task lists"),
        )
        .arg(
            Arg::with_name("global")
                .short("g")
                .long("global")
                .help("Lists Todo lists from all contexts"),
        )
}

/// Lists Todo lists from Todo context while filtering by label and whether or not the task list is
/// completed
pub fn list_command_process(
    args: &ArgMatches,
    config: &Configuration,
) -> Result<(), std::io::Error> {
    trace!("list subcommand");

    let labels = args
        .values_of("label")
        .unwrap_or_default()
        .collect::<Vec<_>>();
    debug!("labels = {:?}", labels);
    debug!("short: {}", args.is_present("short"));
    let short = args.is_present("short");
    let all = args.is_present("all");
    let done = args.is_present("done");
    let global = args.is_present("global");

    for ctx in &config.ctxs {
        if !global && ctx.name != config.active_ctx_name {
            continue;
        }

        println!("Todo lists from {}", ctx.folder_location);
        for entry in WalkDir::new(ctx.folder_location.as_str()) {
            let entry = entry.unwrap();
            if entry.file_type().is_dir() {
                // first entry is the todo folder
                continue;
            }
            let filepath = entry.path().to_str().unwrap();
            debug!("todo: {}", filepath);
            debug!("short: {}", short);
            match read_to_string(filepath) {
                Ok(content) => {
                    let todo = parse_todo_list(&content)?;

                    if labels.iter().all(|l| todo.labels.iter().any(|fl| fl == l)) {
                        let is_done = todo.tasks_are_all_done();
                        // so XOR is a thing: https://doc.rust-lang.org/reference/types/boolean.html#logical-xor
                        if !all && (is_done ^ done) {
                            continue;
                        }

                        if short {
                            print_short(content.as_str());
                        } else {
                            println!("{}", content);
                        }
                    }
                }
                Err(error) => panic!(
                    "Cannot open {}, error: {}",
                    entry.path().to_str().unwrap(),
                    error
                ),
            }
        }
    }

    Ok(())
}

/// Prints a short one-line summary of Todo list
fn print_short(todo_raw: &str) {
    trace!("print_short");
    let todo: ParsedTodoList = parse_todo_list(todo_raw).unwrap();
    println!("{}/{}\t- {}", todo.done, todo.total, todo.title);
}
