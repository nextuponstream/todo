use assert_cmd::prelude::*;
use predicates::prelude::predicate;
// Add methods on commands
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
    cmd.arg("config").arg("get-contexts").arg("--help");
    cmd.assert().success();

    Ok(())
}

#[test]
fn display_correct_active_context() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("todo")?;
    cmd.arg("--with-config")
        .arg(
            r#"active_ctx_name = "ctx1"

[[ctxs]]
ide = ""
name = "ctx1"
timezone = ""
folder_location = ""

[[ctxs]]
ide = ""
name = "ctx2"
timezone = ""
folder_location = ""

[[ctxs]]
ide = ""
name = "ctx3"
timezone = ""
folder_location = """#,
        )
        .arg("config")
        .arg("get-contexts");
    cmd.assert().success().stdout(predicate::eq(
        r#"→ ctx1
  ctx2
  ctx3
"#,
    ));
    let mut cmd = Command::cargo_bin("todo")?;
    cmd.arg("--with-config")
        .arg(
            r#"active_ctx_name = "ctx2"

[[ctxs]]
ide = ""
name = "ctx1"
timezone = ""
folder_location = ""

[[ctxs]]
ide = ""
name = "ctx2"
timezone = ""
folder_location = ""

[[ctxs]]
ide = ""
name = "ctx3"
timezone = ""
folder_location = """#,
        )
        .arg("config")
        .arg("get-contexts");
    cmd.assert().success().stdout(predicate::eq(
        r#"  ctx1
→ ctx2
  ctx3
"#,
    ));
    let mut cmd = Command::cargo_bin("todo")?;
    cmd.arg("--with-config")
        .arg(
            r#"active_ctx_name = "ctx3"

[[ctxs]]
ide = ""
name = "ctx1"
timezone = ""
folder_location = ""

[[ctxs]]
ide = ""
name = "ctx2"
timezone = ""
folder_location = ""

[[ctxs]]
ide = ""
name = "ctx3"
timezone = ""
folder_location = """#,
        )
        .arg("config")
        .arg("get-contexts");
    cmd.assert().success().stdout(predicate::eq(
        r#"  ctx1
  ctx2
→ ctx3
"#,
    ));
    Ok(())
}

#[test]
fn display_correct_active_context_full() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("todo")?;
    cmd.arg("--with-config")
        .arg(
            r#"active_ctx_name = "ctx1"

[[ctxs]]
ide = ""
name = "ctx1"
timezone = ""
folder_location = ""

[[ctxs]]
ide = ""
name = "ctx2"
timezone = ""
folder_location = ""

[[ctxs]]
ide = ""
name = "ctx3"
timezone = ""
folder_location = """#,
        )
        .arg("config")
        .arg("get-contexts")
        .arg("--full");
    cmd.assert().success().stdout(predicate::eq(
        r#"--- Context (active) ---
name: ctx1
ide: 
timezone: 
folder location: 

--- Context ---
name: ctx2
ide: 
timezone: 
folder location: 

--- Context ---
name: ctx3
ide: 
timezone: 
folder location: 

"#,
    ));
    let mut cmd = Command::cargo_bin("todo")?;
    cmd.arg("--with-config")
        .arg(
            r#"active_ctx_name = "ctx2"

[[ctxs]]
ide = ""
name = "ctx1"
timezone = ""
folder_location = ""

[[ctxs]]
ide = ""
name = "ctx2"
timezone = ""
folder_location = ""

[[ctxs]]
ide = ""
name = "ctx3"
timezone = ""
folder_location = """#,
        )
        .arg("config")
        .arg("get-contexts")
        .arg("--full");
    cmd.assert().success().stdout(predicate::eq(
        r#"--- Context ---
name: ctx1
ide: 
timezone: 
folder location: 

--- Context (active) ---
name: ctx2
ide: 
timezone: 
folder location: 

--- Context ---
name: ctx3
ide: 
timezone: 
folder location: 

"#,
    ));
    let mut cmd = Command::cargo_bin("todo")?;
    cmd.arg("--with-config")
        .arg(
            r#"active_ctx_name = "ctx3"

[[ctxs]]
ide = ""
name = "ctx1"
timezone = ""
folder_location = ""

[[ctxs]]
ide = ""
name = "ctx2"
timezone = ""
folder_location = ""

[[ctxs]]
ide = ""
name = "ctx3"
timezone = ""
folder_location = """#,
        )
        .arg("config")
        .arg("get-contexts")
        .arg("--full");
    cmd.assert().success().stdout(predicate::eq(
        r#"--- Context ---
name: ctx1
ide: 
timezone: 
folder location: 

--- Context ---
name: ctx2
ide: 
timezone: 
folder location: 

--- Context (active) ---
name: ctx3
ide: 
timezone: 
folder location: 

"#,
    ));
    Ok(())
}
