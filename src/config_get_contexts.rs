//! Get all available contexts from configuration
use super::parse_configuration_file;
use clap::{crate_authors, App};
use log::trace;

/// Returns get-context subcommand from config command
pub fn get_contexts_command() -> App<'static, 'static> {
    App::new("get-contexts")
        .about("Get all available Todo contexts")
        .author(crate_authors!())
}

/// Shows all available contexts from Todo configuration
pub fn get_contexts_command_process(
    todo_configuration_path: &str,
    raw_config: Option<&str>,
) -> Result<(), std::io::Error> {
    trace!("get-contexts");
    let config = parse_configuration_file(Some(todo_configuration_path), raw_config)?;
    config.ctxs.into_iter().for_each(|ctx| println!("{}", ctx));
    Ok(())
}
