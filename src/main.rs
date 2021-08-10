use clap::{App, Arg, SubCommand};
use log::{debug, trace, warn};
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
    configs: Vec<Configuration>,
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
                        .help("switch todo context"), // TODO implement
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

                trace!("Opening configuration file");
                debug!("todo_configuration_path: {}", todo_configuration_path);
                let mut file = std::fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(todo_configuration_path)?;

                let mut old_content = String::new();
                file.read_to_string(&mut old_content)?;
                debug!("old_content: {}", old_content);
                let (_, old_configs) = old_content.split_once("\n").unwrap_or(("", ""));
                debug!("old_configs: {}", old_configs);

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
                // TODO implement
            }
            ("get-contexts", Some(_)) => {
                trace!("get-contexts");
                // TODO implement
            }
            ("set-context", Some(set_context_matches)) => {
                trace!("set-context");
                debug!("set_context_matches: {:?}", set_context_matches);
                // TODO implement
            }
            ("", None) => {
                trace!("no subcommands");
                // TODO print help
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
        .configs
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
