mod common;

use common::Fixture;
use insta_cmd::assert_cmd_snapshot;

const URI_ONE: &str = "https://github.com/dhth/bmm";

//-------------//
//  SUCCESSES  //
//-------------//

#[test]
fn saving_a_new_bookmark_works() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd(["save", URI_ONE]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    let mut list_cmd = fx.cmd(["list"]);
    assert_cmd_snapshot!(list_cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    https://github.com/dhth/bmm

    ----- stderr -----
    ");
}

#[test]
fn saving_a_new_bookmark_with_title_and_tags_works() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd([
        "save",
        URI_ONE,
        "--title",
        "bmm's github page",
        "--tags",
        "tools,productivity",
    ]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    let mut list_cmd = fx.cmd(["list", "--format", "delimited"]);
    assert_cmd_snapshot!(list_cmd, @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    uri,title,tags
    https://github.com/dhth/bmm,bmm's github page,"productivity,tools"

    ----- stderr -----
    "#);
}

#[test]
fn extending_tags_for_a_saved_bookmark_works() {
    // GIVEN
    let fx = Fixture::new();
    let mut create_cmd = fx.cmd([
        "save",
        URI_ONE,
        "--title",
        "bmm's github page",
        "--tags",
        "tools,productivity",
    ]);
    assert_cmd_snapshot!(create_cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    let mut cmd = fx.cmd(["save", URI_ONE, "--tags", "bookmarks"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    let mut list_cmd = fx.cmd(["list", "--format", "delimited"]);
    assert_cmd_snapshot!(list_cmd, @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    uri,title,tags
    https://github.com/dhth/bmm,bmm's github page,"bookmarks,productivity,tools"

    ----- stderr -----
    "#);
}

#[test]
fn resetting_properties_on_bookmark_update_works() {
    // GIVEN
    let fx = Fixture::new();
    let mut create_cmd = fx.cmd([
        "save",
        URI_ONE,
        "--title",
        "bmm's github page",
        "--tags",
        "tools,productivity",
    ]);
    assert_cmd_snapshot!(create_cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    let mut cmd = fx.cmd([
        "save",
        URI_ONE,
        "--tags",
        "cli,bookmarks",
        "--reset-missing-details",
    ]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    let mut list_cmd = fx.cmd(["list", "--format", "delimited"]);
    assert_cmd_snapshot!(list_cmd, @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    uri,title,tags
    https://github.com/dhth/bmm,,"bookmarks,cli"

    ----- stderr -----
    "#);
}

#[test]
fn force_saving_a_new_bookmark_with_a_long_title_works() {
    // GIVEN
    let fx = Fixture::new();
    let title = "a".repeat(501);
    let mut cmd = fx.cmd([
        "save",
        URI_ONE,
        "--title",
        title.as_str(),
        "--ignore-attribute-errors",
    ]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    let mut show_cmd = fx.cmd(["show", URI_ONE]);
    assert_cmd_snapshot!(show_cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Bookmark details
    ---

    Title: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
    URI  : https://github.com/dhth/bmm
    Tags : <NOT SET>

    ----- stderr -----
    ");
}

#[test]
fn force_saving_a_new_bookmark_with_invalid_tags_works() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd([
        "save",
        URI_ONE,
        "--tags",
        "tag1,invalid tag, another    invalid\t\ttag ",
        "--ignore-attribute-errors",
    ]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    let mut show_cmd = fx.cmd(["show", URI_ONE]);
    assert_cmd_snapshot!(show_cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Bookmark details
    ---

    Title: <NOT SET>
    URI  : https://github.com/dhth/bmm
    Tags : another-invalid-tag,invalid-tag,tag1

    ----- stderr -----
    ");
}

//------------//
//  FAILURES  //
//------------//

#[test]
fn saving_a_new_bookmark_with_a_long_title_fails() {
    // GIVEN
    let fx = Fixture::new();
    let title = "a".repeat(501);
    let mut cmd = fx.cmd(["save", URI_ONE, "--title", title.as_str()]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Error: couldn't save bookmark: title is too long: 501 (max: 500)

    Possible workaround: running with -i/--ignore-attribute-errors might fix some attribute errors.
    If a title is too long, it'll will be trimmed, and some invalid tags might be transformed to fit bmm's requirements.
    ");
}

#[test]
fn saving_a_new_bookmark_with_an_invalid_tag_fails() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd([
        "save",
        URI_ONE,
        "--tags",
        "tag1,invalid tag, another    invalid\t\ttag ",
    ]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r#"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Error: couldn't save bookmark: tags ["invalid tag", " another    invalid\t\ttag "] are invalid (valid regex: ^[a-zA-Z0-9_-]{1,30}$)

    Possible workaround: running with -i/--ignore-attribute-errors might fix some attribute errors.
    If a title is too long, it'll will be trimmed, and some invalid tags might be transformed to fit bmm's requirements.
    "#);
}

#[test]
fn saving_a_new_bookmark_with_no_text_editor_configured_fails() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd(["save", URI_ONE, "--editor"]);
    cmd.env("BMM_EDITOR", "");
    cmd.env("EDITOR", "");

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Error: couldn't save bookmark: no editor configured

    Suggestion: set the environment variables BMM_EDITOR or EDITOR to use this feature
    ");
}

#[test]
fn saving_a_new_bookmark_with_incorrect_text_editor_configured_fails() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd(["save", URI_ONE, "--editor"]);
    cmd.env("BMM_EDITOR", "non-existent-4d56150d");
    cmd.env("EDITOR", "non-existent-4d56150d");

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r#"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Error: couldn't save bookmark: couldn't find editor executable "non-existent-4d56150d": cannot find binary path

    Context: bmm used the environment variable BMM_EDITOR to determine your text editor.
    Check if "non-existent-4d56150d" actually points to your text editor's executable.
    "#);
}
