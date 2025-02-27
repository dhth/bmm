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

pub trait ExpectedSuccess {
    fn print_stderr_if_failed(&self, context: Option<&str>);
}

#[cfg(test)]
impl ExpectedSuccess for std::process::Output {
    fn print_stderr_if_failed(&self, context: Option<&str>) {
        if self.status.success() {
            return;
        }

        let stderr = std::str::from_utf8(&self.stderr).expect("invalid utf-8 stderr");
        match context {
            Some(c) => println!("{} stderr: \n{}", c, stderr),
            None => println!("stderr: \n{}", stderr),
        }
    }
}

pub trait ExpectedFailure {
    #[allow(unused)]
    fn print_stdout_if_succeeded(&self, context: Option<&str>);
}

#[cfg(test)]
impl ExpectedFailure for std::process::Output {
    fn print_stdout_if_succeeded(&self, context: Option<&str>) {
        if !self.status.success() {
            return;
        }

        let stdout = std::str::from_utf8(&self.stdout).expect("invalid utf-8 stdout");
        match context {
            Some(c) => println!("{} stdout: \n{}", c, stdout),
            None => println!("stdout: \n{}", stdout),
        }
    }
}
