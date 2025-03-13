use assert_cmd::Command;
use tempfile::{TempDir, tempdir};

pub struct Fixture {
    _temp_dir: TempDir,
    data_file_path: String,
}

#[cfg(test)]
#[allow(unused)]
impl Fixture {
    pub fn new() -> Self {
        let temp_dir = tempdir().expect("temporary directory should've been created");
        let data_file_path = temp_dir
            .path()
            .join("bmm.db")
            .to_str()
            .expect("temporary directory path is not valid utf-8")
            .to_string();
        let mut command =
            Command::cargo_bin(env!("CARGO_PKG_NAME")).expect("command should've been created");
        command.args(["--db-path", &data_file_path]);

        Self {
            _temp_dir: temp_dir,
            data_file_path,
        }
    }

    pub fn command(&self) -> Command {
        let mut command =
            Command::cargo_bin(env!("CARGO_PKG_NAME")).expect("command should've been created");
        command.args(["--db-path", &self.data_file_path]);
        command
    }
}
