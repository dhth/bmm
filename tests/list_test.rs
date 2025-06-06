mod common;
use common::Fixture;
use predicates::str::contains;

const URI_ONE: &str = "https://github.com/dhth/bmm";
const URI_TWO: &str = "https://github.com/dhth/omm";
const URI_THREE: &str = "https://github.com/dhth/hours";

//-------------//
//  SUCCESSES  //
//-------------//

#[test]
fn listing_bookmarks_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut save_cmd = fixture.command();
    save_cmd.args(["save-all", URI_ONE, URI_TWO, URI_THREE]);
    let save_output = save_cmd.output().expect("save command should've run");
    assert!(save_output.status.success());

    let mut cmd = fixture.command();
    cmd.arg("list");

    // WHEN
    // THEN
    cmd.assert().success().stdout(contains(
        format!(
            "
{}
{}
{}
",
            URI_ONE, URI_TWO, URI_THREE
        )
        .trim(),
    ));
}

#[test]
fn listing_bookmarks_with_queries_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut import_cmd = fixture.command();
    import_cmd.args(["import", "tests/static/import/valid.json"]);
    let import_output = import_cmd.output().expect("import command should've run");
    assert!(import_output.status.success());

    let mut cmd = fixture.command();
    cmd.args([
        "list",
        "--uri",
        "github.com",
        "--title",
        "on-my-mind",
        "--tags",
        "tools,productivity",
    ]);

    // WHEN
    // THEN
    cmd.assert().success().stdout(contains(URI_TWO));
}

#[test]
fn listing_bookmarks_fetches_all_data_for_each_bookmark() {
    // GIVEN
    let fixture = Fixture::new();
    let mut import_cmd = fixture.command();
    import_cmd.args(["import", "tests/static/import/valid.json"]);
    let import_output = import_cmd.output().expect("import command should've run");
    assert!(import_output.status.success());

    let mut cmd = fixture.command();
    cmd.args(["list", "--tags", "tools", "-f", "json"]);

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
fn listing_bookmarks_in_json_format_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut save_cmd = fixture.command();
    save_cmd.args(["save-all", URI_ONE, URI_TWO, URI_THREE]);
    let save_output = save_cmd.output().expect("save command should've run");
    assert!(save_output.status.success());

    let mut cmd = fixture.command();
    cmd.args(["list", "--uri", "hours", "-f", "json"]);

    // WHEN
    // THEN
    cmd.assert().success().stdout(contains(
        r#"
[
  {
    "uri": "https://github.com/dhth/hours",
    "title": null,
    "tags": null
  }
]
"#
        .trim(),
    ));
}

#[test]
fn listing_bookmarks_in_delimited_format_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut import_cmd = fixture.command();
    import_cmd.args(["import", "tests/static/import/valid.json"]);
    let import_output = import_cmd.output().expect("import command should've run");
    assert!(import_output.status.success());

    let mut cmd = fixture.command();
    cmd.args(["list", "--uri", "hours", "-f", "delimited"]);

    // WHEN
    // THEN
    cmd.assert().success().stdout(contains(r#"
uri,title,tags
https://github.com/dhth/hours,GitHub - dhth/hours: A no-frills time tracking toolkit for command line nerds,"productivity,tools"
"#.trim()));
}
