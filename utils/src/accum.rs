use indexmap::map::Entry;
use indexmap::IndexMap;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::BufRead;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("Invalid format")]
    InvalidFormat,
    #[error("Internal {0}")]
    Internal(String),
}

#[derive(Debug)]
struct Trace {
    ip_idx: u64,
    parent_idx: u64,
}

#[derive(Debug)]
struct InstructionPointer {
    ip: u64,
    module_idx: usize,
    frame: Frame,
    inlined: Vec<Frame>,
}

#[derive(Debug)]
pub enum Frame {
    Single {
        function_idx: usize,
    },
    Multiple {
        function_idx: usize,
        file_idx: usize,
        line_number: u16,
    },
}

#[derive(Debug, Default)]
struct AllocationData {
    allocations: u64,
    temporary: u64,
    leaked: u64,
    peak: u64,
    size: u64,
}

#[derive(Debug)]
struct Allocation {
    trace_idx: u64,
    data: AllocationData,
}

impl Allocation {
    pub fn new(trace_idx: u64, size: u64) -> Self {
        let mut data = AllocationData {
            size,
            ..Default::default()
        };

        Self { trace_idx, data }
    }
}

#[derive(Debug)]
struct AccumulatedData {
    strings: Vec<String>,
    traces: Vec<Trace>,
    instruction_pointers: Vec<InstructionPointer>,
    allocation_indices: IndexMap<u64, u64>,
    allocations: Vec<Allocation>,
    total: AllocationData,
    duration: Duration,
    peak_rss: u64,
    page_size: u64,
    pages: u64,
}

impl AccumulatedData {
    pub fn new() -> Self {
        Self {
            strings: Vec::with_capacity(4096),
            traces: Vec::with_capacity(65536),
            instruction_pointers: Vec::with_capacity(16384),
            allocation_indices: IndexMap::with_capacity(16384),
            allocations: Vec::with_capacity(16384),
            total: AllocationData::default(),
            duration: Duration::default(),
            peak_rss: 0,
            page_size: 0,
            pages: 0,
        }
    }
}

pub fn read_trace_file(file_path: impl AsRef<std::path::Path>) -> Result<AccumulatedData, Error> {
    let file = OpenOptions::new().read(true).open(file_path)?;

    Parser::new().parse_file(file)
}

