use std::fs::File;
use std::path::Path;
use utils::ledger::Writer;

pub struct Tracker {
    writer: Writer,
}

impl Tracker {
    pub fn new(filepath: impl AsRef<Path>) -> Self {
        let file = File::open(filepath).expect("Cannot open file");

        Self {
            writer: Writer::new(file),
        }
    }

    pub fn init(&mut self) {
        self.writer.write_version(100);
    }

    pub fn close(&mut self) {
        self.writer.flush()
    }
}
