//! Set active context from available contexts of configuration
use super::parse_configuration_file;
use clap::{crate_authors, App, Arg, ArgMatches};
use log::{debug, trace};
use std::fs::File;
use std::io::Write;

/// Returns set-context subcommand from config commmand
pub fn set_context_command() -> App<'static, 'static> {
    App::new("set-context")
        .about("Set Todo context")
        .author(crate_authors!())
        .arg(
            Arg::with_name("context")
                .takes_value(true)
                .required(true)
                .index(1),
        )
}

/// Processes arguments and set active context if provided context exists within Todo configuration
pub fn set_context_command_process(
    args: &ArgMatches,
    todo_configuration_path: &str,
    raw_config: Option<&str>,
) -> Result<(), std::io::Error> {
    trace!("set-context");
    debug!("set_context_matches: {:?}", args);
    let new_context = args.value_of("context").unwrap().to_string();
    debug!("new context: {}", new_context);
    match parse_configuration_file(Some(todo_configuration_path), raw_config) {
        Ok(mut config) => {
            let update = config.update_active_ctx(&new_context);
            if update.is_err() {
                eprintln!("{}", update.err().unwrap());
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    update.err().unwrap(),
                ));
            }

            trace!("Opening configuration file with write access...");
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(todo_configuration_path)?;
            trace!("Writting to file");
            File::write(&mut file, toml::to_string(&config).unwrap().as_bytes())?;

            println!("Context was set to \"{}\"", config.active_ctx_name);
            Ok(())
        }
        Err(e) => {
            eprintln!("{}", e);
            Err(e)
        }
    }
}
