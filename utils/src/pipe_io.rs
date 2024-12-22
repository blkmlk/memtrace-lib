use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::num::ParseIntError;

const OPERATION_VERSION: u8 = b'v';
const OPERATION_EXEC: u8 = b'x';
const OPERATION_PAGE_INFO: u8 = b'X';
const OPERATION_TRACE: u8 = b't';
const OPERATION_ALLOC: u8 = b'+';
const OPERATION_FREE: u8 = b'-';
const OPERATION_DURATION: u8 = b'c';
const OPERATION_RSS: u8 = b'R';

pub struct PipeReader {
    reader: BufReader<File>,
    line: String,
}

#[derive(Debug)]
pub enum Error {
    InvalidFormat,
    IOError(io::Error),
}

impl From<ParseIntError> for Error {
    fn from(_: ParseIntError) -> Self {
        Self::InvalidFormat
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::IOError(value)
    }
}

#[derive(Debug)]
pub enum Record {
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

    pub fn read_record(&mut self) -> Option<Result<Record, Error>> {
        self.line.clear();
        if let Err(e) = self.reader.read_line(&mut self.line) {
            return Some(Err(e.into()));
        }

        if self.line.is_empty() {
            return None;
        }

        let record = Self::parse_record(&self.line);

        Some(record)
    }

    fn parse_record(line: &str) -> Result<Record, Error> {
        let mut split = line.split_whitespace();

        let cmd = split.next().ok_or(Error::InvalidFormat)?;

        let op = cmd
            .as_bytes()
            .first()
            .copied()
            .ok_or(Error::InvalidFormat)?;

        match op {
            OPERATION_VERSION => {
                let version = split.next().ok_or(Error::InvalidFormat)?.parse()?;
                Ok(Record::Version(version))
            }
            OPERATION_EXEC => {
                _ = split.next().ok_or(Error::InvalidFormat)?;
                let exec = split.next().ok_or(Error::InvalidFormat)?.to_string();
                Ok(Record::Exec(exec))
            }
            OPERATION_PAGE_INFO => {
                let size = usize::from_str_radix(split.next().ok_or(Error::InvalidFormat)?, 16)?;
                let pages = usize::from_str_radix(split.next().ok_or(Error::InvalidFormat)?, 16)?;
                Ok(Record::PageInfo { size, pages })
            }
            OPERATION_TRACE => {
                let ip = usize::from_str_radix(split.next().ok_or(Error::InvalidFormat)?, 16)?;
                let idx = usize::from_str_radix(split.next().ok_or(Error::InvalidFormat)?, 16)?;
                Ok(Record::Trace {
                    ip,
                    parent_idx: idx,
                })
            }
            OPERATION_ALLOC => {
                let size = usize::from_str_radix(split.next().ok_or(Error::InvalidFormat)?, 16)?;
                let idx = usize::from_str_radix(split.next().ok_or(Error::InvalidFormat)?, 16)?;
                let ptr = usize::from_str_radix(split.next().ok_or(Error::InvalidFormat)?, 16)?;
                Ok(Record::Alloc {
                    ptr,
                    size,
                    parent_idx: idx,
                })
            }
            OPERATION_FREE => {
                let ptr = usize::from_str_radix(split.next().ok_or(Error::InvalidFormat)?, 16)?;
                Ok(Record::Free { ptr })
            }
            OPERATION_DURATION => {
                let duration = u128::from_str_radix(split.next().ok_or(Error::InvalidFormat)?, 16)?;
                Ok(Record::Duration(duration))
            }
            OPERATION_RSS => {
                let size = usize::from_str_radix(split.next().ok_or(Error::InvalidFormat)?, 16)?;
                Ok(Record::RSS(size))
            }
            _ => Err(Error::InvalidFormat),
        }
    }
}

pub struct PipeWriter {
    writer: BufWriter<File>,
}

impl PipeWriter {
    pub fn new(file: File) -> Self {
        Self {
            writer: BufWriter::with_capacity(4096, file),
        }
    }

    pub fn write_version(&mut self, version: u16) {
        _ = self
            .writer
            .write_fmt(format_args!("{} {}\n", OPERATION_VERSION, version));
    }

    pub fn write_exec(&mut self, ex: &str) {
        _ = self.writer.write_fmt(format_args!(
            "{} {:x} {}\n",
            OPERATION_EXEC,
            ex.as_bytes().len(),
            ex
        ));
    }

    pub fn write_page_info(&mut self, page_size: usize, phys_pages: usize) {
        _ = self.writer.write_fmt(format_args!(
            "{} {:x} {:x}\n",
            OPERATION_PAGE_INFO, page_size, phys_pages
        ));
    }

    pub fn write_trace(&mut self, ip: usize, parent_idx: usize) {
        _ = self.writer.write_fmt(format_args!(
            "{} {:x} {:x}\n",
            OPERATION_TRACE, ip, parent_idx
        ));
    }

    pub fn write_alloc(&mut self, size: usize, parent_idx: usize, ptr: usize) {
        _ = self.writer.write_fmt(format_args!(
            "{} {:x} {:x} {:x}\n",
            OPERATION_ALLOC, size, parent_idx, ptr
        ));
    }

    pub fn write_free(&mut self, ptr: usize) {
        _ = self
            .writer
            .write_fmt(format_args!("{} {:x}\n", OPERATION_FREE, ptr));
    }

    pub fn write_duration(&mut self, duration: u128) {
        _ = self
            .writer
            .write_fmt(format_args!("{} {}\n", OPERATION_DURATION, duration));
    }

    pub fn write_rss(&mut self, rss: usize) {
        _ = self
            .writer
            .write_fmt(format_args!("{} {:x}\n", OPERATION_RSS, rss));
    }

    pub fn flush(&mut self) {
        _ = self.writer.flush();
    }
}

#[cfg(test)]
mod tests {
    use crate::pipe_io::PipeReader;
    use std::fs::OpenOptions;

    #[test]
    fn test_read_record() {
        let file = OpenOptions::new().read(true).open("/tmp/trace").unwrap();
        let mut reader = PipeReader::new(file);

        let record = reader.read_record().unwrap();
        println!("{:?}", record);

        let record = reader.read_record().unwrap();
        println!("{:?}", record);

        let record = reader.read_record().unwrap();
        println!("{:?}", record);
    }
}
