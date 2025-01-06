mod flamegraph;

use crate::flamegraph::build_flamegraph;
use anyhow::Context;
use clap::Parser;
use interpret::interpret::Interpreter;
use std::fs::remove_file;
use std::path::PathBuf;

#[derive(Parser)]
struct Opt {
    #[clap(short, long)]
    out_file: Option<PathBuf>,

    cmd: String,
    args: Vec<String>,
}

fn main() {
    let opt = Opt::parse();

    let pid = std::process::id();
    let trace_filepath = format!("/tmp/{}.trace", pid);

    let mut interpret = Interpreter::new(&trace_filepath).unwrap();

    let cwd = std::env::current_dir().unwrap();

    interpret.exec(opt.cmd, opt.args, cwd).unwrap();

    let data = utils::parser::Parser::new()
        .parse_file(&trace_filepath)
        .unwrap();

    let output_file = if let Some(file) = opt.out_file {
        PathBuf::from(file)
    } else {
        PathBuf::from("/tmp/flamegraph.svg")
    };

    build_flamegraph(data, &output_file).unwrap();

    println!("stored memory flamegraph to {}", output_file.display());

    remove_file(trace_filepath).unwrap();
}
