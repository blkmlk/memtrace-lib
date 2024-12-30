use std::fs::File;
use std::io::{BufWriter, Write};
use std::time::Duration;

pub struct Output {
    buffer: BufWriter<File>,
}

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

impl Output {
    pub fn new(out: File) -> Self {
        Self {
            buffer: BufWriter::with_capacity(4096, out),
        }
    }

    pub fn write_version(&mut self, version: u16, file_version: u16) -> std::io::Result<()> {
        writeln!(self.buffer, "v {:x} {:x}", version, file_version)
    }

    pub fn write_exec(&mut self, command: &str) -> std::io::Result<()> {
        writeln!(self.buffer, "X {}", command)
    }

    pub fn write_string(&mut self, value: &str) -> std::io::Result<()> {
        let size = value.as_bytes().len();
        writeln!(self.buffer, "s {:x} {}", size, value)
    }

    pub fn write_instruction(
        &mut self,
        ip: u64,
        module_idx: usize,
        frames: &[Frame],
    ) -> std::io::Result<()> {
        write!(self.buffer, "i {:x} {:x}", ip, module_idx)?;
        for frame in frames {
            match frame {
                Frame::Single { function_idx } => write!(self.buffer, " {}", function_idx)?,
                Frame::Multiple {
                    function_idx,
                    file_idx,
                    line_number,
                } => write!(
                    self.buffer,
                    " {} {} {}",
                    function_idx, file_idx, line_number
                )?,
            }
        }

        writeln!(self.buffer)
    }

    pub fn write_trace(&mut self, ip_id: usize, parent_idx: u64) -> std::io::Result<()> {
        writeln!(self.buffer, "t {:x} {:x}", ip_id, parent_idx)
    }

    pub fn write_alloc(&mut self, size: u64, idx: usize) -> std::io::Result<()> {
        writeln!(self.buffer, "a {:x} {:x}", size, idx)
    }

    pub fn write_free(&mut self, idx: usize) -> std::io::Result<()> {
        writeln!(self.buffer, "- {:x}", idx)
    }

    pub fn write_duration(&mut self, duration: Duration) -> std::io::Result<()> {
        writeln!(self.buffer, "c {:x}", duration.as_millis())
    }

    pub fn write_rss(&mut self, rss: usize) -> std::io::Result<()> {
        writeln!(self.buffer, "R {:x}", rss)
    }
}
