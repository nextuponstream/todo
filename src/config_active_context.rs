//! Display active Todo context from configuration
use super::parse::parse_active_context;
use clap::{crate_authors, Command};
use log::trace;

/// Returns active-context subcommand from configuration command
pub fn active_context_command() -> Command<'static> {
    Command::new("active-context")
        .about("Shows active Todo context")
        .author(crate_authors!())
}

/// Shows active context from Todo configuration
pub fn active_context_command_process(
    todo_configuration_path: &str,
    raw_config: Option<&str>,
) -> Result<(), std::io::Error> {
    trace!("active-context");
    let active_ctx = parse_active_context(Some(todo_configuration_path), raw_config)?;
    println!("{}", active_ctx.name);
    Ok(())
}
