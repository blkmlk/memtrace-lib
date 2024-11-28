use crate::tracer::Tracer;
use libc::sysconf;
use std::fs::OpenOptions;
use std::path::Path;
use utils::ledger::Writer;

pub struct Tracker {
    writer: Writer,
}

impl Tracker {
    pub fn new(filepath: impl AsRef<Path>) -> Self {
        let file = OpenOptions::new().write(true).open(filepath).unwrap();

        Self {
            writer: Writer::new(file),
        }
    }

    pub fn init(&mut self) {
        self.writer.write_version(100);

        let sys_info = get_sys_info();

        self.writer.write_exec(&sys_info.exec_path);
        self.writer
            .write_page_info(sys_info.page_size, sys_info.phys_pages);
    }

    pub fn on_malloc(&self) {
        let tracer = Tracer::new();
    }

    pub fn close(&mut self) {
        self.writer.flush()
    }
}

struct SysInfo {
    exec_path: String,
    page_size: usize,
    phys_pages: usize,
}

fn get_sys_info() -> SysInfo {
    let exec_path = process_path::get_executable_path()
        .unwrap()
        .to_string_lossy()
        .to_string();

    let (page_size, phys_pages) = unsafe {
        let page_size = sysconf(libc::_SC_PAGESIZE) as usize;
        let pages = sysconf(libc::_SC_PHYS_PAGES) as usize;

        (page_size, pages)
    };

    SysInfo {
        exec_path,
        page_size,
        phys_pages,
    }
}
