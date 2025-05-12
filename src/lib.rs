mod dylib;
mod trace;
mod trace_tree;
mod tracker;

pub use common;

use crate::tracker::Tracker;
use fishhook::{register, Rebinding};
use libc::{dlsym, size_t, RTLD_NEXT};
use std::env;
use std::ffi::c_void;
use std::sync::{LazyLock, Mutex, Once};

static INIT: Once = Once::new();
static mut ORIGINAL_MALLOC: Option<unsafe extern "C" fn(size: size_t) -> *mut c_void> = None;
static mut ORIGINAL_CALLOC: Option<unsafe extern "C" fn(num: size_t, size: size_t) -> *mut c_void> =
    None;
static mut ORIGINAL_REALLOC: Option<
    unsafe extern "C" fn(ptr: *mut c_void, size: size_t) -> *mut c_void,
> = None;
static mut ORIGINAL_FREE: Option<unsafe extern "C" fn(ptr: *mut c_void)> = None;

// mi-malloc
static mut MI_MALLOC: Option<unsafe extern "C" fn(size: size_t) -> *mut c_void> = None;
static mut MI_CALLOC: Option<unsafe extern "C" fn(num: size_t, size: size_t) -> *mut c_void> = None;
static mut MI_REALLOC: Option<unsafe extern "C" fn(ptr: *mut c_void, size: size_t) -> *mut c_void> =
    None;
static mut MI_FREE: Option<unsafe extern "C" fn(ptr: *mut c_void)> = None;

static TRACKER: LazyLock<Mutex<Option<Tracker>>> = LazyLock::new(|| Mutex::new(None));

macro_rules! gen_malloc {
    ($name:ident, $original:ident) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(size: size_t) -> *mut c_void {
            let original_malloc = $original.unwrap();
            let ptr = original_malloc(size);

            let mut guard = TRACKER.lock().unwrap();
            if let Some(tracker) = guard.as_mut() {
                tracker.on_malloc(size, ptr as usize);
            }

            ptr
        }
    };
}

macro_rules! gen_calloc {
    ($name:ident, $original:ident) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(num: size_t, size: size_t) -> *mut c_void {
            let original_calloc = $original.unwrap();
            let ptr = original_calloc(num, size);

            let mut guard = TRACKER.lock().unwrap();
            if let Some(tracker) = guard.as_mut() {
                tracker.on_malloc(num * size, ptr as usize);
            }

            ptr
        }
    };
}

macro_rules! gen_realloc {
    ($name:ident, $original:ident) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(ptr_in: *mut c_void, size: size_t) -> *mut c_void {
            let original_realloc = $original.unwrap();
            let ptr_out = original_realloc(ptr_in, size);

            let mut guard = TRACKER.lock().unwrap();
            if let Some(tracker) = guard.as_mut() {
                tracker.on_realloc(size, ptr_in as usize, ptr_out as usize);
            }

            ptr_out
        }
    };
}

macro_rules! gen_free {
    ($name:ident, $original:ident) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(ptr: *mut c_void) {
            let original_free = $original.unwrap();
            original_free(ptr);

            let mut guard = TRACKER.lock().unwrap();
            if let Some(tracker) = guard.as_mut() {
                tracker.on_free(ptr as usize);
            }
        }
    };
}

macro_rules! bind_func {
    ($($symbol:ty)*, $var:ident) => {
        let symbol = concat!(stringify!($($symbol)*), "\0").as_bytes();
        let ptr = dlsym(RTLD_NEXT, symbol.as_ptr() as *const _);
        if !ptr.is_null() {
            $var = Some(std::mem::transmute(ptr));
        } else {
            eprintln!(concat!("Error: Could not locate original ", stringify!($($symbol)*)));
        }
    };
}

gen_malloc!(orig_malloc, ORIGINAL_MALLOC);
gen_calloc!(orig_calloc, ORIGINAL_CALLOC);
gen_realloc!(orig_realloc, ORIGINAL_REALLOC);
gen_free!(orig_free, ORIGINAL_FREE);

// mi-malloc
gen_malloc!(mi_malloc, MI_MALLOC);
gen_calloc!(mi_calloc, MI_CALLOC);
gen_realloc!(mi_realloc, MI_REALLOC);
gen_free!(mi_free, MI_FREE);

pub extern "C" fn my_exit() {
    let mut guard = TRACKER.lock().unwrap();
    if let Some(tracker) = guard.as_mut() {
        tracker.on_exit();
    }
}

unsafe fn init_functions() {
    INIT.call_once(|| {
        bind_func!(malloc, ORIGINAL_MALLOC);
        bind_func!(calloc, ORIGINAL_CALLOC);
        bind_func!(realloc, ORIGINAL_REALLOC);
        bind_func!(free, ORIGINAL_FREE);

        bind_func!(mi_malloc, MI_MALLOC);
        bind_func!(mi_calloc, MI_CALLOC);
        bind_func!(mi_realloc, MI_REALLOC);
        bind_func!(mi_free, MI_FREE);

        let pipe_filepath = env::var("PIPE_FILEPATH").expect("PIPE_FILEPATH must be set");

        let mut tracker = Tracker::new(pipe_filepath);
        tracker.init();

        let mut lock = TRACKER.lock().unwrap();
        *lock = Some(tracker);

        libc::atexit(my_exit);
    });
}

#[ctor::ctor]
fn init() {
    unsafe {
        init_functions();

        register(vec![
            Rebinding {
                name: "malloc".to_string(),
                function: orig_malloc as *const c_void,
            },
            Rebinding {
                name: "calloc".to_string(),
                function: orig_calloc as *const c_void,
            },
            Rebinding {
                name: "realloc".to_string(),
                function: orig_realloc as *const c_void,
            },
            Rebinding {
                name: "free".to_string(),
                function: orig_free as *const c_void,
            },
            // mi-malloc
            Rebinding {
                name: "mi_malloc".to_string(),
                function: mi_malloc as *const c_void,
            },
            Rebinding {
                name: "mi_calloc".to_string(),
                function: mi_calloc as *const c_void,
            },
            Rebinding {
                name: "mi_realloc".to_string(),
                function: mi_realloc as *const c_void,
            },
            Rebinding {
                name: "mi_free".to_string(),
                function: mi_free as *const c_void,
            },
            Rebinding {
                name: "atexit".to_string(),
                function: my_exit as *const c_void,
            },
        ]);
    }
}
