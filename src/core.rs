extern crate inference_engine_sys_rust as ffi;

use std::mem;
use std::ffi::CString;
use std::slice;
use std::str;

pub struct Core {
    core: Box<*mut ffi::ie_core_t>,
}

pub struct Network {
    ie_network: Box<*mut ffi::ie_network_t>,
    name: String,
}

use std::collections::HashMap;
impl Network {
    pub fn get_input_info(self) -> HashMap<String, String> {
        let mut inputs_map = HashMap::new();
        inputs_map.insert(String::from("input"), String::from("input"));

        return inputs_map;
    }

    pub fn get_name(self) -> String {
        return self.name;
    }
}

impl Core {
    pub fn new() -> Core {
        unsafe {
            let mut core = Box::<*mut ffi::ie_core_t>::new(mem::zeroed());
            let config_file = CString::new("").unwrap();
            let config_file_ptr = config_file.as_ptr();

            let status = ffi::ie_core_create(config_file_ptr, &mut *core);
            Self::check_status(status);
            Core {
                core: core,
            }
        }
    }

    pub fn get_available_devices(self) -> Vec<String> {
        let mut devices = Vec::new();
        unsafe {
            let mut available_devices = ffi::ie_available_devices{
                devices : std::ptr::null_mut(),
                num_devices : 0,
            };

            let status = ffi::ie_core_get_available_devices(*self.core, &mut available_devices);
            Self::check_status(status);
            devices = Self::convert_double_pointer_to_vec(available_devices.devices,
                available_devices.num_devices as usize).unwrap();
        }

        return devices;
    }

    // TODO: make static or move to a separate entity?
    pub fn read_network(self, xml_filename: &str, bin_filename: &str) -> Network {
        unsafe {
            // FIXME: looks weird but filenames need to wrapped up to pass to a FFI function
            let xml_filename = String::from(xml_filename);
            let bin_filename = String::from(bin_filename);
            let xml_filename_c_str = xml_filename.as_ptr();
            let bin_filename_c_str = bin_filename.as_ptr();

            let mut ie_network = Box::<*mut ffi::ie_network_t>::new(mem::zeroed());
            // TODO: is it possible to avoid dereferencing of core across the file?
            let status = ffi::ie_core_read_network(*self.core, xml_filename_c_str as *const i8,
                bin_filename_c_str as *const i8, &mut *ie_network);
            Self::check_status(status);

            let mut ie_network_name : *mut libc::c_char = std::ptr::null_mut();
            let status = ffi::ie_network_get_name(*ie_network, &mut ie_network_name as *mut *mut libc::c_char);
            Self::check_status(status);

            let network_name = Self::convert_double_pointer_to_vec(&mut ie_network_name as *mut *mut libc::c_char, 1).unwrap();

            Network{
                ie_network: ie_network,
                name: network_name[0].clone(),
            }
        }
    }

    fn check_status(status: ffi::IEStatusCode) {
        match status {
            s if s == (ffi::IEStatusCode_GENERAL_ERROR as _) => panic!("GENERAL_ERROR"),
            s if s == (ffi::IEStatusCode_UNEXPECTED as _) => panic!("UNEXPECTED"),
            s if s == (ffi::IEStatusCode_OK as _) => {},
            s => panic!("Unknown return value = {}", s),
        }
    }

    unsafe fn convert_double_pointer_to_vec(
        data: *mut *mut libc::c_char,
        len: libc::size_t,
    ) -> Result<Vec<String>, std::str::Utf8Error> {
        std::slice::from_raw_parts(data, len)
            .iter()
            .map(|arg| std::ffi::CStr::from_ptr(*arg).to_str().map(ToString::to_string))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Core;
    #[test]
    fn create_core_and_find_device() {
        let core = Core::new();
        let devices = core.get_available_devices();
        assert!(!devices.is_empty());
        assert_eq!("CPU", devices[0]);
    }

    #[test]
    fn read_network_from_IR_and_get_inputs_info() {
        let core = Core::new();
        let network = core.read_network("test_data/resnet-50.xml",
                        "test_data/resnet-50.bin");
        let input_info = network.get_input_info();
        assert!(input_info.len() > 0);
    }

    #[test]
    #[ignore]
    // FIXME: ie_network_get_name returns not a name but something wrong
    fn read_network_from_IR_and_get_network_name() {
        let core = Core::new();
        let network = core.read_network("test_data/resnet-50.xml",
                        "test_data/resnet-50.bin");

        let network_name = network.get_name();
        assert_eq!("ResNet-50", network_name);
    }
}