use crate::output::{Frame, Output};
use crate::resolver::Resolver;
use crate::{executor, resolver};
use indexmap::IndexSet;
use std::fs::OpenOptions;
use std::io;
use std::path::Path;
use thiserror::Error;
use utils::pipe_io::Record;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Execution failed")]
    Exec(#[from] executor::Error),
    #[error("IO error")]
    Io(#[from] io::Error),
    #[error("Resolver")]
    Resolver(#[from] resolver::Error),
    #[error("Custom error: {}")]
    Custom(String),
}

pub struct Interpreter {
    output: Output,
    strings: IndexSet<String>,
    frames: IndexSet<u64>,
    resolver: Resolver,
}

impl Interpreter {
    pub fn new(out_filepath: impl AsRef<Path>) -> io::Result<Self> {
        let file = OpenOptions::new().write(true).open(out_filepath)?;

        Ok(Self {
            output: Output::new(file),
            strings: IndexSet::new(),
            frames: IndexSet::new(),
            resolver: Resolver::new(),
        })
    }

    pub fn execute(&mut self, cmd: &str, cwd: &str) -> Result<(), Error> {
        let mut exec = executor::exec_cmd(cmd, cwd);

        while let Some(item) = exec.next() {
            let record = item?;

            self.handle_record(record)?;
        }

        Ok(())
    }

    fn handle_record(&mut self, record: Record) -> Result<(), Error> {
        match record {
            Record::Version(version) => {
                self.output.write_version(version, 3)?;
            }
            Record::Exec(cmd) => {
                self.write_string(&cmd)?;
            }
            Record::Image {
                name,
                start_address,
                size,
            } => {
                let module_id = self.write_string(&name)?;
                self.resolver.add_module(
                    module_id,
                    &name,
                    start_address as u64,
                    start_address as u64 + size as u64,
                )?;
            }
            Record::PageInfo { .. } => {}
            Record::Trace { ip, parent_idx } => {
                let ip_id = self.add_frame(ip as u64)?;
                self.output.write_trace(ip_id, parent_idx as u64)?;
            }
            Record::Alloc {
                ptr,
                size,
                parent_idx,
            } => {
                self.output.write_alloc(size as u64, parent_idx)?;
            }
            Record::Free { .. } => {}
            Record::Duration(_) => {}
            Record::RSS(_) => {}
        }

        Ok(())
    }

    fn add_frame(&mut self, ip: u64) -> Result<usize, Error> {
        match self.frames.get_full(&ip) {
            None => {
                let (id, _) = self.frames.insert_full(ip);

                let Some(location) = self.resolver.lookup(ip) else {
                    return Err(Error::Custom("ip location not found".to_string()));
                };

                self.output.write_instruction(
                    ip,
                    location.module_id,
                    &[Frame::Single {
                        function_idx: self.write_string(&location.function_name)?,
                    }],
                )?;

                Ok(id)
            }
            Some((id, _)) => Ok(*id),
        }
    }

    fn write_string(&mut self, value: &str) -> Result<usize, Error> {
        match self.strings.get_full(value) {
            None => {
                let (id, _) = self.strings.insert_full(value.to_string());
                self.output.write_string(value)?;

                Ok(id)
            }
            Some((id, _)) => Ok(*id),
        }
    }
}
