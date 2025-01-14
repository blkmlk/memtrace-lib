use libc::{c_char, c_void};
use std::ffi::CStr;

// External declarations for dyld APIs
extern "C" {
    fn _dyld_image_count() -> u32;
    fn _dyld_get_image_name(index: u32) -> *const c_char;
    fn _dyld_get_image_header(index: u32) -> *const c_void;
    fn _dyld_get_image_vmaddr_slide(index: u32) -> isize;
}

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

pub struct Image {
    pub name: String,
    pub start_address: usize,
    pub size: usize,
}

pub fn get_image_slide() -> usize {
    unsafe { _dyld_get_image_vmaddr_slide(1) as usize }
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
            let slide = _dyld_get_image_vmaddr_slide(i);

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
                start_address: header as usize - slide as usize,
                size: image_size as usize,
            });
        }

        images
    }
}
