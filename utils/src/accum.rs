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
struct AccumulatedData {
    strings: Vec<String>,
}

impl AccumulatedData {
    pub fn new() -> Self {
        Self {
            strings: Vec::with_capacity(4096),
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
        _ => {}
    }
    Ok(())
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
