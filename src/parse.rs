//! Parse Todo lists and configuration from raw file content with various functions
//!
//! Todo lists are meant to be edited by a user with the edit command. Therefore, one cannot
//! serialize a Todo list with a crate and expect it to be managed by a human. This module parses also
//! the configuration file.
use super::{Configuration, Context};
use lazy_static::lazy_static;
use log::{debug, trace};
use regex::Regex;
use std::io::Read;

/// Represents a parsed Todo list.
///
/// A parsed Todo list has many relevant informations such as its name and its task list status.
pub struct ParsedTodoList {
    pub raw: String,
    pub title: String,
    pub labels: Vec<String>,
    pub done: usize,
    pub total: usize,
}

impl ParsedTodoList {
    /// Returns true if all items from task list are checked
    pub fn tasks_are_all_done(&self) -> bool {
        self.done == self.total
    }
}

// Regexes which are used at several places
lazy_static! {
    static ref TODO_LIST_RE: Regex =
        Regex::new("\n## Todo list\n\n(?sm)(?P<list>.*?)(?-m:$|\n## .*)").unwrap();
}

/// Returns configuration of all Todo contexts and the name of the active context
///
/// Uses `raw configuration` when supplied instead of `todo_configuration_path`.
pub fn parse_configuration_file(
    todo_configuration_path: Option<&str>,
    raw_configuration: Option<&str>,
) -> Result<Configuration, std::io::Error> {
    let mut file_content = String::new();
    let content = match raw_configuration {
        Some(c) => c,
        None => {
            trace!("Opening configuration file");
            let todo_configuration_path =
                todo_configuration_path.expect("Configuration filepath could not be read");

            debug!("todo_configuration_path: {}", todo_configuration_path);
            let file = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(todo_configuration_path);
            // "Unable to open configuration file. Did you initialize configuration file with `todo config create-context`?",
            match file {
                Ok(mut f) => {
                    f.read_to_string(&mut file_content)?;
                    file_content.as_str()
                }
                Err(e) => return Err(e),
            }
        }
    };

    let configuration: Configuration = toml::from_str(content)?;
    if configuration
        .ctxs
        .iter()
        .find(|&c| c.name == configuration.active_ctx_name)
        .is_none()
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Invalid configuration because no contexts correspond to active context",
        ));
    }
    trace!("Active configuration is valid");
    Ok(configuration)
}

/// Returns active Todo context of configuration
///
/// Uses `raw configuration` when supplied instead of `todo_configuration_path`.
pub fn parse_active_context(
    todo_configuration_path: Option<&str>,
    configuration_raw: Option<&str>,
) -> Result<Context, std::io::Error> {
    let config = parse_configuration_file(todo_configuration_path, configuration_raw)?;
    trace!("Is active configuration valid?");
    let conf = config
        .ctxs
        .iter()
        .find(|&c| c.name == config.active_ctx_name)
        .expect("No configuration matched active context name");
    Ok(conf.clone())
}

/// Returns parsed Todo list
///
/// The motivation for this function is that instead of saving all the content through serializing
/// with a crate like Serde, the user can open the file and find it editable (think editing a json
/// vs xml file).
pub fn parse_todo_list(todo_raw: &str) -> Result<ParsedTodoList, std::io::Error> {
    let title = parse_todo_list_title(todo_raw);
    if title.is_none() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Todo list does not have a title",
        ));
    }
    let labels = parse_todo_list_labels(todo_raw).unwrap();
    let (done, total) = parse_todo_list_tasks_status(todo_raw);
    let todo = ParsedTodoList {
        raw: todo_raw.to_string(),
        title: title.unwrap(),
        labels,
        done,
        total,
    };

    Ok(todo)
}

/// Returns parsed Todo list
///
/// The motivation for this function is that instead of saving all the content through serializing
/// with a crate like Serde, the user can open the file and find it editable (think editing a json
/// vs xml file).
pub fn parse_todo_list_section(
    parsed_todo_list: &ParsedTodoList,
    section: &str,
) -> Result<ParsedTodoList, std::io::Error> {
    let section_re =
        Regex::new(format!("\n### {}\n\n(?sm)(?P<section>.*?)(?-m:$|\n### .*)", section).as_str())
            .unwrap();
    let todo_list_section = match section_re.captures(parsed_todo_list.raw.as_str()) {
        Some(cap) => cap.name("section").unwrap().as_str().to_string(),
        None => return Err(std::io::Error::new(std::io::ErrorKind::Other, "Oh no")),
    };
    let todo_list_section = format!("\n## Todo list\n\n{}", todo_list_section);
    let (done, total) = parse_todo_list_tasks_status(todo_list_section.as_str());
    let todo = ParsedTodoList {
        raw: todo_list_section,
        title: parsed_todo_list.title.to_string(),
        labels: parsed_todo_list.labels.to_owned(),
        done,
        total,
    };

    Ok(todo)
}

