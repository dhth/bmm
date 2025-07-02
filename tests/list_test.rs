mod common;

use common::Fixture;
use insta_cmd::assert_cmd_snapshot;

const URI_ONE: &str = "https://github.com/dhth/bmm";
const URI_TWO: &str = "https://github.com/dhth/omm";
const URI_THREE: &str = "https://github.com/dhth/hours";

//-------------//
//  SUCCESSES  //
//-------------//

#[test]
fn listing_bookmarks_works() {
    // GIVEN
    let fx = Fixture::new();
    let mut save_cmd = fx.cmd(["save-all", URI_ONE, URI_TWO, URI_THREE]);
    assert_cmd_snapshot!(save_cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    saved 3 bookmarks

    ----- stderr -----
    ");

    let mut cmd = fx.cmd(["list"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    https://github.com/dhth/bmm
    https://github.com/dhth/omm
    https://github.com/dhth/hours

    ----- stderr -----
    ");
}

#[test]
fn listing_bookmarks_with_queries_works() {
    // GIVEN
    let fx = Fixture::new();
    let mut import_cmd = fx.cmd(["import", "tests/static/import/valid.json"]);
    assert_cmd_snapshot!(import_cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    imported 4 bookmarks

    ----- stderr -----
    ");

    let mut cmd = fx.cmd([
        "list",
        "--uri",
        "github.com",
        "--title",
        "on-my-mind",
        "--tags",
        "tools,productivity",
    ]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    https://github.com/dhth/omm

    ----- stderr -----
    ");
}

#[test]
fn listing_bookmarks_fetches_all_data_for_each_bookmark() {
    // GIVEN
    let fx = Fixture::new();
    let mut import_cmd = fx.cmd(["import", "tests/static/import/valid.json"]);
    assert_cmd_snapshot!(import_cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    imported 4 bookmarks

    ----- stderr -----
    ");

    let mut cmd = fx.cmd(["list", "--tags", "tools", "--format", "json"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    [
      {
        "uri": "https://github.com/dhth/omm",
        "title": "GitHub - dhth/omm: on-my-mind: a keyboard-driven task manager for the command line",
        "tags": "productivity,tools"
      },
      {
        "uri": "https://github.com/dhth/hours",
        "title": "GitHub - dhth/hours: A no-frills time tracking toolkit for command line nerds",
        "tags": "productivity,tools"
      },
      {
        "uri": "https://github.com/dhth/bmm",
        "title": "GitHub - dhth/bmm: get to your bookmarks in a flash",
        "tags": "tools"
      }
    ]

    ----- stderr -----
    "#);
}

#[test]
fn listing_bookmarks_in_json_format_works() {
    // GIVEN
    let fx = Fixture::new();
    let mut save_cmd = fx.cmd(["save-all", URI_ONE, URI_TWO, URI_THREE]);
    assert_cmd_snapshot!(save_cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    saved 3 bookmarks

    ----- stderr -----
    ");

    let mut cmd = fx.cmd(["list", "--uri", "hours", "--format", "json"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    [
      {
        "uri": "https://github.com/dhth/hours",
        "title": null,
        "tags": null
      }
    ]

    ----- stderr -----
    "#);
}

#[test]
fn listing_bookmarks_in_delimited_format_works() {
    // GIVEN
    let fx = Fixture::new();
    let mut import_cmd = fx.cmd(["import", "tests/static/import/valid.json"]);
    assert_cmd_snapshot!(import_cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    imported 4 bookmarks

    ----- stderr -----
    ");

    let mut cmd = fx.cmd(["list", "--uri", "hours", "--format", "delimited"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    uri,title,tags
    https://github.com/dhth/hours,GitHub - dhth/hours: A no-frills time tracking toolkit for command line nerds,"productivity,tools"

    ----- stderr -----
    "#);
}
