use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::num::ParseIntError;

pub struct PipeReader {
    reader: BufReader<File>,
    line: String,
}

pub enum Error {
    InvalidFormat,
    IOError(io::Error),
}

impl From<ParseIntError> for Error {
    fn from(value: ParseIntError) -> Self {
        Self::InvalidFormat
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::IOError(value)
    }
}

enum Record {
    Version(u16),
    Exec(String),
    PageInfo {
        size: usize,
        pages: usize,
    },
    Trace {
        ip: usize,
        parent_idx: usize,
    },
    Alloc {
        ptr: usize,
        size: usize,
        parent_idx: usize,
    },
    Free {
        ptr: usize,
    },
    Duration(u128),
    RSS(usize),
}

impl PipeReader {
    pub fn new(file: File) -> Self {
        Self {
            reader: BufReader::with_capacity(4096, file),
            line: String::new(),
        }
    }

    pub fn read_record(&mut self) -> Result<Record, Error> {
        self.line.clear();
        self.reader.read_line(&mut self.line)?;

        let mut split = self.line.split_whitespace();

        let cmd = split.next().ok_or(Error::InvalidFormat)?;

        match cmd {
            "v" => {
                let version = u16::from_str_radix(split.next().ok_or(Error::InvalidFormat)?, 16)?;
                Ok(Record::Version(version))
            }
            "x" => {
                let exec = split.next().ok_or(Error::InvalidFormat)?.to_string();
                Ok(Record::Exec(exec))
            }
            "X" => {
                let size = usize::from_str_radix(split.next().ok_or(Error::InvalidFormat)?, 16)?;
                let pages = usize::from_str_radix(split.next().ok_or(Error::InvalidFormat)?, 16)?;
                Ok(Record::PageInfo { size, pages })
            }
            "t" => {
                let ip = usize::from_str_radix(split.next().ok_or(Error::InvalidFormat)?, 16)?;
                let idx = usize::from_str_radix(split.next().ok_or(Error::InvalidFormat)?, 16)?;
                Ok(Record::Trace {
                    ip,
                    parent_idx: idx,
                })
            }
            "+" => {
                let size = usize::from_str_radix(split.next().ok_or(Error::InvalidFormat)?, 16)?;
                let idx = usize::from_str_radix(split.next().ok_or(Error::InvalidFormat)?, 16)?;
                let ptr = usize::from_str_radix(split.next().ok_or(Error::InvalidFormat)?, 16)?;
                Ok(Record::Alloc {
                    ptr,
                    size,
                    parent_idx: idx,
                })
            }
            "-" => {
                let ptr = usize::from_str_radix(split.next().ok_or(Error::InvalidFormat)?, 16)?;
                Ok(Record::Free { ptr })
            }
            "c" => {
                let duration = u128::from_str_radix(split.next().ok_or(Error::InvalidFormat)?, 16)?;
                Ok(Record::Duration(duration))
            }
            "R" => {
                let size = usize::from_str_radix(split.next().ok_or(Error::InvalidFormat)?, 16)?;
                Ok(Record::RSS(size))
            }
            _ => Err(Error::InvalidFormat),
        }
    }
}
