mod common;

use common::Fixture;
use insta_cmd::assert_cmd_snapshot;
#[cfg(unix)]
use std::{os::unix::fs::PermissionsExt, path::PathBuf};
#[cfg(unix)]
use tempfile::{TempDir, tempdir};

const URI_ONE: &str = "https://github.com/dhth/bmm";

//-------------//
//  SUCCESSES  //
//-------------//

#[test]
fn saving_a_new_bookmark_works() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd(["save", URI_ONE]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    let mut list_cmd = fx.cmd(["list"]);
    assert_cmd_snapshot!(list_cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    https://github.com/dhth/bmm

    ----- stderr -----
    ");
}

#[test]
fn saving_a_new_bookmark_with_title_and_tags_works() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd([
        "save",
        URI_ONE,
        "--title",
        "bmm's github page",
        "--tags",
        "tools,productivity",
    ]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    let mut list_cmd = fx.cmd(["list", "--format", "delimited"]);
    assert_cmd_snapshot!(list_cmd, @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    uri,title,tags
    https://github.com/dhth/bmm,bmm's github page,"productivity,tools"

    ----- stderr -----
    "#);
}

#[test]
fn extending_tags_for_a_saved_bookmark_works() {
    // GIVEN
    let fx = Fixture::new();
    let mut create_cmd = fx.cmd([
        "save",
        URI_ONE,
        "--title",
        "bmm's github page",
        "--tags",
        "tools,productivity",
    ]);
    assert_cmd_snapshot!(create_cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    let mut cmd = fx.cmd(["save", URI_ONE, "--tags", "bookmarks"]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    let mut list_cmd = fx.cmd(["list", "--format", "delimited"]);
    assert_cmd_snapshot!(list_cmd, @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    uri,title,tags
    https://github.com/dhth/bmm,bmm's github page,"bookmarks,productivity,tools"

    ----- stderr -----
    "#);
}

#[test]
fn resetting_properties_on_bookmark_update_works() {
    // GIVEN
    let fx = Fixture::new();
    let mut create_cmd = fx.cmd([
        "save",
        URI_ONE,
        "--title",
        "bmm's github page",
        "--tags",
        "tools,productivity",
    ]);
    assert_cmd_snapshot!(create_cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    let mut cmd = fx.cmd([
        "save",
        URI_ONE,
        "--tags",
        "cli,bookmarks",
        "--reset-missing-details",
    ]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    let mut list_cmd = fx.cmd(["list", "--format", "delimited"]);
    assert_cmd_snapshot!(list_cmd, @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    uri,title,tags
    https://github.com/dhth/bmm,,"bookmarks,cli"

    ----- stderr -----
    "#);
}

#[test]
fn force_saving_a_new_bookmark_with_a_long_title_works() {
    // GIVEN
    let fx = Fixture::new();
    let title = "a".repeat(501);
    let mut cmd = fx.cmd([
        "save",
        URI_ONE,
        "--title",
        title.as_str(),
        "--ignore-attribute-errors",
    ]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    let mut show_cmd = fx.cmd(["show", URI_ONE]);
    assert_cmd_snapshot!(show_cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Bookmark details
    ---

    Title: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
    URI  : https://github.com/dhth/bmm
    Tags : <NOT SET>

    ----- stderr -----
    ");
}

#[test]
fn force_saving_a_new_bookmark_with_invalid_tags_works() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd([
        "save",
        URI_ONE,
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

    ----- stderr -----
    ");

    let mut show_cmd = fx.cmd(["show", URI_ONE]);
    assert_cmd_snapshot!(show_cmd, @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Bookmark details
    ---

    Title: <NOT SET>
    URI  : https://github.com/dhth/bmm
    Tags : another-invalid-tag,invalid-tag,tag1

    ----- stderr -----
    ");
}

#[cfg(unix)]
#[test]
fn providing_details_via_editor_should_save_a_new_bookmark() -> anyhow::Result<()> {
    // GIVEN
    let fx = Fixture::new();
    let (_temp_dir, editor) = editor_script(
        r#"printf '%s\n' 'title = "bmm github page"' 'tags = "tools,productivity"' > "$1""#,
    )?;
    let mut cmd = fx.cmd(["save", URI_ONE, "--editor"]);
    cmd.env("BMM_EDITOR", editor).env("EDITOR", "");

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    let mut list_cmd = fx.cmd(["list", "--format", "delimited"]);
    assert_cmd_snapshot!(list_cmd, @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    uri,title,tags
    https://github.com/dhth/bmm,bmm github page,"productivity,tools"

    ----- stderr -----
    "#);
    Ok(())
}

#[cfg(unix)]
#[test]
fn providing_details_via_editor_should_replace_existing_bookmark_details() -> anyhow::Result<()> {
    // GIVEN
    let fx = Fixture::new();
    let mut create_cmd = fx.cmd(["save", URI_ONE, "--title", "old title", "--tags", "old,tag"]);
    assert_cmd_snapshot!(create_cmd, @"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    let (_temp_dir, editor) =
        editor_script(r#"printf '%s\n' 'title = "new title"' 'tags = "new,replacement"' > "$1""#)?;
    let mut cmd = fx.cmd(["save", URI_ONE, "--editor"]);
    cmd.env("BMM_EDITOR", editor).env("EDITOR", "");

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    let mut list_cmd = fx.cmd(["list", "--format", "delimited"]);
    assert_cmd_snapshot!(list_cmd, @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    uri,title,tags
    https://github.com/dhth/bmm,new title,"new,replacement"

    ----- stderr -----
    "#);
    Ok(())
}

#[cfg(unix)]
#[test]
fn clearing_details_via_editor_should_remove_existing_bookmark_details() -> anyhow::Result<()> {
    // GIVEN
    let fx = Fixture::new();
    let mut create_cmd = fx.cmd(["save", URI_ONE, "--title", "old title", "--tags", "old,tag"]);
    assert_cmd_snapshot!(create_cmd, @"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    let (_temp_dir, editor) = editor_script(r#"printf '%s\n' 'title = ""' 'tags = ""' > "$1""#)?;
    let mut cmd = fx.cmd(["save", URI_ONE, "--editor"]);
    cmd.env("BMM_EDITOR", editor).env("EDITOR", "");

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    let mut list_cmd = fx.cmd(["list", "--format", "delimited"]);
    assert_cmd_snapshot!(list_cmd, @"
    success: true
    exit_code: 0
    ----- stdout -----
    uri,title,tags
    https://github.com/dhth/bmm,,

    ----- stderr -----
    ");
    Ok(())
}

#[cfg(unix)]
#[test]
fn closing_editor_without_changes_should_not_save_a_new_bookmark() -> anyhow::Result<()> {
    // GIVEN
    let fx = Fixture::new();
    let (_temp_dir, editor) = editor_script("exit 0")?;
    let mut cmd = fx.cmd(["save", URI_ONE, "--editor"]);
    cmd.env("BMM_EDITOR", editor).env("EDITOR", "");

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    let mut list_cmd = fx.cmd(["list"]);
    assert_cmd_snapshot!(list_cmd, @"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");
    Ok(())
}

#[cfg(unix)]
#[test]
fn closing_editor_without_changes_should_preserve_existing_bookmark_details() -> anyhow::Result<()>
{
    // GIVEN
    let fx = Fixture::new();
    let mut create_cmd = fx.cmd(["save", URI_ONE, "--title", "old title", "--tags", "old,tag"]);
    assert_cmd_snapshot!(create_cmd, @"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    let (_temp_dir, editor) = editor_script("exit 0")?;
    let mut cmd = fx.cmd(["save", URI_ONE, "--editor"]);
    cmd.env("BMM_EDITOR", editor).env("EDITOR", "");

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    let mut list_cmd = fx.cmd(["list", "--format", "delimited"]);
    assert_cmd_snapshot!(list_cmd, @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    uri,title,tags
    https://github.com/dhth/bmm,old title,"old,tag"

    ----- stderr -----
    "#);
    Ok(())
}

#[cfg(unix)]
#[test]
fn providing_malformed_toml_via_editor_should_fail_without_saving() -> anyhow::Result<()> {
    // GIVEN
    let fx = Fixture::new();
    let (_temp_dir, editor) = editor_script(r#"printf '%s\n' 'title = "unterminated' > "$1""#)?;
    let mut cmd = fx.cmd(["save", URI_ONE, "--editor"]);
    cmd.env("BMM_EDITOR", editor).env("EDITOR", "");

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r#"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Error: couldn't save bookmark: couldn't parse editor document: TOML parse error at line 1, column 22
      |
    1 | title = "unterminated
      |                      ^
    invalid basic string, expected `"`
    "#);

    let mut list_cmd = fx.cmd(["list"]);
    assert_cmd_snapshot!(list_cmd, @"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");
    Ok(())
}

#[cfg(unix)]
#[test]
fn providing_unknown_toml_field_via_editor_should_fail_without_saving() -> anyhow::Result<()> {
    // GIVEN
    let fx = Fixture::new();
    let (_temp_dir, editor) = editor_script(
        r#"printf '%s\n' 'uri = "https://different.example"' 'title = "title"' 'tags = ""' > "$1""#,
    )?;
    let mut cmd = fx.cmd(["save", URI_ONE, "--editor"]);
    cmd.env("BMM_EDITOR", editor).env("EDITOR", "");

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r#"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Error: couldn't save bookmark: couldn't parse editor document: TOML parse error at line 1, column 1
      |
    1 | uri = "https://different.example"
      | ^^^
    unknown field `uri`, expected `title` or `tags`
    "#);

    let mut list_cmd = fx.cmd(["list"]);
    assert_cmd_snapshot!(list_cmd, @"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");
    Ok(())
}

#[cfg(unix)]
#[test]
fn providing_invalid_details_via_editor_should_fail_without_saving() -> anyhow::Result<()> {
    // GIVEN
    let fx = Fixture::new();
    let (_temp_dir, editor) =
        editor_script(r#"printf '%s\n' 'title = "title"' 'tags = "invalid tag"' > "$1""#)?;
    let mut cmd = fx.cmd(["save", URI_ONE, "--editor"]);
    cmd.env("BMM_EDITOR", editor).env("EDITOR", "");

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r#"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Error: couldn't save bookmark: tags ["invalid tag"] are invalid (valid regex: ^[a-zA-Z0-9_-]{1,30}$)

    Possible workaround: running with -i/--ignore-attribute-errors might fix some attribute errors.
    If a title is too long, it'll will be trimmed, and some invalid tags might be transformed to fit bmm's requirements.
    "#);

    let mut list_cmd = fx.cmd(["list"]);
    assert_cmd_snapshot!(list_cmd, @"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");
    Ok(())
}

#[cfg(unix)]
#[test]
fn editor_exiting_unsuccessfully_should_fail_without_saving() -> anyhow::Result<()> {
    // GIVEN
    let fx = Fixture::new();
    let (_temp_dir, editor) = editor_script("exit 23")?;
    let mut cmd = fx.cmd(["save", URI_ONE, "--editor"]);
    cmd.env("BMM_EDITOR", editor).env("EDITOR", "");

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Error: couldn't save bookmark: text editor exited unsuccessfully: exit status: 23
    ");

    let mut list_cmd = fx.cmd(["list"]);
    assert_cmd_snapshot!(list_cmd, @"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");
    Ok(())
}

#[cfg(unix)]
#[test]
fn empty_bmm_editor_should_fall_back_to_editor() -> anyhow::Result<()> {
    // GIVEN
    let fx = Fixture::new();
    let (_temp_dir, editor) =
        editor_script(r#"printf '%s\n' 'title = "fallback editor"' 'tags = ""' > "$1""#)?;
    let mut cmd = fx.cmd(["save", URI_ONE, "--editor"]);
    cmd.env("BMM_EDITOR", "").env("EDITOR", editor);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    let mut show_cmd = fx.cmd(["show", URI_ONE]);
    assert_cmd_snapshot!(show_cmd, @"
    success: true
    exit_code: 0
    ----- stdout -----
    Bookmark details
    ---

    Title: fallback editor
    URI  : https://github.com/dhth/bmm
    Tags : <NOT SET>

    ----- stderr -----
    ");
    Ok(())
}

#[cfg(unix)]
#[test]
fn configured_bmm_editor_should_take_precedence_over_editor() -> anyhow::Result<()> {
    // GIVEN
    let fx = Fixture::new();
    let (_bmm_temp_dir, bmm_editor) =
        editor_script(r#"printf '%s\n' 'title = "preferred editor"' 'tags = ""' > "$1""#)?;
    let (_editor_temp_dir, editor) = editor_script("exit 23")?;
    let mut cmd = fx.cmd(["save", URI_ONE, "--editor"]);
    cmd.env("BMM_EDITOR", bmm_editor).env("EDITOR", editor);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    ");

    let mut show_cmd = fx.cmd(["show", URI_ONE]);
    assert_cmd_snapshot!(show_cmd, @"
    success: true
    exit_code: 0
    ----- stdout -----
    Bookmark details
    ---

    Title: preferred editor
    URI  : https://github.com/dhth/bmm
    Tags : <NOT SET>

    ----- stderr -----
    ");
    Ok(())
}

//------------//
//  FAILURES  //
//------------//

#[test]
fn saving_a_new_bookmark_with_a_long_title_fails() {
    // GIVEN
    let fx = Fixture::new();
    let title = "a".repeat(501);
    let mut cmd = fx.cmd(["save", URI_ONE, "--title", title.as_str()]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Error: couldn't save bookmark: title is too long: 501 (max: 500)

    Possible workaround: running with -i/--ignore-attribute-errors might fix some attribute errors.
    If a title is too long, it'll will be trimmed, and some invalid tags might be transformed to fit bmm's requirements.
    ");
}

#[test]
fn saving_a_new_bookmark_with_an_invalid_tag_fails() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd([
        "save",
        URI_ONE,
        "--tags",
        "tag1,invalid tag, another    invalid\t\ttag ",
    ]);

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r#"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Error: couldn't save bookmark: tags ["invalid tag", " another    invalid\t\ttag "] are invalid (valid regex: ^[a-zA-Z0-9_-]{1,30}$)

    Possible workaround: running with -i/--ignore-attribute-errors might fix some attribute errors.
    If a title is too long, it'll will be trimmed, and some invalid tags might be transformed to fit bmm's requirements.
    "#);
}

#[test]
fn saving_a_new_bookmark_with_no_text_editor_configured_fails() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd(["save", URI_ONE, "--editor"]);
    cmd.env("BMM_EDITOR", "");
    cmd.env("EDITOR", "");

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Error: couldn't save bookmark: no editor configured

    Suggestion: set the environment variables BMM_EDITOR or EDITOR to use this feature
    ");
}

#[test]
fn saving_a_new_bookmark_with_incorrect_text_editor_configured_fails() {
    // GIVEN
    let fx = Fixture::new();
    let mut cmd = fx.cmd(["save", URI_ONE, "--editor"]);
    cmd.env("BMM_EDITOR", "non-existent-4d56150d");
    cmd.env("EDITOR", "non-existent-4d56150d");

    // WHEN
    // THEN
    assert_cmd_snapshot!(cmd, @r#"
    success: false
    exit_code: 1
    ----- stdout -----

    ----- stderr -----
    Error: couldn't save bookmark: couldn't find editor executable "non-existent-4d56150d": cannot find binary path

    Context: bmm used the environment variable BMM_EDITOR to determine your text editor.
    Check if "non-existent-4d56150d" actually points to your text editor's executable.
    "#);
}

#[cfg(unix)]
fn editor_script(contents: &str) -> Result<(TempDir, PathBuf), std::io::Error> {
    let temp_dir = tempdir()?;
    let script_path = temp_dir.path().join("editor.sh");
    let script = format!("#!/bin/sh\n{contents}\n");
    std::fs::write(&script_path, script)?;

    let mut permissions = std::fs::metadata(&script_path)?.permissions();
    permissions.set_mode(0o700);
    std::fs::set_permissions(&script_path, permissions)?;

    Ok((temp_dir, script_path))
}
