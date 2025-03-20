mod common;
use common::Fixture;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;

const URI_ONE: &str = "https://github.com/dhth/bmm";
pub const LOCAL_SERVER_ADDRESS: &str = "http://127.0.0.1:8200";

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
fn updating_bookmarks_with_no_new_data_works() {
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
    cmd.args(["save", URI_ONE]);

    // WHEN
    // THEN
    cmd.assert()
        .success()
        .stdout(contains("nothing to update!"));

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
fn resetting_all_data_for_a_bookmark_works() {
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
    cmd.args(["save", URI_ONE, "-r"]);

    // WHEN
    // THEN
    cmd.assert().success();

    let mut list_cmd = fixture.command();
    list_cmd.args(["list", "-f", "delimited"]);
    list_cmd
        .assert()
        .success()
        .stdout(contains(format!("{},,", URI_ONE)));
}

#[test]
fn resetting_title_on_bookmark_update_works() {
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
    cmd.args(["save", URI_ONE, "--tags", "updated,tags", "-r"]);

    // WHEN
    // THEN
    cmd.assert().success();

    let mut list_cmd = fixture.command();
    list_cmd.args(["list", "-f", "delimited"]);
    list_cmd
        .assert()
        .success()
        .stdout(contains(format!(r#"{},,"tags,updated"#, URI_ONE)));
}

#[test]
fn resetting_tags_on_bookmark_update_works() {
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
    cmd.args(["save", URI_ONE, "--title", "updated title", "-r"]);

    // WHEN
    // THEN
    cmd.assert().success();

    let mut list_cmd = fixture.command();
    list_cmd.args(["list", "-f", "delimited"]);
    list_cmd
        .assert()
        .success()
        .stdout(contains(format!("{},updated title,", URI_ONE)));
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

#[test]
#[ignore = "requires a local http server"]
fn fetching_title_from_remote_server_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut cmd = fixture.command();
    let uri = format!("{}/simple.html", LOCAL_SERVER_ADDRESS);
    cmd.args(["save", &uri, "--tags", "auto,fetch", "-F"]);

    // WHEN
    // THEN
    cmd.assert().success();

    let mut show_cmd = fixture.command();
    show_cmd.args(["show", &uri]);
    show_cmd.assert().success().stdout(contains(
        format!(
            r#"
Bookmark details
---

Title: dhth/bmm: get to your bookmarks in a flash
URI  : {}
Tags : auto,fetch
        "#,
            uri,
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
fn updating_a_bookmarks_with_no_new_details_fails_if_requested() {
    // GIVEN
    let fixture = Fixture::new();
    let mut create_cmd = fixture.command();
    create_cmd.args(["save", URI_ONE]);
    create_cmd.output().expect("command should've run");
    let mut cmd = fixture.command();
    cmd.args(["save", URI_ONE, "-f"]);

    // WHEN
    // THEN
    cmd.assert().failure().stderr(contains("uri already saved"));
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

#[test]
#[ignore = "sends an HTTP request to localhost"]
fn fetching_details_for_non_existent_uri_should_fail() {
    // GIVEN
    let fixture = Fixture::new();
    let mut cmd = fixture.command();
    let uri = "http://127.0.0.1:9999/non-existent.html";
    cmd.args(["save", uri, "-F"]);

    // WHEN
    // THEN
    cmd.assert()
        .failure()
        .stderr(contains("couldn't fetch details"));
}
