use anyhow::Context;
use inferno::flamegraph::Options;
use interpret::interpret::Interpreter;
use std::fs::OpenOptions;
use std::path::Path;
use utils::parser::{AccumulatedData, Frame, InstructionPointer};

fn main() {
    let mut interpret = Interpreter::new("/tmp/pipe.out").unwrap();

    interpret
        .execute(
            // "/Users/id/devel/Rust/memtrack-rs/.local/simple",
            "/Users/id/devel/Rust/memtrack-rs/.local/math_cmp",
            "/Users/id/devel/ALT/backtest/backtest",
        )
        .unwrap();

    let data = utils::parser::Parser::new()
        .parse_file("/tmp/pipe.out")
        .unwrap();

    build_flamegraph(data, "/tmp/flamegraph.svg").unwrap()
}

struct Line {
    frames: Vec<String>,
    value: u64,
}

impl Line {
    pub fn new(value: u64) -> Self {
        Self {
            frames: Vec::new(),
            value,
        }
    }
}

impl ToString for Line {
    fn to_string(&self) -> String {
        let frames = self
            .frames
            .iter()
            .rev()
            .map(|v| v.to_string())
            .collect::<Vec<String>>()
            .join(";");

        format!("{} {}", frames, self.value)
    }
}

fn build_flamegraph(data: AccumulatedData, output_file: impl AsRef<Path>) -> anyhow::Result<()> {
    let mut lines = Vec::new();
    for info in &data.allocation_infos {
        let allocation = &data.allocations[info.allocation_idx as usize];
        let mut trace_idx = allocation.trace_idx;

        let mut line = Line::new(info.size);
        while trace_idx != 0 {
            let trace = &data.traces[trace_idx as usize - 1];
            let ip_info = &data.instruction_pointers[trace.ip_idx as usize - 1];

            let frames = get_frames_from_ip_info(&data, ip_info);
            line.frames.extend(frames);

            trace_idx = trace.parent_idx;
        }

        lines.push(line.to_string());
    }

    let mut opts = Options::default();
    opts.count_name = "bytes".to_string();

    let output = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(output_file)
        .context("failed to open new file")?;

    inferno::flamegraph::from_lines(&mut opts, lines.iter().map(|v| v.as_str()), output)
        .context("failed to create a flame graph")?;

    Ok(())
}

fn get_frames_from_ip_info(data: &AccumulatedData, ip_info: &InstructionPointer) -> Vec<String> {
    [&ip_info.frame]
        .into_iter()
        .chain(ip_info.inlined.iter())
        .map(|frame| {
            let function_idx = match frame {
                Frame::Single { function_idx } => function_idx,
                Frame::Multiple { function_idx, .. } => function_idx,
            };
            data.strings[function_idx - 1].clone()
        })
        .collect()
}
