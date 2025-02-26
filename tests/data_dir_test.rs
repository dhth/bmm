mod common;
use assert_cmd::Command;
use common::{ExpectedFailure, ExpectedSuccess};
use pretty_assertions::assert_eq;
use tempfile::tempdir;

const URI: &str = "https://crates.io/crates/sqlx";

//-------------//
//  SUCCESSES  //
//-------------//

#[cfg(target_os = "macos")]
#[test]
fn xdg_config_is_respected_on_darwin() {
    // GIVEN
    let temp_dir = tempdir().expect("temporary directory should've been created");
    let data_dir_path = temp_dir
        .path()
        .to_str()
        .expect("temporary directory path is not valid utf-8")
        .to_string();
    let mut import_cmd =
        Command::cargo_bin(env!("CARGO_PKG_NAME")).expect("command should've been created");
    import_cmd.args(["import", "tests/static/import/valid.json"]);
    import_cmd.env("XDG_DATA_HOME", &data_dir_path);

    let import_output = import_cmd.output().expect("import command should've run");
    assert!(import_output.status.success());

    let mut cmd =
        Command::cargo_bin(env!("CARGO_PKG_NAME")).expect("command should've been created");
    cmd.args(["show", URI]);
    cmd.env("XDG_DATA_HOME", &data_dir_path);

    // WHEN
    let output = cmd.output().expect("command should've run");

    // THEN
    output.print_stderr_if_failed(None);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("invalid utf-8 stdout");
    assert_eq!(
        stdout.trim(),
        "
Bookmark details
---

Title: sqlx - crates.io: Rust Package Registry
URI  : https://crates.io/crates/sqlx
Tags : crates,rust
"
        .trim()
    );
}

//------------//
//  FAILURES  //
//------------//

#[cfg(target_os = "macos")]
#[test]
fn fails_if_xdg_data_home_is_non_absolute() {
    // GIVEN
    let mut cmd =
        Command::cargo_bin(env!("CARGO_PKG_NAME")).expect("command should've been created");
    cmd.args(["show", URI]);
    cmd.env("XDG_DATA_HOME", "../not/an/absolute/path");

    // WHEN
    let output = cmd.output().expect("command should've run");

    // THEN
    output.print_stdout_if_succeeded(None);
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("invalid utf-8 stderr");
    assert!(stderr.contains("XDG_DATA_HOME is not an absolute path"));
}