pub struct Parser {
    data: AccumulatedData,
    last_ptr: u64,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            data: AccumulatedData::new(),
            last_ptr: 0,
        }
    }

    fn parse_file(mut self, file: File) -> Result<AccumulatedData, Error> {
        let reader = io::BufReader::new(file);

        for line in reader.lines() {
            self.parse_line(&line?)?
        }

        Ok(self.data)
    }

    fn parse_line(&mut self, line: &str) -> Result<(), Error> {
        let mut split = line.split_whitespace();

        let Some(first) = split.next() else {
            return Ok(());
        };

        match first {
            "s" => {
                split.next();
                self.data
                    .strings
                    .push(split.next().ok_or(Error::InvalidFormat)?.to_string());
            }
            "t" => {
                let ip_idx = u64::from_str_radix(split.next().ok_or(Error::InvalidFormat)?, 16)
                    .map_err(|_| Error::InvalidFormat)?;
                let parent_idx = u64::from_str_radix(split.next().ok_or(Error::InvalidFormat)?, 16)
                    .map_err(|_| Error::InvalidFormat)?;

                self.data.traces.push(Trace { ip_idx, parent_idx })
            }
            "i" => {
                let ip = u64::from_str_radix(split.next().ok_or(Error::InvalidFormat)?, 16)
                    .map_err(|_| Error::InvalidFormat)?;
                let module_idx =
                    usize::from_str_radix(split.next().ok_or(Error::InvalidFormat)?, 16)
                        .map_err(|_| Error::InvalidFormat)?;

                let frame = Self::parse_frame(&mut split)?.ok_or(Error::InvalidFormat)?;
                let mut inlined = Vec::new();

                while let Some(frame) = Self::parse_frame(&mut split)? {
                    inlined.push(frame);
                }

                self.data.instruction_pointers.push(InstructionPointer {
                    ip,
                    module_idx,
                    frame,
                    inlined,
                })
            }
            "a" => {
                let size = u64::from_str_radix(split.next().ok_or(Error::InvalidFormat)?, 16)
                    .map_err(|_| Error::InvalidFormat)?;
                let trace_idx = u64::from_str_radix(split.next().ok_or(Error::InvalidFormat)?, 16)
                    .map_err(|_| Error::InvalidFormat)?;

                self.add_allocation(trace_idx, size);
            }
            "+" => {
                let allocation_idx =
                    u64::from_str_radix(split.next().ok_or(Error::InvalidFormat)?, 16)
                        .map_err(|_| Error::InvalidFormat)?;

                let allocation: &mut Allocation = self
                    .data
                    .allocations
                    .get_mut(allocation_idx as usize)
                    .ok_or_else(|| Error::Internal("allocation not found".into()))?;

                self.last_ptr = allocation_idx;

                allocation.data.leaked += allocation.data.size;
                allocation.data.allocations += 1;

                self.data.total.leaked += allocation.data.size;
                self.data.total.allocations += 1;
                if self.data.total.leaked > self.data.total.peak {
                    self.data.total.peak = self.data.total.leaked;
                }
            }
            "-" => {
                let allocation_idx =
                    u64::from_str_radix(split.next().ok_or(Error::InvalidFormat)?, 16)
                        .map_err(|_| Error::InvalidFormat)?;

                let allocation = self
                    .data
                    .allocations
                    .get_mut(allocation_idx as usize)
                    .ok_or_else(|| Error::Internal("allocation not found".into()))?;

                self.data.total.leaked -= allocation.data.size;

                let temporary = self.last_ptr == allocation_idx;
                self.last_ptr = 0;

                if temporary {
                    self.data.total.temporary += 1;
                }

                allocation.data.leaked -= allocation.data.size;
                if temporary {
                    allocation.data.temporary += 1;
                }
            }
            "c" => {
                let timestamp = u64::from_str_radix(split.next().ok_or(Error::InvalidFormat)?, 16)
                    .map_err(|_| Error::InvalidFormat)?;
                self.data.duration = Duration::from_millis(timestamp);
            }
            "R" => {
                let rss = u64::from_str_radix(split.next().ok_or(Error::InvalidFormat)?, 16)
                    .map_err(|_| Error::InvalidFormat)?;
                if rss > self.data.peak_rss {
                    self.data.peak_rss = rss;
                }
            }
            "I" => {
                self.data.page_size =
                    u64::from_str_radix(split.next().ok_or(Error::InvalidFormat)?, 16)
                        .map_err(|_| Error::InvalidFormat)?;
                self.data.pages =
                    u64::from_str_radix(split.next().ok_or(Error::InvalidFormat)?, 16)
                        .map_err(|_| Error::InvalidFormat)?;
            }
            "#" => {
                // comment
            }
            _ => {}
        }
        Ok(())
    }

    fn add_allocation(&mut self, trace_idx: u64, size: u64) {
        match self.data.allocation_indices.entry(trace_idx) {
            Entry::Occupied(e) => {
                let idx = *e.get();
                self.data.allocations[idx as usize].data.size += size;
            }
            Entry::Vacant(e) => {
                let idx = self.data.allocations.len() as u64;
                e.insert(idx);
                let allocation = Allocation::new(trace_idx, size);
                self.data.allocations.push(allocation);
            }
        }
    }

    fn parse_frame<'a>(mut iter: impl Iterator<Item = &'a str>) -> Result<Option<Frame>, Error> {
        let Some(first) = iter.next() else {
            return Ok(None);
        };

        let function_idx = usize::from_str_radix(first, 16).map_err(|_| Error::InvalidFormat)?;

        let Some(file_val) = iter.next() else {
            return Ok(Some(Frame::Single { function_idx }));
        };

        let file_idx = usize::from_str_radix(file_val, 16).map_err(|_| Error::InvalidFormat)?;
        let line_number = iter
            .next()
            .unwrap_or_default()
            .parse()
            .map_err(|_| Error::InvalidFormat)?;

        Ok(Some(Frame::Multiple {
            function_idx,
            file_idx,
            line_number,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_read_trace_file() {
        let file = "/tmp/pipe.out";
        let data = read_trace_file(file).unwrap();

        println!("{:#?}", data);
    }
}
