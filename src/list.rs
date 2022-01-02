//! List all Todo lists in active Todo context
use crate::{
    parse::{parse_todo_list, parse_todo_list_tasks},
    Configuration, Context,
};
use clap::{crate_authors, App, Arg, ArgMatches};
use log::{debug, trace};
use std::{fs::read_to_string, path::Path};
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
        .arg(
            Arg::with_name("completed-tasks")
                .short("c")
                .long("completed-tasks")
                .help(
                    "Shows only completed tasks in the lists (default shows the entire task list)",
                ),
        )
        .arg(
            Arg::with_name("open-tasks")
                .short("o")
                .long("open-tasks")
                .conflicts_with("completed-tasks")
                .help("Shows only open tasks in the lists (default shows the entire task list)"),
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
    let completed = args.is_present("completed-tasks");
    let open = args.is_present("open-tasks");

    list_message(
        &mut std::io::stdout(),
        &config,
        labels,
        short,
        all,
        done,
        global,
        completed,
        open,
        None,
    )
}

/// Returns message when `todo list` command is invoked
///
/// `Todo list` command prints Todo lists in the active Todo context. There are many filters that
/// can be applied.
///
/// * `labels` - filter by label
/// * `short` - print short view of Todo lists
/// * `all` - do not filter out any Todo lists within context
/// * `done` - filter Todo lists with all tasks done
/// * `global` - disable filtering by Todo context
/// * `entries` - when provided, don't use Todo list file entries at Todo context folder location
fn list_message(
    stdout: &mut dyn std::io::Write,
    config: &Configuration,
    labels: Vec<&str>,
    short: bool,
    all: bool,
    done: bool,
    global: bool,
    completed: bool,
    open: bool,
    entries: Option<Vec<Vec<&str>>>,
) -> Result<(), std::io::Error> {
    debug!("short: {}", short);
    assert!(!(completed && open));

    if !config.is_valid() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Bad configuration file",
        ));
    }

    if entries.is_some() {
        let mut entries = entries.unwrap();
        assert_eq!(
            entries.len(),
            config.ctxs.len(),
            "entries and configuration contexts number do not match"
        );
        let mut ctxs = config.ctxs.clone();
        ctxs.reverse();
        entries.reverse();

        for ctx in config.ctxs.clone() {
            if !global && ctx.name != config.active_ctx_name {
                continue;
            }

            print_todo_folder_location(stdout, &ctx)?;
            let directory = entries.pop().unwrap();
            debug!("directory: {}\n- files:\n{:?}", ctx.name, directory);
            for todo_raw in directory {
                print_todo(stdout, todo_raw, &labels, all, done, short, completed, open)?;
            }
        }

        return Ok(());
    }

    for ctx in &config.ctxs {
        if !global && ctx.name != config.active_ctx_name {
            continue;
        }

        print_todo_folder_location(stdout, ctx)?;

        for entry in WalkDir::new(ctx.folder_location.as_str()) {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("{}", e);
                    return Err(std::io::Error::new(std::io::ErrorKind::Other, e));
                }
            };
            if !entry.file_type().is_file() {
                // first entry is the todo folder which should be skipped
                continue;
            }
            let filepath = entry.path().to_str().unwrap();
            debug!("todo: {}", filepath);
            let extension = Path::new(&filepath).extension().unwrap().to_str().unwrap();
            // avoid coercing .jpg files into Todo list
            if !is_valid_extension(&extension) {
                continue;
            }
            let todo_raw = match read_to_string(filepath) {
                Ok(content) => content,
                Err(error) => panic!(
                    "Cannot open {}, error: {}",
                    entry.path().to_str().unwrap(),
                    error
                ),
            };

            print_todo(
                stdout,
                todo_raw.as_str(),
                &labels,
                all,
                done,
                short,
                completed,
                open,
            )?;
        }
    }

    Ok(())
}

/// Returns true if the file is markdown or in txt format
fn is_valid_extension(ext: &str) -> bool {
    let valid_extensions: Vec<&str> = vec!["md", "txt"];

    valid_extensions.contains(&ext)
}

