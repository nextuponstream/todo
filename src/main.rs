use clap::{crate_authors, crate_version, App, AppSettings, Arg};
use log::{debug, warn};
//use simplelog::*;
use todo::config::{config_command, config_command_process};
use todo::create::{create_command, create_command_process};
use todo::delete::{delete_command, delete_command_process};
use todo::edit::{edit_command, edit_command_process};
use todo::list::{list_command, list_command_process};
use todo::parse::{parse_active_context, parse_configuration_file};
use todo::r#move::{move_command, move_command_process};

fn main() -> Result<(), std::io::Error> {
    // TODO comment before release
    //let _ = TermLogger::init(
    //    LevelFilter::Trace,
    //    Config::default(),
    //    TerminalMode::Mixed,
    //    ColorChoice::Auto,
    //);
    let home = std::env::var("HOME").unwrap(); // can't use '~' since it needs to be expanded
    let with_config_path_help_text = format!(
        "Uses configuration file at CONFIG_PATH instead of default at \"{}/.todo\"",
        home
    );

    let app = App::new("todo Program")
        .version(crate_version!())
        .author(crate_authors!())
        .setting(AppSettings::GlobalVersion)
        .long_about("Tool to manage todo lists from multiple contexts

This tool was inspired from kubectl and git. This tool hopes to save some ink from your whiteboard.")
        .about("Tool to manage todo lists from multiple contexts");
    let app = app
        .setting(AppSettings::SubcommandRequired)
        // this command is mostly for testing purposes
        .arg(
            Arg::with_name("with-config")
                .short("r")
                .long("with-config")
                .value_name("CONFIG_RAW")
                .help("Use <CONFIG_RAW> instead of configuration file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("with-config-path")
                .short("p")
                .long("with-config-path")
                .value_name("CONFIG_PATH")
                .help(with_config_path_help_text.as_str())
                .takes_value(true),
        )
        .subcommand(create_command())
        .subcommand(config_command())
        .subcommand(edit_command())
        .subcommand(delete_command())
        .subcommand(list_command())
        .subcommand(move_command());
    let matches = app.get_matches();

    let default_todo_configuration_path = format!("{}/.todo", home.as_str());
    let todo_configuration_path = matches
        .value_of("with-config-path")
        .unwrap_or_else(|| default_todo_configuration_path.as_str());

    // other subcommands than config requires a working configuration file
    let raw_config = matches.value_of("with-config");
    debug!("raw_config = {:?}", raw_config);

    if let Some(args) = matches.subcommand_matches("config") {
        return config_command_process(args, todo_configuration_path, raw_config);
    }

    let ctx = parse_active_context(Some(todo_configuration_path), raw_config)?;
    let config = parse_configuration_file(Some(todo_configuration_path), raw_config)?;

    if let Some(args) = matches.subcommand_matches("create") {
        return create_command_process(args, &ctx);
    }

    if let Some(args) = matches.subcommand_matches("delete") {
        return delete_command_process(args, &ctx);
    }

    if let Some(args) = matches.subcommand_matches("edit") {
        if let Err(e) = edit_command_process(args, &ctx, &config) {
            eprintln!("Error: {e}");
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Edit command could not complete.",
            ));
        } else {
            return Ok(());
        }
    }

    if let Some(args) = matches.subcommand_matches("list") {
        return list_command_process(args, &config);
    }

    if let Some(args) = matches.subcommand_matches("move") {
        if let Err(e) = move_command_process(args, &config) {
            eprintln!("Error: {e}");
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Move command could not complete.",
            ));
        } else {
            return Ok(());
        }
    }

    warn!("Unrecognised subcommand");
    Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "unrecognised subcommand",
    ))
}
