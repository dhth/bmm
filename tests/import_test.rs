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
    let mut cmd = fixture.command();
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
    let mut cmd = fixture.command();
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
    let mut cmd = fixture.command();
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
    let mut cmd = fixture.command();
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
    let mut cmd = fixture.command();
    cmd.args(["import", "tests/static/import/valid.txt"]);

    // WHEN
    let output = cmd.output().expect("command should've run");

    // THEN
    output.print_stderr_if_failed(None);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("invalid utf-8 stdout");
    assert_eq!(stdout.trim(), "imported 4 bookmarks");
}

#[test]
fn importing_extends_previously_saved_info() {
    // GIVEN
    let fixture = Fixture::new();
    let uri = "https://github.com/dhth/bmm";
    let mut create_cmd = fixture.command();
    create_cmd.args([
        "save",
        uri,
        "--title",
        "bmm's github page",
        "--tags",
        "productivity",
    ]);
    create_cmd.output().expect("command should've run");

    let mut cmd = fixture.command();
    cmd.args(["import", "tests/static/import/valid.json"]);

    // WHEN
    let output = cmd.output().expect("command should've run");

    // THEN
    output.print_stderr_if_failed(None);
    assert!(output.status.success());

    let mut show_cmd = fixture.command();
    show_cmd.args(["show", uri]);
    let show_output = show_cmd.output().expect("list command should've run");
    assert!(show_output.status.success());
    let show_stdout = String::from_utf8(show_output.stdout).expect("invalid utf-8 list_stdout");
    assert_eq!(
        show_stdout.trim(),
        format!(
            r#"
Bookmark details
---

Title: GitHub - dhth/bmm: get to your bookmarks in a flash
URI  : {}
Tags : productivity,tools
"#,
            uri
        )
        .trim()
    );
}

#[test]
fn importing_resets_previously_saved_info_if_requested() {
    // GIVEN
    let fixture = Fixture::new();
    let uri = "https://github.com/dhth/omm";
    let mut create_cmd = fixture.command();
    create_cmd.args([
        "save",
        uri,
        "--title",
        "omm's github page",
        "--tags",
        "task-management,productivity",
    ]);
    create_cmd.output().expect("command should've run");

    let mut cmd = fixture.command();
    cmd.args(["import", "tests/static/import/only-mandatory.json", "-r"]);

    // WHEN
    let output = cmd.output().expect("command should've run");

    // THEN
    output.print_stderr_if_failed(None);
    assert!(output.status.success());

    let mut show_cmd = fixture.command();
    show_cmd.args(["show", uri]);
    let show_output = show_cmd.output().expect("list command should've run");
    assert!(show_output.status.success());
    let show_stdout = String::from_utf8(show_output.stdout).expect("invalid utf-8 list_stdout");
    assert_eq!(
        show_stdout.trim(),
        format!(
            r#"
Bookmark details
---

Title: <NOT SET>
URI  : {}
Tags : <NOT SET>
"#,
            uri
        )
        .trim()
    );
}

//------------//
//  FAILURES  //
//------------//

#[test]
fn importing_from_an_invalid_json_file_fails() {
    // GIVEN
    let fixture = Fixture::new();
    let mut cmd = fixture.command();
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
    let mut cmd = fixture.command();
    cmd.args(["import", "tests/static/import/missing-uri.json"]);

    // WHEN
    let output = cmd.output().expect("command should've run");

    // THEN
    output.print_stdout_if_succeeded(None);
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("invalid utf-8 stderr");
    assert!(stderr.contains("missing field `uri`"));
}
