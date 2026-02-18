#[cfg_attr(target_os = "macos", path = "darwin.rs")]
#[cfg_attr(target_os = "linux", path = "linux.rs")]
mod platform;

pub use platform::*;

pub struct Image {
    pub name: String,
    pub start_address: usize,
    pub size: usize,
}

pub struct SysInfo {
    pub exec_path: String,
    pub page_size: usize,
    pub phys_pages: usize,
}
