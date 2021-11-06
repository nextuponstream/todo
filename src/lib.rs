//! Todo is a tool to help you create todos, todo contexts and manage them. Just
//! like Git, Todo is comprised of multiple subcommmands arranged in their
//! respective modules.
//!
//! Compile the tool and start using it with TODO more details needed<br>
//! `todo --version`
use parse::{parse_configuration_file, parse_done_tasks, parse_title};
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
        let todo = Todo {
            title: String::from(""),
            label: vec![],
            content: String::from(""),
            items: vec![],
            motives: vec![],
        };
        let expected = TODO_BAREBONES;
        let output = format!("{}", todo);
        assert_eq!(output, expected);
    }

    #[test]
    fn all_options_todo() {
        init();
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
        assert_eq!(output, expected);
    }
}
