use std::mem;

pub mod infer_request;
use infer_request::InferRequest;
use crate::utils;

#[derive(Clone)]
pub struct InputInfo {
    pub dims: Vec<usize>,
}

pub struct ExecutableNetwork {
    // TODO: a factory method in Core instead
    pub ie_executable_network: Box<*mut ffi::ie_executable_network_t>,
}

impl ExecutableNetwork {
    pub fn create_infer_request(&self) -> InferRequest {
        unsafe {
            let mut ie_infer_request = Box::<*mut ffi::ie_infer_request_t>::new(mem::zeroed());
            let status = ffi::ie_exec_network_create_infer_request(*self.ie_executable_network,
                &mut *ie_infer_request);
            utils::check_status(status);
            InferRequest{
                ie_infer_request: ie_infer_request,
            }
        }
    }
}
