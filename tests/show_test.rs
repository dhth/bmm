mod common;
use common::{ExpectedFailure, ExpectedSuccess, Fixture};
use pretty_assertions::assert_eq;

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

Title: <NOT SET>
URI  : https://crates.io/crates/sqlx
Tags : <NOT SET>
"
        .trim()
    );
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
    let output = cmd.output().expect("command should've run");

    // THEN
    output.print_stdout_if_succeeded(None);
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("invalid utf-8 stderr");
    assert!(stderr.contains("bookmark doesn't exist"));
}
