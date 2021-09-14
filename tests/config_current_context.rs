use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions
use std::process::Command; // Run programs

#[test]
fn missing_config_file_throws_error() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("todo")?;

    cmd.arg("--with-config-path")
        .arg("/path/to/missing/.todo/config")
        .arg("config")
        .arg("current-context");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("No such file or directory"));

    Ok(())
}

#[test]
fn empty_config_file_has_no_context() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("todo")?;

    cmd.arg("--with-config")
        .arg("")
        .arg("config")
        .arg("current-context");
    cmd.assert().failure();

    Ok(())
}

#[test]
fn parses_config1() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("todo")?;

    cmd.arg("--with-config")
        .arg(
            "current_config = \"config1\"

[[config]]
name = \"config1\"
ide = \"config1_ide\"
timezone = \"config1_timezone\"
todo_folder = \"/path/to/config1/folder\"

[[config]]
name = \"config2\"
ide = \"config2_ide\"
timezone = \"config2_timezone\"
todo_folder = \"/path/to/config2/folder\"",
        )
        .arg("config")
        .arg("current-context");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("config1"));

    Ok(())
}

#[test]
fn parses_config2() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("todo")?;

    cmd.arg("--with-config")
        .arg(
            "current_config = \"config2\"

[[config]]
name = \"config1\"
ide = \"config1_ide\"
timezone = \"config1_timezone\"
todo_folder = \"/path/to/config1/folder\"

[[config]]
name = \"config2\"
ide = \"config2_ide\"
timezone = \"config2_timezone\"
todo_folder = \"/path/to/config2/folder\"",
        )
        .arg("config")
        .arg("current-context");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("config2"));

    Ok(())
}

#[test]
fn unknown_current_context_throws_error() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("todo")?;

    cmd.arg("--with-config")
        .arg(
            "current_config = \"config3\"

[[config]]
name = \"config1\"
ide = \"config1_ide\"
timezone = \"config1_timezone\"
todo_folder = \"/path/to/config1/folder\"

[[config]]
name = \"config2\"
ide = \"config2_ide\"
timezone = \"config2_timezone\"
todo_folder = \"/path/to/config2/folder\"",
        )
        .arg("config")
        .arg("current-context");
    cmd.assert().failure();

    Ok(())
}
