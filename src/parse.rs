//! Parse Todo lists and configuration from raw file content with various functions
//!
//! Todo lists are meant to be edited by a user with the edit command. Therefore, one cannot
//! serialize a Todo list with a crate and expect it to be managed by a human. This module parses also
//! the configuration file.
use super::{Configuration, Context};
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

    debug!("content: {}", content);
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

/// Returns title from Todo list
fn parse_todo_list_title(todo_raw: &str) -> Option<String> {
    let title_reg: Regex = Regex::new(r"^# (.+)\n").unwrap();
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

/// Returns the detailed informations about the task list of given Todo list. Tasks can be spread throughout the
/// file.
fn parse_todo_list_tasks_status(todo_raw: &str) -> (usize, usize) {
    trace!("parse_remaining_tasks");
    debug!("todo_raw: {:?}", todo_raw);
    let todo_list_reg: Regex = Regex::new(r"(?s)\n\n## Todo list\n\n(.*)\n").unwrap();
    let todo_list = todo_list_reg.captures(todo_raw);
    debug!("{:?}", todo_list);
    match todo_list {
        None => (0, 0),
        Some(caps) => {
            if caps.len() == 1 {
                (0, 0)
            } else {
                debug!("caps: {:?}", caps);
                let done_reg: Regex = Regex::new(r"(?m)^\* \[(.{1})\] .+$").unwrap();
                let mut done = 0;
                let todo_list = &caps[1];
                let matches = done_reg.find_iter(todo_list);
                let total = matches.count();
                for mat in done_reg.find_iter(todo_list) {
                    if mat.as_str().get(0..6).unwrap().eq("* [x] ") {
                        done = done + 1;
                    }
                }
                (done, total)
            }
        }
    }
}

/// Returns labels of Todo list
fn parse_todo_list_labels(todo_raw: &str) -> Result<Vec<String>, std::io::Error> {
    let label_re: Regex = Regex::new(r"## Description\n\nLABEL=(.*)").unwrap();
    let label_matches = label_re.captures(todo_raw).unwrap();
    debug!("label_matches: {:?}", label_matches);
    if label_matches.len() == 1 {
        eprintln!("Error while parsing labels");
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Error while parsing labels",
        ));
    }

    debug!("get 1: {}", label_matches.get(1).unwrap().as_str());
    // does collect to vec[""] but empty label is not a valid label
    debug!(
        "get 1: {:?}",
        label_matches
            .get(1)
            .unwrap()
            .as_str()
            .split(",")
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<String>>()
    );
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
    }
}
