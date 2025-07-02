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
fn saving_multiple_bookmarks_works() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd(["save-all", URI_ONE, URI_TWO, URI_THREE]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    saved 3 bookmarks

    ----- stderr -----
    ");

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
fn saving_multiple_bookmarks_with_tags_works() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd([
        "save-all",
        URI_ONE,
        URI_TWO,
        URI_THREE,
        "--tags",
        "tools,productivity",
    ]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    saved 3 bookmarks

    ----- stderr -----
    ");

    let mut list_tags_cmd = fx.cmd(["tags", "list"]);
    assert_cmd_snapshot!(list_tags_cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    productivity
    tools

    ----- stderr -----
    ");
}

#[test]
fn saving_multiple_bookmarks_extends_previously_saved_tags() {
    // GIVEN
    let fx = Fixture::new();
    let mut create_cmd = fx.cmd([
        "save",
        URI_ONE,
        "--title",
        "bmm's github page",
        "--tags",
        "productivity",
    ]);
    assert_cmd_snapshot!(create_cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    let mut cmd = fx.cmd(["save-all", URI_ONE, URI_TWO, URI_THREE, "--tags", "tools"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    saved 3 bookmarks

    ----- stderr -----
    ");

    let mut show_cmd = fx.cmd(["show", URI_ONE]);
    assert_cmd_snapshot!(show_cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Bookmark details
    ---

    Title: bmm's github page
    URI  : https://github.com/dhth/bmm
    Tags : productivity,tools

    ----- stderr -----
    ");
}

#[test]
fn saving_multiple_bookmarks_resets_previously_saved_tags_if_requested() {
    // GIVEN
    let fx = Fixture::new();
    let mut create_cmd = fx.cmd([
        "save",
        URI_ONE,
        "--title",
        "bmm's github page",
        "--tags",
        "productivity",
    ]);
    assert_cmd_snapshot!(create_cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    let mut cmd = fx.cmd([
        "save-all",
        URI_ONE,
        URI_TWO,
        URI_THREE,
        "--tags",
        "tools",
        "--reset-missing-details",
    ]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    saved 3 bookmarks

    ----- stderr -----
    ");

    let mut show_cmd = fx.cmd(["show", URI_ONE]);
    assert_cmd_snapshot!(show_cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Bookmark details
    ---

    Title: bmm's github page
    URI  : https://github.com/dhth/bmm
    Tags : tools

    ----- stderr -----
    ");
}

#[test]
fn force_saving_multiple_bookmarks_with_invalid_tags_works() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd([
        "save-all",
        URI_ONE,
        URI_TWO,
        URI_THREE,
        "--tags",
        "tag1,invalid tag, another    invalid\t\ttag ",
        "--ignore-attribute-errors",
    ]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    saved 3 bookmarks

    ----- stderr -----
    ");

    let mut list_tags_cmd = fx.cmd(["tags", "list"]);
    assert_cmd_snapshot!(list_tags_cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    another-invalid-tag
    invalid-tag
    tag1

    ----- stderr -----
    ");
}

//------------//
//  FAILURES  //
//------------//

#[test]
fn saving_multiple_bookmarks_fails_for_incorrect_uris() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd([
        "save-all",
        "this is not a uri",
        URI_TWO,
        "https:/ this!!isn't-either.com",
    ]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Error: couldn't save bookmarks: there were 2 validation errors

    - entry 1: couldn't parse provided uri value: relative URL without a base
    - entry 3: couldn't parse provided uri value: invalid international domain name

    Possible workaround: running with -i/--ignore-attribute-errors might fix some attribute errors.
    If a title is too long, it'll will be trimmed, and some invalid tags might be transformed to fit bmm's requirements.
    ");
}
