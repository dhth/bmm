mod common;
use common::{ExpectedSuccess, Fixture};
use pretty_assertions::assert_eq;

const URI_ONE: &str = "https://github.com/dhth/bmm";
const URI_TWO: &str = "https://github.com/dhth/omm";
const URI_THREE: &str = "https://github.com/dhth/hours";

//-------------//
//  SUCCESSES  //
//-------------//

#[test]
fn deleting_multiple_bookmarks_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut save_cmd = fixture.command();
    save_cmd.args(["save-all", URI_ONE, URI_TWO, URI_THREE]);
    let save_output = save_cmd.output().expect("save command should've run");
    assert!(save_output.status.success());

    let mut cmd = fixture.command();
    cmd.args(["delete", "--yes", URI_ONE, URI_TWO]);

    // WHEN
    let output = cmd.output().expect("command should've run");

    // THEN
    output.print_stderr_if_failed(None);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("invalid utf-8 stdout");
    assert_eq!(stdout.trim(), "deleted 2 bookmarks");
}

#[test]
fn deleting_shouldnt_fail_if_bookmarks_dont_exist() {
    // GIVEN
    let fixture = Fixture::new();
    let mut save_cmd = fixture.command();
    save_cmd.args(["save-all", URI_ONE, URI_TWO, URI_THREE]);
    let save_output = save_cmd.output().expect("save command should've run");
    assert!(save_output.status.success());

    let mut cmd = fixture.command();
    cmd.args(["delete", "--yes", "https://nonexistent-uri.com"]);

    // WHEN
    let output = cmd.output().expect("command should've run");

    // THEN
    output.print_stderr_if_failed(None);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("invalid utf-8 stdout");
    assert_eq!(stdout.trim(), "nothing got deleted");
}
