use libc::{c_char, c_void};
use std::ffi::CStr;

// External declarations for dyld APIs
extern "C" {
    fn _dyld_image_count() -> u32;
    fn _dyld_get_image_name(index: u32) -> *const c_char;
    fn _dyld_get_image_header(index: u32) -> *const c_void;
    fn _dyld_get_image_vmaddr_slide(index: u32) -> isize;
}

pub struct Image {
    pub name: String,
    pub header_address: usize,
    pub slide: isize,
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

            images.push(Image {
                name: image_name,
                header_address: header as usize,
                slide,
            });
        }

        images
    }
}
