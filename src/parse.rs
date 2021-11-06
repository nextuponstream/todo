use super::{Configuration, Context};
use log::{debug, trace};
use regex::Regex;
use std::io::Read;

/// Opens configuration file and returns active Todo context and configurations. Uses `raw
/// configuration` when supplied.
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

/// Parses configuration file at `todo_configuration_path`. Uses supplied `configuration_raw` when
/// provided. Fails when configuration file is either badly formatted or the active context is invalid.
pub fn parse_active_ctx(
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

pub struct ParsedTodo {
    pub raw: String,
    pub title: String,
    pub labels: Vec<String>,
    done: usize,
    total: usize,
}

/// Parse raw input from todo file and extract relevant fields.
///
/// The motivation for this function is that instead of saving all the content through serializing
/// with a crate like Serde, the user can open the file and find it editable (think editing a json
/// vs xml file).
pub fn parse_todo(todo_raw: &str) -> Result<ParsedTodo, std::io::Error> {
    let title = parse_title(todo_raw);
    if title.is_none() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Todo list does not have a title",
        ));
    }
    let labels = parse_labels(todo_raw).unwrap();
    let (done, total) = parse_done_tasks(todo_raw);
    let todo = ParsedTodo {
        raw: todo_raw.to_string(),
        title: title.unwrap(),
        labels,
        done,
        total,
    };

    Ok(todo)
}

impl ParsedTodo {
    pub fn tasks_are_all_done(&self) -> bool {
        self.done == self.total
    }
}

/// Returns title from todo raw content
pub fn parse_title(todo_raw: &str) -> Option<String> {
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
pub fn parse_done_tasks(todo_raw: &str) -> (usize, usize) {
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

/// Returns labels of Todo list.
pub fn parse_labels(todo_raw: &str) -> Result<Vec<String>, std::io::Error> {
    let label_re: Regex = Regex::new(r"LABEL=(.*)\n---").unwrap();
    let label_matches = label_re.captures(todo_raw).unwrap();
    if label_matches.len() == 1 {
        eprintln!("Error while parsing labels");
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Error while parsing labels",
        ));
    }

    let file_labels = label_matches
        .get(1)
        .unwrap()
        .as_str()
        .split(",")
        .map(|s| s.to_string())
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
            LevelFilter::Debug,
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

    const TODO_BAREBONES: &'static str = "\
---
TITLE=
LABEL=
---
";

    #[test]
    fn empty_configuration_file_throws_error() {
        init();
        let mut config = Configuration {
            active_ctx_name: String::from(""),
            ctxs: vec![],
        };
        assert!(config.update_active_ctx("").is_err());
    }

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
                assert_eq!(c1.todo_folder, "/path/to/config1/folder");
                let c2 = configuration.ctxs[2].clone();
                assert_eq!(c2.name, "config1");
                assert_eq!(c2.ide, "config1_ide");
                assert_eq!(c2.timezone, "config1_timezone");
                assert_eq!(c2.todo_folder, "/path/to/config1/folder");
            }
        }
    }

    #[test]
    fn update_config_with_empty_title_fails() {
        init();
        let mut config = Configuration {
            active_ctx_name: String::from("config1"),
            ctxs: vec![
                Context {
                    ide: String::from(""),
                    name: String::from("config1"),
                    timezone: String::from(""),
                    todo_folder: String::from(""),
                },
                Context {
                    ide: String::from(""),
                    name: String::from("config2"),
                    timezone: String::from(""),
                    todo_folder: String::from(""),
                },
            ],
        };
        assert!(config.update_active_ctx("").is_err());
    }

    #[test]
    fn update_config_with_existing_config() {
        init();
        let mut config = Configuration {
            active_ctx_name: String::from("config1"),
            ctxs: vec![
                Context {
                    ide: String::from(""),
                    name: String::from("config1"),
                    timezone: String::from(""),
                    todo_folder: String::from(""),
                },
                Context {
                    ide: String::from(""),
                    name: String::from("config2"),
                    timezone: String::from(""),
                    todo_folder: String::from(""),
                },
            ],
        };
        assert!(config.update_active_ctx("config2").is_ok());
        assert_eq!(config.active_ctx_name, "config2");
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
                    todo_folder: String::from(""),
                },
                Context {
                    ide: String::from(""),
                    name: String::from("config2"),
                    timezone: String::from(""),
                    todo_folder: String::from(""),
                },
            ],
        };
        assert!(config.update_active_ctx("missing_config").is_err());
    }

    #[test]
    fn parse_todo_empty_title_produces_error() {
        init();
        let todo_raw = TODO_BAREBONES;
        let todo = parse_todo(todo_raw);
        assert!(todo.is_err());
    }

    #[test]
    fn parse_todo_simple() {
        init();
        let todo_raw = "\
---
TITLE=simple title
LABEL=l1,l2,l3
---
";
        let todo = parse_todo(todo_raw);
        assert!(todo.is_ok());
        let todo = todo.unwrap();
        assert_eq!(todo.title, "simple title");
        assert_eq!(todo.raw, todo_raw);
        assert!(todo.labels.contains(&String::from("l1")));
        assert!(todo.labels.contains(&String::from("l2")));
        assert!(todo.labels.contains(&String::from("l3")));
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
TITLE=title
LABEL=
---

* [ ] idk man

---
";
        let (done, total) = parse_done_tasks(todo_raw);
        assert_eq!(0, done);
        assert_eq!(1, total);

        assert!(!parse_todo(todo_raw).unwrap().tasks_are_all_done());
    }

    #[test]
    fn parse_one_done_tasks() {
        init();
        let todo_raw = "\
---
TITLE=title
LABEL=
---

* [x] idk man

---
";
        let (done, total) = parse_done_tasks(todo_raw);
        assert_eq!(1, done);
        assert_eq!(1, total);

        assert!(parse_todo(todo_raw).unwrap().tasks_are_all_done());
    }

    #[test]
    fn parse_multiple_remaining_tasks() {
        init();
        let todo_raw = "\
---
TITLE=title
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

        assert!(!parse_todo(todo_raw).unwrap().tasks_are_all_done());
    }

    #[test]
    fn parse_multiple_all_done_tasks() {
        init();
        let todo_raw = "\
---
TITLE=title
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

        assert!(parse_todo(todo_raw).unwrap().tasks_are_all_done());
    }
}
