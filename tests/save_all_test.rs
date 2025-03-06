mod common;
use common::{ExpectedFailure, ExpectedSuccess, Fixture};
use pretty_assertions::assert_eq;

const URI_ONE: &str = "https://github.com/dhth/bmm";
const URI_TWO: &str = "https://github.com/dhth/omm";
const URI_THREE: &str = "https://github.com/dhth/hours";

//-------------//
//  SUCCESSES  //
//-------------//

#[test]
fn saving_multiple_bookmarks_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut cmd = fixture.command();
    cmd.args(["save-all", URI_ONE, URI_TWO, URI_THREE]);

    // WHEN
    let output = cmd.output().expect("command should've run");

    // THEN
    output.print_stderr_if_failed(None);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("invalid utf-8 stdout");
    assert_eq!(stdout.trim(), "saved 3 bookmarks");

    let mut list_cmd = fixture.command();
    list_cmd.arg("list");
    let list_output = list_cmd.output().expect("list command should've run");
    assert!(list_output.status.success());
    let list_stdout = String::from_utf8(list_output.stdout).expect("invalid utf-8 list_stdout");
    assert_eq!(
        list_stdout.trim(),
        format!(
            "
{}
{}
{}
",
            URI_ONE, URI_TWO, URI_THREE
        )
        .trim()
    );
}

#[test]
fn saving_multiple_bookmarks_with_tags_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut cmd = fixture.command();
    cmd.args([
        "save-all",
        URI_ONE,
        URI_TWO,
        URI_THREE,
        "-t",
        "tools,productivity",
    ]);

    // WHEN
    let output = cmd.output().expect("command should've run");

    // THEN
    output.print_stderr_if_failed(None);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("invalid utf-8 stdout");
    assert_eq!(stdout.trim(), "saved 3 bookmarks");

    let mut list_tags_cmd = fixture.command();
    list_tags_cmd.args(["tags", "list"]);
    let list_tags_output = list_tags_cmd
        .output()
        .expect("list tags command should've run");
    assert!(list_tags_output.status.success());
    let list_tags_stdout =
        String::from_utf8(list_tags_output.stdout).expect("invalid utf-8 list_stdout");
    assert_eq!(
        list_tags_stdout.trim(),
        "
productivity
tools
"
        .trim()
    );
}

#[test]
fn saving_multiple_bookmarks_extends_previously_saved_tags() {
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
    cmd.args(["save-all", URI_ONE, URI_TWO, URI_THREE, "-t", "tools"]);

    // WHEN
    let output = cmd.output().expect("command should've run");

    // THEN
    output.print_stderr_if_failed(None);
    assert!(output.status.success());

    let mut show_cmd = fixture.command();
    show_cmd.args(["show", URI_ONE]);
    let show_output = show_cmd.output().expect("list command should've run");
    assert!(show_output.status.success());
    let show_stdout = String::from_utf8(show_output.stdout).expect("invalid utf-8 list_stdout");
    assert_eq!(
        show_stdout.trim(),
        format!(
            r#"
Bookmark details
---

Title: bmm's github page
URI  : {}
Tags : productivity,tools
"#,
            URI_ONE
        )
        .trim()
    );
}

#[test]
fn saving_multiple_bookmarks_resets_previously_saved_tags_if_requested() {
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
    cmd.args(["save-all", URI_ONE, URI_TWO, URI_THREE, "-t", "tools", "-r"]);

    // WHEN
    let output = cmd.output().expect("command should've run");

    // THEN
    output.print_stderr_if_failed(None);
    assert!(output.status.success());

    let mut show_cmd = fixture.command();
    show_cmd.args(["show", URI_ONE]);
    let show_output = show_cmd.output().expect("list command should've run");
    assert!(show_output.status.success());
    let show_stdout = String::from_utf8(show_output.stdout).expect("invalid utf-8 list_stdout");
    assert_eq!(
        show_stdout.trim(),
        format!(
            r#"
Bookmark details
---

Title: bmm's github page
URI  : {}
Tags : tools
"#,
            URI_ONE
        )
        .trim()
    );
}

#[test]
fn force_saving_multiple_bookmarks_with_invalid_tags_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut cmd = fixture.command();
    cmd.args([
        "save-all",
        URI_ONE,
        URI_TWO,
        URI_THREE,
        "--tags",
        "tag1,invalid tag, another    invalid\t\ttag ",
        "-i",
    ]);

    // WHEN
    let output = cmd.output().expect("command should've run");

    // THEN
    output.print_stderr_if_failed(None);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("invalid utf-8 stdout");
    assert_eq!(stdout.trim(), "saved 3 bookmarks");

    let mut list_tags_cmd = fixture.command();
    list_tags_cmd.args(["tags", "list"]);
    let list_tags_output = list_tags_cmd
        .output()
        .expect("list tags command should've run");
    assert!(list_tags_output.status.success());
    let list_tags_stdout =
        String::from_utf8(list_tags_output.stdout).expect("invalid utf-8 list_stdout");
    assert_eq!(
        list_tags_stdout.trim(),
        "
another-invalid-tag
invalid-tag
tag1
"
        .trim()
    );
}

//------------//
//  FAILURES  //
//------------//

#[test]
fn saving_multiple_bookmarks_fails_for_incorrect_uris() {
    // GIVEN
    let fixture = Fixture::new();
    let mut cmd = fixture.command();
    cmd.args([
        "save-all",
        "this is not a uri",
        URI_TWO,
        "https:/ this!!isn't-either.com",
    ]);

    // WHEN
    let output = cmd.output().expect("command should've run");

    // THEN
    output.print_stdout_if_succeeded(None);
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("invalid utf-8 stderr");
    assert!(stderr.contains("- entry 1: couldn't parse provided uri value"));
    assert!(stderr.contains("- entry 3: couldn't parse provided uri value"));
}
