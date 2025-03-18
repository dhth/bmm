mod common;
use common::Fixture;
use predicates::str::contains;

//-------------//
//  SUCCESSES  //
//-------------//

#[test]
fn listing_tags_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut import_cmd = fixture.command();
    import_cmd.args(["import", "tests/static/import/valid.json"]);
    import_cmd.assert().success();

    let mut cmd = fixture.command();
    cmd.args(["tags", "list"]);

    // WHEN
    // THEN
    cmd.assert().success().stdout(contains(
        "
crates
productivity
rust
tools
"
        .trim(),
    ));
}

#[test]
fn listing_tags_with_stats_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut import_cmd = fixture.command();
    import_cmd.args(["import", "tests/static/import/valid.json"]);
    import_cmd.assert().success();

    let mut cmd = fixture.command();
    cmd.args(["tags", "list", "-s"]);

    // WHEN
    // THEN
    cmd.assert().success().stdout(contains(
        "
crates (1 bookmark)
productivity (2 bookmarks)
rust (1 bookmark)
tools (3 bookmarks)
"
        .trim(),
    ));
}

#[test]
fn deleting_tags_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut import_cmd = fixture.command();
    import_cmd.args(["import", "tests/static/import/valid.json"]);
    import_cmd.assert().success();

    let mut cmd = fixture.command();
    cmd.args(["tags", "delete", "--yes", "productivity", "crates"]);

    // WHEN
    // THEN
    cmd.assert().success();

    let mut list_cmd = fixture.command();
    list_cmd.args(["tags", "list"]);
    list_cmd.assert().success().stdout(contains(
        "
rust
tools
"
        .trim(),
    ));
}

#[test]
fn renaming_tags_works() {
    // GIVEN
    let fixture = Fixture::new();
    let mut import_cmd = fixture.command();
    import_cmd.args(["import", "tests/static/import/valid.json"]);
    import_cmd.assert().success();

    let mut cmd = fixture.command();
    cmd.args(["tags", "rename", "tools", "cli-tools"]);

    // WHEN
    // THEN
    cmd.assert().success();

    let mut list_cmd = fixture.command();
    list_cmd.args(["tags", "list"]);
    list_cmd.assert().success().stdout(contains(
        "
cli-tools
crates
productivity
rust
"
        .trim(),
    ));
}

//------------//
//  FAILURES  //
//------------//

#[test]
fn deleting_tags_fails_if_tag_doesnt_exist() {
    // GIVEN
    let fixture = Fixture::new();
    let mut import_cmd = fixture.command();
    import_cmd.args(["import", "tests/static/import/valid.json"]);
    import_cmd.assert().success();

    let mut cmd = fixture.command();
    cmd.args(["tags", "delete", "--yes", "productivity", "absent"]);

    // WHEN
    // THEN
    cmd.assert()
        .failure()
        .stderr(contains(r#"tags do not exist: ["absent"]"#));
}

#[test]
fn renaming_tags_fails_if_tag_doesnt_exist() {
    // GIVEN
    let fixture = Fixture::new();
    let mut import_cmd = fixture.command();
    import_cmd.args(["import", "tests/static/import/valid.json"]);
    import_cmd.assert().success();

    let mut cmd = fixture.command();
    cmd.args(["tags", "rename", "absent", "target"]);

    // WHEN
    // THEN
    cmd.assert().failure().stderr(contains("no such tag"));
}
