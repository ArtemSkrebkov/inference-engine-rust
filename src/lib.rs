extern crate inference_engine_sys_rust as ffi;

use std::{mem, str, slice};
use std::ffi::CString;
use std::collections::HashMap;
use ndarray::{Array, ArrayD, ArrayViewMut, IxDyn};
use rulinalg;
use ndarray_image;

mod infer_request;
pub use infer_request::InferRequest;

fn check_status(status: ffi::IEStatusCode) {
    match status {
        s if s == (ffi::IEStatusCode_GENERAL_ERROR as _) => panic!("GENERAL_ERROR"),
        s if s == (ffi::IEStatusCode_UNEXPECTED as _) => panic!("UNEXPECTED"),
        s if s == (ffi::IEStatusCode_OK as _) => {},
        s if s == (ffi::IEStatusCode_NOT_FOUND as _) => panic!("NOT_FOUND"),
        s => panic!("Unknown return value = {}", s),
    }
}

pub struct Core {
    core: Box<*mut ffi::ie_core_t>,
}

pub struct Network {
    ie_network: Box<*mut ffi::ie_network_t>,
    name: String,
    inputs_info: HashMap<String, InputInfo>,
}

#[derive(Clone)]
pub struct InputInfo {
    pub dims: Vec<usize>,
}

pub struct ExecutableNetwork {
    ie_executable_network: Box<*mut ffi::ie_executable_network_t>,
}

