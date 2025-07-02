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

    let mut cmd = fx.cmd(["delete", "--yes", "https://nonexistent-uri.com"]);

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
