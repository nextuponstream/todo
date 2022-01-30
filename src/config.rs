//! Manage todo contexts from configuration
use crate::config_active_context::{active_context_command, active_context_command_process};
use crate::config_create_context::{config_create_context_process, create_context_command};
use crate::config_get_contexts::{get_contexts_command, get_contexts_command_process};
use crate::config_set_context::{set_context_command, set_context_command_process};
use clap::{crate_authors, App, AppSettings, ArgMatches};
use log::warn;

/// Returns configuration command which is comprised of multiple subcommands
pub fn config_command() -> App<'static, 'static> {
    App::new("config")
        .about("Manage your todo configuration")
        .author(crate_authors!())
        .setting(AppSettings::SubcommandRequired)
        .subcommand(create_context_command())
        .subcommand(active_context_command())
        .subcommand(get_contexts_command())
        .subcommand(set_context_command())
}

/// Executes configuration command
pub fn config_command_process(
    args: &ArgMatches,
    todo_configuration_path: &str,
    raw_config: Option<&str>,
) -> Result<(), std::io::Error> {
    if let Some(args) = args.subcommand_matches("create-context") {
        return config_create_context_process(args, todo_configuration_path, raw_config);
    }

    if args.subcommand_matches("active-context").is_some() {
        return active_context_command_process(todo_configuration_path, raw_config);
    }

    if let Some(args) = args.subcommand_matches("get-contexts") {
        return get_contexts_command_process(args, todo_configuration_path, raw_config);
    }

    if let Some(set_context_matches) = args.subcommand_matches("set-context") {
        return set_context_command_process(
            set_context_matches,
            todo_configuration_path,
            raw_config,
        );
    }

    warn!("unrecognised command");
    Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Unrecognised command",
    ))
}
