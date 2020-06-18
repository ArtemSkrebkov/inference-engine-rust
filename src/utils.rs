extern crate inference_engine_sys_rust as ffi;
use ndarray::{ArrayD, IxDyn, ArrayView};
extern crate num;

use num::{Zero, NumCast, cast};
// TODO make public utils and private utils

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

pub fn convert_layout_from_nhwc_to_nchw<T: NumCast + Copy,
            U: NumCast + Zero + Copy>(input: ArrayView<T, IxDyn>) -> ArrayD<U> {
    let dims = input.dim();
    let width = dims[0];
    let height = dims[1];
    let channels = dims[2];
    let mut output = ArrayD::<U>::zeros(IxDyn(&[1, channels, height, width]));
    for c in 0..channels {
        for h in 0..height {
            for w in 0..width {
                output[[0, c, h, w]] = cast(input[[h, w, c]]).unwrap();
            }
        }
    }
    output
}

pub fn argmax<T: PartialOrd>(v: &[T]) -> Vec<usize> {
    let mut idx = (0..v.len()).collect::<Vec<_>>();
    idx.sort_by(|&i, &j| v[j].partial_cmp(&v[i]).unwrap());
    idx
}

