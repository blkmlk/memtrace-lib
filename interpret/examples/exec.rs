use utils::executor::exec_cmd;
use utils::pipe_io::Record;
use utils::resolver::Resolver;

fn main() {
    let mut res = exec_cmd(
        "/Users/id/devel/Rust/memtrack-rs/.local/math_cmp",
        "/Users/id/devel/ALT/backtest/backtest",
    );

    let mut resolver = Resolver::new();

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
                println!(
                    "Image: {} 0x{:x} 0x{:x}",
                    name,
                    start_address,
                    start_address * 2
                );
                _ = resolver.add_module(&name, start_address as u64, size as u64);
            }
            Record::PageInfo { size, pages } => println!("PageInfo: {} {}", size, pages),
            Record::Trace { ip, parent_idx } => {
                if let Some(location) = resolver.lookup(ip as u64) {
                    println!("Location: (0x{:x}) {:?}", ip, location);
                } else {
                    println!("Trace: ip: 0x{:x}, parent_idx: {}", ip, parent_idx);
                };
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
