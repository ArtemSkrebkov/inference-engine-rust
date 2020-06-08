pub mod infer_request;
pub use infer_request::InferRequest;
use std::{mem, str, slice};

// TODO: dupliction
fn check_status(status: ffi::IEStatusCode) {
    match status {
        s if s == (ffi::IEStatusCode_GENERAL_ERROR as _) => panic!("GENERAL_ERROR"),
        s if s == (ffi::IEStatusCode_UNEXPECTED as _) => panic!("UNEXPECTED"),
        s if s == (ffi::IEStatusCode_OK as _) => {},
        s if s == (ffi::IEStatusCode_NOT_FOUND as _) => panic!("NOT_FOUND"),
        s => panic!("Unknown return value = {}", s),
    }
}

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
            check_status(status);
            InferRequest{
                ie_infer_request: ie_infer_request,
            }
        }
    }
}