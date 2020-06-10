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
            utils::check_status_with_error_message(status, "ExecutableNetwork object is invalid.");
            InferRequest{
                ie_infer_request: ie_infer_request,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "GENERAL_ERROR: ExecutableNetwork object is invalid.")]
    fn create_infer_request_from_invalid_executable_network() {
        let executable_network = ExecutableNetwork{
            ie_executable_network: unsafe {
                Box::<*mut ffi::ie_executable_network_t>::new(mem::zeroed())
            },
        };

        let _infer_request = executable_network.create_infer_request();
    }
}