impl ExecutableNetwork {
    pub fn create_infer_request(&self) -> InferRequest {
        unsafe {
            let mut ie_infer_request = Box::<*mut ffi::ie_infer_request_t>::new(mem::zeroed());
            let status = ffi::ie_exec_network_create_infer_request(*self.ie_executable_network,
                &mut *ie_infer_request);
            check_status(status);
            InferRequest{
                ie_infer_request: ie_infer_request,
            }
        }
    }
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
            check_status(status);
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
            check_status(status);
            let devices = Self::convert_double_pointer_to_vec(available_devices.devices,
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
            check_status(status);

            let mut ie_network_name : *mut libc::c_char = std::ptr::null_mut();
            let status = ffi::ie_network_get_name(*ie_network, &mut ie_network_name as *mut *mut libc::c_char);
            check_status(status);

            let network_name = Self::convert_double_pointer_to_vec(&mut ie_network_name as *mut *mut libc::c_char, 1).unwrap();

            let mut input_name : *mut libc::c_char = std::ptr::null_mut();
            let status = ffi::ie_network_get_input_name(*ie_network, 0,
                &mut input_name as *mut *mut libc::c_char);
            check_status(status);
            let mut raw_dims: ffi::dimensions = ffi::dimensions_t{
                ranks: 0,
                dims: [0, 0, 0, 0, 0, 0, 0, 0]
            };
            let status = ffi::ie_network_get_input_dims(*ie_network,
                            input_name as *const i8, &mut raw_dims as *mut ffi::dimensions);
            check_status(status);

            let ranks: usize = raw_dims.ranks as usize;
            let mut dims = Vec::with_capacity(ranks);
            for i in 0..ranks {
                let dim: usize = raw_dims.dims[i] as usize;
                dims.push(dim);
            }

            let input_name = Self::convert_double_pointer_to_vec(&mut input_name as *mut *mut libc::c_char, 1).unwrap();
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
            check_status(status);

            check_status(status);
            ExecutableNetwork {
                ie_executable_network: ie_executable_network,
            }
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

    #[test]
    fn create_executable_network() {
        let core = Core::new();
        let network = core.read_network("test_data/resnet-50.xml",
                        "test_data/resnet-50.bin");
        let _executable_network = core.load_network(network, "CPU");
        // TODO: check something?
    }
    
    #[test]
    fn create_infer_request() {
        let core = Core::new();
        let network = core.read_network("test_data/resnet-50.xml",
                        "test_data/resnet-50.bin");
        let executable_network = core.load_network(network, "CPU");
        let infer_request: InferRequest = executable_network.create_infer_request();

        let input = infer_request.get_blob("data");
        assert_eq!(input.dim(), IxDyn(&[1, 3, 224, 224]));
    }

    #[test]
    fn get_output_blob() {
        let core = Core::new();
        let network = core.read_network("test_data/resnet-50.xml",
                        "test_data/resnet-50.bin");
        let executable_network = core.load_network(network, "CPU");
        let infer_request: InferRequest = executable_network.create_infer_request();

        let output = infer_request.get_blob("prob");
        assert_eq!(output.dim(), IxDyn(&[1, 1000]));
    }
    // gratefully throw an error if no blob
    
    #[test]
    fn can_change_input_blob() {
        let core = Core::new();
        let network = core.read_network("test_data/resnet-50.xml",
                        "test_data/resnet-50.bin");
        let executable_network = core.load_network(network, "CPU");
        let infer_request: InferRequest = executable_network.create_infer_request();
        {
            let mut input = infer_request.get_blob("data");
            input.fill(1.0);
        }
        let input = infer_request.get_blob("data");

        assert_eq!(input[[0, 0, 0, 0]], 1.0);
    }

    #[test]
    fn get_correct_inference_results() {
        let core = Core::new();
        let network = core.read_network("test_data/resnet-50.xml",
                        "test_data/resnet-50.bin");
        let executable_network = core.load_network(network, "CPU");
        let infer_request: InferRequest = executable_network.create_infer_request();

        let mut input = infer_request.get_blob("data");
        let input_image = ndarray_image::open_image("test_data/cat.png",
            ndarray_image::Colors::Bgr).unwrap();
        // FIXME: move repacking into a helper
        let dims = input_image.dim();
        let width = dims.0;
        let height = dims.1;
        let channels = dims.2;
        for c in 0..channels {
            for h in 0..height {
                for w in 0..width {
                    input[[0, c, h, w]] = input_image[[h, w, c]] as f32;
                }
            }
        }

        infer_request.infer();
        let output = infer_request.get_blob("prob");
        // FIXME: rulinalg is used only for argmax
        let argmax = rulinalg::utils::argmax(output.into_slice().unwrap());
        assert_eq!(argmax.0, 283);
    }

    #[test]
    fn can_set_blob_for_input() {
        let core = Core::new();
        let network = core.read_network("test_data/resnet-50.xml",
                        "test_data/resnet-50.bin");
        let executable_network = core.load_network(network, "CPU");
        let infer_request: InferRequest = executable_network.create_infer_request();
        let input_image = ndarray_image::open_image("test_data/cat.png",
            ndarray_image::Colors::Bgr).unwrap();
        // FIXME: move repacking into a helper
        let dims = input_image.dim();
        let width = dims.0;
        let height = dims.1;
        let channels = dims.2;
        let mut input = ArrayD::<f32>::zeros(IxDyn(&[1, channels, height, width]));
        for c in 0..channels {
            for h in 0..height {
                for w in 0..width {
                    input[[0, c, h, w]] = input_image[[h, w, c]] as f32;
                }
            }
        }

        let elem = input[[0, 0, 0, 0]];
        infer_request.set_blob("data", input);

        let input_from_get = infer_request.get_blob("data");

        assert_eq!(input_from_get[[0, 0, 0, 0]], elem);
    }

    #[test]
    fn get_correct_inference_result_with_set_blob_for_input() {
        let core = Core::new();
        let network = core.read_network("test_data/resnet-50.xml",
                        "test_data/resnet-50.bin");
        let executable_network = core.load_network(network, "CPU");
        let infer_request: InferRequest = executable_network.create_infer_request();
        let input_image = ndarray_image::open_image("test_data/cat.png",
            ndarray_image::Colors::Bgr).unwrap();
        // FIXME: move repacking into a helper
        let dims = input_image.dim();
        let width = dims.0;
        let height = dims.1;
        let channels = dims.2;
        let mut input = ArrayD::<f32>::zeros(IxDyn(&[1, channels, height, width]));
        // TODO: move to a helper
        for c in 0..channels {
            for h in 0..height {
                for w in 0..width {
                    input[[0, c, h, w]] = input_image[[h, w, c]] as f32;
                }
            }
        }

        infer_request.set_blob("data", input);
        infer_request.infer();
        let output = infer_request.get_blob("prob");
        // FIXME: rulinalg is used only for argmax
        let argmax = rulinalg::utils::argmax(output.into_slice().unwrap());
        assert_eq!(argmax.0, 283);
    }
}
