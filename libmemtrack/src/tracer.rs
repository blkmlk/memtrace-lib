use libunwind::{_Unwind_Reason_Code, _Unwind_Reason_Code__URC_NO_REASON};
use std::ffi::c_void;

const MAX_SIZE: usize = 64;

pub struct Tracer {
    stack: [u64; MAX_SIZE],
    len: usize,
}

impl Tracer {
    pub fn new() -> Self {
        let mut tracer = Self {
            stack: [0; MAX_SIZE],
            len: 0,
        };

        tracer.init();

        tracer
    }

    fn init(&mut self) {
        unsafe {
            let tracer: *mut c_void = std::mem::transmute(self);
            libunwind::_Unwind_Backtrace(Some(callback), tracer);
        }
    }

    fn push(&mut self, ip: u64) {
        if self.len >= MAX_SIZE {
            return;
        }

        self.stack[self.len] = ip;
        self.len += 1;
    }
}

unsafe extern "C" fn callback(
    ctx: *mut libunwind::_Unwind_Context,
    arg: *mut c_void,
) -> _Unwind_Reason_Code {
    let tracer = &mut *(arg as *mut Tracer);

    let pc = libunwind::_Unwind_GetIP(ctx) as u64;

    if pc > 0 {
        tracer.push(pc);
    }

    _Unwind_Reason_Code__URC_NO_REASON
}
