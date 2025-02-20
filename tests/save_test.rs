mod common;
use common::{ExpectedSuccess, Fixture};
use pretty_assertions::assert_eq;

const URI_ONE: &str = "https://github.com/dhth/bmm";

//-------------//
//  SUCCESSES  //
//-------------//

#[test]
fn saving_a_new_bookmark_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut cmd = fixture.command();
    cmd.args(["save", URI_ONE]);

    // WHEN
    let output = cmd.output().expect("command should've run");

    // THEN
    output.print_stderr_if_failed(None);
    assert!(output.status.success());

    let mut list_cmd = fixture.command();
    list_cmd.arg("list");
    let list_output = list_cmd.output().expect("list command should've run");
    assert!(list_output.status.success());
    let list_stdout = String::from_utf8(list_output.stdout).expect("invalid utf-8 list_stdout");
    assert_eq!(list_stdout.trim(), URI_ONE);
}

#[test]
fn saving_a_new_bookmark_with_title_and_tags_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut cmd = fixture.command();
    cmd.args([
        "save",
        URI_ONE,
        "--title",
        "bmm's github page",
        "--tags",
        "tools,productivity",
    ]);

    // WHEN
    let output = cmd.output().expect("command should've run");

    // THEN
    output.print_stderr_if_failed(None);
    assert!(output.status.success());

    let mut list_cmd = fixture.command();
    list_cmd.args(["list", "-f", "delimited"]);
    let list_output = list_cmd.output().expect("list command should've run");
    assert!(list_output.status.success());
    let list_stdout = String::from_utf8(list_output.stdout).expect("invalid utf-8 list_stdout");
    assert!(list_stdout.contains(&format!(
        r#"{},bmm's github page,"productivity,tools"#,
        URI_ONE
    )));
}

#[test]
fn extending_tags_for_a_saved_bookmark_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut create_cmd = fixture.command();
    create_cmd.args([
        "save",
        URI_ONE,
        "--title",
        "bmm's github page",
        "--tags",
        "tools,productivity",
    ]);
    create_cmd.output().expect("command should've run");

    // WHEN
    let mut cmd = fixture.command();
    cmd.args(["save", URI_ONE, "--tags", "bookmarks"]);
    let output = cmd.output().expect("command should've run");

    // THEN
    output.print_stderr_if_failed(None);
    assert!(output.status.success());

    let mut list_cmd = fixture.command();
    list_cmd.args(["list", "-f", "delimited"]);
    let list_output = list_cmd.output().expect("list command should've run");
    assert!(list_output.status.success());
    let list_stdout = String::from_utf8(list_output.stdout).expect("invalid utf-8 list_stdout");
    assert!(list_stdout.contains(&format!(
        r#"{},bmm's github page,"bookmarks,productivity,tools"#,
        URI_ONE
    )));
}

#[test]
fn resetting_properties_on_bookmark_update_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut create_cmd = fixture.command();
    create_cmd.args([
        "save",
        URI_ONE,
        "--title",
        "bmm's github page",
        "--tags",
        "tools,productivity",
    ]);
    create_cmd.output().expect("command should've run");

    // WHEN
    let mut cmd = fixture.command();
    cmd.args(["save", URI_ONE, "--tags", "cli,bookmarks", "-r"]);
    let output = cmd.output().expect("command should've run");

    // THEN
    output.print_stderr_if_failed(None);
    assert!(output.status.success());

    let mut list_cmd = fixture.command();
    list_cmd.args(["list", "-f", "delimited"]);
    let list_output = list_cmd.output().expect("list command should've run");
    assert!(list_output.status.success());
    let list_stdout = String::from_utf8(list_output.stdout).expect("invalid utf-8 list_stdout");
    assert!(list_stdout.contains(&format!(r#"{},,"bookmarks,cli"#, URI_ONE)));
}
