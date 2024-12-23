use crate::pipe_io;
use crate::pipe_io::{PipeReader, Record};
use nix::sys::stat::Mode;
use nix::unistd::mkfifo;
use std::fs::{remove_file, OpenOptions};
use std::io;
use std::process::{Child, Command, ExitStatus, Stdio};

#[derive(Debug)]
pub enum Error {
    CmdFailed(ExitStatus),
    CmdError(io::Error),
    PipeError(pipe_io::Error),
}

impl From<pipe_io::Error> for Error {
    fn from(value: pipe_io::Error) -> Self {
        Error::PipeError(value)
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::CmdError(value)
    }
}

pub fn exec_cmd(program: &str, cwd: &str) -> ExecResult {
    let pid = std::process::id();
    let pipe_file_path = format!("/tmp/{}.pipe", pid);

    mkfifo(pipe_file_path.as_str(), Mode::S_IRUSR | Mode::S_IWUSR).unwrap();

    let envs = [
        ("PIPE_FILEPATH", pipe_file_path.as_str()),
        (
            "DYLD_INSERT_LIBRARIES",
            "/Users/id/devel/Rust/memtrack-rs/libmemtrack/target/release/liblibmemtrack.dylib",
        ),
    ];

    let mut cmd = Command::new(program);
    cmd.stdout(Stdio::null());
    cmd.envs(envs);
    cmd.current_dir(cwd);

    let child = cmd.spawn().unwrap();

    ExecResult::new(child, pipe_file_path)
}

pub struct ExecResult {
    child: Child,
    pipe_filepath: String,
    reader: Option<PipeReader>,
}

impl ExecResult {
    pub fn new(child: Child, pipe_filepath: String) -> Self {
        Self {
            child,
            pipe_filepath,
            reader: None,
        }
    }

    pub fn next(&mut self) -> Option<Result<Record, Error>> {
        let record = match &mut self.reader {
            None => {
                let pipe_file = OpenOptions::new()
                    .read(true)
                    .open(&self.pipe_filepath)
                    .unwrap();

                let mut reader = PipeReader::new(pipe_file);

                let record = reader.read_record()?.map_err(Error::from);

                self.reader = Some(reader);

                record
            }
            Some(reader) => match self.child.try_wait() {
                Ok(result) => {
                    if let Some(exit) = result {
                        return if exit.success() {
                            None
                        } else {
                            Some(Err(Error::CmdFailed(exit)))
                        };
                    }
                    reader.read_record()?.map_err(Error::from)
                }
                Err(e) => return Some(Err(e.into())),
            },
        };

        Some(record)
    }
}

impl Drop for ExecResult {
    fn drop(&mut self) {
        _ = remove_file(&self.pipe_filepath);
    }
}

#[cfg(test)]
mod tests {
    use crate::executor::exec_cmd;

    #[test]
    fn test_exec() {
        let mut res = exec_cmd(
            "/Users/id/devel/ALT/backtest/backtest/target/release/examples/math_cmp",
            "/Users/id/devel/ALT/backtest/backtest",
        );

        while let Some(result) = res.next() {
            println!("{:?}", result.unwrap());
        }
    }
}