/// Prints folder location from which Todo lists are being parsed
///
/// Note: there is two references, one for tests and one for list command. We avoid the petty case
/// where modifying at one place might not affect the other (imagine tests running fine but actual
/// logic is different).
fn print_todo_folder_location(
    stdout: &mut dyn std::io::Write,
    ctx: &Context,
) -> Result<(), std::io::Error> {
    writeln!(stdout, "Todo lists from {}", ctx.folder_location)
}

/// Prints out a Todo list. By default, only Todo lists with open tasks will be
/// printed out.
///
/// * `stdout` - The output be printed to (usually stdout)
/// * `todo_raw` - The content of the Todo list in plain text
/// * `labels` - The set of label the Todo list must have for it to be printed
/// * `all` - Condition to print Todo list even when it has no open tasks
/// * `done` - Condition to print only Todo list with no open tasks
/// * `short` - Print a short summary of Todo list which indicates the number of
/// task done and the total number of tasks in the list
/// * `completed` - Print the summary of the completed tasks in the list
/// * `open` - Print the summary of the open tasks in the list
fn print_todo(
    stdout: &mut dyn std::io::Write,
    todo_raw: &str,
    labels: &Vec<&str>,
    all: bool,
    done: bool,
    short: bool,
    completed: bool,
    open: bool,
) -> Result<(), std::io::Error> {
    trace!("print_todo");
    let todo_list = parse_todo_list(&todo_raw).unwrap();
    debug!("labels count: {}", labels.len());
    debug!(
        "All labels matches: {}",
        labels
            .iter()
            .all(|l| todo_list.labels.iter().any(|fl| fl == l))
    );
    if labels
        .iter()
        .all(|l| todo_list.labels.iter().any(|fl| fl == l))
    {
        let is_done = todo_list.tasks_are_all_done();
        debug!("all: {}", all);
        debug!("is_done: {}", is_done);
        debug!("done: {}", done);
        debug!("!all && (is_done ^ done): {}", !all && (is_done ^ done));
        // so XOR is a thing: https://doc.rust-lang.org/reference/types/boolean.html#logical-xor
        if !all && (is_done ^ done) {
            trace!("skipped");
            return Ok(());
        }
        if completed ^ open {
            writeln!(stdout, "# {}", todo_list.title)?;
            let tasks = parse_todo_list_tasks(todo_raw, completed, open, short).unwrap();
            for task in tasks {
                // trim_end avoid cluttering the output with all whitespace the
                // user might have used to make his Todo list more readable or
                // the accidental trailing spaces he might have left
                writeln!(stdout, "{}", task.as_str().trim_end())?;
            }
        } else {
            if short {
                writeln!(
                    stdout,
                    "{}/{}\t- {}",
                    todo_list.done, todo_list.total, todo_list.title
                )?;
            } else {
                writeln!(stdout, "{}", todo_raw)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use simplelog::*;

    // TODO wait for before/after_test macro
    // https://github.com/rust-lang/rfcs/issues/1664
    fn init() {
        let _ = TermLogger::init(
            LevelFilter::Trace,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        );
    }

    // Note: we test the short message everytime because Todo display message might be subject to
    // change
    const SHORT: bool = true;

    // Note: testing buffered output idea https://stackoverflow.com/a/48393114
    // One could write to a string then display the message but the string can possibly take a lot
    // of memory before being written to stdout. Therefore, it is better to println as you iterate
    // through Todo lists. Testing is then a little more complicated than comparing two strings.
    // String does not implement std::io::Write so can't use Strings

    #[test]
    fn empty_configuration() {
        init();
        let mut stdout = vec![];
        let config = Configuration {
            active_ctx_name: String::from("ctx1"),
            ctxs: vec![],
        };
        let labels: Vec<&str> = Vec::new();
        let all = false;
        let done = false;
        let global = false;
        let completed = false;
        let open = false;
        let entries = vec![];
        assert!(list_message(
            &mut stdout,
            &config,
            labels,
            SHORT,
            all,
            done,
            global,
            completed,
            open,
            Some(entries),
        )
        .is_err());
    }

    #[test]
    fn todo_context_with_no_todo_lists() {
        init();
        let mut stdout = vec![];
        let config = Configuration {
            active_ctx_name: String::from("ctx1"),
            ctxs: vec![Context {
                ide: String::from(""),
                name: String::from("ctx1"),
                timezone: String::from("CET"),
                folder_location: String::from("fake/folder"),
            }],
        };
        let labels: Vec<&str> = Vec::new();
        let all = false;
        let done = false;
        let global = false;
        let completed = false;
        let open = false;
        let entries = vec![vec![]];

        assert!(list_message(
            &mut stdout,
            &config,
            labels,
            SHORT,
            all,
            done,
            global,
            completed,
            open,
            Some(entries),
        )
        .is_ok());
        let expected = b"Todo lists from fake/folder\n";
        assert_eq!(
            stdout,
            expected,
            "\ngot     : \"{}\"\nexpected: \"{}\"",
            String::from_utf8(stdout.to_vec()).unwrap(),
            String::from_utf8(expected.to_vec()).unwrap()
        );
    }

    #[test]
    fn list_todo_lists_from_one_config() {
        init();
        let mut stdout = vec![];
        let config = Configuration {
            active_ctx_name: String::from("ctx1"),
            ctxs: vec![Context {
                ide: String::from(""),
                name: String::from("ctx1"),
                timezone: String::from("CET"),
                folder_location: String::from("fake/folder"),
            }],
        };
        let labels: Vec<&str> = Vec::new();
        let all = false;
        let done = false;
        let global = false;
        let completed = false;
        let open = false;
        let entries = vec![vec![
            "# title1\n\n## Description\n\nLABEL=\n\n## Todo list\n\n* [ ] first",
            "# title2\n\n## Description\n\nLABEL=\n\n## Todo list\n\n* [x] first",
        ]];

        assert!(list_message(
            &mut stdout,
            &config,
            labels,
            SHORT,
            all,
            done,
            global,
            completed,
            open,
            Some(entries),
        )
        .is_ok());
        let expected = b"Todo lists from fake/folder\n0/1\t- title1\n";
        assert_eq!(
            stdout,
            expected,
            "\ngot     : \"{}\"\nexpected: \"{}\"",
            String::from_utf8(stdout.to_vec()).unwrap(),
            String::from_utf8(expected.to_vec()).unwrap()
        );

        // with done
        let mut stdout = vec![];
        let config = Configuration {
            active_ctx_name: String::from("ctx1"),
            ctxs: vec![Context {
                ide: String::from(""),
                name: String::from("ctx1"),
                timezone: String::from("CET"),
                folder_location: String::from("fake/folder"),
            }],
        };
        let labels: Vec<&str> = Vec::new();
        let all = false;
        let done = true;
        let global = false;
        let completed = false;
        let open = false;
        let entries = vec![vec![
            "# title1\n\n## Description\n\nLABEL=\n\n## Todo list\n\n* [ ] first",
            "# title2\n\n## Description\n\nLABEL=\n\n## Todo list\n\n* [x] first",
        ]];

        assert!(list_message(
            &mut stdout,
            &config,
            labels,
            SHORT,
            all,
            done,
            global,
            completed,
            open,
            Some(entries),
        )
        .is_ok());
        let expected = b"Todo lists from fake/folder\n1/1\t- title2\n";
        assert_eq!(
            stdout,
            expected,
            "\ngot     : \"{}\"\nexpected: \"{}\"",
            String::from_utf8(stdout.to_vec()).unwrap(),
            String::from_utf8(expected.to_vec()).unwrap()
        );

        // with all
        let mut stdout = vec![];
        let config = Configuration {
            active_ctx_name: String::from("ctx1"),
            ctxs: vec![Context {
                ide: String::from(""),
                name: String::from("ctx1"),
                timezone: String::from("CET"),
                folder_location: String::from("fake/folder"),
            }],
        };
        let labels: Vec<&str> = Vec::new();
        let all = true;
        let done = false;
        let global = false;
        let completed = false;
        let open = false;
        let entries = vec![vec![
            "# title1\n\n## Description\n\nLABEL=\n\n## Todo list\n\n* [ ] first",
            "# title2\n\n## Description\n\nLABEL=\n\n## Todo list\n\n* [x] first",
        ]];

        assert!(list_message(
            &mut stdout,
            &config,
            labels,
            SHORT,
            all,
            done,
            global,
            completed,
            open,
            Some(entries),
        )
        .is_ok());
        let expected = b"Todo lists from fake/folder\n0/1\t- title1\n1/1\t- title2\n";
        assert_eq!(
            stdout,
            expected,
            "\ngot     : \"{}\"\nexpected: \"{}\"",
            String::from_utf8(stdout.to_vec()).unwrap(),
            String::from_utf8(expected.to_vec()).unwrap()
        );

        // with labels
        let mut stdout = vec![];
        let config = Configuration {
            active_ctx_name: String::from("ctx1"),
            ctxs: vec![Context {
                ide: String::from(""),
                name: String::from("ctx1"),
                timezone: String::from("CET"),
                folder_location: String::from("fake/folder"),
            }],
        };
        let labels: Vec<&str> = vec!["l2"];
        let all = true;
        let done = false;
        let global = false;
        let completed = false;
        let open = false;
        let entries = vec![vec![
            "# title1\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n* [ ] first",
            "# title2\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [x] first",
        ]];

        assert!(list_message(
            &mut stdout,
            &config,
            labels,
            SHORT,
            all,
            done,
            global,
            completed,
            open,
            Some(entries),
        )
        .is_ok());
        let expected = b"Todo lists from fake/folder\n1/1\t- title2\n";
        assert_eq!(
            stdout,
            expected,
            "\ngot     : \"{}\"\nexpected: \"{}\"",
            String::from_utf8(stdout.to_vec()).unwrap(),
            String::from_utf8(expected.to_vec()).unwrap()
        );

        let mut stdout = vec![];
        let config = Configuration {
            active_ctx_name: String::from("ctx1"),
            ctxs: vec![Context {
                ide: String::from(""),
                name: String::from("ctx1"),
                timezone: String::from("CET"),
                folder_location: String::from("fake/folder"),
            }],
        };
        let labels: Vec<&str> = vec!["l1"];
        let all = true;
        let done = false;
        let global = false;
        let completed = false;
        let open = false;
        let entries = vec![vec![
            "# title1\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n* [ ] first",
            "# title2\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [x] first",
        ]];

        assert!(list_message(
            &mut stdout,
            &config,
            labels,
            SHORT,
            all,
            done,
            global,
            completed,
            open,
            Some(entries),
        )
        .is_ok());
        let expected = b"Todo lists from fake/folder\n0/1\t- title1\n";
        assert_eq!(
            stdout,
            expected,
            "\ngot     : \"{}\"\nexpected: \"{}\"",
            String::from_utf8(stdout.to_vec()).unwrap(),
            String::from_utf8(expected.to_vec()).unwrap()
        );
    }

    #[test]
    fn list_todo_lists_from_all_config() {
        init();
        let mut stdout = vec![];
        let config = Configuration {
            active_ctx_name: String::from("ctx1"),
            ctxs: vec![
                Context {
                    ide: String::from(""),
                    name: String::from("ctx1"),
                    timezone: String::from("CET"),
                    folder_location: String::from("fake/folder1"),
                },
                Context {
                    ide: String::from(""),
                    name: String::from("ctx2"),
                    timezone: String::from("CET"),
                    folder_location: String::from("fake/folder2"),
                },
            ],
        };
        let labels: Vec<&str> = vec!["l1"];
        let all = true;
        let done = false;
        let global = true;
        let completed = false;
        let open = false;
        let entries = vec![
            vec![
                "# title1\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n* [ ] first",
                "# title2\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [x] first",
            ],
            vec![
                "# title3\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n* [x] first",
                "# title4\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [ ] first",
            ],
        ];

        assert!(list_message(
            &mut stdout,
            &config,
            labels,
            SHORT,
            all,
            done,
            global,
            completed,
            open,
            Some(entries),
        )
        .is_ok());
        let expected = b"Todo lists from fake/folder1\n0/1\t- title1\nTodo lists from fake/folder2\n1/1\t- title3\n";
        assert_eq!(
            stdout,
            expected,
            "\ngot     : \"{}\"\nexpected: \"{}\"",
            String::from_utf8(stdout.to_vec()).unwrap(),
            String::from_utf8(expected.to_vec()).unwrap()
        );

        let mut stdout = vec![];
        let config = Configuration {
            active_ctx_name: String::from("ctx1"),
            ctxs: vec![
                Context {
                    ide: String::from(""),
                    name: String::from("ctx1"),
                    timezone: String::from("CET"),
                    folder_location: String::from("fake/folder1"),
                },
                Context {
                    ide: String::from(""),
                    name: String::from("ctx2"),
                    timezone: String::from("CET"),
                    folder_location: String::from("fake/folder2"),
                },
            ],
        };
        let labels: Vec<&str> = vec!["l2"];
        let all = true;
        let done = false;
        let global = true;
        let completed = false;
        let open = false;
        let entries = vec![
            vec![
                "# title1\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n* [ ] first",
                "# title2\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [x] first",
            ],
            vec![
                "# title3\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n* [x] first",
                "# title4\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [ ] first",
            ],
        ];

        assert!(list_message(
            &mut stdout,
            &config,
            labels,
            SHORT,
            all,
            done,
            global,
            completed,
            open,
            Some(entries),
        )
        .is_ok());
        let expected = b"Todo lists from fake/folder1\n1/1\t- title2\nTodo lists from fake/folder2\n0/1\t- title4\n";
        assert_eq!(
            stdout,
            expected,
            "\ngot     : \"{}\"\nexpected: \"{}\"",
            String::from_utf8(stdout.to_vec()).unwrap(),
            String::from_utf8(expected.to_vec()).unwrap()
        );
    }

    #[test]
    fn valid_extension() {
        assert!(is_valid_extension("md"));
        assert!(is_valid_extension("txt"));
        assert!(!is_valid_extension(""));
        assert!(!is_valid_extension("jpg"));
    }

    #[test]
    fn list_open_tasks() {
        init();
        // TODO implement test
        let mut stdout = vec![];
        let config = Configuration {
            active_ctx_name: String::from("ctx1"),
            ctxs: vec![
                Context {
                    ide: String::from(""),
                    name: String::from("ctx1"),
                    timezone: String::from("CET"),
                    folder_location: String::from("fake/folder1"),
                },
                Context {
                    ide: String::from(""),
                    name: String::from("ctx2"),
                    timezone: String::from("CET"),
                    folder_location: String::from("fake/folder2"),
                },
            ],
        };
        let labels: Vec<&str> = vec!["l1"];
        let all = false;
        let done = false;
        let global = false;
        let completed = false;
        let open = true;
        let entries = vec![
            vec![
                "# title1\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n* [ ] open1\n* [x] completed1\n* [ ] open2\n* [x] completed2",
                "# title2\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [x] completed1\n* [ ] open1\n* [x] completed2\n* [ ] open2",
            ],
            vec![
                "# title3\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n* [ ] open1\n* [x] completed1\n* [ ] open2\n* [x] completed2",
                "# title4\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [x] completed1\n* [ ] open1\n* [x] completed2\n* [ ] open2",
            ],
        ];

        assert!(list_message(
            &mut stdout,
            &config,
            labels,
            SHORT,
            all,
            done,
            global,
            completed,
            open,
            Some(entries),
        )
        .is_ok());
        let expected = b"Todo lists from fake/folder1\n# title1\n* [ ] open1\n* [ ] open2\n";
        assert_eq!(
            stdout,
            expected,
            "\ngot     : \"{}\"\nexpected: \"{}\"",
            String::from_utf8(stdout.to_vec()).unwrap(),
            String::from_utf8(expected.to_vec()).unwrap()
        );

        let mut stdout = vec![];
        let config = Configuration {
            active_ctx_name: String::from("ctx1"),
            ctxs: vec![
                Context {
                    ide: String::from(""),
                    name: String::from("ctx1"),
                    timezone: String::from("CET"),
                    folder_location: String::from("fake/folder1"),
                },
                Context {
                    ide: String::from(""),
                    name: String::from("ctx2"),
                    timezone: String::from("CET"),
                    folder_location: String::from("fake/folder2"),
                },
            ],
        };
        let labels: Vec<&str> = vec!["l2"];
        let all = false;
        let done = false;
        let global = false;
        let completed = false;
        let open = true;
        let entries = vec![
            vec![
                "# title1\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n* [ ] open1\n* [x] completed1\n* [ ] open2\n* [x] completed2",
                "# title2\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [x] completed1\n* [ ] open1\n* [x] completed2\n* [ ] open2",
            ],
            vec![
                "# title3\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n* [ ] open1\n* [x] completed1\n* [ ] open2\n* [x] completed2",
                "# title4\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [x] completed1\n* [ ] open1\n* [x] completed2\n* [ ] open2",
            ],
        ];

        assert!(list_message(
            &mut stdout,
            &config,
            labels,
            SHORT,
            all,
            done,
            global,
            completed,
            open,
            Some(entries),
        )
        .is_ok());
        let expected = b"Todo lists from fake/folder1\n# title2\n* [ ] open1\n* [ ] open2\n";
        assert_eq!(
            stdout,
            expected,
            "\ngot     : \"{}\"\nexpected: \"{}\"",
            String::from_utf8(stdout.to_vec()).unwrap(),
            String::from_utf8(expected.to_vec()).unwrap()
        );
    }

    #[test]
    fn list_completed_tasks() {
        init();
        // TODO implement test
        let mut stdout = vec![];
        let config = Configuration {
            active_ctx_name: String::from("ctx1"),
            ctxs: vec![
                Context {
                    ide: String::from(""),
                    name: String::from("ctx1"),
                    timezone: String::from("CET"),
                    folder_location: String::from("fake/folder1"),
                },
                Context {
                    ide: String::from(""),
                    name: String::from("ctx2"),
                    timezone: String::from("CET"),
                    folder_location: String::from("fake/folder2"),
                },
            ],
        };
        let labels: Vec<&str> = vec!["l1"];
        let all = false;
        let done = false;
        let global = false;
        let completed = true;
        let open = false;
        let entries = vec![
            vec![
                "# title1\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n* [ ] open1\n* [x] completed1\n* [ ] open2\n* [x] completed2",
                "# title2\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [x] completed1\n* [ ] open1\n* [x] completed2\n* [ ] open2",
            ],
            vec![
                "# title3\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n* [ ] open1\n* [x] completed1\n* [ ] open2\n* [x] completed2",
                "# title4\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [x] completed1\n* [ ] open1\n* [x] completed2\n* [ ] open2",
            ],
        ];

        assert!(list_message(
            &mut stdout,
            &config,
            labels,
            SHORT,
            all,
            done,
            global,
            completed,
            open,
            Some(entries),
        )
        .is_ok());
        let expected =
            b"Todo lists from fake/folder1\n# title1\n* [x] completed1\n* [x] completed2\n";
        assert_eq!(
            stdout,
            expected,
            "\ngot     : \"{}\"\nexpected: \"{}\"",
            String::from_utf8(stdout.to_vec()).unwrap(),
            String::from_utf8(expected.to_vec()).unwrap()
        );

        let mut stdout = vec![];
        let config = Configuration {
            active_ctx_name: String::from("ctx1"),
            ctxs: vec![
                Context {
                    ide: String::from(""),
                    name: String::from("ctx1"),
                    timezone: String::from("CET"),
                    folder_location: String::from("fake/folder1"),
                },
                Context {
                    ide: String::from(""),
                    name: String::from("ctx2"),
                    timezone: String::from("CET"),
                    folder_location: String::from("fake/folder2"),
                },
            ],
        };
        let labels: Vec<&str> = vec!["l2"];
        let all = false;
        let done = false;
        let global = false;
        let completed = true;
        let open = false;
        let entries = vec![
            vec![
                "# title1\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n* [ ] open1\n* [x] completed1\n* [ ] open2\n* [x] completed2",
                "# title2\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [x] completed1\n* [ ] open1\n* [x] completed2\n* [ ] open2",
            ],
            vec![
                "# title3\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n* [ ] open1\n* [x] completed1\n* [ ] open2\n* [x] completed2",
                "# title4\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [x] completed1\n* [ ] open1\n* [x] completed2\n* [ ] open2",
            ],
        ];

        assert!(list_message(
            &mut stdout,
            &config,
            labels,
            SHORT,
            all,
            done,
            global,
            completed,
            open,
            Some(entries),
        )
        .is_ok());
        let expected =
            b"Todo lists from fake/folder1\n# title2\n* [x] completed1\n* [x] completed2\n";
        assert_eq!(
            stdout,
            expected,
            "\ngot     : \"{}\"\nexpected: \"{}\"",
            String::from_utf8(stdout.to_vec()).unwrap(),
            String::from_utf8(expected.to_vec()).unwrap()
        );
    }
}
