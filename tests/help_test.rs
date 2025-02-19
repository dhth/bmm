use assert_cmd::Command;

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
    let output = cmd.output().expect("command should've run");

    // THEN
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("invalid utf-8 stdout");
    assert!(stdout.contains("lets you get to your bookmarks in a flash"));
}
