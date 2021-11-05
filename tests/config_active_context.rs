use assert_cmd::prelude::*; // Add methods on commands
use simplelog::*;
use std::process::Command; // Run programs

// TODO wait for before/after_test macro
// https://github.com/rust-lang/rfcs/issues/1664
fn init() {
    let _ = TermLogger::init(
        LevelFilter::Warn,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    );
}

#[test]
fn has_help() -> Result<(), Box<dyn std::error::Error>> {
    init();
    let mut cmd = Command::cargo_bin("todo")?;
    cmd.arg("config").arg("active-context").arg("--help");
    cmd.assert().success();

    Ok(())
}
