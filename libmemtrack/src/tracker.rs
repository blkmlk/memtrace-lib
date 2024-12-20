use crate::trace::Trace;
use crate::trace_tree::TraceTree;
use libc::{
    mach_msg_type_number_t, mach_task_basic_info_data_t, mach_task_self, sysconf, task_info,
    task_info_t, time_value_t, MACH_TASK_BASIC_INFO, MACH_TASK_BASIC_INFO_COUNT,
};
use std::fs::OpenOptions;
use std::path::Path;
use std::time::Instant;
use utils::pipe_writer::PipeWriter;

pub struct Tracker {
    writer: PipeWriter,
    trace_tree: TraceTree,
    started_at: Instant,
}

impl Tracker {
    pub fn new(filepath: impl AsRef<Path>) -> Self {
        let file = OpenOptions::new().write(true).open(filepath).unwrap();

        Self {
            writer: PipeWriter::new(file),
            trace_tree: TraceTree::new(),
            started_at: Instant::now(),
        }
    }

    pub fn init(&mut self) {
        self.writer.write_version(100);

        let sys_info = get_sys_info();

        self.writer.write_exec(&sys_info.exec_path);
        self.writer
            .write_page_info(sys_info.page_size, sys_info.phys_pages);
    }

    pub fn on_malloc(&mut self, size: usize, ptr: usize) {
        let trace = Trace::new();

        let idx = self
            .trace_tree
            .index(trace, |ip, parent| self.writer.write_trace(ip, parent));

        self.writer.write_alloc(size, idx, ptr);
    }

    pub fn on_realloc(&mut self, size: usize, ptr_in: usize, ptr_out: usize) {
        let trace = Trace::new();

        if ptr_in > 0 {
            self.on_free(ptr_in);
        }

        let idx = self
            .trace_tree
            .index(trace, |ip, parent| self.writer.write_trace(ip, parent));

        self.writer.write_alloc(size, idx, ptr_out);
    }

    pub fn on_free(&mut self, ptr: usize) {
        self.writer.write_free(ptr)
    }

    pub fn on_exit(&mut self) {
        let elapsed = Instant::now().duration_since(self.started_at).as_millis();
        self.writer.write_duration(elapsed);

        let rss = self.get_rss();
        self.writer.write_rss(rss);

        self.writer.flush()
    }

    fn get_rss(&self) -> usize {
        let mut info = mach_task_basic_info_data_t {
            virtual_size: 0,
            resident_size: 0,
            resident_size_max: 0,
            user_time: time_value_t {
                seconds: 0,
                microseconds: 0,
            },
            system_time: time_value_t {
                seconds: 0,
                microseconds: 0,
            },
            policy: 0,
            suspend_count: 0,
        };

        let mut count = MACH_TASK_BASIC_INFO_COUNT;

        unsafe {
            task_info(
                mach_task_self(),
                MACH_TASK_BASIC_INFO,
                &raw mut info as task_info_t,
                &mut count as *mut mach_msg_type_number_t,
            );
        }

        info.resident_size as usize
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
