mod common;

use common::Fixture;
use insta_cmd::assert_cmd_snapshot;

//-------------//
//  SUCCESSES  //
//-------------//

#[test]
fn shows_help() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd(["--help"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    bmm (stands for "bookmarks manager") lets you get to your bookmarks in a flash.

    It does so by storing your bookmarks locally, allowing you to quickly access,
    manage, and search through them using various commands.

    bmm has a traditional command line interface that can be used standalone or
    integrated with other tools, and a textual user interface for easy browsing.

    Usage: bmm [OPTIONS] <COMMAND>

    Commands:
      import    Import bookmarks from various sources
      delete    Delete bookmarks
      list      List bookmarks based on several kinds of queries
      save      Save/update a bookmark
      save-all  Save/update multiple bookmarks
      search    Search bookmarks by matching over terms
      show      Show bookmark details
      tags      Interact with tags
      tui       Open bmm's TUI
      help      Print this message or the help of the given subcommand(s)

    Options:
          --db-path <STRING>
              Override bmm's database location (default: <DATA_DIR>/bmm/bmm.db)

          --debug
              Output debug information without doing anything

      -h, --help
              Print help (see a summary with '-h')

    ----- stderr -----
    "#);
}
