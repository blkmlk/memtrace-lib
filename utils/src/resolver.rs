use addr2line::Loader;
use rangemap::RangeMap;
use std::collections::HashMap;

pub enum Error {
    ModuleNotFound,
}

#[derive(Clone, Eq, PartialEq)]
struct Module {
    pub start_address: u64,
    pub end_address: u64,
    path: String,
}

impl Module {
    pub fn new(path: String, start_address: u64, size: u64) -> Self {
        Self {
            path,
            start_address,
            end_address: start_address + size,
        }
    }

    pub fn lookup(&self, ip: u64, loader: &Loader) -> Option<Location> {
        let symbol = loader.find_symbol(ip)?;
        let function_name = rustc_demangle::demangle(symbol).to_string();

        Some(Location { function_name })
    }
}

#[derive(Clone, Debug)]
pub struct Location {
    function_name: String,
}

pub struct Resolver {
    modules: RangeMap<u64, Module>,
    cached: HashMap<u64, Location>,
    loaders: HashMap<u64, Loader>,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            modules: RangeMap::new(),
            cached: HashMap::new(),
            loaders: HashMap::new(),
        }
    }

    pub fn add_module(
        &mut self,
        file_path: &str,
        start_address: u64,
        size: u64,
    ) -> Result<(), Error> {
        let module = Module::new(file_path.to_string(), start_address, size);

        let Ok(loader) = Loader::new(file_path.to_string()) else {
            return Err(Error::ModuleNotFound);
        };

        self.loaders.insert(start_address, loader);

        self.modules
            .insert(module.start_address..module.end_address, module);

        Ok(())
    }

    pub fn lookup(&mut self, ip: u64) -> Option<Location> {
        if let Some(location) = self.cached.get(&ip).cloned() {
            return Some(location);
        }

        let module = self.modules.get(&ip)?;
        let loader = self.loaders.get(&module.start_address)?;
        module.lookup(ip, loader)
    }
}

#[cfg(test)]
mod tests {
    use crate::resolver::Resolver;
    use std::ffi::c_void;

    extern "C" {
        fn _dyld_get_image_header(index: u32) -> *const c_void;
        fn _dyld_get_image_vmaddr_slide(index: u32) -> isize;
    }

    fn boo() {}

    #[test]
    fn test_lookup() {
        let exe = std::env::current_exe()
            .unwrap()
            .to_string_lossy()
            .to_string();

        let slide = unsafe { _dyld_get_image_vmaddr_slide(0) } as u64;

        let addr = unsafe { _dyld_get_image_header(0) } as u64 - slide;

        let mut resolver = Resolver::new();
        resolver.add_module(&exe, addr, 0x1000000);

        let ip = boo as u64 - slide;

        let location = resolver.lookup(ip);
        println!("{:#?}", location);
    }
}