/// Returns tasks description of completed tasks and/or open tasks.
///
/// If `complete` and `open` are both false, this function will return an error.
pub fn parse_todo_list_tasks(
    todo_raw: &str,
    completed: bool,
    open: bool,
    short: bool,
    section: Option<&str>,
) -> Result<Vec<String>, std::io::Error> {
    if !completed && !open {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "complete and open parameters are not mutually exclusive",
        ));
    }
    let mut tasks = vec![];
    let todo_list = match TODO_LIST_RE.captures(todo_raw) {
        Some(cap) => cap,
        None => return Ok(tasks),
    };
    let mut todo_list = todo_list.name("list").unwrap().as_str().to_string();
    let mut todo_section = "".to_string();
    if let Some(s) = section {
        let section_re: Regex =
            Regex::new(format!("\n### {}\n\n(?sm)(?P<section>.*?)(?-m:$|\n### .*)", s).as_str())
                .unwrap();
        todo_section = match section_re.captures(todo_list.as_str()) {
            Some(cap) => cap.name("section").unwrap().as_str().to_string(),
            None => return Ok(tasks),
        };
    }

    if !todo_section.is_empty() {
        todo_list = todo_section;
    }
    lazy_static! {
        // Note: after 1-2 days, I figured out that the regex crate 1.5.4 does
        // not offer the required functionality to capture a bullet point in a
        // markdown file (delimited by '* [ ]' or the end of string). To capture
        // a section, you need the look-ahead feature of a regex engine (I have
        // not found a good workaround). Look-ahead does not evaluate in linear
        // time, which is against what the regex crate wants to offer.
        // Therefore, you need to import the fancy_regex crate for this type of
        // regexes (there is two of them).
        static ref COMPLETED_TASK_FRE: fancy_regex::Regex = fancy_regex::Regex::new(
            r"(?ms)(?P<summary>^\* \[x\] (?-m).*?)(?=\n\* \[(x|\s)\].*?|$)",
        )
        .unwrap();
        static ref COMPLETED_TASK_SHORT_RE: Regex =
            Regex::new(r"(?m)^(?P<summary>\* \[x\] .+)$").unwrap();
        static ref OPEN_TASK_FRE: fancy_regex::Regex = fancy_regex::Regex::new(
            r"(?ms)(?P<summary>^\* \[\s\] (?-m).*?)(?=\n\* \[(x|\s)\].*?|$)",
        )
        .unwrap();
        static ref OPEN_TASK_SHORT_RE: Regex =
            Regex::new(r"(?m)(?P<summary>^\* \[\s\] .+)$").unwrap();
        static ref EITHER_TASK_SHORT_RE: Regex =
            Regex::new(r"(?m)(?P<summary>^\* \[[x|\s]\] .+)$").unwrap();
        static ref EITHER_TASK_FRE: fancy_regex::Regex = fancy_regex::Regex::new(
            r"(?ms)(?P<summary>^\* \[[x|\s]\] (?-m).*?)(?=\n\* \[(x|\s)\].*?|$)",
        )
        .unwrap();
    }

    if short {
        let re = match (completed, open) {
            (true, false) => COMPLETED_TASK_SHORT_RE.clone(),
            (false, true) => OPEN_TASK_SHORT_RE.clone(),
            (true, true) => EITHER_TASK_SHORT_RE.clone(),
            _ => unreachable!(),
        };
        // You cannot return static items in a match, hence the
        // need to copy from them
        for caps in re.captures_iter(todo_list.as_str()) {
            trace!("CAP");
            let task = caps["summary"].to_string();
            tasks.push(task);
        }
    } else {
        let fre = match (completed, open) {
            (true, false) => COMPLETED_TASK_FRE.clone(),
            (false, true) => OPEN_TASK_FRE.clone(),
            (true, true) => EITHER_TASK_FRE.clone(),
            _ => unreachable!(),
        };
        // You cannot return static items in a match, hence the
        // need to copy from them
        for caps in fre.captures_iter(todo_list.as_str()) {
            let task = caps.unwrap()["summary"].to_string();
            tasks.push(task);
        }
    }

    Ok(tasks)
}

