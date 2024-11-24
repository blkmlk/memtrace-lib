use std::fs::File;
use std::io::{BufWriter, Write};

#[repr(u8)]
enum Operation {
    Version = b'v',
    SystemInfo = b'a',
}

pub struct Reader {}

pub struct Writer {
    writer: BufWriter<File>,
}

impl Writer {
    pub fn new(file: File) -> Self {
        Self {
            writer: BufWriter::with_capacity(4096, file),
        }
    }

    pub fn write_version(&mut self, version: usize) {
        _ = self.writer.write_fmt(format_args!("t {}", version));
    }

    pub fn flush(&mut self) {
        _ = self.writer.flush();
    }
}
