//! List all Todo lists in active Todo context
use crate::{
    parse::{parse_todo_list, parse_todo_list_section, parse_todo_list_tasks},
    Configuration, Context,
};
use clap::{crate_authors, App, Arg, ArgMatches};
use log::debug;
use std::{fs::read_to_string, path::Path};
use walkdir::WalkDir;

/// The list of parameters for the `todo list` subcommand
//
// This struct is introduced to avoid development pain where adding a new
// parameter forces the developer to update 30+ test cases with 30 default
// values, which also look messy when doing a `git diff HEAD~1` (30 lines
// updated which don't change much). With the help of the `new()` command, you
// initialize default values and then set the values you need, avoiding a lot of
// boilerplate initialization.
//
// NOTE: This struct does not inclue the byte stream output (stdout or Vec<u8>)
// because trying to put a mutable shared reference to std::io::stdout() is a
// real pain that can be avoided (I don't know enough about lifetimes and the
// various advanced types of Rust to make it work). Besides, I see very few
// reasons why the list argument would need more shared mutable references so
// it's ok if the list_command_process function only has two arguments.
//
// The reason to preserve this reference to either stdout or a Vec<u8> is that
// during tests, you need to check the correctness of what is printed out
// (the Vec<u8> substituting as stdout).
#[derive(Debug)]
pub struct Parameters<'a> {
    pub all: bool,
    pub completed: bool,
    pub config: Configuration,
    pub done: bool,
    entries: Option<Vec<Vec<&'a str>>>,
    pub global: bool,
    pub labels: Vec<&'a str>,
    pub open: bool,
    pub short: bool,
    pub task_lists: Option<Vec<&'a str>>,
    pub sections: Option<Vec<&'a str>>,
}

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
            Arg::with_name("open-tasks")
                .short("o")
                .long("open")
                .help("Shows only open tasks in the lists (default shows the entire task list)"),
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
            Arg::with_name("sections")
                .long("section")
                .multiple(true)
                .number_of_values(1)
                .takes_value(true)
                .help("Shows specified section of task list"),
        )
        .arg(
            Arg::with_name("task-lists")
                .short("t")
                .long("task-lists")
                .help("Show only specified task lists.")
                .takes_value(true)
                .multiple(true)
                .index(1),
        )
}

