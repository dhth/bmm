mod common;
use common::{ExpectedFailure, ExpectedSuccess, Fixture};
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

#[test]
fn force_saving_a_new_bookmark_with_a_long_title_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut cmd = fixture.command();
    let title = "a".repeat(501);
    cmd.args(["save", URI_ONE, "--title", title.as_str(), "-i"]);

    // WHEN
    let output = cmd.output().expect("command should've run");

    // THEN
    output.print_stderr_if_failed(None);
    assert!(output.status.success());

    let mut show_cmd = fixture.command();
    show_cmd.args(["show", URI_ONE]);
    let show_output = show_cmd.output().expect("show command should've run");
    assert!(show_output.status.success());
    let list_stdout = String::from_utf8(show_output.stdout).expect("invalid utf-8 list_stdout");
    assert_eq!(
        list_stdout.trim(),
        format!(
            r#"
Bookmark details
---

Title: {}
URI  : {}
Tags : <NOT SET>
        "#,
            "a".repeat(500),
            URI_ONE
        )
        .trim()
    );
}

#[test]
fn force_saving_a_new_bookmark_with_invalid_tags_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut cmd = fixture.command();
    cmd.args([
        "save",
        URI_ONE,
        "--tags",
        "tag1,invalid tag, another    invalid\t\ttag ",
        "-i",
    ]);

    // WHEN
    let output = cmd.output().expect("command should've run");

    // THEN
    output.print_stderr_if_failed(None);
    assert!(output.status.success());

    let mut show_cmd = fixture.command();
    show_cmd.args(["show", URI_ONE]);
    let show_output = show_cmd.output().expect("show command should've run");
    assert!(show_output.status.success());
    let list_stdout = String::from_utf8(show_output.stdout).expect("invalid utf-8 list_stdout");
    assert_eq!(
        list_stdout.trim(),
        format!(
            r#"
Bookmark details
---

Title: <NOT SET>
URI  : {}
Tags : another-invalid-tag,invalid-tag,tag1
        "#,
            URI_ONE
        )
        .trim()
    );
}

//------------//
//  FAILURES  //
//------------//

#[test]
fn saving_a_new_bookmark_with_a_long_title_fails() {
    // GIVEN
    let fixture = Fixture::new();
    let mut cmd = fixture.command();
    let title = "a".repeat(501);
    cmd.args(["save", URI_ONE, "--title", title.as_str()]);

    // WHEN
    let output = cmd.output().expect("command should've run");

    // THEN
    output.print_stdout_if_succeeded(None);
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("invalid utf-8 list_stderr");
    assert!(stderr.contains("title is too long"))
}

#[test]
fn saving_a_new_bookmark_with_an_invalid_tag_fails() {
    // GIVEN
    let fixture = Fixture::new();
    let mut cmd = fixture.command();
    cmd.args([
        "save",
        URI_ONE,
        "--tags",
        "tag1,invalid tag, another    invalid\t\ttag ",
    ]);

    // WHEN
    let output = cmd.output().expect("command should've run");

    // THEN
    output.print_stdout_if_succeeded(None);
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("invalid utf-8 list_stderr");
    assert!(stderr.contains(r#"tags ["invalid tag", " another    invalid\t\ttag "] are invalid"#))
}
