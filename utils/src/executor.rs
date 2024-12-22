use crate::pipe_io;
use crate::pipe_io::{PipeReader, Record};
use nix::sys::stat::Mode;
use nix::unistd::mkfifo;
use std::fs::{remove_file, OpenOptions};
use std::io;
use std::path::Path;
use std::process::{Child, Command, ExitStatus};

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
    let path = Path::new(&pipe_file_path);
    mkfifo(path, Mode::S_IRUSR | Mode::S_IWUSR).unwrap();

    let pipe_file = OpenOptions::new().read(true).open(&pipe_file_path).unwrap();
    let reader = PipeReader::new(pipe_file);

    let envs = [
        ("PIPE_FILEPATH", pipe_file_path.as_str()),
        (
            "DYLD_INSERT_LIBRARIES",
            "/Users/id/devel/Rust/memtrack-rs/libmemtrack/target/release/liblibmemtrack.dylib",
        ),
    ];

    let mut cmd = Command::new(program);
    cmd.envs(envs);
    cmd.current_dir(cwd);

    ExecResult::new(cmd, reader, pipe_file_path)
}

pub struct ExecResult {
    reader: PipeReader,
    cmd: Command,
    pipe_filepath: String,
    child: Option<Child>,
}

impl ExecResult {
    pub fn new(cmd: Command, reader: PipeReader, pipe_filepath: String) -> Self {
        Self {
            reader,
            cmd,
            pipe_filepath,
            child: None,
        }
    }

    pub fn next(&mut self) -> Option<Result<Record, Error>> {
        match &mut self.child {
            None => {
                let child = match self.cmd.spawn() {
                    Ok(v) => v,
                    Err(e) => return Some(Err(e.into())),
                };

                self.child = Some(child);
            }
            Some(child) => match child.try_wait() {
                Ok(result) => {
                    if let Some(exit) = result {
                        return if exit.success() {
                            None
                        } else {
                            Some(Err(Error::CmdFailed(exit)))
                        };
                    }
                }
                Err(e) => return Some(Err(e.into())),
            },
        }

        let record = self.reader.read_record()?;

        Some(record.map_err(Error::from))
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
