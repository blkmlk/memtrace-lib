use std::{
    ffi::{c_void, CStr},
    fs,
    path::PathBuf,
};

use libc::{c_int, dl_iterate_phdr, dl_phdr_info};

use super::{Image, SysInfo};

pub fn get_image_slide() -> usize {
    0
}

pub fn get_images() -> Vec<Image> {
    let mut images = Vec::<Image>::new();

    unsafe extern "C" fn callback(
        info: *mut dl_phdr_info,
        _size: usize,
        data: *mut c_void,
    ) -> c_int {
        let images = &mut *(data as *mut Vec<Image>);
        let info = &*info;

        // Get name
        let name = if info.dlpi_name.is_null() || (*info.dlpi_name == 0) {
            main_executable_path()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|_| "<program>".to_string())
        } else {
            CStr::from_ptr(info.dlpi_name)
                .to_string_lossy()
                .into_owned()
        };

        let mut image_size = 0usize;

        // Sum PT_LOAD segments (equivalent to LC_SEGMENT_64)
        for i in 0..info.dlpi_phnum {
            let phdr = &*info.dlpi_phdr.add(i as usize);
            if phdr.p_type == libc::PT_LOAD {
                image_size += phdr.p_memsz as usize;
            }
        }

        images.push(Image {
            name,
            start_address: info.dlpi_addr as usize,
            size: image_size,
        });

        0
    }

    unsafe {
        dl_iterate_phdr(Some(callback), &mut images as *mut _ as *mut c_void);
    }

    images
}

pub fn get_rss() -> usize {
    let statm = fs::read_to_string("/proc/self/statm").unwrap_or_default();
    let mut it = statm.split_whitespace();

    // skip "size"
    let _size_pages = it.next();
    let rss_pages: usize = it.next().and_then(|s| s.parse().ok()).unwrap_or(0);

    let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    if page_size <= 0 {
        return 0;
    }

    rss_pages.saturating_mul(page_size as usize)
}

pub fn get_sys_info() -> SysInfo {
    let exec_path = std::fs::read_link("/proc/self/exe")
        .unwrap_or_else(|_| PathBuf::from(""))
        .to_string_lossy()
        .into_owned();

    // System info
    let (page_size, phys_pages) = unsafe {
        let page_size = libc::sysconf(libc::_SC_PAGESIZE);
        let pages = libc::sysconf(libc::_SC_PHYS_PAGES);

        (
            if page_size > 0 { page_size as usize } else { 0 },
            if pages > 0 { pages as usize } else { 0 },
        )
    };

    SysInfo {
        exec_path,
        page_size,
        phys_pages,
    }
}

fn main_executable_path() -> std::io::Result<PathBuf> {
    fs::read_link("/proc/self/exe")
}
