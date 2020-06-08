extern crate inference_engine_sys_rust as ffi;

pub fn check_status(status: ffi::IEStatusCode) {
    match status {
        s if s == (ffi::IEStatusCode_GENERAL_ERROR as _) => panic!("GENERAL_ERROR"),
        s if s == (ffi::IEStatusCode_UNEXPECTED as _) => panic!("UNEXPECTED"),
        s if s == (ffi::IEStatusCode_OK as _) => {},
        s if s == (ffi::IEStatusCode_NOT_FOUND as _) => panic!("NOT_FOUND"),
        s => panic!("Unknown return value = {}", s),
    }
}