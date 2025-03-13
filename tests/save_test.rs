mod common;
use common::Fixture;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;

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
    // THEN
    cmd.assert().success();

    let mut list_cmd = fixture.command();
    list_cmd.arg("list");
    list_cmd.assert().success().stdout(contains(URI_ONE));
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
    // THEN
    cmd.assert().success();

    let mut list_cmd = fixture.command();
    list_cmd.args(["list", "-f", "delimited"]);
    list_cmd.assert().success().stdout(contains(format!(
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
    let mut cmd = fixture.command();
    cmd.args(["save", URI_ONE, "--tags", "bookmarks"]);

    // WHEN
    // THEN
    cmd.assert().success();

    let mut list_cmd = fixture.command();
    list_cmd.args(["list", "-f", "delimited"]);
    list_cmd.assert().success().stdout(contains(format!(
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
    let mut cmd = fixture.command();
    cmd.args(["save", URI_ONE, "--tags", "cli,bookmarks", "-r"]);

    // WHEN
    // THEN
    cmd.assert().success();

    let mut list_cmd = fixture.command();
    list_cmd.args(["list", "-f", "delimited"]);
    list_cmd
        .assert()
        .success()
        .stdout(contains(format!(r#"{},,"bookmarks,cli"#, URI_ONE)));
}

#[test]
fn force_saving_a_new_bookmark_with_a_long_title_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut cmd = fixture.command();
    let title = "a".repeat(501);
    cmd.args(["save", URI_ONE, "--title", title.as_str(), "-i"]);

    // WHEN
    // THEN
    cmd.assert().success();

    let mut show_cmd = fixture.command();
    show_cmd.args(["show", URI_ONE]);
    show_cmd.assert().success().stdout(contains(
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
        .trim(),
    ));
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
    // THEN
    cmd.assert().success();

    let mut show_cmd = fixture.command();
    show_cmd.args(["show", URI_ONE]);
    show_cmd.assert().success().stdout(contains(
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
        .trim(),
    ));
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
    // THEN
    cmd.assert().failure().stderr(contains("title is too long"));
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
    // THEN
    cmd.assert().failure().stderr(contains(
        r#"tags ["invalid tag", " another    invalid\t\ttag "] are invalid"#,
    ));
}

#[test]
fn saving_a_new_bookmark_with_no_text_editor_configured_fails() {
    // GIVEN
    let fixture = Fixture::new();
    let mut cmd = fixture.command();
    cmd.args(["save", URI_ONE, "-e"]);
    cmd.env("BMM_EDITOR", "");
    cmd.env("EDITOR", "");

    // WHEN
    // THEN
    cmd.assert()
        .failure()
        .stderr(contains("no editor configured").and(contains(
            "Suggestion: set the environment variables BMM_EDITOR or EDITOR to use this feature",
        )));
}

#[test]
fn saving_a_new_bookmark_with_incorrect_text_editor_configured_fails() {
    // GIVEN
    let fixture = Fixture::new();
    let mut cmd = fixture.command();
    cmd.args(["save", URI_ONE, "-e"]);
    cmd.env("BMM_EDITOR", "non-existent-4d56150d");
    cmd.env("EDITOR", "non-existent-4d56150d");

    // WHEN
    // THEN
    cmd.assert()
        .failure()
        .stderr(contains("cannot find binary path"));
}
