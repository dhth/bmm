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
    assert_cmd_snapshot!(cmd, @r"
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
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    nothing got deleted

    ----- stderr -----
    ");
}

#[test]
fn deleting_bookmarks_by_multiple_patterns_works_without_confirmation() {
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
    assert_cmd_snapshot!(cmd, @r"
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
fn confirming_exact_deletion_deletes_resolved_bookmarks() {
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

    let mut cmd = fx.cmd(["delete", URI_ONE, "https://nonexistent-uri.com"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd.pass_stdin("y\n"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Deleting 1 bookmark:
    https://github.com/dhth/bmm
    Enter "y" to confirm.
    deleted 1 bookmark

    ----- stderr -----
    "#);

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
fn declining_pattern_deletion_keeps_resolved_bookmarks() {
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
    Deleting 3 bookmarks:
    https://github.com/dhth/bmm
    https://github.com/dhth/hours
    https://github.com/dhth/omm
    Enter "y" to confirm.

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
    Deleting 1 bookmark:
    https://github.com/dhth/bmm
    Enter "y" to confirm.

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
