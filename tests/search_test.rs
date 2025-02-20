mod common;
use common::{ExpectedSuccess, Fixture};
use pretty_assertions::assert_eq;

const URI_ONE: &str = "https://crates.io/crates/sqlx";
const URI_TWO: &str = "https://github.com/dhth/omm";
const URI_THREE: &str = "https://github.com/dhth/hours";
const URI_FOUR: &str = "https://github.com/dhth/bmm";

//-------------//
//  SUCCESSES  //
//-------------//

#[test]
fn searching_bookmarks_by_uri_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut import_cmd = fixture.command();
    import_cmd.args(["import", "tests/static/import/valid.json"]);
    let import_output = import_cmd.output().expect("import command should've run");
    assert!(import_output.status.success());

    let mut cmd = fixture.command();
    cmd.args(["search", "crates"]);

    // WHEN
    let output = cmd.output().expect("command should've run");

    // THEN
    output.print_stderr_if_failed(None);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("invalid utf-8 stdout");
    assert_eq!(stdout.trim(), URI_ONE,);
}

#[test]
fn searching_bookmarks_by_title_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut import_cmd = fixture.command();
    import_cmd.args(["import", "tests/static/import/valid.json"]);
    let import_output = import_cmd.output().expect("import command should've run");
    assert!(import_output.status.success());

    let mut cmd = fixture.command();
    cmd.args(["search", "keyboard-driven"]);

    // WHEN
    let output = cmd.output().expect("command should've run");

    // THEN
    output.print_stderr_if_failed(None);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("invalid utf-8 stdout");
    assert_eq!(stdout.trim(), URI_TWO,);
}

#[test]
fn searching_bookmarks_by_tags_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut import_cmd = fixture.command();
    import_cmd.args(["import", "tests/static/import/valid.json"]);
    let import_output = import_cmd.output().expect("import command should've run");
    assert!(import_output.status.success());

    let mut cmd = fixture.command();
    cmd.args(["search", "tools"]);

    // WHEN
    let output = cmd.output().expect("command should've run");

    // THEN
    output.print_stderr_if_failed(None);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("invalid utf-8 stdout");
    assert_eq!(
        stdout.trim(),
        format!(
            "
{}
{}
{}
",
            URI_TWO, URI_THREE, URI_FOUR
        )
        .trim()
    );
}
