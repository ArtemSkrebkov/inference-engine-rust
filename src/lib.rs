extern crate inference_engine_sys_rust as ffi;

use std::{mem, str};
use std::ffi::CString;
use std::collections::HashMap;
// TODO make this private
pub mod utils;
pub mod executable_network;

use executable_network::{ExecutableNetwork, InputInfo};

pub struct Core {
    core: Box<*mut ffi::ie_core_t>,
}

pub struct Network {
    ie_network: Box<*mut ffi::ie_network_t>,
    name: String,
    inputs_info: HashMap<String, InputInfo>,
}

impl Network {
    // TODO: replace getter with a public field?
    pub fn get_inputs_info(self) -> HashMap<String, InputInfo> {
        return self.inputs_info;
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
            utils::check_status(status);
            Core {
                core: core,
            }
        }
    }

    pub fn get_available_devices(self) -> Vec<String> {
        unsafe {
            let mut available_devices = ffi::ie_available_devices{
                devices : std::ptr::null_mut(),
                num_devices : 0,
            };

            let status = ffi::ie_core_get_available_devices(*self.core, &mut available_devices);
            utils::check_status(status);
            let devices = utils::convert_double_pointer_to_vec(available_devices.devices,
                available_devices.num_devices as usize).unwrap();
            return devices;
        }
    }

    // TODO: make static or move to a separate entity?
    pub fn read_network(&self, xml_filename: &str, bin_filename: &str) -> Network {
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
            utils::check_status(status);

            let mut ie_network_name : *mut libc::c_char = std::ptr::null_mut();
            let status = ffi::ie_network_get_name(*ie_network, &mut ie_network_name as *mut *mut libc::c_char);
            utils::check_status(status);

            let network_name = utils::convert_double_pointer_to_vec(&mut ie_network_name as *mut *mut libc::c_char, 1).unwrap();

            let mut input_name : *mut libc::c_char = std::ptr::null_mut();
            let status = ffi::ie_network_get_input_name(*ie_network, 0,
                &mut input_name as *mut *mut libc::c_char);
            utils::check_status(status);
            let mut raw_dims: ffi::dimensions = ffi::dimensions_t{
                ranks: 0,
                dims: [0, 0, 0, 0, 0, 0, 0, 0]
            };
            let status = ffi::ie_network_get_input_dims(*ie_network,
                            input_name as *const i8, &mut raw_dims as *mut ffi::dimensions);
            utils::check_status(status);

            let ranks: usize = raw_dims.ranks as usize;
            let mut dims = Vec::with_capacity(ranks);
            for i in 0..ranks {
                let dim: usize = raw_dims.dims[i] as usize;
                dims.push(dim);
            }

            let input_name = utils::convert_double_pointer_to_vec(&mut input_name as *mut *mut libc::c_char, 1).unwrap();
            // FIXME: try to use &str (requires to learn lifetime concept)
            let inputs_info: HashMap<String, InputInfo> = [(input_name[0].clone(),
                                                            InputInfo{dims: dims})]
                                                            .iter().cloned().collect();
            // FIXME: need a function to convert raw pointer to Rust string/string slice
            Network{
                ie_network: ie_network,
                name: network_name[0].clone(),
                inputs_info: inputs_info,
            }
        }
    }

    pub fn load_network(&self, network: Network, device_name: &str) -> ExecutableNetwork {
        let config: ffi::ie_config_t = ffi::ie_config{
            name: std::ptr::null_mut(),
            next: std::ptr::null_mut(),
            value: std::ptr::null_mut(),
        };
        let device_name = CString::new(device_name).unwrap();
        let device_name = device_name.as_ptr();
        unsafe {
            let mut ie_executable_network = Box::<*mut ffi::ie_executable_network_t>::new(mem::zeroed());
            let status = ffi::ie_core_load_network(*self.core, *network.ie_network,
                device_name as *const i8,
                &config as *const ffi::ie_config_t,
                &mut *ie_executable_network);
            utils::check_status(status);

            utils::check_status(status);
            ExecutableNetwork {
                ie_executable_network: ie_executable_network,
            }
        } 
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_network_from_file_and_get_inputs_info() {
        let core = Core::new();
        let network = core.read_network("test_data/resnet-50.xml",
                        "test_data/resnet-50.bin");
        let inputs_info = network.get_inputs_info();
        assert!(inputs_info.len() == 1);
        for (name, info) in &inputs_info {
            assert_eq!("data", name);
            assert_eq!(vec![1, 3, 224, 224], info.dims);
        }
    }

    #[test]
    #[ignore]
    // FIXME: ie_network_get_name returns not a name but something wrong
    fn read_network_from_file_and_get_network_name() {
        let core = Core::new();
        let network = core.read_network("test_data/resnet-50.xml",
                        "test_data/resnet-50.bin");

        let network_name = network.get_name();
        assert_eq!("ResNet-50", network_name);
    }
}
