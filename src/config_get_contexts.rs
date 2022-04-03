//! Display all available Todo contexts from configuration
use super::parse_configuration_file;
use clap::{crate_authors, Arg, ArgMatches, Command};
use log::{debug, trace};

/// Returns get-context subcommand from config command
pub fn get_contexts_command() -> Command<'static> {
    Command::new("get-contexts")
        .about("Get all available Todo contexts")
        .author(crate_authors!())
        .arg(
            Arg::new("full")
                .short('f')
                .long("full")
                .help("Display all information about Todo context"),
        )
}

/// Shows all available contexts from Todo configuration
pub fn get_contexts_command_process(
    args: &ArgMatches,
    todo_configuration_path: &str,
    raw_config: Option<&str>,
) -> Result<(), std::io::Error> {
    trace!("get-contexts");
    let config = parse_configuration_file(Some(todo_configuration_path), raw_config)?;
    let full = args.is_present("full");
    debug!("args: {:?}", args);
    debug!("full: {}", full);

    if full {
        config.ctxs.into_iter().for_each(|ctx| {
            if config.active_ctx_name == ctx.name {
                println!(
            "--- Context (active) ---\nname: {}\nide: {}\ntimezone: {}\nfolder location: {}\n",
            ctx.name, ctx.ide, ctx.timezone, ctx.folder_location
        )
            } else {
                println!("{}", ctx)
            }
        });
    } else {
        config.ctxs.into_iter().for_each(|ctx| {
            println!(
                "{}{}",
                if config.active_ctx_name == ctx.name {
                    "→ "
                } else {
                    "  "
                },
                ctx.short(),
            )
        });
    }
    Ok(())
}
