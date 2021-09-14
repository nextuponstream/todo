use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions
use std::process::Command; // Run programs

#[test]
fn missing_config_file_throws_error() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("todo")?;

    cmd.arg("--with-config-path")
        .arg("/path/to/missing/.todo/config")
        .arg("config")
        .arg("set-context")
        .arg("config1");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("No such file or directory"));

    Ok(())
}

// TODO add tests
