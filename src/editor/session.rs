use crate::common::{ENV_VAR_BMM_EDITOR, ENV_VAR_EDITOR};
use std::path::PathBuf;
use std::process::{Command, ExitStatus};
use tempfile::tempdir;
use which::{Error as WhichError, which};

#[derive(Debug, PartialEq, Eq)]
pub(super) enum EditorOutcome {
    Unchanged,
    Changed(String),
}

#[derive(Debug, thiserror::Error)]
pub(super) enum EditorError {
    #[error("couldn't create temporary directory for editor file: {0}")]
    CreateTempDir(std::io::Error),
    #[error("couldn't write temporary editor file: {0}")]
    WriteTempFile(std::io::Error),
    #[error("editor environment variable \"{0}\" is invalid")]
    InvalidEditorEnvVar(String),
    #[error("no editor configured")]
    NoEditorConfigured,
    #[error("couldn't find editor executable \"{0}\": {2}")]
    CouldntFindEditorExe(String, String, WhichError),
    #[error("couldn't open text editor ({0}): {1}")]
    OpenTextEditor(PathBuf, std::io::Error),
    #[error("text editor exited unsuccessfully: {0}")]
    EditorFailed(ExitStatus),
    #[error("couldn't read temporary editor file: {0}")]
    ReadTempFile(std::io::Error),
}

pub(super) fn edit_text(initial_contents: &str) -> Result<EditorOutcome, EditorError> {
    let temp_dir = tempdir().map_err(EditorError::CreateTempDir)?;
    let file_path = temp_dir.path().join("bmm.toml");
    std::fs::write(&file_path, initial_contents).map_err(EditorError::WriteTempFile)?;

    let (editor_exe, env_var_used) = get_text_editor_exe()?;
    let editor_exe_path = which(&editor_exe)
        .map_err(|error| EditorError::CouldntFindEditorExe(editor_exe, env_var_used, error))?;

    let status = Command::new(&editor_exe_path)
        .arg(&file_path)
        .status()
        .map_err(|error| EditorError::OpenTextEditor(editor_exe_path, error))?;

    if !status.success() {
        return Err(EditorError::EditorFailed(status));
    }

    let edited_contents = std::fs::read_to_string(file_path).map_err(EditorError::ReadTempFile)?;
    if edited_contents == initial_contents {
        Ok(EditorOutcome::Unchanged)
    } else {
        Ok(EditorOutcome::Changed(edited_contents))
    }
}

fn get_text_editor_exe() -> Result<(String, String), EditorError> {
    fn get_env_var(key: &str) -> Result<String, EditorError> {
        match std::env::var(key) {
            Ok(value) => Ok(value),
            Err(std::env::VarError::NotPresent) => Ok(String::new()),
            Err(std::env::VarError::NotUnicode(_)) => {
                Err(EditorError::InvalidEditorEnvVar(key.to_string()))
            }
        }
    }

    let bmm_editor = get_env_var(ENV_VAR_BMM_EDITOR)?;
    if !bmm_editor.trim().is_empty() {
        return Ok((bmm_editor, ENV_VAR_BMM_EDITOR.to_string()));
    }

    let editor = get_env_var(ENV_VAR_EDITOR)?;
    if !editor.trim().is_empty() {
        return Ok((editor, ENV_VAR_EDITOR.to_string()));
    }

    Err(EditorError::NoEditorConfigured)
}
