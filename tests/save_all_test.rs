mod common;
use common::Fixture;
use predicates::boolean::PredicateBooleanExt;
use predicates::str::contains;

const URI_ONE: &str = "https://github.com/dhth/bmm";
const URI_TWO: &str = "https://github.com/dhth/omm";
const URI_THREE: &str = "https://github.com/dhth/hours";

//-------------//
//  SUCCESSES  //
//-------------//

#[test]
fn saving_multiple_bookmarks_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut cmd = fixture.command();
    cmd.args(["save-all", URI_ONE, URI_TWO, URI_THREE]);

    // WHEN
    // THEN
    cmd.assert().success().stdout(contains("saved 3 bookmarks"));
    let mut list_cmd = fixture.command();
    list_cmd.arg("list");
    list_cmd.assert().success().stdout(contains(
        format!(
            "
{}
{}
{}
",
            URI_ONE, URI_TWO, URI_THREE,
        )
        .trim(),
    ));
}

#[test]
fn saving_multiple_bookmarks_with_tags_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut cmd = fixture.command();
    cmd.args([
        "save-all",
        URI_ONE,
        URI_TWO,
        URI_THREE,
        "-t",
        "tools,productivity",
    ]);

    // WHEN
    // THEN
    cmd.assert().success().stdout(contains("saved 3 bookmarks"));

    let mut list_tags_cmd = fixture.command();
    list_tags_cmd.args(["tags", "list"]);
    list_tags_cmd.assert().success().stdout(contains(
        "
productivity
tools
"
        .trim(),
    ));
}

#[test]
fn saving_multiple_bookmarks_extends_previously_saved_tags() {
    // GIVEN
    let fixture = Fixture::new();
    let uri = "https://github.com/dhth/bmm";
    let mut create_cmd = fixture.command();
    create_cmd.args([
        "save",
        uri,
        "--title",
        "bmm's github page",
        "--tags",
        "productivity",
    ]);
    create_cmd.output().expect("command should've run");

    let mut cmd = fixture.command();
    cmd.args(["save-all", URI_ONE, URI_TWO, URI_THREE, "-t", "tools"]);

    // WHEN
    // THEN
    cmd.assert().success();

    let mut show_cmd = fixture.command();
    show_cmd.args(["show", URI_ONE]);
    show_cmd.assert().success().stdout(contains(
        format!(
            r#"
Bookmark details
---

Title: bmm's github page
URI  : {}
Tags : productivity,tools
"#,
            URI_ONE
        )
        .trim(),
    ));
}

#[test]
fn saving_multiple_bookmarks_resets_previously_saved_tags_if_requested() {
    // GIVEN
    let fixture = Fixture::new();
    let uri = "https://github.com/dhth/bmm";
    let mut create_cmd = fixture.command();
    create_cmd.args([
        "save",
        uri,
        "--title",
        "bmm's github page",
        "--tags",
        "productivity",
    ]);
    create_cmd.output().expect("command should've run");

    let mut cmd = fixture.command();
    cmd.args(["save-all", URI_ONE, URI_TWO, URI_THREE, "-t", "tools", "-r"]);

    // WHEN
    // THEN
    cmd.assert().success();

    let mut show_cmd = fixture.command();
    show_cmd.args(["show", URI_ONE]);
    show_cmd.assert().success().stdout(contains(
        format!(
            r#"
Bookmark details
---

Title: bmm's github page
URI  : {}
Tags : tools
"#,
            URI_ONE
        )
        .trim(),
    ));
}

#[test]
fn force_saving_multiple_bookmarks_with_invalid_tags_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut cmd = fixture.command();
    cmd.args([
        "save-all",
        URI_ONE,
        URI_TWO,
        URI_THREE,
        "--tags",
        "tag1,invalid tag, another    invalid\t\ttag ",
        "-i",
    ]);

    // WHEN
    // THEN
    cmd.assert().success().stdout(contains("saved 3 bookmarks"));

    let mut list_tags_cmd = fixture.command();
    list_tags_cmd.args(["tags", "list"]);
    list_tags_cmd.assert().success().stdout(contains(
        "
another-invalid-tag
invalid-tag
tag1
"
        .trim(),
    ));
}

//------------//
//  FAILURES  //
//------------//

#[test]
fn saving_multiple_bookmarks_fails_for_incorrect_uris() {
    // GIVEN
    let fixture = Fixture::new();
    let mut cmd = fixture.command();
    cmd.args([
        "save-all",
        "this is not a uri",
        URI_TWO,
        "https:/ this!!isn't-either.com",
    ]);

    // WHEN
    // THEN
    cmd.assert().failure().stderr(
        contains("- entry 1: couldn't parse provided uri value")
            .and(contains("- entry 3: couldn't parse provided uri value")),
    );
}
