use interpret::interpret::Interpreter;

fn main() {
    let mut interpret = Interpreter::new("/tmp/pipe.out").unwrap();

    interpret
        .execute(
            "/Users/id/devel/Rust/memtrack-rs/.local/simple",
            // "/Users/id/devel/Rust/memtrack-rs/.local/math_cmp",
            "/Users/id/devel/ALT/backtest/backtest",
        )
        .unwrap();
}
