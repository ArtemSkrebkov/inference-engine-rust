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

pub fn check_status_with_error_message(status: ffi::IEStatusCode, msg: &str) {
    match status {
        s if s == (ffi::IEStatusCode_GENERAL_ERROR as _) => panic!("GENERAL_ERROR: {}", msg),
        s if s == (ffi::IEStatusCode_UNEXPECTED as _) => panic!("UNEXPECTED: {}", msg),
        s if s == (ffi::IEStatusCode_OK as _) => {},
        s if s == (ffi::IEStatusCode_NOT_FOUND as _) => panic!("NOT_FOUND: {}", msg),
        s => panic!("Unknown return value = {}", s),
    }
}

pub unsafe fn convert_double_pointer_to_vec(data: *mut *mut libc::c_char,
                    len: libc::size_t) -> Result<Vec<String>, std::str::Utf8Error> {
    std::slice::from_raw_parts(data, len)
        .iter()
        .map(|arg| std::ffi::CStr::from_ptr(*arg).to_str().map(ToString::to_string))
    .collect()
}