extern crate inference_engine_sys_rust as ffi;

use std::mem;
use std::ffi::CString;

pub struct Core {
    core: Box<*mut ffi::ie_core_t>,
}

pub struct Network {
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
            // TODO: check for return value
            ffi::ie_core_create(config_file_ptr, &mut *core);
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
            // TODO: check for return value
            ffi::ie_core_get_available_devices(*self.core, &mut available_devices);
            devices = Self::convert_double_pointer_to_vec(available_devices.devices,
                available_devices.num_devices as usize).unwrap();
        }

        return devices;
    }

    // TODO: make static or move to a separate entity?
    pub fn read_network(self, xml_filename: String , bin_filename: String) -> Network {
        let network = Network{name : String::from("network")};

        return network;
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
        assert_eq!(String::from("CPU"), devices[0]);
    }

    #[test]
    fn read_network_from_IR_and_get_inputs_info() {
        let core = Core::new();
        let network = core.read_network(String::from("resnet50.xml"),
                        String::from("resnet50.bin"));
        let input_info = network.get_input_info();
        assert!(input_info.len() > 0);
    }

    #[test]
    fn read_network_from_IR_and_get_network_name() {
        let core = Core::new();
        let network = core.read_network(String::from("resnet50.xml"),
                        String::from("resnet50.bin"));

        let network_name = network.get_name();
        assert_eq!("resnet50", network_name);
    }
}