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
fn shows_help() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd(["delete", "--help"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @"
    success: true
    exit_code: 0
    ----- stdout -----
    Delete bookmarks

    Usage: bmm delete [OPTIONS] [URI]...

    Arguments:
      [URI]...  URIs to delete

    Options:
      -p, --pattern           Treat provided values as URI patterns instead of exact URIs
      -y, --yes               Whether to skip confirmation
          --db-path <STRING>  Override bmm's database location (default: <DATA_DIR>/bmm/bmm.db)
          --debug             Output debug information without doing anything
      -h, --help              Print help

    Examples:
      Delete bookmarks by exact URIs:
        bmm delete https://example.com https://example.org

      Delete bookmarks matching URI patterns:
        bmm delete --pattern example.com github.com

      Delete without confirmation:
        bmm delete --yes https://example.com

    ----- stderr -----
    ");
}

#[test]
fn deleting_multiple_bookmarks_works() {
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

    let mut cmd = fx.cmd(["delete", "--yes", URI_ONE, URI_TWO]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @"
    success: true
    exit_code: 0
    ----- stdout -----
    deleted 2 bookmarks

    ----- stderr -----
    ");

    let mut list_cmd = fx.cmd(["list"]);
    assert_cmd_snapshot!(list_cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    https://github.com/dhth/hours

    ----- stderr -----
    ");
}

#[test]
fn deleting_shouldnt_fail_if_bookmarks_dont_exist() {
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

    let mut cmd = fx.cmd(["delete", "https://nonexistent-uri.com"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @"
    success: true
    exit_code: 0
    ----- stdout -----
    no bookmarks matched

    ----- stderr -----
    ");
}

#[test]
fn deleting_bookmarks_by_multiple_patterns_works() {
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

    let mut cmd = fx.cmd(["delete", "--yes", "--pattern", "bmm", "dhth/o"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @"
    success: true
    exit_code: 0
    ----- stdout -----
    deleted 2 bookmarks

    ----- stderr -----
    ");

    let mut list_cmd = fx.cmd(["list"]);
    assert_cmd_snapshot!(list_cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    https://github.com/dhth/hours

    ----- stderr -----
    ");
}

#[test]
fn deleting_by_pattern_treats_like_metacharacters_literally() {
    // GIVEN
    let fx = Fixture::new();
    let mut save_cmd = fx.cmd([
        "save-all",
        "https://example.com/under_score",
        "https://example.com/underXscore",
        "https://example.com/percent%20value",
        "https://example.com/percentZZ20value",
    ]);
    assert_cmd_snapshot!(save_cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    saved 4 bookmarks

    ----- stderr -----
    ");

    let mut cmd = fx.cmd(["delete", "--yes", "--pattern", "under_score", "%20"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @"
    success: true
    exit_code: 0
    ----- stdout -----
    deleted 2 bookmarks

    ----- stderr -----
    ");

    let mut list_cmd = fx.cmd(["list"]);
    assert_cmd_snapshot!(list_cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    https://example.com/underXscore
    https://example.com/percentZZ20value

    ----- stderr -----
    ");
}

#[test]
fn deleting_bookmarks_by_exact_match_works() {
    // GIVEN
    let fx = Fixture::new();
    let mut save_cmd = fx.cmd(["save-all", URI_ONE, URI_TWO]);
    assert_cmd_snapshot!(save_cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    saved 2 bookmarks

    ----- stderr -----
    ");

    let mut cmd = fx.cmd(["delete", "--yes", URI_ONE, "https://nonexistent-uri.com"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @"
    success: true
    exit_code: 0
    ----- stdout -----
    deleted 1 bookmark

    ----- stderr -----
    ");

    let mut list_cmd = fx.cmd(["list"]);
    assert_cmd_snapshot!(list_cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    https://github.com/dhth/omm

    ----- stderr -----
    ");
}

#[test]
fn deleting_bookmarks_asks_for_confirmation_by_default() {
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

    let mut cmd = fx.cmd(["delete", "--pattern", "bmm", "dhth/o"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd.pass_stdin("y\n"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Will delete 2 bookmarks:
      - https://github.com/dhth/bmm
      - https://github.com/dhth/omm

    Type "y" to confirm.
    deleted 2 bookmarks

    ----- stderr -----
    "#);

    let mut list_cmd = fx.cmd(["list"]);
    assert_cmd_snapshot!(list_cmd, @"
    success: true
    exit_code: 0
    ----- stdout -----
    https://github.com/dhth/hours

    ----- stderr -----
    ");
}

#[test]
fn declining_deletion_keeps_resolved_bookmarks() {
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

    let mut cmd = fx.cmd(["delete", "--pattern", "dhth"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd.pass_stdin("n\n"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Will delete 3 bookmarks:
      - https://github.com/dhth/bmm
      - https://github.com/dhth/hours
      - https://github.com/dhth/omm

    Type "y" to confirm.
    cancelled

    ----- stderr -----
    "#);

    let mut list_cmd = fx.cmd(["list"]);
    assert_cmd_snapshot!(list_cmd, @r"
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
fn input_other_than_y_does_not_confirm_deletion() {
    // GIVEN
    let fx = Fixture::new();
    let mut save_cmd = fx.cmd(["save-all", URI_ONE]);
    assert_cmd_snapshot!(save_cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    saved 1 bookmark

    ----- stderr -----
    ");

    let mut cmd = fx.cmd(["delete", URI_ONE]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd.pass_stdin("yes\n"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Will delete 1 bookmark:
      - https://github.com/dhth/bmm

    Type "y" to confirm.
    cancelled

    ----- stderr -----
    "#);

    let mut list_cmd = fx.cmd(["list"]);
    assert_cmd_snapshot!(list_cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    https://github.com/dhth/bmm

    ----- stderr -----
    ");
}