/// Lists Todo lists from Todo context while filtering by label and whether or not the task list is
/// completed
pub fn list_command_process(
    args: &ArgMatches,
    config: &Configuration,
) -> Result<(), std::io::Error> {
    let parameters = Parameters {
        all: args.is_present("all"),
        completed: args.is_present("completed-tasks"),
        config: config.to_owned(),
        done: args.is_present("done"),
        entries: None,
        global: args.is_present("global"),
        labels: args
            .values_of("label")
            .unwrap_or_default()
            .collect::<Vec<_>>(),
        open: args.is_present("open-tasks"),
        short: args.is_present("short"),
        task_lists: match args.values_of("task-lists") {
            Some(tls) => Some(tls.collect::<Vec<_>>()),
            None => None,
        },
        sections: match args.values_of("sections") {
            Some(ss) => Some(ss.collect::<Vec<_>>()),
            None => None,
        },
    };

    list_message(&mut std::io::stdout(), parameters)
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
/// * `task_lists` - when provided, show only specified task lists
fn list_message(stdout: &mut dyn std::io::Write, p: Parameters) -> Result<(), std::io::Error> {
    if !p.config.is_valid() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Bad configuration file",
        ));
    }

    let task_lists = p.task_lists.unwrap_or(vec![]);
    let sections = p.sections.unwrap_or(vec![]);

    if p.entries.is_some() {
        let mut entries = p.entries.unwrap();
        assert_eq!(
            entries.len(),
            p.config.ctxs.len(),
            "entries and configuration contexts number do not match"
        );
        let mut ctxs = p.config.ctxs.clone();
        ctxs.reverse();
        entries.reverse();

        for ctx in p.config.ctxs.clone() {
            let directory = entries.pop().unwrap();
            if !p.global && ctx.name != p.config.active_ctx_name {
                continue;
            }

            print_todo_folder_location(stdout, &ctx)?;
            debug!("directory: {}\n- files:\n{:?}", ctx.name, directory);
            for todo_raw in directory {
                let todo_list = parse_todo_list(todo_raw).unwrap();
                if task_lists.is_empty() || task_lists.contains(&todo_list.title.as_str()) {
                    print_todo(
                        stdout,
                        todo_raw,
                        &p.labels,
                        p.all,
                        p.done,
                        p.short,
                        p.completed,
                        p.open,
                        &sections,
                    )?;
                }
            }
        }

        return Ok(());
    }

    for ctx in &p.config.ctxs {
        if !p.global && ctx.name != p.config.active_ctx_name {
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

            // NOTE: one could form directly the path to the file and directly
            // check if it exists or not to avoid iterating through all the
            // files in the context.
            let todo_list = parse_todo_list(todo_raw.as_str()).unwrap();
            if task_lists.is_empty() || task_lists.contains(&todo_list.title.as_str()) {
                print_todo(
                    stdout,
                    todo_raw.as_str(),
                    &p.labels,
                    p.all,
                    p.done,
                    p.short,
                    p.completed,
                    p.open,
                    &sections,
                )?;
            }
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
/// NOTE: there is two references, one for tests and one for list command. We avoid the petty case
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
    sections: &Vec<&str>,
) -> Result<(), std::io::Error> {
    let todo_list = parse_todo_list(&todo_raw).unwrap();
    if labels
        .iter()
        .all(|l| todo_list.labels.iter().any(|fl| fl == l))
    {
        let is_done = todo_list.tasks_are_all_done();
        // so XOR is a thing: https://doc.rust-lang.org/reference/types/boolean.html#logical-xor
        if !all && (is_done ^ done) {
            return Ok(());
        }

        if completed || open {
            writeln!(stdout, "# {}", todo_list.title)?;
            if sections.is_empty() {
                let tasks = parse_todo_list_tasks(todo_raw, completed, open, short, None).unwrap();
                for task in tasks {
                    // trim_end avoid cluttering the output with all whitespace the
                    // user might have used to make his Todo list more readable or
                    // the accidental trailing spaces he might have left
                    writeln!(stdout, "{}", task.as_str().trim_end())?;
                }
            } else {
                for section in sections {
                    writeln!(stdout, "\n## {section}\n")?;
                    let tasks =
                        parse_todo_list_tasks(todo_raw, completed, open, short, Some(section))
                            .unwrap();
                    for task in tasks {
                        // trim_end avoid cluttering the output with all whitespace the
                        // user might have used to make his Todo list more readable or
                        // the accidental trailing spaces he might have left
                        writeln!(stdout, "{}", task.as_str().trim_end())?;
                    }
                }
            }
        } else {
            if sections.is_empty() {
                if short {
                    writeln!(
                        stdout,
                        "{}/{}\t- {}",
                        todo_list.done, todo_list.total, todo_list.title
                    )?;
                } else {
                    writeln!(stdout, "{}", todo_raw)?;
                }
            } else {
                for section in sections {
                    let todo_list_section = parse_todo_list_section(&todo_list, section).unwrap();
                    if short {
                        writeln!(
                            stdout,
                            "{}/{}\t- {} ({section})",
                            todo_list_section.done,
                            todo_list_section.total,
                            todo_list_section.title
                        )?;
                    } else {
                        writeln!(stdout, "{}", todo_raw)?;
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lazy_static::lazy_static;
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

    // The builder pattern makes it easy to create new structs for a growing
    // number of test cases with nice default values that may need tweaking.
    // Because it is only for testing, then the builder methods do not need to
    // be public when processing a `todo list` issued by the user since all
    // relevant fields are public already.
    impl<'a> Parameters<'a> {
        /// Set `all` parameter to true
        fn all(mut self) -> Parameters<'a> {
            self.all = true;
            self
        }

        /// Set `completed` parameter to true
        fn completed(mut self) -> Parameters<'a> {
            self.completed = true;
            self
        }

        /// Set `config` parameter to true
        fn config(mut self, config: Configuration) -> Parameters<'a> {
            self.config = config;
            self
        }

        /// Set `done` parameter to true
        fn done(mut self) -> Parameters<'a> {
            self.done = true;
            self
        }

        /// Set entries for testing purposes
        fn entries(mut self, entries: Vec<Vec<&'a str>>) -> Parameters {
            self.entries = Some(entries);
            self
        }

        /// Set `global` parameter to true
        fn global(mut self) -> Parameters<'a> {
            self.global = true;
            self
        }

        /// Set labels
        fn labels(mut self, labels: Vec<&'a str>) -> Parameters {
            self.labels = labels;
            self
        }

        /// Build a new Parameter struct.
        fn new() -> Parameters<'a> {
            Parameters {
                all: false,
                completed: false,
                config: Configuration::new(),
                done: false,
                entries: None,
                global: false,
                labels: vec![],
                open: false,
                short: false,
                task_lists: None,
                sections: None,
            }
        }

        /// Set `open` parameter to true
        fn open(mut self) -> Parameters<'a> {
            self.open = true;
            self
        }

        /// Set `short` parameter to true
        fn short(mut self) -> Parameters<'a> {
            self.short = true;
            self
        }

        /// Set task lists in Parameters struct:
        fn task_lists(mut self, task_lists: Vec<&'a str>) -> Parameters {
            self.task_lists = Some(task_lists);
            self
        }

        /// Set task lists in Parameters struct:
        fn sections(mut self, sections: Vec<&'a str>) -> Parameters {
            self.sections = Some(sections);
            self
        }
    }

    lazy_static! {
        static ref CONFIG_TWO_CTX_1: Configuration = Configuration {
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
        static ref CONFIG_TWO_CTX_2: Configuration = Configuration {
            active_ctx_name: String::from("ctx2"),
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
        static ref CONFIG_ONE_CTX: Configuration = Configuration {
            active_ctx_name: String::from("ctx1"),
            ctxs: vec![Context {
                ide: String::from(""),
                name: String::from("ctx1"),
                timezone: String::from("CET"),
                folder_location: String::from("fake/folder"),
            }],
        };
    }

    // NOTE: testing buffered output idea https://stackoverflow.com/a/48393114
    // One could write to a string then display the message but the string can possibly take a lot
    // of memory before being written to stdout. Therefore, it is better to println as you iterate
    // through Todo lists. Testing is then a little more complicated than comparing two strings.
    // String does not implement std::io::Write so can't use Strings

    #[test]
    fn empty_configuration() {
        init();
        let mut stdout = vec![];
        let entries = vec![];
        let parameters = Parameters::new()
            .config(Configuration {
                active_ctx_name: String::from("ctx1"),
                ctxs: vec![],
            })
            .entries(entries);
        assert!(list_message(&mut stdout, parameters).is_err());
    }

    #[test]
    fn todo_context_with_no_todo_lists() {
        init();
        let mut stdout = vec![];
        let parameters = Parameters::new()
            .entries(vec![vec![]])
            .config(CONFIG_ONE_CTX.to_owned());

        assert!(list_message(&mut stdout, parameters).is_ok());
        let expected = b"Todo lists from fake/folder\n";
        assert_eq!(
            stdout,
            expected,
            "\ngot     : \"{}\"\nexpected: \"{}\"",
            String::from_utf8(stdout.to_owned()).unwrap(),
            String::from_utf8(expected.to_vec()).unwrap()
        );
    }

    #[test]
    fn list_todo_lists_from_one_config() {
        init();
        let mut stdout = vec![];
        let parameters = Parameters::new()
            .entries(vec![vec![
                "# title1\n\n## Description\n\nLABEL=\n\n## Todo list\n\n* [ ] first",
                "# title2\n\n## Description\n\nLABEL=\n\n## Todo list\n\n* [x] first",
            ]])
            .config(CONFIG_ONE_CTX.to_owned())
            .short();

        assert!(list_message(&mut stdout, parameters).is_ok());
        let expected = b"Todo lists from fake/folder\n0/1\t- title1\n";
        assert_eq!(
            stdout,
            expected,
            "\ngot     : \"{}\"\nexpected: \"{}\"",
            String::from_utf8(stdout.to_owned()).unwrap(),
            String::from_utf8(expected.to_vec()).unwrap()
        );

        let mut stdout = vec![];
        let parameters = Parameters::new()
            .entries(vec![vec![
                "# title1\n\n## Description\n\nLABEL=\n\n## Todo list\n\n* [ ] first",
                "# title2\n\n## Description\n\nLABEL=\n\n## Todo list\n\n* [x] first",
            ]])
            .config(CONFIG_ONE_CTX.to_owned())
            .short()
            .done();

        assert!(list_message(&mut stdout, parameters).is_ok());
        let expected = b"Todo lists from fake/folder\n1/1\t- title2\n";
        assert_eq!(
            stdout,
            expected,
            "\ngot     : \"{}\"\nexpected: \"{}\"",
            String::from_utf8(stdout.to_owned()).unwrap(),
            String::from_utf8(expected.to_vec()).unwrap()
        );

        let mut stdout = vec![];
        let parameters = Parameters::new()
            .entries(vec![vec![
                "# title1\n\n## Description\n\nLABEL=\n\n## Todo list\n\n* [ ] first",
                "# title2\n\n## Description\n\nLABEL=\n\n## Todo list\n\n* [x] first",
            ]])
            .config(CONFIG_ONE_CTX.to_owned())
            .short()
            .all();

        assert!(list_message(&mut stdout, parameters).is_ok());
        let expected = b"Todo lists from fake/folder\n0/1\t- title1\n1/1\t- title2\n";
        assert_eq!(
            stdout,
            expected,
            "\ngot     : \"{}\"\nexpected: \"{}\"",
            String::from_utf8(stdout.to_owned()).unwrap(),
            String::from_utf8(expected.to_vec()).unwrap()
        );

        let mut stdout = vec![];
        let parameters = Parameters::new()
            .entries(vec![vec![
                "# title1\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n* [ ] first",
                "# title2\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [x] first",
            ]])
            .config(CONFIG_ONE_CTX.to_owned())
            .labels(vec!["l2"])
            .short()
            .all();

        assert!(list_message(&mut stdout, parameters).is_ok());
        let expected = b"Todo lists from fake/folder\n1/1\t- title2\n";
        assert_eq!(
            stdout,
            expected,
            "\ngot     : \"{}\"\nexpected: \"{}\"",
            String::from_utf8(stdout.to_owned()).unwrap(),
            String::from_utf8(expected.to_vec()).unwrap()
        );

        let mut stdout = vec![];
        let parameters = Parameters::new()
            .entries(vec![vec![
                "# title1\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n* [ ] first",
                "# title2\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [x] first",
            ]])
            .config(CONFIG_ONE_CTX.to_owned())
            .labels(vec!["l1"])
            .short()
            .all();

        assert!(list_message(&mut stdout, parameters).is_ok());
        let expected = b"Todo lists from fake/folder\n0/1\t- title1\n";
        assert_eq!(
            stdout,
            expected,
            "\ngot     : \"{}\"\nexpected: \"{}\"",
            String::from_utf8(stdout.to_owned()).unwrap(),
            String::from_utf8(expected.to_vec()).unwrap()
        );
    }

    #[test]
    fn list_todo_lists_from_all_config() {
        init();
        let mut stdout = vec![];
        let parameters = Parameters::new()
            .entries(vec![
                vec![
                    "# title1\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n* [ ] first",
                    "# title2\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [x] first",
                ],
                vec![
                    "# title3\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n* [x] first",
                    "# title4\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [ ] first",
                ],
            ])
            .config(CONFIG_TWO_CTX_1.to_owned())
            .labels(vec!["l1"])
            .short()
            .all()
            .global();

        assert!(list_message(&mut stdout, parameters).is_ok());
        let expected = b"Todo lists from fake/folder1\n0/1\t- title1\nTodo lists from fake/folder2\n1/1\t- title3\n";
        assert_eq!(
            stdout,
            expected,
            "\ngot     : \"{}\"\nexpected: \"{}\"",
            String::from_utf8(stdout.to_owned()).unwrap(),
            String::from_utf8(expected.to_vec()).unwrap()
        );

        let mut stdout = vec![];
        let parameters = Parameters::new()
            .entries(vec![
                vec![
                    "# title1\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n* [ ] first",
                    "# title2\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [x] first",
                ],
                vec![
                    "# title3\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n* [x] first",
                    "# title4\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [ ] first",
                ],
            ])
            .config(CONFIG_TWO_CTX_1.to_owned())
            .labels(vec!["l2"])
            .short()
            .all()
            .global();

        assert!(list_message(&mut stdout, parameters).is_ok());
        let expected = b"Todo lists from fake/folder1\n1/1\t- title2\nTodo lists from fake/folder2\n0/1\t- title4\n";
        assert_eq!(
            stdout,
            expected,
            "\ngot     : \"{}\"\nexpected: \"{}\"",
            String::from_utf8(stdout.to_owned()).unwrap(),
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
        let mut stdout = vec![];
        let parameters=Parameters::new().entries(vec![
            vec![
                "# title1\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n* [ ] open1\n* [x] completed1\n* [ ] open2\n* [x] completed2",
                "# title2\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [x] completed1\n* [ ] open1\n* [x] completed2\n* [ ] open2",
            ],
            vec![
                "# title3\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n* [ ] open1\n* [x] completed1\n* [ ] open2\n* [x] completed2",
                "# title4\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [x] completed1\n* [ ] open1\n* [x] completed2\n* [ ] open2",
            ],
        ]).config(CONFIG_TWO_CTX_1.to_owned()).labels(vec!["l1"]).open();

        assert!(list_message(&mut stdout, parameters).is_ok());
        let expected = b"Todo lists from fake/folder1\n# title1\n* [ ] open1\n* [ ] open2\n";
        assert_eq!(
            stdout,
            expected,
            "\ngot     : \"{}\"\nexpected: \"{}\"",
            String::from_utf8(stdout.to_owned()).unwrap(),
            String::from_utf8(expected.to_vec()).unwrap()
        );

        let mut stdout = vec![];
        let parameters=Parameters::new().entries(vec![
            vec![
                "# title1\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n* [ ] open1\n* [x] completed1\n* [ ] open2\n* [x] completed2",
                "# title2\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [x] completed1\n* [ ] open1\n* [x] completed2\n* [ ] open2",
            ],
            vec![
                "# title3\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n* [ ] open1\n* [x] completed1\n* [ ] open2\n* [x] completed2",
                "# title4\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [x] completed1\n* [ ] open1\n* [x] completed2\n* [ ] open2",
            ],
        ]).config(CONFIG_TWO_CTX_1.to_owned()).labels(vec!["l2"]).open();

        assert!(list_message(&mut stdout, parameters).is_ok());
        let expected = b"Todo lists from fake/folder1\n# title2\n* [ ] open1\n* [ ] open2\n";
        assert_eq!(
            stdout,
            expected,
            "\ngot     : \"{}\"\nexpected: \"{}\"",
            String::from_utf8(stdout.to_owned()).unwrap(),
            String::from_utf8(expected.to_vec()).unwrap()
        );
    }

    #[test]
    fn list_completed_tasks() {
        init();
        let mut stdout = vec![];
        let parameters=Parameters::new().config(CONFIG_TWO_CTX_1.to_owned()).entries(vec![
            vec![
                "# title1\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n* [ ] open1\n* [x] completed1\n* [ ] open2\n* [x] completed2",
                "# title2\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [x] completed1\n* [ ] open1\n* [x] completed2\n* [ ] open2",
            ],
            vec![
                "# title3\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n* [ ] open1\n* [x] completed1\n* [ ] open2\n* [x] completed2",
                "# title4\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [x] completed1\n* [ ] open1\n* [x] completed2\n* [ ] open2",
            ],
        ]).labels(vec!["l1"]).completed();

        assert!(list_message(&mut stdout, parameters).is_ok());
        let expected =
            b"Todo lists from fake/folder1\n# title1\n* [x] completed1\n* [x] completed2\n";
        assert_eq!(
            stdout,
            expected,
            "\ngot     : \"{}\"\nexpected: \"{}\"",
            String::from_utf8(stdout.to_owned()).unwrap(),
            String::from_utf8(expected.to_vec()).unwrap()
        );

        let mut stdout = vec![];
        let parameters=Parameters::new().entries(vec![
            vec![
                "# title1\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n* [ ] open1\n* [x] completed1\n* [ ] open2\n* [x] completed2",
                "# title2\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [x] completed1\n* [ ] open1\n* [x] completed2\n* [ ] open2",
            ],
            vec![
                "# title3\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n* [ ] open1\n* [x] completed1\n* [ ] open2\n* [x] completed2",
                "# title4\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [x] completed1\n* [ ] open1\n* [x] completed2\n* [ ] open2",
            ],
        ]).config(CONFIG_TWO_CTX_1.to_owned()).labels(vec!["l2"]).completed();

        assert!(list_message(&mut stdout, parameters).is_ok());
        let expected =
            b"Todo lists from fake/folder1\n# title2\n* [x] completed1\n* [x] completed2\n";
        assert_eq!(
            stdout,
            expected,
            "\ngot     : \"{}\"\nexpected: \"{}\"",
            String::from_utf8(stdout.to_owned()).unwrap(),
            String::from_utf8(expected.to_vec()).unwrap()
        );
    }

    #[test]
    fn show_one_task_list() {
        init();
        let mut stdout = vec![];
        let parameters=Parameters::new().entries(vec![
            vec![
                "# title1\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n* [ ] open1\n* [x] completed1\n* [ ] open2\n* [x] completed2",
                "# title2\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [x] completed1\n* [ ] open1\n* [x] completed2\n* [ ] open2",
            ],
            vec![
                "# title3\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n* [ ] open1\n* [x] completed1\n* [ ] open2\n* [x] completed2",
                "# title4\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [x] completed1\n* [ ] open1\n* [x] completed2\n* [ ] open2",
            ],
        ]).config(CONFIG_TWO_CTX_2.to_owned()).open().task_lists(vec!["title3"]);

        assert!(list_message(&mut stdout, parameters).is_ok());
        let expected = b"Todo lists from fake/folder2\n# title3\n* [ ] open1\n* [ ] open2\n";
        assert_eq!(
            stdout,
            expected,
            "\ngot     : \"{}\"\nexpected: \"{}\"",
            String::from_utf8(stdout.to_owned()).unwrap(),
            String::from_utf8(expected.to_vec()).unwrap()
        );
    }

    #[test]
    fn show_many_task_lists() {
        init();
        let mut stdout = vec![];
        let parameters=Parameters::new().entries(vec![
            vec![
                "# title1\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n* [ ] open1\n* [x] completed1\n* [ ] open2\n* [x] completed2",
                "# title2\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [x] completed1\n* [ ] open1\n* [x] completed2\n* [ ] open2",
            ],
            vec![
                "# title3\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n* [ ] open1\n* [x] completed1\n* [ ] open2\n* [x] completed2",
                "# title4\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [x] completed1\n* [ ] open1\n* [x] completed2\n* [ ] open2",
                "# title5\n\n## Description\n\nLABEL=l3\n\n## Todo list\n\n* [x] completed1\n* [ ] open1\n* [x] completed2\n* [ ] open2\n* [ ] open3",
            ],
        ])
            .config(CONFIG_TWO_CTX_2.to_owned()).open()
        .task_lists (vec!["title3", "title5"]);

        assert!(list_message(&mut stdout, parameters).is_ok());
        let expected = b"Todo lists from fake/folder2\n# title3\n* [ ] open1\n* [ ] open2\n# title5\n* [ ] open1\n* [ ] open2\n* [ ] open3\n";
        assert_eq!(
            stdout,
            expected,
            "\ngot     : \"{}\"\nexpected: \"{}\"",
            String::from_utf8(stdout.to_owned()).unwrap(),
            String::from_utf8(expected.to_vec()).unwrap()
        );
    }

    #[test]
    fn show_section_open() {
        init();
        let mut stdout = vec![];
        let parameters = Parameters::new()
            .entries(vec![
            vec![
                "# title1\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n* [ ] open1\n* [x] completed1\n* [ ] open2\n* [x] completed2",
                "# title2\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [x] completed1\n* [ ] open1\n* [x] completed2\n* [ ] open2",
            ],
            vec![
                "# title3\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n\
* [ ] open1\n* [x] completed1\n\n### Section1\n\n* [ ] open2\n* [x] completed2\n\
\n### Section2\n\n* [ ] open3\n* [x] completed3",
                "# title4\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [x] completed1\n* [ ] open1\n* [x] completed2\n* [ ] open2",
                "# title5\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n\
* [x] completed1\n* [ ] open1\n\n### Section1\n\n* [x] completed2\n* [ ] open2\n\
\n### Section2\n\n* [x] completed3\n* [ ] open3",
            ],
        ])
            .config(CONFIG_TWO_CTX_2.to_owned())
            .sections(vec!["Section2"])
            .open()
            .task_lists(vec!["title3", "title5"]);

        assert!(list_message(&mut stdout, parameters).is_ok());
        let expected = b"Todo lists from fake/folder2\n# title3\n\n## Section2\
\n\n* [ ] open3\n# title5\n\n## Section2\n\n* [ ] open3\n";
        assert_eq!(
            stdout,
            expected,
            "\ngot     : \"{}\"\nexpected: \"{}\"",
            String::from_utf8(stdout.to_vec()).unwrap(),
            String::from_utf8(expected.to_vec()).unwrap()
        );
    }

    #[test]
    fn show_section_completed() {
        init();
        let mut stdout = vec![];
        let parameters = Parameters::new()
            .entries(vec![
            vec![
                "# title1\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n* [ ] open1\n* [x] completed1\n* [ ] open2\n* [x] completed2",
                "# title2\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [x] completed1\n* [ ] open1\n* [x] completed2\n* [ ] open2",
            ],
            vec![
                "# title3\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n\
* [ ] open1\n* [x] completed1\n\n### Section1\n\n* [ ] open2\n* [x] completed2\n\
\n### Section2\n\n* [ ] open3\n* [x] completed3",
                "# title4\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [x] completed1\n* [ ] open1\n* [x] completed2\n* [ ] open2",
                "# title5\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n\
* [x] completed1\n* [ ] open1\n\n### Section1\n\n* [x] completed2\n* [ ] open2\n\
\n### Section2\n\n* [x] completed3\n* [ ] open3",
            ],
        ])
            .config(CONFIG_TWO_CTX_2.to_owned()).sections(vec!["Section2"]).completed().task_lists(vec!["title3", "title5"]);

        assert!(list_message(&mut stdout, parameters).is_ok());
        let expected = b"Todo lists from fake/folder2\n# title3\n\n## Section2\
\n\n* [x] completed3\n# title5\n\n## Section2\n\n* [x] completed3\n";
        assert_eq!(
            stdout,
            expected,
            "\ngot     : \"{}\"\nexpected: \"{}\"",
            String::from_utf8(stdout.to_vec()).unwrap(),
            String::from_utf8(expected.to_vec()).unwrap()
        );
    }

    #[test]
    fn show_section() {
        init();
        let mut stdout = vec![];
        let parameters = Parameters::new()
            .entries(vec![
            vec![
                "# title1\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n* [ ] open1\n* [x] completed1\n* [ ] open2\n* [x] completed2",
                "# title2\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [x] completed1\n* [ ] open1\n* [x] completed2\n* [ ] open2",
            ],
            vec![
                "# title3\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n\
* [ ] open1\n* [x] completed1\n\n### Section1\n\n* [ ] open2\n* [x] completed2\n\
\n### Section2\n\n* [ ] open3\n* [x] completed3",
                "# title4\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [x] completed1\n* [ ] open1\n* [x] completed2\n* [ ] open2",
                "# title5\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n\
* [x] completed1\n* [ ] open1\n\n### Section1\n\n* [x] completed2\n* [ ] open2\n\
\n### Section2\n\n* [x] completed3\n* [ ] open3",
            ],
        ])
            .config(CONFIG_TWO_CTX_2.to_owned()).sections(vec!["Section2"]).completed().open().task_lists(vec!["title3", "title5"]);

        assert!(list_message(&mut stdout, parameters).is_ok());
        let expected = b"Todo lists from fake/folder2\n# title3\n\n## Section2\
\n\n* [ ] open3\n* [x] completed3\n# title5\n\n## Section2\n\n* [x] completed3\n* [ ] open3\n";
        assert_eq!(
            stdout,
            expected,
            "\ngot     : \"{}\"\nexpected: \"{}\"",
            String::from_utf8(stdout.to_vec()).unwrap(),
            String::from_utf8(expected.to_vec()).unwrap()
        );

        let mut stdout = vec![];
        let parameters = Parameters::new()
            .entries(vec![
            vec![
                "# title1\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n* [ ] open1\n* [x] completed1\n* [ ] open2\n* [x] completed2",
                "# title2\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [x] completed1\n* [ ] open1\n* [x] completed2\n* [ ] open2",
            ],
            vec![
                "# title3\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n\
* [ ] open1\n* [x] completed1\n\n### Section1\n\n* [ ] open2\n* [x] completed2\n\
\n### Section2\n\n* [ ] open3\n* [x] completed3\n\n### Section3\n\n* [ ] open4",
                "# title4\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [x] completed1\n* [ ] open1\n* [x] completed2\n* [ ] open2",
                "# title5\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n\
* [x] completed1\n* [ ] open1\n\n### Section1\n\n* [x] completed2\n* [ ] open2\n\
\n### Section2\n\n* [x] completed3\n* [ ] open3\n\n### Section3\n\n* [ ] open4",
            ],
        ])
            .config(CONFIG_TWO_CTX_2.to_owned()).sections(vec!["Section2"]).completed().open().task_lists(vec!["title3", "title5"]);

        assert!(list_message(&mut stdout, parameters).is_ok());
        let expected = b"Todo lists from fake/folder2\n# title3\n\n## Section2\
\n\n* [ ] open3\n* [x] completed3\n# title5\n\n## Section2\n\n* [x] completed3\n* [ ] open3\n";
        assert_eq!(
            stdout,
            expected,
            "\ngot     : \"{}\"\nexpected: \"{}\"",
            String::from_utf8(stdout.to_vec()).unwrap(),
            String::from_utf8(expected.to_vec()).unwrap()
        );

        let mut stdout = vec![];
        let parameters = Parameters::new()
            .entries(vec![
            vec![
                "# title1\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n* [ ] open1\n* [x] completed1\n* [ ] open2\n* [x] completed2",
                "# title2\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [x] completed1\n* [ ] open1\n* [x] completed2\n* [ ] open2",
            ],
            vec![
                "# title3\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n\
* [ ] open1\n* [x] completed1\n\n### Section1\n\n* [ ] open2\n* [x] completed2\n\
\n### Section 2\n\n* [ ] open3\n* [x] completed3\n\n### Section3\n\n* [ ] open4",
                "# title4\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [x] completed1\n* [ ] open1\n* [x] completed2\n* [ ] open2",
                "# title5\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n\
* [x] completed1\n* [ ] open1\n\n### Section1\n\n* [x] completed2\n* [ ] open2\n\
\n### Section 2\n\n* [x] completed3 long description\n* [ ] open3\n\n### Section3\n\n* [ ] open4",
            ],
        ])
            .config(CONFIG_TWO_CTX_2.to_owned()).sections(vec!["Section 2"]).completed().open().task_lists(vec!["title3", "title5"]);

        assert!(list_message(&mut stdout, parameters).is_ok());
        let expected = b"Todo lists from fake/folder2\n# title3\n\n## Section 2\
\n\n* [ ] open3\n* [x] completed3\n# title5\n\n## Section 2\n\n* [x] completed3 long description\n* [ ] open3\n";
        assert_eq!(
            stdout,
            expected,
            "\ngot     : \"{}\"\nexpected: \"{}\"",
            String::from_utf8(stdout.to_vec()).unwrap(),
            String::from_utf8(expected.to_vec()).unwrap()
        );
    }

    #[test]
    fn show_short_section() {
        let mut stdout = vec![];
        let parameters = Parameters::new()
            .entries(vec![
            vec![
                "# title1\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n* [ ] open1\n* [x] completed1\n* [ ] open2\n* [x] completed2",
                "# title2\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [x] completed1\n* [ ] open1\n* [x] completed2\n* [ ] open2",
            ],
            vec![
                "# title3\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n\
* [ ] open1\n* [x] completed1\n\n### Section1\n\n* [ ] open2\n* [x] completed2\n\
\n### Section 2\n\n* [ ] open3\n* [x] completed3\n\n### Section3\n\n* [ ] open4",
                "# title4\n\n## Description\n\nLABEL=l2\n\n## Todo list\n\n* [x] completed1\n* [ ] open1\n* [x] completed2\n* [ ] open2",
                "# title5\n\n## Description\n\nLABEL=l1\n\n## Todo list\n\n\
* [x] completed1\n* [ ] open1\n\n### Section1\n\n* [x] completed2\n* [ ] open2\n\
\n### Section 2\n\n* [x] completed3 long description\n* [ ] open3\n\n### Section3\n\n* [ ] open4",
            ],
        ])
            .config(CONFIG_TWO_CTX_2.to_owned())
            .sections(vec!["Section 2", "Section3"]).short()
            .task_lists(vec!["title3", "title5"]);

        assert!(list_message(&mut stdout, parameters).is_ok());
        let expected = b"Todo lists from fake/folder2\n1/2\t- title3 (Section 2\
)\n0/1\t- title3 (Section3)\n1/2\t- title5 (Section 2)\n0/1\t- title5 (Section3\
)\n";
        assert_eq!(
            stdout,
            expected,
            "\ngot     : \"{}\"\nexpected: \"{}\"",
            String::from_utf8(stdout.to_vec()).unwrap(),
            String::from_utf8(expected.to_vec()).unwrap()
        );
    }
}
