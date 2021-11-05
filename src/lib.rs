//! Todo is a tool to help you create todos, todo contexts and manage them. Just
//! like Git, Todo is comprised of multiple subcommmands arranged in their
//! respective modules.
//!
//! Compile the tool and start using it with TODO more details needed<br>
//! `todo --version`
use log::{debug, trace};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::io::Read;

pub mod config;
pub mod config_active_context;
pub mod config_create_context;
pub mod config_get_contexts;
pub mod config_set_context;
pub mod create;
pub mod delete;
pub mod edit;
pub mod list;

#[derive(Deserialize, Debug)]
/// ActiveContext represents the active Todo context in the Todo configuration. The active context
/// is identified by its unique name
pub struct ActiveContext {
    pub active_ctx_name: String,
}

impl fmt::Display for ActiveContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "active_ctx_name = \"{}\"", self.active_ctx_name)
    }
}

#[derive(Clone, Deserialize, Debug, Serialize)]
/// Todo context posseses its own set of Todos and parameters. A context is part of a Todo configuration.
pub struct Context {
    pub ide: String,
    pub name: String,
    pub timezone: String,
    pub todo_folder: String,
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "[[config]]\nname = \"{}\"\nide = \"{}\"\ntimezone = \"{}\"\ntodo_folder = \"{}\"",
            self.name, self.ide, self.timezone, self.todo_folder
        )
    }
}

#[derive(Deserialize, Debug, Serialize)]
/// A Todo Configuration is a set of Todo contexts.
pub struct Configuration {
    pub active_ctx_name: String,
    pub ctxs: Vec<Context>,
}

impl fmt::Display for Configuration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "active_ctx_name = {}\n", self.active_ctx_name)?;
        for config in self.ctxs.iter() {
            writeln!(
                f,
                "==={} context===\nide\t\t: {}\ntimezone\t: {}\nfolder\t\t: {}",
                config.name, config.ide, config.timezone, config.todo_folder
            )?;
        }
        Ok(())
    }
}

impl Configuration {
    /// Update active context with new_active_ctx_name if there is one contexts name where its name
    /// matches
    fn update_active_ctx(&mut self, new_active_ctx_name: &str) -> Result<(), &str> {
        if new_active_ctx_name.is_empty() {
            return Err("Active context has no name");
        }

        if self
            .ctxs
            .iter()
            .find(|ctx| ctx.name == new_active_ctx_name)
            .is_none()
        {
            return Err("No matching context could be found among available contexts");
        }

        self.active_ctx_name = new_active_ctx_name.to_string();
        Ok(())
    }
}

#[derive(Deserialize, Debug)]
/// Todo object is uniquely identified by its name in its Todo context
pub struct Todo {
    pub title: String,
    pub label: Vec<String>,
    pub content: String,
    pub items: Vec<String>,
    pub motives: Vec<String>,
}

impl fmt::Display for Todo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "---\nTITLE={}\nLABEL={}\n---",
            self.title,
            self.label.join(","),
        )?;

        if !self.content.is_empty() {
            writeln!(f, "{}\n---\n", self.content)?;
        }

        if !self.items.is_empty() {
            writeln!(f, "# Todo\n")?;
            for i in self.items.iter() {
                writeln!(f, "* [ ] {}", i)?;
            }
            writeln!(f, "\n---\n")?;
        }

        if !self.motives.is_empty() {
            writeln!(f, "# Motives\n")?;
            let mut i = self.motives.len();
            for m in self.motives.iter().rev() {
                writeln!(f, "{}. {}", i, m)?;
                i = i - 1;
            }
            writeln!(f, "\n---")?;
        }

        Ok(())
    }
}

/// joins todo folder path and todo title into a filepath. The file is in markdown format.
pub fn todo_path(todo_folder: &str, todo_title: &str) -> String {
    format!("{}/{}.md", todo_folder, todo_title)
}

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
    fn barebones_todo() {
        let todo = Todo {
            title: String::from(""),
            label: vec![],
            content: String::from(""),
            items: vec![],
            motives: vec![],
        };
        let expected = String::from(
            "\
---
TITLE=
LABEL=
---
",
        );
        let output = format!("{}", todo);
        assert_eq!(expected, output);
    }

    #[test]
    fn all_options_todo() {
        let todo = Todo {
            title: String::from("hello"),
            label: vec![String::from("l1"), String::from("l2")],
            content: String::from("This is the hello todo list"),
            items: vec![String::from("i1 first"), String::from("i2 second")],
            motives: vec![String::from("m1 first"), String::from("m2 second")],
        };
        let expected = String::from(
            "\
---
TITLE=hello
LABEL=l1,l2
---
This is the hello todo list
---

# Todo

* [ ] i1 first
* [ ] i2 second

---

# Motives

2. m2 second
1. m1 first

---
",
        );
        let output = format!("{}", todo);
        assert_eq!(expected, output);
    }

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
                assert_eq!("config1", configuration.active_ctx_name);
                assert!(configuration.ctxs.len() == 2);
                let c1 = configuration.ctxs[1].clone();
                assert_eq!("config1", c1.name);
                assert_eq!("config1_ide", c1.ide);
                assert_eq!("config1_timezone", c1.timezone);
                assert_eq!("/path/to/config1/folder", c1.todo_folder);
                let c2 = configuration.ctxs[2].clone();
                assert_eq!("config2", c2.name);
                assert_eq!("config2_ide", c2.ide);
                assert_eq!("config2_timezone", c2.timezone);
                assert_eq!("/path/to/config2/folder", c2.todo_folder);
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
        assert_eq!("config2", config.active_ctx_name);
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
}
