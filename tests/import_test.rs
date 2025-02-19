mod common;
use common::{ExpectedFailure, ExpectedSuccess, Fixture};
use pretty_assertions::assert_eq;

//-------------//
//  SUCCESSES  //
//-------------//

#[test]
fn importing_from_an_html_file_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut cmd = fixture.cmd();
    cmd.args(["import", "tests/static/import/valid.html"]);

    // WHEN
    let output = cmd.output().expect("command should've run");

    // THEN
    output.print_stderr_if_failed(None);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("invalid utf-8 stdout");
    assert_eq!(stdout.trim(), "imported 4 bookmarks");
}

#[test]
fn importing_from_an_invalid_html_file_doesnt_fail() {
    // GIVEN
    let fixture = Fixture::new();
    let mut cmd = fixture.cmd();
    cmd.args(["import", "tests/static/import/invalid.html"]);

    // WHEN
    let output = cmd.output().expect("command should've run");

    // THEN
    output.print_stderr_if_failed(None);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("invalid utf-8 stdout");
    assert_eq!(stdout.trim(), "imported 0 bookmarks");
}

#[test]
fn importing_from_a_valid_json_file_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut cmd = fixture.cmd();
    cmd.args(["import", "tests/static/import/valid.json"]);

    // WHEN
    let output = cmd.output().expect("command should've run");

    // THEN
    output.print_stderr_if_failed(None);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("invalid utf-8 stdout");
    assert_eq!(stdout.trim(), "imported 4 bookmarks");
}

#[test]
fn importing_from_a_json_file_with_only_mandatory_details_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut cmd = fixture.cmd();
    cmd.args(["import", "tests/static/import/only-mandatory.json"]);

    // WHEN
    let output = cmd.output().expect("command should've run");

    // THEN
    output.print_stderr_if_failed(None);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("invalid utf-8 stdout");
    assert_eq!(stdout.trim(), "imported 2 bookmarks");
}

#[test]
fn importing_from_a_valid_txt_file_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut cmd = fixture.cmd();
    cmd.args(["import", "tests/static/import/valid.txt"]);

    // WHEN
    let output = cmd.output().expect("command should've run");

    // THEN
    output.print_stderr_if_failed(None);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("invalid utf-8 stdout");
    assert_eq!(stdout.trim(), "imported 4 bookmarks");
}

//------------//
//  FAILURES  //
//------------//

#[test]
fn importing_from_an_invalid_json_file_fails() {
    // GIVEN
    let fixture = Fixture::new();
    let mut cmd = fixture.cmd();
    cmd.args(["import", "tests/static/import/invalid.json"]);

    // WHEN
    let output = cmd.output().expect("command should've run");

    // THEN
    output.print_stdout_if_succeeded(None);
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("invalid utf-8 stderr");
    assert!(stderr.contains("couldn't parse JSON input"));
}

#[test]
fn importing_from_a_json_file_fails_if_missing_uri() {
    // GIVEN
    let fixture = Fixture::new();
    let mut cmd = fixture.cmd();
    cmd.args(["import", "tests/static/import/missing-uri.json"]);

    // WHEN
    let output = cmd.output().expect("command should've run");

    // THEN
    output.print_stdout_if_succeeded(None);
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("invalid utf-8 stderr");
    assert!(stderr.contains("missing field `uri`"));
}
