use clap::{App, AppSettings, Arg, SubCommand};
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
    name: String,
}

#[derive(Deserialize, Debug)]
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
                        // TODO use https://docs.rs/clap/2.33.3/clap/struct.Arg.html#method.value_delimiter
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
                        .help("shows current todo context"), // TODO implement
                )
                .subcommand(
                    SubCommand::with_name("get-contexts")
                        .about("get all available todo contexts")
                        .help("get all available todo contexts"), // TODO implement
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
        .subcommand(SubCommand::with_name("list"))
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

                let (_, old_configs) = config_file_content(todo_configuration_path)?;
                let mut file = std::fs::OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(todo_configuration_path)?;
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
                let (current_config, _) = config_file_content(todo_configuration_path)?;
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
                let (_, configs_raw) = config_file_content(todo_configuration_path)?;
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
                let (_, configs) = config_file_content(todo_configuration_path)?;

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

    let raw = match read_to_string(todo_configuration_path) {
        Ok(r) => r,
        Err(e) => {
            // Nice error message because forgetting configuration will happen (panic! macro is
            // ugly)
            eprintln!("Missing configuration file or unable to open \"{}\", did you initialize it with `todo config`?\nError: {}", todo_configuration_path, e);
            std::process::exit(1)
        }
    };
    let (current_config_name_raw, configs_raw) = raw.split_once('\n').unwrap();
    let current_config: CurrentConfig = toml::from_str(current_config_name_raw)?;
    let cv: ConfigurationVec = toml::from_str(configs_raw).unwrap();
    debug!("{:?}", cv);
    let c = cv
        .config
        .iter()
        .find(|&c| c.name == current_config.name)
        .expect("Bad configuration file: no current config name found");

    println!("Nothing was saved in {}", c.todo_folder); // TODO place somewhere after saving file
    println!("... ({})", c.timezone); // TODO place somewhere after saving todo with deadline

    match matches.subcommand() {
        ("create", Some(create_matches)) => {
            trace!("create subcommand");
            // TODO prevent overwritting
            let title = create_matches.value_of("title").unwrap_or("untitled");
            let content = create_matches.value_of("content").unwrap_or("");
            let label = create_matches.value_of("label").unwrap_or("");

            // TODO change panic! to ?
            let mut file = match File::create(todo_path(c.todo_folder.as_str(), title)) {
                Err(why) => panic!("couldn't create {}: {}", title, why),
                Ok(f) => f,
            };
            let file_content = format!("+++\n{}\n{}\n+++\n{}", title, label, content);

            let _ = match file.write_all(file_content.as_bytes()) {
                Err(why) => panic!("couldn't write to file: {}", why),
                Ok(_) => {}
            };

            println!("Created todo \"{}\", stored at {}", title, c.todo_folder);
        }
        ("delete", Some(delete_matches)) => {
            trace!("delete subcommand");
            println!("Listing all todo's from {}", c.todo_folder);

            let title = delete_matches.value_of("title").unwrap();
            remove_file(todo_path(c.todo_folder.as_str(), title)).unwrap();
        }
        ("edit", Some(edit_matches)) => {
            trace!("edit subcommand");
            println!("Listing all todo's from {}", c.todo_folder);

            let title = edit_matches.value_of("title").unwrap();

            Command::new(c.ide.as_str())
                .arg(todo_path(c.todo_folder.as_str(), title))
                .status()
                .expect("IDE error");
        }
        ("list", Some(list_matches)) => {
            trace!("list subcommand");
            println!("Listing all todo's from {}", c.todo_folder);

            for entry in WalkDir::new(c.todo_folder.as_str()) {
                let entry = entry.unwrap();
                if entry.file_type().is_dir() {
                    // first entry is the todo folder
                    continue;
                }
                debug!("{}", entry.path().to_str().unwrap());
                match read_to_string(entry.path().to_str().unwrap()) {
                    Ok(content) => println!("{}", content),
                    Err(error) => panic!(
                        "Cannot open {}, error: {}",
                        entry.path().to_str().unwrap(),
                        error
                    ),
                }
            }

            // TODO read one or many
            // TODO filter by label
        }
        ("", None) => {
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

/// Opens configuration file, returns current configuration and configurations
fn config_file_content(todo_configuration_path: &str) -> Result<(String, String), std::io::Error> {
    trace!("Opening configuration file");
    debug!("todo_configuration_path: {}", todo_configuration_path);
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(todo_configuration_path)?;

    let mut content = String::new();
    file.read_to_string(&mut content)?;
    debug!("content: {}", content);
    let (current_config_name, configs) = &content.split_once("\n").unwrap_or(("", ""));
    debug!("current_config_name: {}", current_config_name);
    debug!("configs: {}", configs);
    Ok((current_config_name.to_string(), configs.to_string()))
}
