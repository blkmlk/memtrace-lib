use crate::arch;
use crate::trace::Trace;
use crate::trace_tree::TraceTree;
use memtrace_utils::pipe_io::PipeWriter;
use std::fs::OpenOptions;
use std::path::Path;
use std::time::Instant;

pub struct Tracker {
    writer: PipeWriter,
    trace_tree: TraceTree,
    started_at: Instant,
    slide: usize,
}

impl Tracker {
    pub fn new(filepath: impl AsRef<Path>) -> Self {
        let file = OpenOptions::new().write(true).open(filepath).unwrap();

        Self {
            writer: PipeWriter::new(file),
            trace_tree: TraceTree::new(),
            started_at: Instant::now(),
            slide: 0,
        }
    }

    pub fn init(&mut self) {
        self.writer.write_version(100);

        let sys_info = arch::get_sys_info();

        self.writer.write_exec(&sys_info.exec_path);
        self.writer
            .write_page_info(sys_info.page_size, sys_info.phys_pages);

        self.slide = arch::get_image_slide();

        self.write_images();
    }

    pub fn on_malloc(&mut self, size: usize, ptr: usize) {
        let trace = Trace::new();

        let idx = self.trace_tree.index(trace, |ip, parent| {
            let ip = ip - self.slide;
            self.writer.write_trace(ip, parent)
        });

        self.writer.write_alloc(size, idx, ptr);
    }

    pub fn on_realloc(&mut self, size: usize, ptr_in: usize, ptr_out: usize) {
        let trace = Trace::new();

        if ptr_in > 0 {
            self.on_free(ptr_in);
        }

        let idx = self.trace_tree.index(trace, |ip, parent| {
            let ip = ip - self.slide;
            self.writer.write_trace(ip, parent)
        });

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
        arch::get_rss()
    }

    fn write_images(&mut self) {
        for image in arch::get_images() {
            self.writer
                .write_image(image.name, image.start_address, image.size)
        }
    }
}
