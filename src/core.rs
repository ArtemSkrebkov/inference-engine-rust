extern crate inference_engine_sys_rust as ffi;

use std::mem;
use std::ffi::CString;

pub struct Core {
    core: Box<*mut ffi::ie_core_t>,
}


impl Core {
    pub fn new() -> Core {
        unsafe {
            let mut core = Box::<*mut ffi::ie_core_t>::new(mem::zeroed());
            let config_file = CString::new("").unwrap();
            let config_file_ptr = config_file.as_ptr();
            let status = ffi::ie_core_create(config_file_ptr, &mut *core);
            Core {
                core: core,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Core;
    #[test]
    fn create_core() {
        let core = Core::new();
    }
}