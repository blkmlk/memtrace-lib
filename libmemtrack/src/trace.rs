use std::ffi::c_void;

extern "C" {
    fn backtrace(buffer: *mut *mut c_void, size: libc::c_int) -> libc::c_int;
}

const MAX_SIZE: usize = 64;

pub struct Trace {
    stack: [u64; MAX_SIZE],
    len: usize,
}

impl Trace {
    pub fn new() -> Self {
        let mut tracer = Self {
            stack: [0; MAX_SIZE],
            len: 0,
        };

        tracer.init();

        tracer
    }

    pub fn as_slice(&self) -> &[u64] {
        &self.stack[..self.len]
    }

    fn init(&mut self) {
        unsafe {
            let n = backtrace(
                self.stack.as_mut_ptr() as *mut *mut c_void,
                MAX_SIZE as libc::c_int,
            );
            self.len = n as usize
        }
    }
}
