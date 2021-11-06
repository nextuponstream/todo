//! List all todo lists in active Todo context inside configuration
use super::{parse_done_tasks, parse_title, Context};
use crate::parse::parse_todo;
use clap::{crate_authors, App, Arg, ArgMatches};
use log::{debug, trace};
use regex::Regex;
use std::fs::read_to_string;
use walkdir::WalkDir;

/// Todo list subcommand
pub fn list_command() -> App<'static, 'static> {
    App::new("list")
        .about("List all todo list within Todo context with tasks remaining")
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
            Arg::with_name("short")
                .short("s")
                .long("short")
                .help("one line"),
        )
        .arg(
            Arg::with_name("all")
                .short("a")
                .long("all")
                .help("show all Todo lists"),
        )
        .arg(
            Arg::with_name("done")
                .short("d")
                .long("done")
                .help("Show only fully completed task lists"),
        )
}

/// Process arguments and list Todos from context according to given filters
pub fn list_command_process(args: &ArgMatches, ctx: &Context) -> Result<(), std::io::Error> {
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

    let label_re: Regex = Regex::new(r"LABEL=(.*)\n---").unwrap();

    println!("Todo lists from {}", ctx.todo_folder);
    for entry in WalkDir::new(ctx.todo_folder.as_str()) {
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
                let todo = parse_todo(&content).unwrap();
                let label_matches = label_re.captures(content.as_str()).unwrap();
                if label_matches.len() == 1 {
                    eprintln!("Cannot parse {} labels", filepath);
                    std::process::exit(1);
                }

                let file_labels = label_matches
                    .get(1)
                    .unwrap()
                    .as_str()
                    .split(",")
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();
                if labels.iter().all(|f| file_labels.iter().any(|fl| fl == f)) {
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

    Ok(())
}

/// Prints a short one-line summary of Todo item
fn print_short(todo_raw: &str) {
    trace!("print_short");
    let (finished, total) = parse_done_tasks(todo_raw);
    println!(
        "{}/{}\t- {}",
        finished,
        total,
        parse_title(todo_raw).unwrap()
    );
}
