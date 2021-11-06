//! Todo is a tool to help you create todos, todo contexts and manage them. Just
//! like Git, Todo is comprised of multiple subcommmands arranged in their
//! respective modules.
//!
//! Compile the tool and start using it with TODO more details needed<br>
//! `todo --version`
use parse::parse_configuration_file;
use serde::{Deserialize, Serialize};
use std::fmt;

pub mod config;
pub mod config_active_context;
pub mod config_create_context;
pub mod config_get_contexts;
pub mod config_set_context;
pub mod create;
pub mod delete;
pub mod edit;
pub mod list;
pub mod parse;

#[derive(Clone, Deserialize, Debug, Serialize)]
/// Represents a themed set of Todo lists
///
/// Context is uniquely identified by its name. All related Todo lists are stored inside the same
/// folder.
pub struct Context {
    pub ide: String,
    pub name: String,
    pub timezone: String,
    pub folder_location: String,
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "[[config]]\nname = \"{}\"\nide = \"{}\"\ntimezone = \"{}\"\ntodo_folder = \"{}\"",
            self.name, self.ide, self.timezone, self.folder_location
        )
    }
}

#[derive(Deserialize, Debug, Serialize)]
/// Represents all Todo contexts and the active context of the configuration
pub struct Configuration {
    active_ctx_name: String,
    ctxs: Vec<Context>,
}

impl fmt::Display for Configuration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "active_ctx_name = {}\n", self.active_ctx_name)?;
        for config in self.ctxs.iter() {
            writeln!(
                f,
                "==={} context===\nide\t\t: {}\ntimezone\t: {}\nfolder\t\t: {}",
                config.name, config.ide, config.timezone, config.folder_location
            )?;
        }
        Ok(())
    }
}

impl Configuration {
    /// Updates active context in configuration
    ///
    /// The active context is updated when the given name matches the one of the context inside the configuration.
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
/// Represents a Todo list
///
/// Todo lists are uniquely identified by their name. Labels allows to theme your Todo list and
/// allow to be filtered out when listing all Todo lists with the `list` command.<br>
/// Todo list items are initially are not unchecked.
pub struct TodoList {
    title: String,
    description: String,
    labels: Vec<String>,
    list_items: Vec<String>,
    motives: Vec<String>,
}

impl fmt::Display for TodoList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO put title on top of the file with first level of heading like so `# Title`
        writeln!(
            f,
            "---\nTITLE={}\nLABEL={}\n---",
            self.title,
            self.labels.join(","),
        )?;

        if !self.description.is_empty() {
            writeln!(f, "{}\n---\n", self.description)?;
        }

        if !self.list_items.is_empty() {
            writeln!(f, "# Todo\n")?;
            for i in self.list_items.iter() {
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

/// Returns the path to the Todo list from given Todo context
///
/// The Todo list is always a markdown file for usability.
pub fn todo_path(todo_folder_of_todo_ctx: &str, todo_list_name: &str) -> String {
    format!("{}/{}.md", todo_folder_of_todo_ctx, todo_list_name)
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

    const TODO_BAREBONES: &'static str = "\
---
TITLE=
LABEL=
---
";

    #[test]
    fn barebones_todo() {
        init();
        let todo = TodoList {
            title: String::from(""),
            labels: vec![],
            description: String::from(""),
            list_items: vec![],
            motives: vec![],
        };
        let expected = TODO_BAREBONES;
        let output = format!("{}", todo);
        assert_eq!(output, expected);
    }

    #[test]
    fn all_options_todo() {
        init();
        let todo = TodoList {
            title: String::from("hello"),
            labels: vec![String::from("l1"), String::from("l2")],
            description: String::from("This is the hello todo list"),
            list_items: vec![String::from("i1 first"), String::from("i2 second")],
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
        assert_eq!(output, expected);
    }

    #[test]
    fn update_config_with_empty_title_fails() {
        init();
        let mut config = Configuration {
            active_ctx_name: String::from(""),
            ctxs: vec![],
        };
        assert!(config.update_active_ctx("").is_err());

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
                    name: String::from(""),
                    timezone: String::from(""),
                    folder_location: String::from(""),
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
        assert!(config.update_active_ctx("config2").is_ok());
        assert_eq!(config.active_ctx_name, "config2");
    }
}