/// Returns title from Todo list
fn parse_todo_list_title(todo_raw: &str) -> Option<String> {
    lazy_static! {
        static ref TITLE_RE: Regex = Regex::new(r"^# (.+)\n").unwrap();
    }
    match TITLE_RE.captures(todo_raw) {
        None => None,
        Some(caps) => {
            if caps.len() == 1 {
                None
            } else {
                Some(String::from(&caps[1]))
            }
        }
    }
}

/// Returns the detailed informations about the task list of given Todo list. Tasks can be spread throughout the
/// file.
fn parse_todo_list_tasks_status(todo_raw: &str) -> (usize, usize) {
    let todo_list = match TODO_LIST_RE.captures(todo_raw) {
        Some(cap) => cap,
        None => return (0, 0),
    };
    let todo_list = todo_list.name("list").unwrap();
    lazy_static! {
        static ref DONE_RE: Regex = Regex::new(r"(?m)^\* \[(.{1})\] .+$").unwrap();
    }
    let mut done = 0;
    let matches = DONE_RE.find_iter(todo_list.as_str());
    let total = matches.count();
    for mat in DONE_RE.find_iter(todo_list.as_str()) {
        if mat.as_str().get(0..6).unwrap().eq("* [x] ") {
            done = done + 1;
        }
    }
    (done, total)
}

/// Returns labels of Todo list
fn parse_todo_list_labels(todo_raw: &str) -> Result<Vec<String>, std::io::Error> {
    lazy_static! {
        static ref LABEL_RE: Regex = Regex::new(r"## Description\n\nLABEL=(.*)").unwrap();
    }
    let label_matches = LABEL_RE.captures(todo_raw).unwrap();
    if label_matches.len() == 1 {
        eprintln!("Error while parsing labels");
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Error while parsing labels",
        ));
    }

    // does collect to vec[""] but empty label is not a valid label
    let file_labels = label_matches
        .get(1)
        .unwrap()
        .as_str()
        .split(",")
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .collect::<Vec<String>>();

    Ok(file_labels)
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

    const SAMPLE_CONFIG: &'static str = "active_ctx_name = \"config1\"

[[config]]
name = \"config1\"
ide = \"config1_ide\"
timezone = \"config1_timezone\"
todo_folder = \"/path/to/config1/folder\"

