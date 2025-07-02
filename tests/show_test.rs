mod common;

use common::Fixture;
use insta_cmd::assert_cmd_snapshot;

const URI: &str = "https://crates.io/crates/sqlx";

//-------------//
//  SUCCESSES  //
//-------------//

#[test]
fn showing_bookmarks_details_works() {
    // GIVEN
    let fx = Fixture::new();
    let mut import_cmd = fx.cmd(["import", "tests/static/import/valid.json"]);
    assert_cmd_snapshot!(import_cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    imported 4 bookmarks

    ----- stderr -----
    ");

    let mut cmd = fx.cmd(["show", URI]);

    // WHEN
    // THEN
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

#[test]
fn show_details_output_marks_attributes_that_are_missing() {
    // GIVEN
    let fx = Fixture::new();
    let mut save_cmd = fx.cmd(["save", URI]);
    assert_cmd_snapshot!(save_cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    let mut cmd = fx.cmd(["show", URI]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Bookmark details
    ---

    Title: <NOT SET>
    URI  : https://crates.io/crates/sqlx
    Tags : <NOT SET>

    ----- stderr -----
    ");
}

//------------//
//  FAILURES  //
//------------//

#[test]
fn showing_bookmarks_fails_if_bookmark_doesnt_exist() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd(["show", URI]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Error: couldn't show bookmark details: bookmark doesn't exist
    ");
}
