use clap::{App, AppSettings, Arg, SubCommand};
use dialoguer::Confirm;
use log::{debug, trace, warn};
use regex::Regex;
use serde::Deserialize;
use simplelog::*;
use std::fmt;
use std::fs::{read_to_string, remove_file, File};
use std::io::prelude::*;
use std::process::Command;
use walkdir::WalkDir;

#[derive(Deserialize, Debug)]
struct CurrentConfig {
    current_config: String,
}

#[derive(Clone, Deserialize, Debug)]
struct Configuration {
    ide: String,
    name: String,
    timezone: String,
    todo_folder: String,
}

impl fmt::Display for Configuration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "[[config]]\nname = \"{}\"\nide = \"{}\"\ntimezone = \"{}\"\ntodo_folder = \"{}\"",
            self.name, self.ide, self.timezone, self.todo_folder
        )
    }
}

#[derive(Deserialize, Debug)]
struct ConfigurationVec {
    config: Vec<Configuration>,
}

impl fmt::Display for ConfigurationVec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for config in self.config.iter() {
            writeln!(
                f,
                "==={} context===\nide\t\t: {}\ntimezone\t: {}\nfolder\t\t: {}",
                config.name, config.ide, config.timezone, config.todo_folder
            )?;
        }
        Ok(())
    }
}

#[derive(Deserialize, Debug)]
struct Todo {
    title: String,
    label: Vec<String>,
    content: String,
}

impl fmt::Display for Todo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "+++\nTITLE={}\nLABEL={}\n+++\n{}",
            self.title,
            self.label.join(","),
            self.content
        )
    }
}

