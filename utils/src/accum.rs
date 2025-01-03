use std::fs::OpenOptions;
use std::io;
use std::io::BufRead;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("Invalid format")]
    InvalidFormat,
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

#[derive(Debug)]
struct AccumulatedData {
    strings: Vec<String>,
    traces: Vec<Trace>,
    instruction_pointers: Vec<InstructionPointer>,
}

impl AccumulatedData {
    pub fn new() -> Self {
        Self {
            strings: Vec::with_capacity(4096),
            traces: Vec::with_capacity(65536),
            instruction_pointers: Vec::with_capacity(16384),
        }
    }
}

pub fn read_trace_file(file_path: impl AsRef<std::path::Path>) -> Result<AccumulatedData, Error> {
    let file = OpenOptions::new().read(true).open(file_path)?;

    let buff = io::BufReader::new(file);

    parse_file(buff)
}

fn parse_file(mut reader: impl BufRead) -> Result<AccumulatedData, Error> {
    let mut data = AccumulatedData::new();

    for line in reader.lines() {
        parse_line(&line?, &mut data)?
    }

    Ok(data)
}

fn parse_line(line: &str, data: &mut AccumulatedData) -> Result<(), Error> {
    let mut split = line.split_whitespace();

    let Some(first) = split.next() else {
        return Ok(());
    };

    match first {
        "s" => {
            split.next();
            data.strings
                .push(split.next().ok_or(Error::InvalidFormat)?.to_string());
        }
        "t" => {
            let ip_idx = u64::from_str_radix(split.next().ok_or(Error::InvalidFormat)?, 16)
                .map_err(|_| Error::InvalidFormat)?;
            let parent_idx = u64::from_str_radix(split.next().ok_or(Error::InvalidFormat)?, 16)
                .map_err(|_| Error::InvalidFormat)?;

            data.traces.push(Trace { ip_idx, parent_idx })
        }
        "i" => {
            let ip = u64::from_str_radix(split.next().ok_or(Error::InvalidFormat)?, 16)
                .map_err(|_| Error::InvalidFormat)?;
            let module_idx = usize::from_str_radix(split.next().ok_or(Error::InvalidFormat)?, 16)
                .map_err(|_| Error::InvalidFormat)?;

            let frame = parse_frame(&mut split)?.ok_or(Error::InvalidFormat)?;
            let mut inlined = Vec::new();

            while let Some(frame) = parse_frame(&mut split)? {
                inlined.push(frame);
            }

            data.instruction_pointers.push(InstructionPointer {
                ip,
                module_idx,
                frame,
                inlined,
            })
        }
        _ => {}
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_read_trace_file() {
        let file = "/tmp/pipe.out";
        let data = read_trace_file(file).unwrap();

        println!("{:?}", data);
    }
}
