use assert_cmd::Command;
use predicates::str::contains;

//-------------//
//  SUCCESSES  //
//-------------//

#[test]
fn shows_help() {
    // GIVEN
    let mut cmd =
        Command::cargo_bin(env!("CARGO_PKG_NAME")).expect("command should've been created");
    cmd.arg("--help");

    // WHEN
    // THEN
    cmd.assert()
        .success()
        .stdout(contains("lets you get to your bookmarks in a flash"));
}
