use utils::executor::exec_cmd;
use utils::pipe_io::Record;

fn main() {
    let mut res = exec_cmd(
        "/Users/id/devel/ALT/backtest/backtest/target/release/examples/math_cmp",
        "/Users/id/devel/ALT/backtest/backtest",
    );

    while let Some(result) = res.next() {
        let record = result.unwrap();
        match record {
            Record::Version(version) => println!("Version: {}", version),
            Record::Exec(exe) => println!("Exec: {}", exe),
            Record::Image {
                name,
                header,
                slide,
            } => println!("Image: {} {} {}", name, header, slide),
            Record::PageInfo { size, pages } => println!("PageInfo: {} {}", size, pages),
            Record::Trace { ip, parent_idx } => println!("Trace: {} {}", ip, parent_idx),
            Record::Alloc {
                ptr,
                size,
                parent_idx,
            } => println!("Alloc: {} {} {}", ptr, size, parent_idx),
            Record::Free { ptr } => println!("Free: {}", ptr),
            Record::Duration(duration) => println!("Duration: {}", duration),
            Record::RSS(rss) => println!("RSS: {}", rss),
        }
    }
}
