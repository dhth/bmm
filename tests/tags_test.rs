mod common;

use common::Fixture;
use insta_cmd::assert_cmd_snapshot;

//-------------//
//  SUCCESSES  //
//-------------//

#[test]
fn listing_tags_works() {
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

    let mut cmd = fx.cmd(["tags", "list"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    crates
    productivity
    rust
    tools

    ----- stderr -----
    ");
}

#[test]
fn listing_tags_with_stats_works() {
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

    let mut cmd = fx.cmd(["tags", "list", "--show-stats"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    crates (1 bookmark)
    productivity (2 bookmarks)
    rust (1 bookmark)
    tools (3 bookmarks)

    ----- stderr -----
    ");
}

#[test]
fn deleting_tags_works() {
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

    // WHEN
    // THEN
    let mut cmd = fx.cmd(["tags", "delete", "--yes", "productivity", "crates"]);
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    deleted 2 tags

    ----- stderr -----
    ");

    let mut list_cmd = fx.cmd(["tags", "list"]);
    assert_cmd_snapshot!(list_cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    rust
    tools

    ----- stderr -----
    ");
}

#[test]
fn renaming_tags_works() {
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

    let mut cmd = fx.cmd(["tags", "rename", "tools", "cli-tools"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    let mut list_cmd = fx.cmd(["tags", "list"]);
    assert_cmd_snapshot!(list_cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    cli-tools
    crates
    productivity
    rust

    ----- stderr -----
    ");
}

//------------//
//  FAILURES  //
//------------//

#[test]
fn deleting_tags_fails_if_tag_doesnt_exist() {
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

    let mut cmd = fx.cmd(["tags", "delete", "--yes", "productivity", "absent"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r#"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Error: couldn't delete tag(s): tags do not exist: ["absent"]
    "#);
}

#[test]
fn renaming_tags_fails_if_tag_doesnt_exist() {
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

    let mut cmd = fx.cmd(["tags", "rename", "absent", "target"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Error: couldn't rename tag: no such tag
    ");
}
