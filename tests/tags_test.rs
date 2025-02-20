mod common;
use common::{ExpectedFailure, ExpectedSuccess, Fixture};
use pretty_assertions::assert_eq;

//-------------//
//  SUCCESSES  //
//-------------//

#[test]
fn listing_tags_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut import_cmd = fixture.command();
    import_cmd.args(["import", "tests/static/import/valid.json"]);
    let import_output = import_cmd.output().expect("import command should've run");
    assert!(import_output.status.success());

    let mut cmd = fixture.command();
    cmd.args(["tags", "list"]);

    // WHEN
    let output = cmd.output().expect("command should've run");

    // THEN
    output.print_stderr_if_failed(None);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("invalid utf-8 stdout");
    assert_eq!(
        stdout.trim(),
        "
crates
productivity
rust
tools
"
        .trim()
    );
}

#[test]
fn listing_tags_with_stats_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut import_cmd = fixture.command();
    import_cmd.args(["import", "tests/static/import/valid.json"]);
    let import_output = import_cmd.output().expect("import command should've run");
    assert!(import_output.status.success());

    let mut cmd = fixture.command();
    cmd.args(["tags", "list", "-s"]);

    // WHEN
    let output = cmd.output().expect("command should've run");

    // THEN
    output.print_stderr_if_failed(None);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("invalid utf-8 stdout");
    assert_eq!(
        stdout.trim(),
        "
crates (1 bookmark)
productivity (2 bookmarks)
rust (1 bookmark)
tools (3 bookmarks)
"
        .trim()
    );
}

#[test]
fn deleting_tags_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut import_cmd = fixture.command();
    import_cmd.args(["import", "tests/static/import/valid.json"]);
    let import_output = import_cmd.output().expect("import command should've run");
    assert!(import_output.status.success());

    let mut cmd = fixture.command();
    cmd.args(["tags", "delete", "--yes", "productivity", "crates"]);

    // WHEN
    let output = cmd.output().expect("command should've run");

    // THEN
    output.print_stderr_if_failed(None);
    assert!(output.status.success());

    let mut list_cmd = fixture.command();
    list_cmd.args(["tags", "list"]);
    let list_output = list_cmd.output().expect("list command should've run");
    let list_stdout = String::from_utf8(list_output.stdout).expect("invalid utf-8 stdout");
    assert_eq!(
        list_stdout.trim(),
        "
rust
tools
"
        .trim()
    );
}

#[test]
fn renaming_tags_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut import_cmd = fixture.command();
    import_cmd.args(["import", "tests/static/import/valid.json"]);
    let import_output = import_cmd.output().expect("import command should've run");
    assert!(import_output.status.success());

    let mut cmd = fixture.command();
    cmd.args(["tags", "rename", "tools", "cli-tools"]);

    // WHEN
    let output = cmd.output().expect("command should've run");

    // THEN
    output.print_stderr_if_failed(None);
    assert!(output.status.success());

    let mut list_cmd = fixture.command();
    list_cmd.args(["tags", "list"]);
    let list_output = list_cmd.output().expect("list command should've run");
    let list_stdout = String::from_utf8(list_output.stdout).expect("invalid utf-8 stdout");
    assert_eq!(
        list_stdout.trim(),
        "
cli-tools
crates
productivity
rust
"
        .trim()
    );
}

//------------//
//  FAILURES  //
//------------//

#[test]
fn deleting_tags_fails_if_tag_doesnt_exist() {
    // GIVEN
    let fixture = Fixture::new();
    let mut import_cmd = fixture.command();
    import_cmd.args(["import", "tests/static/import/valid.json"]);
    let import_output = import_cmd.output().expect("import command should've run");
    assert!(import_output.status.success());

    let mut cmd = fixture.command();
    cmd.args(["tags", "delete", "--yes", "productivity", "absent"]);

    // WHEN
    let output = cmd.output().expect("command should've run");

    // THEN
    output.print_stdout_if_succeeded(None);
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("invalid utf-8 stderr");
    assert!(stderr.contains(r#"tags do not exist: ["absent"]"#))
}

#[test]
fn renaming_tags_fails_if_tag_doesnt_exist() {
    // GIVEN
    let fixture = Fixture::new();
    let mut import_cmd = fixture.command();
    import_cmd.args(["import", "tests/static/import/valid.json"]);
    let import_output = import_cmd.output().expect("import command should've run");
    assert!(import_output.status.success());

    let mut cmd = fixture.command();
    cmd.args(["tags", "rename", "absent", "target"]);

    // WHEN
    let output = cmd.output().expect("command should've run");

    // THEN
    output.print_stdout_if_succeeded(None);
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("invalid utf-8 stderr");
    assert!(stderr.contains("no such tag"))
}
