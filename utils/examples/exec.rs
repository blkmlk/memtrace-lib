use utils::executor::exec_cmd;
use utils::pipe_io::Record;

fn main() {
    let mut res = exec_cmd(
        "/Users/id/devel/Rust/memtrack-rs/.local/math_cmp",
        "/Users/id/devel/ALT/backtest/backtest",
    );

    let mut global_slide = 0;

    while let Some(result) = res.next() {
        let record = result.unwrap();
        match record {
            Record::Version(version) => println!("Version: {}", version),
            Record::Exec(exe) => println!("Exec: {}", exe),
            Record::Image {
                name,
                start_address,
                size,
            } => {
                println!("Image: {} {} {}", name, start_address, size)
            }
            Record::PageInfo { size, pages } => println!("PageInfo: {} {}", size, pages),
            Record::Trace { ip, parent_idx } => {
                let loader =
                    addr2line::Loader::new("/Users/id/devel/Rust/memtrack-rs/.local/math_cmp")
                        .unwrap();
                let location = loader
                    .find_location(ip as u64 - global_slide as u64)
                    .unwrap();
                if let Some(loc) = location {
                    println!("Location: {:?} {:?} {:?}", loc.file, loc.line, loc.column);
                }
            }
            Record::Alloc {
                ptr,
                size,
                parent_idx,
            } => {
                // println!("Alloc: {} {} {}", ptr, size, parent_idx)
            }
            Record::Free { ptr } => {
                // println!("Free: {}", ptr)
            }
            Record::Duration(duration) => println!("Duration: {}", duration),
            Record::RSS(rss) => println!("RSS: {}", rss),
        }
    }
}