fn main() -> Result<(), std::io::Error> {
    trace!("Program start");
    let _ = TermLogger::init(
        LevelFilter::Trace, // TODO set to appropriate level before release
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    );

    // TODO parse other arguments such as deadline, subcommands...
    // TODO autoversion
    // TODO autoauthors
    let app = App::new("todo Program")
        .version("0.1")
        .author("Nextuponstream")
        .about("This CLI tool was inspired by kubectl apply/delete/get...");
    let app = app
        .subcommand(
            SubCommand::with_name("create")
                .arg(
                    Arg::with_name("label")
                        .short("l")
                        .long("label")
                        .value_name("LABEL")
                        .help("Filter by label")
                        .value_delimiter(",")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("title")
                        .short("t")
                        .long("title")
                        .value_name("TITLE")
                        .help("Sets title of todo")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("content")
                        .short("c")
                        .long("content")
                        .value_name("CONTENT")
                        .help("Sets content of todo")
                        .takes_value(true),
                ), // TODO enumeration variant
                   // TODO checklist variant
        )
        .subcommand(
            SubCommand::with_name("config")
                .about("Manage your todo configuration")
                .setting(AppSettings::SubcommandRequired)
                .subcommand(
                    SubCommand::with_name("create-context")
                        .about("create a new todo context")
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
                        ),
                )
                .subcommand(
                    SubCommand::with_name("current-context")
                        .about("shows current todo context")
                        .help("shows current todo context"),
                )
                .subcommand(
                    SubCommand::with_name("get-contexts")
                        .about("get all available todo contexts")
                        .help("get all available todo contexts"),
                )
                .subcommand(
                    SubCommand::with_name("set-context")
                        .about("switch todo context")
                        .help("switch todo context")
                        .arg(
                            Arg::with_name("new context")
                                .takes_value(true)
                                .required(true)
                                .help("new context")
                                .index(1),
                        ),
                ),
        )
        .subcommand(
            SubCommand::with_name("list").arg(
                Arg::with_name("label")
                    .short("l")
                    .long("label")
                    .value_name("LABEL")
                    .help("Filter by label")
                    .value_delimiter(",")
                    .takes_value(true),
            ),
        )
        .subcommand(
            SubCommand::with_name("edit").arg(
                Arg::with_name("title")
                    .short("t")
                    .long("title")
                    .value_name("TITLE")
                    .help("Title of todo to open")
                    .takes_value(true)
                    .required(true),
            ),
        )
        .subcommand(
            SubCommand::with_name("delete").arg(
                Arg::with_name("title")
                    .short("t")
                    .long("title")
                    .value_name("TITLE")
                    .help("Title of todo to delete")
                    .takes_value(true)
                    .required(true),
            ),
        );
    let matches = app.get_matches();

    let home = std::env::var("HOME").unwrap(); // can't use '~' since it needs to be expanded
    let tcp = format!("{}/.todo", home.as_str());
    let todo_configuration_path = tcp.as_str(); // borrow checker at it again

    match matches.subcommand() {
        ("config", Some(config_matches)) => match config_matches.subcommand() {
            ("create-context", Some(create_context_submatches)) => {
                trace!("config subcommand");
                let config = Configuration {
                    ide: create_context_submatches
                        .value_of("ide")
                        .unwrap()
                        .to_string(),
                    name: create_context_submatches
                        .value_of("name")
                        .unwrap()
                        .to_string(),
                    timezone: create_context_submatches
                        .value_of("timezone")
                        .unwrap()
                        .to_string(),
                    todo_folder: create_context_submatches
                        .value_of("todo_folder")
                        .unwrap()
                        .to_string(),
                };

                let (_, old_configs) = config_file_raw(todo_configuration_path)?;
                let mut file = std::fs::OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(todo_configuration_path)?;
                // TODO use fmt::Display from CurrentConfig struct
                File::write(
                    &mut file,
                    format!(
                        "current_config = \"{}\"\n{}\n{}",
                        config.name, old_configs, config
                    )
                    .as_bytes(),
                )?;

                println!(
                "Successfully updated configuration at \"{}\"\nConfiguration was switched to `{}`",
                todo_configuration_path, config.name
            );

                return Ok(());
            }
            ("current-context", Some(_)) => {
                trace!("current-context");
                let (current_config, _) = config_file_raw(todo_configuration_path)?;
                debug!("current_config = {}", current_config);
                let current_config_name = Regex::new(r#""(.*)""#)
                    .unwrap()
                    .find(current_config.as_str());
                match current_config_name {
                    Some(m) => {
                        trace!("match found");
                        let mut name = String::from(m.as_str());
                        name.remove(0);
                        name.pop();
                        debug!("name = {}", name);
                        if name.is_empty() {
                            warn!("Context is not set");
                            eprintln!("Context is not set");
                            std::process::exit(1)
                        } else {
                            println!("{}", name);
                            return Ok(());
                        }
                    }
                    None => {
                        warn!("No match was found. Bad configuration file");
                        eprintln!("Bad configuration file: could not parse current configuration");
                        std::process::exit(1)
                    }
                };
            }
            ("get-contexts", Some(_)) => {
                trace!("get-contexts");
                let (_, configs_raw) = config_file_raw(todo_configuration_path)?;
                trace!("parsing toml table");
                let configs: ConfigurationVec = toml::from_str(configs_raw.as_str())?;
                debug!("parsed toml = {:?}", configs);
                println!("{}", configs);
                return Ok(());
            }
            ("set-context", Some(set_context_matches)) => {
                trace!("set-context");
                debug!("set_context_matches: {:?}", set_context_matches);
                let new_context = set_context_matches
                    .value_of("new context")
                    .unwrap()
                    .to_string();
                let (_, configs) = config_file_raw(todo_configuration_path)?;

                trace!("Opening configuration file with write access...");
                let mut file = std::fs::OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(todo_configuration_path)?;
                trace!("Writting to file");
                File::write(
                    &mut file,
                    format!("current_config = \"{}\"\n{}", new_context, configs).as_bytes(),
                )?;

                println!("Context was set to \"{}\"", new_context);
                return Ok(());
            }
            _ => unreachable!(),
        },
        _ => {}
    }

    let current_config = parse_config_file(todo_configuration_path)?;

    match matches.subcommand() {
        ("create", Some(create_matches)) => {
            trace!("create subcommand");
            let todo = Todo {
                title: create_matches
                    .value_of("title")
                    .unwrap_or("untitled")
                    .to_string(),
                content: create_matches.value_of("content").unwrap_or("").to_string(),
                // https://stackoverflow.com/a/37547426/16631150
                label: create_matches
                    .values_of("label")
                    .unwrap_or_default()
                    .map(|s| s.to_string())
                    .collect(),
            };
            debug!("{}", todo);

            // Individual files allow for manual editing without the pain of scrolling through
            // all other todo's.
            let filepath = todo_path(current_config.todo_folder.as_str(), todo.title.as_str());
            match read_to_string(&filepath) {
                Ok(_) => {
                    trace!("Potential overwrite detected");
                    if !Confirm::new()
                        .with_prompt(format!(
                            "This operation will overwrite todo \"{}\". Continue?",
                            todo.title
                        ))
                        .interact()?
                    {
                        return Ok(());
                    }
                }
                Err(e) => {
                    trace!("File cannot be open: {}", e);
                }
            }

            std::fs::write(&filepath, format!("{}", todo))?;
            println!(
                "Saved todo \"{}\" ({})",
                todo.title, current_config.todo_folder
            );
        }
        ("delete", Some(delete_matches)) => {
            trace!("delete subcommand");
            println!("Listing all todo's from {}", current_config.todo_folder);

            let title = delete_matches.value_of("title").unwrap();
            remove_file(todo_path(current_config.todo_folder.as_str(), title)).unwrap();
        }
        ("edit", Some(edit_matches)) => {
            trace!("edit subcommand");
            println!("Listing all todo's from {}", current_config.todo_folder);

            let title = edit_matches.value_of("title").unwrap();

            Command::new(current_config.ide.as_str())
                .arg(todo_path(current_config.todo_folder.as_str(), title))
                .status()
                .expect("IDE error");
        }
        ("list", Some(list_matches)) => {
            trace!("list subcommand");

            let labels = list_matches
                .values_of("label")
                .unwrap_or_default()
                .collect::<Vec<_>>();
            debug!("labels = {:?}", labels);

            let re: Regex = Regex::new(format!(r"LABEL=(.*)\n\+\+\+").as_str()).unwrap();

            println!("Listing all todo's from {}", current_config.todo_folder);
            for entry in WalkDir::new(current_config.todo_folder.as_str()) {
                let entry = entry.unwrap();
                if entry.file_type().is_dir() {
                    // first entry is the todo folder
                    continue;
                }
                let filepath = entry.path().to_str().unwrap();
                debug!("todo: {}", filepath);
                match read_to_string(filepath) {
                    Ok(content) => {
                        let cs = re.captures(content.as_str()).unwrap();
                        if cs.len() == 1 {
                            eprintln!("Cannot parse {} labels", filepath);
                            std::process::exit(1);
                        }

                        let file_labels = cs
                            .get(1)
                            .unwrap()
                            .as_str()
                            .split(",")
                            .map(|s| s.to_string())
                            .collect::<Vec<String>>();
                        if labels.iter().all(|f| file_labels.iter().any(|fl| fl == f)) {
                            println!("{}", content);
                        }
                    }
                    Err(error) => panic!(
                        "Cannot open {}, error: {}",
                        entry.path().to_str().unwrap(),
                        error
                    ),
                }
            }
        }
        ("", None) => {
            // TODO force subcommand
            trace!("no subcommand was used");
        }
        _ => unreachable!(),
    }

    Ok(())
}

