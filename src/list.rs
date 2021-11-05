//! List all todo lists in active Todo context inside configuration
use super::Context;
use clap::{crate_authors, App, Arg, ArgMatches};
use log::{debug, trace};
use regex::Regex;
use std::fs::read_to_string;
use walkdir::WalkDir;

/// Todo list subcommand
pub fn list_command() -> App<'static, 'static> {
    App::new("list")
        .about("List all todo list within Todo context")
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

    let re: Regex = Regex::new(r"LABEL=(.*)\n---").unwrap();

    println!("Todo list from {}", ctx.todo_folder);
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
                let cs = re.captures(content.as_str()).unwrap();
                if cs.len() == 1 {
                    eprintln!("Cannot parse {} labels", filepath);
                    std::process::exit(1);
                }

                let file_labels = cs
                    .get(1)
                    .unwrap()
                    .as_str()
                    .split(",")
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();
                if labels.iter().all(|f| file_labels.iter().any(|fl| fl == f)) {
                    if short {
                        // FIXME short not printed
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
        "-- {}/{} - {}",
        finished,
        total,
        parse_title(todo_raw).unwrap()
    );
}

/// Returns title from raw Todo content
fn parse_title(todo_raw: &str) -> Option<String> {
    let title_reg: Regex = Regex::new(r"---\nTITLE=(.+)\n").unwrap();
    debug!("todo_raw: {}", todo_raw);
    match title_reg.captures(todo_raw) {
        None => None,
        Some(caps) => {
            debug!("caps len: {}", caps.len());
            if caps.len() == 1 {
                None
            } else {
                debug!("&caps[1]: {}", &caps[1]);
                Some(String::from(&caps[1]))
            }
        }
    }
}

/// Returns how many done tasks and total number of tasks. Tasks can be spread throughout the
/// file.
fn parse_done_tasks(todo_raw: &str) -> (usize, usize) {
    trace!("parse_remaining_tasks");
    debug!("todo_raw: {}", todo_raw);
    let done_reg: Regex = Regex::new(r"(?m)^\* \[(.{1})\] .+$").unwrap();
    let mut done = 0;
    let matches = done_reg.find_iter(todo_raw);
    let total = matches.count();
    for mat in done_reg.find_iter(todo_raw) {
        if mat.as_str().get(0..6).unwrap().eq("* [x] ") {
            done = done + 1;
        }
    }
    (done, total)
}

/// Returns true if all todo items are done
pub fn tasks_are_all_done(todo_raw: &str) -> bool {
    let (remaining, done) = parse_done_tasks(todo_raw);
    return remaining == done;
}

#[cfg(test)]
mod tests {
    use super::*;
    use simplelog::*;

    // TODO wait for before/after_test macro
    // https://github.com/rust-lang/rfcs/issues/1664
    fn init() {
        let _ = TermLogger::init(
            LevelFilter::Warn,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        );
    }

    #[test]
    fn empty_title() {
        init();
        let todo_raw = "\
---
TITLE=
LABEL=
---
";
        assert!(parse_title(todo_raw).is_none());

        assert!(tasks_are_all_done(todo_raw));
    }

    #[test]
    fn parse_no_tasks() {
        init();
        let todo_raw = "\
---
TITLE=
LABEL=
---
";
        let (done, total) = parse_done_tasks(todo_raw);
        assert_eq!(0, done);
        assert_eq!(0, total);
    }

    #[test]
    fn parse_one_remaining_tasks() {
        init();
        let todo_raw = "\
---
TITLE=
LABEL=
---

* [ ] idk man

---
";
        let (done, total) = parse_done_tasks(todo_raw);
        assert_eq!(0, done);
        assert_eq!(1, total);

        assert!(!tasks_are_all_done(todo_raw));
    }

    #[test]
    fn parse_one_done_tasks() {
        init();
        let todo_raw = "\
---
TITLE=
LABEL=
---

* [x] idk man

---
";
        let (done, total) = parse_done_tasks(todo_raw);
        assert_eq!(1, done);
        assert_eq!(1, total);

        assert!(tasks_are_all_done(todo_raw));
    }

    #[test]
    fn parse_multiple_remaining_tasks() {
        init();
        let todo_raw = "\
---
TITLE=
LABEL=
---

* [ ] idk man
* [x] idk man
* [ ] idk man
* [x] idk man
* [ ] idk man

---
";
        let (done, total) = parse_done_tasks(todo_raw);
        assert_eq!(2, done, "wrong number of done tasks");
        assert_eq!(5, total);

        assert!(!tasks_are_all_done(todo_raw));
    }

    #[test]
    fn parse_multiple_all_done_tasks() {
        init();
        let todo_raw = "\
---
TITLE=
LABEL=
---

* [x] idk man
* [x] idk man
* [x] idk man
* [x] idk man
* [x] idk man

---
";
        let (done, total) = parse_done_tasks(todo_raw);
        assert_eq!(5, done);
        assert_eq!(5, total);

        assert!(tasks_are_all_done(todo_raw));
    }
}
