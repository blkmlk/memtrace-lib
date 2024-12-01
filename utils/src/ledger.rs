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
        _ = self.writer.write_fmt(format_args!("v {}\n", version));
    }

    pub fn write_exec(&mut self, ex: &str) {
        _ = self
            .writer
            .write_fmt(format_args!("x {:x} {}\n", ex.as_bytes().len(), ex));
    }

    pub fn write_page_info(&mut self, page_size: usize, phys_pages: usize) {
        _ = self
            .writer
            .write_fmt(format_args!("x {:x} {:x}\n", page_size, phys_pages));
    }

    pub fn write_trace(&mut self, ip: u64, parent_idx: u64) {
        _ = self
            .writer
            .write_fmt(format_args!("t {:x} {:x}\n", ip, parent_idx));
    }

    pub fn write_alloc(&mut self, size: usize, parent_idx: u64, ptr: usize) {
        _ = self
            .writer
            .write_fmt(format_args!("+ {:x} {:x} {:x}\n", size, parent_idx, ptr));
    }

    pub fn write_free(&mut self, ptr: usize) {
        _ = self.writer.write_fmt(format_args!("- {:x}\n", ptr));
    }

    pub fn write_duration(&mut self, duration: u128) {
        _ = self.writer.write_fmt(format_args!("c {}\n", duration));
    }

    pub fn flush(&mut self) {
        _ = self.writer.flush();
    }
}
