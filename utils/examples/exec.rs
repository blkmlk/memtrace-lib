use utils::executor::exec_cmd;

fn main() {
    let mut res = exec_cmd(
        "/Users/id/devel/ALT/backtest/backtest/target/release/examples/math_cmp",
        "/Users/id/devel/ALT/backtest/backtest",
    );

    while let Some(result) = res.next() {
        println!("{:?}", result.unwrap());
    }
}
