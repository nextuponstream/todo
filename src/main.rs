use clap::{App, AppSettings, Arg, SubCommand};
use dialoguer::Confirm;
use log::{debug, trace};
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

impl fmt::Display for CurrentConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "current_config = \"{}\"", self.current_config)
    }
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
    items: Vec<String>,
    motives: Vec<String>,
}

impl fmt::Display for Todo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "+++\nTITLE={}\nLABEL={}\n+++\n{}\n",
            self.title,
            self.label.join(","),
            self.content
        )?;
        for i in self.items.iter() {
            writeln!(f, "- [ ] {}", i)?;
        }
        if self.items.len() > 0 && self.motives.len() > 0 {
            writeln!(f, "---")?;
        }
        let mut i = self.motives.len();
        for m in self.motives.iter().rev() {
            writeln!(f, "{} {}", i, m)?;
            i = i - 1;
        }

        Ok(())
    }
}

fn main() -> Result<(), std::io::Error> {
    trace!("Program start");
    let _ = TermLogger::init(
        LevelFilter::Debug, // TODO set to appropriate level before release
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    );

    let home = std::env::var("HOME").unwrap(); // can't use '~' since it needs to be expanded
    let with_config_path_help_text = format!(
        "Uses configuration file at CONFIG_PATH instead of default at \"{}/.todo\"",
        home
    );

    // TODO autoversion
    // TODO autoauthors
    // TODO document create/edit/delete
    let app = App::new("todo Program")
        .version("0.1")
        .author("Nextuponstream")
        .about("This CLI tool was inspired by kubectl apply/delete/get...");
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
                        .takes_value(true)
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("content")
                        .short("c")
                        .long("content")
                        .value_name("CONTENT")
                        .help("Sets content of todo")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("item")
                        .short("i")
                        .long("item")
                        .multiple(true)
                        .value_name("ITEM")
                        .help("An item of your todo list")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("motives")
                        .short("m")
                        .long("motives")
                        .multiple(true)
                        .value_name("MOTIVE")
                        .help("list of motives that appears in reverse order of the todo")
                        .takes_value(true),
                ),
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
                    .index(1)
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
                    .index(1)
                    .help("Title of todo to delete")
                    .takes_value(true)
                    .required(true),
            ),
        );
    let matches = app.get_matches();

    let default_todo_configuration_path = format!("{}/.todo", home.as_str());
    let todo_configuration_path = matches
        .value_of("with-config-path")
        .unwrap_or(default_todo_configuration_path.as_str());

    // other subcommands than config requires a working configuration file
    let raw_config = matches.value_of("with-config");
    debug!("raw_config = {:?}", raw_config);

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
                let current_config = CurrentConfig {
                    current_config: config.name.clone(),
                };

                let (_, old_configs) = config_file_raw(todo_configuration_path, raw_config)?;
                let mut file = std::fs::OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(todo_configuration_path)?;
                File::write(
                    &mut file,
                    format!("{}{}\n{}", current_config, old_configs, config).as_bytes(),
                )?;

                println!(
                "Successfully updated configuration at \"{}\"\nConfiguration was switched to `{}`",
                todo_configuration_path, current_config.current_config
            );

                return Ok(());
            }
            ("current-context", Some(_)) => {
                trace!("current-context");
                let current_config = parse_config_file(todo_configuration_path, raw_config)?;
                println!("{}", current_config.name);
                return Ok(());
            }
            ("get-contexts", Some(_)) => {
                trace!("get-contexts");
                let (_, configs_raw) = config_file_raw(todo_configuration_path, raw_config)?;
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
                let (_, configs) = config_file_raw(todo_configuration_path, raw_config)?;
                let current_config = CurrentConfig {
                    current_config: new_context,
                };

                trace!("Opening configuration file with write access...");
                let mut file = std::fs::OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(todo_configuration_path)?;
                trace!("Writting to file");
                File::write(
                    &mut file,
                    format!("{}{}", current_config, configs).as_bytes(),
                )?;

                println!("Context was set to \"{}\"", current_config.current_config);
                return Ok(());
            }
            _ => unreachable!(),
        },
        _ => {}
    }

    let current_config = parse_config_file(todo_configuration_path, raw_config)?;

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
                items: create_matches
                    .values_of("item")
                    .unwrap_or_default()
                    .map(|s| s.to_string())
                    .collect(),
                motives: create_matches
                    .values_of("motives")
                    .unwrap_or_default()
                    .map(|s| s.to_string())
                    .collect(),
            };
            debug!("todo to create:\n{}", todo);

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

            let title = delete_matches.value_of("title").unwrap();
            match remove_file(todo_path(current_config.todo_folder.as_str(), title)) {
                Ok(_) => println!("Successfully removed {}", title),
                Err(_) => eprintln!("Error: File does not exist"),
            }
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
        _ => unreachable!(),
    }

    Ok(())
}

/// joins todo folder path and todo title into a filepath. The file is in markdown format.
fn todo_path(todo_folder: &str, todo_title: &str) -> String {
    format!("{}/{}.md", todo_folder, todo_title)
}

/// Opens configuration file and returns current configuration and configurations. Uses `raw
/// configuration` when supplied.
fn config_file_raw(
    todo_configuration_path: &str,
    raw_configuration: Option<&str>,
) -> Result<(String, String), std::io::Error> {
    let mut file_content = String::new();
    let content = match raw_configuration {
        Some(c) => c,
        None => {
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
            file.read_to_string(&mut file_content)?;
            file_content.as_str()
        }
    };

    debug!("content: {}", content);
    let (current_config_name, configs) = &content.split_once("\n").unwrap_or(("", ""));
    debug!("current_config_name: {}", current_config_name);
    debug!("configs: {}", configs);
    Ok((current_config_name.to_string(), configs.to_string()))
}

/// Parses configuration file at `todo_configuration_path`. Uses supplied `configuration_raw` when
/// provided. Fails when configuration file is either badly formatted or the current context is invalid.
fn parse_config_file(
    todo_configuration_path: &str,
    configuration_raw: Option<&str>,
) -> Result<Configuration, std::io::Error> {
    let (current_config_name_raw, configs_raw) =
        config_file_raw(todo_configuration_path, configuration_raw)?;
    trace!("Parsing current configuration name");
    let current_config: CurrentConfig = toml::from_str(current_config_name_raw.as_str())?;
    trace!("Parsing configurations");
    let configs_raw = configuration_raw.unwrap_or(configs_raw.as_str());
    let cv: ConfigurationVec = toml::from_str(configs_raw)?;
    trace!("Is current configuration valid?");
    let conf = cv
        .config
        .iter()
        .find(|&c| c.name == current_config.current_config)
        .expect("No configuration matched current configuration name");
    trace!("Current configuration is valid");
    Ok(conf.clone())
}
