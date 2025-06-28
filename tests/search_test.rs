mod common;
use common::Fixture;
use predicates::str::contains;

const URI_ONE: &str = "https://crates.io/crates/sqlx";
const URI_TWO: &str = "https://github.com/dhth/omm";
const URI_THREE: &str = "https://github.com/dhth/hours";
const URI_FOUR: &str = "https://github.com/dhth/bmm";

//-------------//
//  SUCCESSES  //
//-------------//

#[test]
fn searching_bookmarks_by_uri_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut import_cmd = fixture.command();
    import_cmd.args(["import", "tests/static/import/valid.json"]);
    let import_output = import_cmd.output().expect("import command should've run");
    assert!(import_output.status.success());

    let mut cmd = fixture.command();
    cmd.args(["search", "crates"]);

    // WHEN
    // THEN
    cmd.assert().success().stdout(contains(URI_ONE));
}

#[test]
fn searching_bookmarks_by_title_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut import_cmd = fixture.command();
    import_cmd.args(["import", "tests/static/import/valid.json"]);
    let import_output = import_cmd.output().expect("import command should've run");
    assert!(import_output.status.success());

    let mut cmd = fixture.command();
    cmd.args(["search", "keyboard-driven"]);

    // WHEN
    // THEN
    cmd.assert().success().stdout(contains(URI_TWO));
}

#[test]
fn searching_bookmarks_by_tags_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut import_cmd = fixture.command();
    import_cmd.args(["import", "tests/static/import/valid.json"]);
    let import_output = import_cmd.output().expect("import command should've run");
    assert!(import_output.status.success());

    let mut cmd = fixture.command();
    cmd.args(["search", "tools"]);

    // WHEN
    // THEN
    cmd.assert().success().stdout(contains(
        format!(
            "
{URI_TWO}
{URI_THREE}
{URI_FOUR}
"
        )
        .trim(),
    ));
}

#[test]
fn search_shows_all_details_for_each_bookmark() {
    // GIVEN
    let fixture = Fixture::new();
    let mut import_cmd = fixture.command();
    import_cmd.args(["import", "tests/static/import/valid.json"]);
    let import_output = import_cmd.output().expect("import command should've run");
    assert!(import_output.status.success());

    let mut cmd = fixture.command();
    cmd.args(["search", "tools", "-f", "json"]);

    // WHEN
    // THEN
    cmd.assert().success().stdout(contains(
        r#"
[
  {
    "uri": "https://github.com/dhth/omm",
    "title": "GitHub - dhth/omm: on-my-mind: a keyboard-driven task manager for the command line",
    "tags": "productivity,tools"
  },
  {
    "uri": "https://github.com/dhth/hours",
    "title": "GitHub - dhth/hours: A no-frills time tracking toolkit for command line nerds",
    "tags": "productivity,tools"
  },
  {
    "uri": "https://github.com/dhth/bmm",
    "title": "GitHub - dhth/bmm: get to your bookmarks in a flash",
    "tags": "tools"
  }
]
"#
        .trim(),
    ));
}

#[test]
fn searching_bookmarks_by_multiple_terms_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut import_cmd = fixture.command();
    import_cmd.args(["import", "tests/static/import/valid.json"]);
    let import_output = import_cmd.output().expect("import command should've run");
    assert!(import_output.status.success());

    let mut cmd = fixture.command();
    cmd.args([
        "search",
        "github",
        "tools",
        "productivity",
        "command",
        "time",
    ]);

    // WHEN
    // THEN
    cmd.assert().success().stdout(contains(URI_THREE));
}

//------------//
//  FAILURES  //
//------------//

#[test]
fn searching_bookmarks_fails_if_search_terms_exceeds_limit() {
    // GIVEN
    let fixture = Fixture::new();
    let mut import_cmd = fixture.command();
    import_cmd.args(["import", "tests/static/import/valid.json"]);
    let import_output = import_cmd.output().expect("import command should've run");
    assert!(import_output.status.success());

    let mut cmd = fixture.command();
    cmd.arg("search");
    cmd.args((1..=11).map(|i| format!("term-{i}")).collect::<Vec<_>>());

    // WHEN
    // THEN
    cmd.assert().failure().stderr(contains("too many terms"));
}

#[test]
fn searching_bookmarks_fails_if_search_query_empty() {
    // GIVEN
    let fixture = Fixture::new();
    let mut import_cmd = fixture.command();
    import_cmd.args(["import", "tests/static/import/valid.json"]);
    let import_output = import_cmd.output().expect("import command should've run");
    assert!(import_output.status.success());

    let mut cmd = fixture.command();
    cmd.args(["search"]);

    // WHEN
    // THEN
    cmd.assert().failure().stderr(contains("query is empty"));
}
