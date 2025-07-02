mod common;

use common::Fixture;
use insta_cmd::assert_cmd_snapshot;

//-------------//
//  SUCCESSES  //
//-------------//

#[test]
fn searching_bookmarks_by_uri_works() {
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

    let mut cmd = fx.cmd(["search", "crates"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    https://crates.io/crates/sqlx

    ----- stderr -----
    ");
}

#[test]
fn searching_bookmarks_by_title_works() {
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

    let mut cmd = fx.cmd(["search", "keyboard-driven"]);

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
fn searching_bookmarks_by_tags_works() {
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

    let mut cmd = fx.cmd(["search", "tools"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    https://github.com/dhth/omm
    https://github.com/dhth/hours
    https://github.com/dhth/bmm

    ----- stderr -----
    ");
}

#[test]
fn search_shows_all_details_for_each_bookmark() {
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

    let mut cmd = fx.cmd(["search", "tools", "--format", "json"]);

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
fn searching_bookmarks_by_multiple_terms_works() {
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
        "search",
        "github",
        "tools",
        "productivity",
        "command",
        "time",
    ]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    https://github.com/dhth/hours

    ----- stderr -----
    ");
}

//------------//
//  FAILURES  //
//------------//

#[test]
fn searching_bookmarks_fails_if_search_terms_exceeds_limit() {
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

    let mut cmd = fx.cmd(["search"]);
    cmd.args((1..=11).map(|i| format!("term-{i}")).collect::<Vec<_>>());

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Error: couldn't search bookmarks: search query is invalid: too many terms (maximum allowed: 10)
    ");
}

#[test]
fn searching_bookmarks_fails_if_search_query_empty() {
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

    let mut cmd = fx.cmd(["search"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Error: couldn't search bookmarks: search query is invalid: query is empty
    ");
}
