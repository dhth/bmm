use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum DataDirError {
    #[error("XDG_DATA_HOME is not an absolute path")]
    XDGDataHomeNotAbsolute,
    #[error("couldn't get your data directory")]
    CouldntGetDataDir,
}

pub fn get_data_dir() -> Result<PathBuf, DataDirError> {
    #[cfg(target_os = "macos")]
    let data_dir = match std::env::var_os("XDG_DATA_HOME").map(PathBuf::from) {
        Some(p) => {
            if p.is_absolute() {
                Ok(p)
            } else {
                Err(DataDirError::XDGDataHomeNotAbsolute)
            }
        }
        None => match dirs::data_dir() {
            Some(p) => Ok(p),
            None => Err(DataDirError::CouldntGetDataDir),
        },
    }?;

    #[cfg(not(target_os = "macos"))]
    let data_dir = dirs::data_dir().ok_or(DataDirError::CouldntGetDataDir)?;

    Ok(data_dir)
}
