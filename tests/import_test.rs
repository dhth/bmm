mod common;
use common::Fixture;
use predicates::str::contains;

//-------------//
//  SUCCESSES  //
//-------------//

#[test]
fn importing_from_an_html_file_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut cmd = fixture.command();
    cmd.args(["import", "tests/static/import/valid.html"]);

    // WHEN
    // THEN
    cmd.assert()
        .success()
        .stdout(contains("imported 4 bookmarks"));
}

#[test]
fn importing_from_an_invalid_html_file_doesnt_fail() {
    // GIVEN
    let fixture = Fixture::new();
    let mut cmd = fixture.command();
    cmd.args(["import", "tests/static/import/invalid.html"]);

    // WHEN
    // THEN
    cmd.assert()
        .success()
        .stdout(contains("imported 0 bookmarks"));
}

#[test]
fn force_importing_from_an_html_file_with_some_invalid_attrs_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut cmd = fixture.command();
    cmd.args([
        "import",
        "tests/static/import/valid-with-some-invalid-attributes.html",
        "-i",
    ]);

    // WHEN
    // THEN
    cmd.assert()
        .success()
        .stdout(contains("imported 4 bookmarks"));
}

#[test]
fn importing_from_a_valid_json_file_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut cmd = fixture.command();
    cmd.args(["import", "tests/static/import/valid.json"]);

    // WHEN
    // THEN
    cmd.assert()
        .success()
        .stdout(contains("imported 4 bookmarks"));
}

#[test]
fn importing_from_a_json_file_with_only_mandatory_details_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut cmd = fixture.command();
    cmd.args(["import", "tests/static/import/only-mandatory.json"]);

    // WHEN
    // THEN
    cmd.assert()
        .success()
        .stdout(contains("imported 2 bookmarks"));
}

#[test]
fn force_importing_from_a_json_file_with_some_invalid_attrs_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut cmd = fixture.command();
    cmd.args([
        "import",
        "tests/static/import/valid-with-some-invalid-attributes.json",
        "-i",
    ]);

    // WHEN
    // THEN
    cmd.assert()
        .success()
        .stdout(contains("imported 4 bookmarks"));
}

#[test]
fn importing_from_a_valid_txt_file_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut cmd = fixture.command();
    cmd.args(["import", "tests/static/import/valid.txt"]);

    // WHEN
    // THEN
    cmd.assert()
        .success()
        .stdout(contains("imported 4 bookmarks"));
}

#[test]
fn importing_extends_previously_saved_info() {
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
    cmd.args(["import", "tests/static/import/valid.json"]);

    // WHEN
    // THEN
    cmd.assert().success();

    let mut show_cmd = fixture.command();
    show_cmd.args(["show", uri]);
    show_cmd.assert().success().stdout(contains(
        format!(
            r#"
Bookmark details
---

Title: GitHub - dhth/bmm: get to your bookmarks in a flash
URI  : {}
Tags : productivity,tools
"#,
            uri
        )
        .trim(),
    ));
}

#[test]
fn importing_resets_previously_saved_info_if_requested() {
    // GIVEN
    let fixture = Fixture::new();
    let uri = "https://github.com/dhth/omm";
    let mut create_cmd = fixture.command();
    create_cmd.args([
        "save",
        uri,
        "--title",
        "omm's github page",
        "--tags",
        "task-management,productivity",
    ]);
    create_cmd.output().expect("command should've run");

    let mut cmd = fixture.command();
    cmd.args(["import", "tests/static/import/only-mandatory.json", "-r"]);

    // WHEN
    // THEN
    cmd.assert()
        .success()
        .stdout(contains("imported 2 bookmarks"));

    let mut show_cmd = fixture.command();
    show_cmd.args(["show", uri]);
    show_cmd.assert().success().stdout(contains(
        format!(
            r#"
Bookmark details
---

Title: <NOT SET>
URI  : {}
Tags : <NOT SET>
"#,
            uri
        )
        .trim(),
    ));
}

//------------//
//  FAILURES  //
//------------//

#[test]
fn importing_from_an_invalid_json_file_fails() {
    // GIVEN
    let fixture = Fixture::new();
    let mut cmd = fixture.command();
    cmd.args(["import", "tests/static/import/invalid.json"]);

    // WHEN
    // THEN
    cmd.assert()
        .failure()
        .stderr(contains("couldn't parse JSON input"));
}

#[test]
fn importing_from_a_json_file_fails_if_missing_uri() {
    // GIVEN
    let fixture = Fixture::new();
    let mut cmd = fixture.command();
    cmd.args(["import", "tests/static/import/missing-uri.json"]);

    // WHEN
    // THEN
    cmd.assert()
        .failure()
        .stderr(contains("missing field `uri`"));
}

#[test]
fn importing_from_a_json_file_fails_if_missing_uri_even_when_forced() {
    // GIVEN
    let fixture = Fixture::new();
    let mut cmd = fixture.command();
    cmd.args(["import", "tests/static/import/missing-uri.json", "-i"]);

    // WHEN
    // THEN
    cmd.assert()
        .failure()
        .stderr(contains("missing field `uri`"));
}