/// joins todo folder path and todo title into a filepath. The file is in markdown format.
fn todo_path(todo_folder: &str, todo_title: &str) -> String {
    format!("{}/{}.md", todo_folder, todo_title)
}

/// Opens configuration file and returns current configuration and configurations.
fn config_file_raw(todo_configuration_path: &str) -> Result<(String, String), std::io::Error> {
    trace!("Opening configuration file");
    debug!("todo_configuration_path: {}", todo_configuration_path);
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(todo_configuration_path)
        .expect(
            "Missing configuration file or unable to open \"{}\", 
did you initialize it with `todo config`?\nError: {}",
        );

    let mut content = String::new();
    file.read_to_string(&mut content)?;
    debug!("content: {}", content);
    let (current_config_name, configs) = &content.split_once("\n").unwrap_or(("", ""));
    debug!("current_config_name: {}", current_config_name);
    debug!("configs: {}", configs);
    Ok((current_config_name.to_string(), configs.to_string()))
}

/// Takes raw input from configuration file and parse its content. This method will fail if the
/// configuration file is badly formatted or the current configuration is invalid.
fn parse_config_file(todo_configuration_path: &str) -> Result<Configuration, std::io::Error> {
    let (current_config_name_raw, configs_raw) = config_file_raw(todo_configuration_path)?;
    trace!("Parsing current configuration name");
    let current_config: CurrentConfig = toml::from_str(current_config_name_raw.as_str())?;
    trace!("Parsing configurations");
    let cv: ConfigurationVec = toml::from_str(configs_raw.as_str())?;
    trace!("Is current configuration valid?");
    let conf = cv
        .config
        .iter()
        .find(|&c| c.name == current_config.current_config)
        .expect("No configuration matched current configuration name");
    trace!("Current configuration is valid");
    Ok(conf.clone())
}
