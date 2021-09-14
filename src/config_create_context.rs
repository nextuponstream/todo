//! Create todo context inside configuration
use super::{parse_configuration_file, Configuration, Context};
use clap::{crate_authors, App, Arg, ArgMatches};
use log::{debug, trace, warn};
use read_input::prelude::*;
use std::fs::File;
use std::io::Write;

/// Returns create context subcommand from config command
pub fn create_context_command() -> App<'static, 'static> {
    App::new("create-context")
        .about("Create a new Todo context")
        .author(crate_authors!())
        .arg(
            Arg::with_name("ide")
                .short("i")
                .long("ide")
                .value_name("IDE")
                .help("IDE configuration")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("name")
                .short("n")
                .long("name")
                .value_name("NAME")
                .help("Name of configuration")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("timezone")
                .short("t")
                .long("timezone")
                .value_name("TIMEZONE")
                .help("Timezone for configuration")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("todo_folder")
                .short("f")
                .long("todo-folder")
                .value_name("TODO_FOLDER")
                .help("Folder where todo's of configuration will be saved")
                .takes_value(true)
                .required(true),
        )
}

/// Processes arguments and creates new Todo context in configuration. After Todo context creation,
/// sets active context to the newly created context.
pub fn config_create_context_process(
    args: &ArgMatches,
    todo_configuration_path: &str,
    raw_config: Option<&str>,
) -> Result<(), std::io::Error> {
    trace!("create-context subsubcommand");
    let new_ctx = Context {
        ide: args.value_of("ide").unwrap().to_string(),
        name: args.value_of("name").unwrap().to_string(),
        timezone: args.value_of("timezone").unwrap().to_string(),
        todo_folder: args.value_of("todo_folder").unwrap().to_string(),
    };

    let config = parse_configuration_file(Some(todo_configuration_path), raw_config);
    debug!("config.is_ok: {}", config.is_ok());
    let mut config: Configuration = match config {
        Err(e) => {
            if e.kind() != std::io::ErrorKind::NotFound {
                return Err(e);
            }

            if "n"
                == input::<String>()
                    .msg("Do you want to create a new configuration file [y/n]? ")
                    .add_test(|user_input| user_input == "y" || user_input == "n")
                    .err("Please input \"y\" or \"n\".")
                    .get()
            {
                println!("No configuration file was created. Aborting command.");
                warn!("User aborted command");
                return Ok(());
            }

            Configuration {
                active_ctx_name: String::from(""),
                ctxs: vec![],
            }
        }
        Ok(config) => config,
    };

    config.ctxs.push(new_ctx.clone());
    config.update_active_ctx(new_ctx.name.as_str()).unwrap();

    debug!("config: {}", config);
    if config
        .ctxs
        .iter()
        .find(|&c| {
            debug!("{}, {}", c.name, config.active_ctx_name);
            c.name == config.active_ctx_name
        })
        .is_none()
    {
        warn!("No contexts matched active context");
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "No contexts matched active context",
        ));
    }
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(todo_configuration_path)?;
    let raw_config = toml::to_string(&config).unwrap();
    debug!("raw_config:\n{}", raw_config);
    File::write(&mut file, raw_config.as_bytes())?;

    println!(
        "Successfully updated configuration at \"{}\"\nConfiguration was switched to `{}`",
        todo_configuration_path, config.active_ctx_name
    );

    Ok(())
}