[[config]]
name = \"config2\"
ide = \"config2_ide\"
timezone = \"config2_timezone\"
todo_folder = \"/path/to/config2/folder\"";

    #[test]
    fn configuration_file_parses_configuration() {
        init();
        let todo_configuration_path = None;
        let raw_configuration = Some(SAMPLE_CONFIG);
        match parse_configuration_file(todo_configuration_path, raw_configuration) {
            Err(e) => eprintln!("{}", e),
            Ok(configuration) => {
                assert_eq!(configuration.active_ctx_name, "config1");
                assert!(configuration.ctxs.len() == 2);
                let c1 = configuration.ctxs[1].clone();
                assert_eq!(c1.name, "config1");
                assert_eq!(c1.ide, "config1_ide");
                assert_eq!(c1.timezone, "config1_timezone");
                assert_eq!(c1.folder_location, "/path/to/config1/folder");
                let c2 = configuration.ctxs[2].clone();
                assert_eq!(c2.name, "config1");
                assert_eq!(c2.ide, "config1_ide");
                assert_eq!(c2.timezone, "config1_timezone");
                assert_eq!(c2.folder_location, "/path/to/config1/folder");
            }
        }
    }

    #[test]
    fn update_config_with_missing_config() {
        init();
        let mut config = Configuration {
            active_ctx_name: String::from("config1"),
            ctxs: vec![
                Context {
                    ide: String::from(""),
                    name: String::from("config1"),
                    timezone: String::from(""),
                    folder_location: String::from(""),
                },
                Context {
                    ide: String::from(""),
                    name: String::from("config2"),
                    timezone: String::from(""),
                    folder_location: String::from(""),
                },
            ],
        };
        assert!(config.update_active_ctx("missing_config").is_err());
    }

    #[test]
    fn parse_todo_empty_title_produces_error() {
        init();
        let todo_raw = "\
# 

## Description

LABEL=
";
        let todo = parse_todo_list(todo_raw);
        assert!(todo.is_err());
    }

    #[test]
    fn parse_todo_simple() {
        init();
        let todo_raw = "\
# Title

## Description

LABEL=l1,l2,l3
";
        let todo_list = parse_todo_list(todo_raw);
        assert!(todo_list.is_ok());
        let todo = todo_list.unwrap();
        assert_eq!(todo.title, "Title");
        assert_eq!(todo.raw, todo_raw);
        assert!(todo.labels.contains(&String::from("l1")));
        assert!(todo.labels.contains(&String::from("l2")));
        assert!(todo.labels.contains(&String::from("l3")));
        assert_eq!(todo.labels.len(), 3);
    }

    #[test]
    fn empty_title() {
        init();
        let todo_raw = "\
# 

## Description

LABEL=
";
        assert!(parse_todo_list_title(todo_raw).is_none());
    }

    #[test]
    fn parse_todo_no_labels() {
        init();
        let todo_raw = "\
# Title

## Description

LABEL=
";
        let todo = parse_todo_list(todo_raw).unwrap();
        assert_eq!(todo.labels.len(), 0);
    }

    #[test]
    fn parse_no_tasks() {
        init();
        let todo_raw = "\
# Title

## Description

LABEL=
";
        let (done, total) = parse_todo_list_tasks_status(todo_raw);
        assert_eq!(0, done);
        assert_eq!(0, total);
    }

    #[test]
    fn parse_one_remaining_tasks() {
        init();
        let todo_raw = "\
# Title

## Description

LABEL=

## Todo list

* [ ] idk man

";
        let (done, total) = parse_todo_list_tasks_status(todo_raw);
        assert_eq!(0, done);
        assert_eq!(1, total);

        assert!(!parse_todo_list(todo_raw).unwrap().tasks_are_all_done());
    }

    #[test]
    fn parse_one_done_tasks() {
        init();
        let todo_raw = "\
# Title

## Description

LABEL=

## Todo list

* [x] idk man

";
        let (done, total) = parse_todo_list_tasks_status(todo_raw);
        assert_eq!(1, done);
        assert_eq!(1, total);

        assert!(parse_todo_list(todo_raw).unwrap().tasks_are_all_done());
    }

    #[test]
    fn parse_multiple_remaining_tasks() {
        init();
        let todo_raw = "\
# Title

## Description

LABEL=

## Todo list

* [ ] idk man
* [x] idk man
* [ ] idk man
* [x] idk man
* [ ] idk man

";
        let (done, total) = parse_todo_list_tasks_status(todo_raw);
        assert_eq!(2, done, "wrong number of done tasks");
        assert_eq!(5, total);

        assert!(!parse_todo_list(todo_raw).unwrap().tasks_are_all_done());
    }

    #[test]
    fn parse_multiple_all_done_tasks() {
        init();
        let todo_raw = "\
# Title

## Description

LABEL=

## Todo list

* [x] idk man
* [x] idk man
* [x] idk man
* [x] idk man
* [x] idk man

";
        let (done, total) = parse_todo_list_tasks_status(todo_raw);
        assert_eq!(5, done);
        assert_eq!(5, total);

        assert!(parse_todo_list(todo_raw).unwrap().tasks_are_all_done());
    }

    #[test]
    fn parse_tasks_only_in_todo_list_section() {
        init();
        let todo_raw = "\
# Title

## Description

LABEL=

## Description 

Confusing description

* [x] confusing point

## Todo list

* [x] idk man
* [x] idk man
* [x] idk man
* [x] idk man
* [x] idk man

";
        let (done, total) = parse_todo_list_tasks_status(todo_raw);
        assert_eq!(5, done);
        assert_eq!(5, total);

        assert!(parse_todo_list(todo_raw).unwrap().tasks_are_all_done());

        let todo_raw = "\
# Title

## Description

LABEL=

## Description 

Confusing description

* [x] confusing point

## Todo list

* [x] idk man
* [x] idk man
* [x] idk man
* [x] idk man
* [x] idk man

## Some other section

* [x] this should not be counted
";
        let (done, total) = parse_todo_list_tasks_status(todo_raw);
        assert_eq!(5, done);
        assert_eq!(5, total);

        assert!(parse_todo_list(todo_raw).unwrap().tasks_are_all_done());
    }

    #[test]
    fn parse_todo_list_tasks_assertion() {
        init();
        let todo_raw = "";
        let completed = true;
        let open = true;
        let short = true;
        assert!(parse_todo_list_tasks(&todo_raw, completed, open, short, None).is_ok());
        let completed = true;
        let open = true;
        let short = false; // testing if short modifies this behavior
        assert!(parse_todo_list_tasks(&todo_raw, completed, open, short, None).is_ok());
        let completed = false;
        let open = true;
        let short = false;
        assert!(parse_todo_list_tasks(&todo_raw, completed, open, short, None).is_ok());
        let completed = true;
        let open = false;
        let short = false;
        assert!(parse_todo_list_tasks(&todo_raw, completed, open, short, None).is_ok());
        let completed = false;
        let open = false;
        let short = false;
        assert!(parse_todo_list_tasks(&todo_raw, completed, open, short, None).is_err());
        let completed = false;
        let open = false;
        let short = true;
        assert!(parse_todo_list_tasks(&todo_raw, completed, open, short, None).is_err());
    }

    #[test]
    fn parse_todo_list_completed_tasks_short_description() {
        init();
        let todo_raw = "\
# Title

## Description

LABEL=

## Description

Confusing description

* [x] confusing point

## Todo list

* [x] completed1
* [x] completed2
* [ ] open1
* [x] completed3 long description
this line should not be caught
* [x] completed4 long description
this line should not be caught either

";
        let completed = true;
        let open = false;
        let short = true;
        let tasks = parse_todo_list_tasks(todo_raw, completed, open, short, None).unwrap();
        let expected: Vec<String> = vec![
            String::from("* [x] completed1"),
            String::from("* [x] completed2"),
            String::from("* [x] completed3 long description"),
            String::from("* [x] completed4 long description"),
        ];
        assert_eq!(tasks, expected);
    }

    #[test]
    fn parse_todo_list_open_tasks_short_description() {
        init();
        let todo_raw = "\
# Title

## Description

LABEL=

## Description

Confusing description

* [x] confusing point

## Todo list

* [ ] open1
* [x] completed1
* [x] completed2
* [ ] open2 long description
this line should not be caught
* [ ] open3 long description
this line should not be caught either

";
        let completed = false;
        let open = true;
        let short = true;
        let tasks = parse_todo_list_tasks(todo_raw, completed, open, short, None).unwrap();
        let expected: Vec<String> = vec![
            String::from("* [ ] open1"),
            String::from("* [ ] open2 long description"),
            String::from("* [ ] open3 long description"),
        ];
        assert_eq!(tasks, expected);
    }

    #[test]
    fn parse_todo_list_completed_tasks_description() {
        init();
        let todo_raw = "\
# Title

## Description

LABEL=

## Description

Confusing description

* [x] confusing point

## Todo list

* [x] completed1
* [x] completed2
* [ ] open1
* [x] completed3 long description
this line should be caught
* [x] completed4 long description
this line should also be caught

";
        let completed = true;
        let open = false;
        let short = false;
        let tasks = parse_todo_list_tasks(todo_raw, completed, open, short, None).unwrap();
        let expected: Vec<String> = vec![
            String::from("* [x] completed1"),
            String::from("* [x] completed2"),
            String::from("* [x] completed3 long description\nthis line should be caught"),
            String::from("* [x] completed4 long description\nthis line should also be caught\n\n"),
        ];
        assert_eq!(tasks, expected);
    }

    #[test]
    fn parse_todo_list_open_tasks_description() {
        init();
        let todo_raw = "\
# Title

## Description

LABEL=

## Description

Confusing description

* [x] confusing point

## Todo list

* [ ] open1
* [x] completed1
* [x] completed2
* [ ] open2 long description
this line should be caught
* [ ] open3 long description
this line should also be caught

";
        let completed = false;
        let open = true;
        let short = false;
        let tasks = parse_todo_list_tasks(todo_raw, completed, open, short, None).unwrap();
        let expected: Vec<String> = vec![
            String::from("* [ ] open1"),
            String::from("* [ ] open2 long description\nthis line should be caught"),
            String::from("* [ ] open3 long description\nthis line should also be caught\n\n"),
        ];
        assert_eq!(tasks, expected);
    }
}
