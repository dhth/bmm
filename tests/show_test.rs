mod common;
use common::Fixture;
use predicates::str::contains;

const URI: &str = "https://crates.io/crates/sqlx";

//-------------//
//  SUCCESSES  //
//-------------//

#[test]
fn showing_bookmarks_details_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut import_cmd = fixture.command();
    import_cmd.args(["import", "tests/static/import/valid.json"]);
    let import_output = import_cmd.output().expect("import command should've run");
    assert!(import_output.status.success());

    let mut cmd = fixture.command();
    cmd.args(["show", URI]);

    // WHEN
    // THEN
    cmd.assert().success().stdout(contains(
        "
Bookmark details
---

Title: sqlx - crates.io: Rust Package Registry
URI  : https://crates.io/crates/sqlx
Tags : crates,rust
"
        .trim(),
    ));
}

#[test]
fn show_details_output_marks_attributes_that_are_missing() {
    // GIVEN
    let fixture = Fixture::new();
    let mut save_cmd = fixture.command();
    save_cmd.args(["save", URI]);
    let save_output = save_cmd.output().expect("save command should've run");
    assert!(save_output.status.success());

    let mut cmd = fixture.command();
    cmd.args(["show", URI]);

    // WHEN
    // THEN
    cmd.assert().success().stdout(contains(
        "
Bookmark details
---

Title: <NOT SET>
URI  : https://crates.io/crates/sqlx
Tags : <NOT SET>
"
        .trim(),
    ));
}

//------------//
//  FAILURES  //
//------------//

#[test]
fn showing_bookmarks_fails_if_bookmark_doesnt_exist() {
    // GIVEN
    let fixture = Fixture::new();
    let mut cmd = fixture.command();
    cmd.args(["show", URI]);

    // WHEN
    // THEN
    cmd.assert()
        .failure()
        .stderr(contains("bookmark doesn't exist"));
}
