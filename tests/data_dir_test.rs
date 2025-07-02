mod common;

use common::Fixture;
use insta_cmd::assert_cmd_snapshot;
use tempfile::tempdir;

const URI: &str = "https://crates.io/crates/sqlx";

//-------------//
//  SUCCESSES  //
//-------------//

#[test]
fn xdg_data_home_is_respected() {
    // GIVEN
    let fx = Fixture::new();
    let temp_dir = tempdir().expect("temporary directory should've been created");
    let data_dir_path = temp_dir
        .path()
        .to_str()
        .expect("temporary directory path is not valid utf-8")
        .to_string();
    let mut import_cmd = fx.base_cmd();
    import_cmd.args(["import", "tests/static/import/valid.json"]);
    import_cmd.env("XDG_DATA_HOME", &data_dir_path);
    assert_cmd_snapshot!(import_cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    imported 4 bookmarks

    ----- stderr -----
    ");

    let mut cmd_without_env_var = fx.cmd(["show", URI]);
    let mut cmd = fx.base_cmd();
    cmd.args(["show", URI]);
    cmd.env("XDG_DATA_HOME", &data_dir_path);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd_without_env_var, @r"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Error: couldn't show bookmark details: bookmark doesn't exist
    ");

    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Bookmark details
    ---

    Title: sqlx - crates.io: Rust Package Registry
    URI  : https://crates.io/crates/sqlx
    Tags : crates,rust

    ----- stderr -----
    ");
}

//------------//
//  FAILURES  //
//------------//

#[cfg(target_family = "unix")]
#[test]
fn fails_if_xdg_data_home_is_non_absolute() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.base_cmd();
    cmd.args(["show", URI]);
    cmd.env("XDG_DATA_HOME", "../not/an/absolute/path");

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Error: XDG_DATA_HOME is not an absolute path

    Context: XDG specifications dictate that XDG_DATA_HOME must be an absolute path.
    Read more here: https://specifications.freedesktop.org/basedir-spec/latest/#basics
    ");
}
