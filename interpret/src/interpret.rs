use crate::output::Output;
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
}

pub struct Interpreter {
    output: Output,
    strings: IndexSet<String>,
    resolver: Resolver,
}

impl Interpreter {
    pub fn new(out_filepath: impl AsRef<Path>) -> io::Result<Self> {
        let file = OpenOptions::new().write(true).open(out_filepath)?;

        Ok(Self {
            output: Output::new(file),
            strings: IndexSet::new(),
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
                let id = self.write_string(&name)?;
                self.resolver.add_module(
                    id,
                    &name,
                    start_address as u64,
                    start_address as u64 + size as u64,
                )?;
            }
            Record::PageInfo { .. } => {}
            Record::Trace { .. } => {}
            Record::Alloc { .. } => {}
            Record::Free { .. } => {}
            Record::Duration(_) => {}
            Record::RSS(_) => {}
        }

        Ok(())
    }

    fn write_string(&mut self, value: &str) -> Result<usize, Error> {
        let (id, _) = self.strings.insert_full(value.to_string());
        self.output.write_string(value)?;

        Ok(id)
    }
}
