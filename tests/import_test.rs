mod common;

use common::Fixture;
use insta_cmd::assert_cmd_snapshot;

//-------------//
//  SUCCESSES  //
//-------------//

#[test]
fn importing_from_an_html_file_works() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd(["import", "tests/static/import/valid.html"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    imported 4 bookmarks

    ----- stderr -----
    ");
}

#[test]
fn importing_from_an_invalid_html_file_doesnt_fail() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd(["import", "tests/static/import/invalid.html"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    imported 0 bookmarks

    ----- stderr -----
    ");
}

#[test]
fn force_importing_from_an_html_file_with_some_invalid_attrs_works() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd([
        "import",
        "tests/static/import/valid-with-some-invalid-attributes.html",
        "--ignore-attribute-errors",
    ]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    imported 4 bookmarks

    ----- stderr -----
    ");
}

#[test]
fn importing_from_a_valid_json_file_works() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd(["import", "tests/static/import/valid.json"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    imported 4 bookmarks

    ----- stderr -----
    ");
}

#[test]
fn importing_from_a_json_file_with_only_mandatory_details_works() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd(["import", "tests/static/import/only-mandatory.json"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    imported 2 bookmarks

    ----- stderr -----
    ");
}

#[test]
fn force_importing_from_a_json_file_with_some_invalid_attrs_works() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd([
        "import",
        "tests/static/import/valid-with-some-invalid-attributes.json",
        "--ignore-attribute-errors",
    ]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    imported 4 bookmarks

    ----- stderr -----
    ");
}

#[test]
fn importing_from_a_valid_txt_file_works() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd(["import", "tests/static/import/valid.txt"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    imported 4 bookmarks

    ----- stderr -----
    ");
}

#[test]
fn importing_extends_previously_saved_info() {
    // GIVEN
    let uri = "https://github.com/dhth/bmm";
    let fx = Fixture::new();
    let mut create_cmd = fx.cmd([
        "save",
        uri,
        "--title",
        "bmm's github page",
        "--tags",
        "productivity",
    ]);
    assert_cmd_snapshot!(create_cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    let mut cmd = fx.cmd(["import", "tests/static/import/valid.json"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    imported 4 bookmarks

    ----- stderr -----
    ");

    let mut show_cmd = fx.cmd(["show", uri]);
    assert_cmd_snapshot!(show_cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Bookmark details
    ---

    Title: GitHub - dhth/bmm: get to your bookmarks in a flash
    URI  : https://github.com/dhth/bmm
    Tags : productivity,tools

    ----- stderr -----
    ");
}

#[test]
fn importing_resets_previously_saved_info_if_requested() {
    // GIVEN
    let uri = "https://github.com/dhth/omm";
    let fx = Fixture::new();
    let mut create_cmd = fx.cmd([
        "save",
        uri,
        "--title",
        "omm's github page",
        "--tags",
        "task-management,productivity",
    ]);
    assert_cmd_snapshot!(create_cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    let mut cmd = fx.cmd(["import", "tests/static/import/only-mandatory.json", "-r"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    imported 2 bookmarks

    ----- stderr -----
    ");

    let mut show_cmd = fx.cmd(["show", uri]);
    assert_cmd_snapshot!(show_cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Bookmark details
    ---

    Title: <NOT SET>
    URI  : https://github.com/dhth/omm
    Tags : <NOT SET>

    ----- stderr -----
    ");
}

//------------//
//  FAILURES  //
//------------//

#[test]
fn importing_from_an_invalid_json_file_fails() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd(["import", "tests/static/import/invalid.json"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r#"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Error: couldn't import bookmarks: couldn't parse JSON input: EOF while parsing a list at line 8 column 0

    Suggestion: ensure the file is valid JSON and looks like the following:

    [
      {
        "uri": "https://github.com/dhth/bmm",
        "title": null,
        "tags": "tools,bookmarks"
      },
      {
        "uri": "https://github.com/dhth/omm",
        "title": "on-my-mind: a keyboard-driven task manager for the command line",
        "tags": "tools,productivity"
      }
    ]
    "#);
}

#[test]
fn importing_from_a_json_file_fails_if_missing_uri() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd(["import", "tests/static/import/missing-uri.json"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r#"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Error: couldn't import bookmarks: couldn't parse JSON input: missing field `uri` at line 12 column 3

    Suggestion: ensure the file is valid JSON and looks like the following:

    [
      {
        "uri": "https://github.com/dhth/bmm",
        "title": null,
        "tags": "tools,bookmarks"
      },
      {
        "uri": "https://github.com/dhth/omm",
        "title": "on-my-mind: a keyboard-driven task manager for the command line",
        "tags": "tools,productivity"
      }
    ]
    "#);
}

#[test]
fn importing_from_a_json_file_fails_if_missing_uri_even_when_forced() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd([
        "import",
        "tests/static/import/missing-uri.json",
        "--ignore-attribute-errors",
    ]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r#"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Error: couldn't import bookmarks: couldn't parse JSON input: missing field `uri` at line 12 column 3

    Suggestion: ensure the file is valid JSON and looks like the following:

    [
      {
        "uri": "https://github.com/dhth/bmm",
        "title": null,
        "tags": "tools,bookmarks"
      },
      {
        "uri": "https://github.com/dhth/omm",
        "title": "on-my-mind: a keyboard-driven task manager for the command line",
        "tags": "tools,productivity"
      }
    ]
    "#);
}
