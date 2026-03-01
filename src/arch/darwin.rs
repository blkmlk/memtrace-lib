use libc::{c_char, c_void};
use libc::{
    mach_msg_type_number_t, mach_task_basic_info_data_t, sysconf, task_info, task_info_t,
    time_value_t, MACH_TASK_BASIC_INFO, MACH_TASK_BASIC_INFO_COUNT,
};
use mach2::traps::mach_task_self;
use std::ffi::CStr;

// External declarations for dyld APIs
extern "C" {
    fn _dyld_image_count() -> u32;
    fn _dyld_get_image_name(index: u32) -> *const c_char;
    fn _dyld_get_image_header(index: u32) -> *const c_void;
    fn _dyld_get_image_vmaddr_slide(index: u32) -> isize;
}
use super::{Image, SysInfo};

// Mach-O header structures
#[repr(C)]
struct MachHeader {
    magic: u32,
    cputype: i32,
    cpusubtype: i32,
    filetype: u32,
    ncmds: u32,
    sizeofcmds: u32,
    flags: u32,
    reserved: u32, // Only present in 64-bit headers
}

#[repr(C)]
struct LoadCommand {
    cmd: u32,
    cmdsize: u32,
}

#[repr(C)]
struct SegmentCommand64 {
    cmd: u32,
    cmdsize: u32,
    segname: [u8; 16],
    vmaddr: u64,
    vmsize: u64,
    fileoff: u64,
    filesize: u64,
    maxprot: i32,
    initprot: i32,
    nsects: u32,
    flags: u32,
}

pub fn get_images() -> Vec<Image> {
    unsafe {
        let image_count = _dyld_image_count();

        let mut images = Vec::with_capacity(image_count as usize);
        for i in 0..image_count {
            // Get image name
            let image_name_ptr = _dyld_get_image_name(i);
            let image_name = if image_name_ptr.is_null() {
                "<unknown>".to_string()
            } else {
                CStr::from_ptr(image_name_ptr)
                    .to_string_lossy()
                    .into_owned()
            };

            let header = _dyld_get_image_header(i);
            let vmaddr = _dyld_get_image_vmaddr_slide(i);

            let image_header = header as *const MachHeader;
            let mut image_size = 0;
            let mut load_command =
                (header as *const u8).add(size_of::<MachHeader>()) as *const LoadCommand;

            for _ in 0..(*image_header).ncmds {
                if (*load_command).cmd == 0x19 {
                    // LC_SEGMENT_64
                    let segment = load_command as *const SegmentCommand64;
                    image_size += (*segment).vmsize;
                }

                load_command = (load_command as *const u8).add((*load_command).cmdsize as usize)
                    as *const LoadCommand;
            }

            images.push(Image {
                name: image_name,
                start_address: vmaddr as usize,
                size: image_size as usize,
            });
        }

        images
    }
}

pub fn get_rss() -> usize {
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

pub fn get_sys_info() -> SysInfo {
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
