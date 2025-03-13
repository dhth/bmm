mod common;
use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use tempfile::tempdir;

const URI: &str = "https://crates.io/crates/sqlx";

//-------------//
//  SUCCESSES  //
//-------------//

#[test]
fn xdg_data_home_is_respected() {
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

    let mut cmd_without_env_var =
        Command::cargo_bin(env!("CARGO_PKG_NAME")).expect("command should've been created");
    cmd_without_env_var.args(["show", URI]);
    let mut cmd =
        Command::cargo_bin(env!("CARGO_PKG_NAME")).expect("command should've been created");
    cmd.args(["show", URI]);
    cmd.env("XDG_DATA_HOME", &data_dir_path);

    // WHEN
    // THEN
    cmd_without_env_var.assert().failure();
    cmd.assert().success();
}

//------------//
//  FAILURES  //
//------------//

#[cfg(target_family = "unix")]
#[test]
fn fails_if_xdg_data_home_is_non_absolute() {
    // GIVEN

    let mut cmd =
        Command::cargo_bin(env!("CARGO_PKG_NAME")).expect("command should've been created");
    cmd.args(["show", URI]);
    cmd.env("XDG_DATA_HOME", "../not/an/absolute/path");

    // WHEN
    // THEN
    cmd.assert()
        .failure()
        .stderr(contains("XDG_DATA_HOME is not an absolute path").and(contains("Context: ")));
}
