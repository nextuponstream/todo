//! Display Active Todo context from configuration
use super::parse_active_ctx;
use clap::{crate_authors, App};
use log::trace;

/// Returns active-context subcommand from configuration command
pub fn active_context_command() -> App<'static, 'static> {
    App::new("active-context")
        .about("shows active Todo context")
        .author(crate_authors!())
}

/// Shows active context from Todo configuration
pub fn active_context_command_process(
    todo_configuration_path: &str,
    raw_config: Option<&str>,
) -> Result<(), std::io::Error> {
    trace!("active-context");
    let active_ctx = parse_active_ctx(Some(todo_configuration_path), raw_config)?;
    println!("{}", active_ctx.name);
    Ok(())
}