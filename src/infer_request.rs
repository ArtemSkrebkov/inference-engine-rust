use ndarray::{Array, ArrayD, ArrayViewMut, IxDyn};
use std::ffi::CString;
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

pub struct InferRequest {
    // TODO: encapsulate this by introducing factory method in executable network
    // for creation of infer request
    // question: how to call a private contructor from another module? 
    pub ie_infer_request: Box<*mut ffi::ie_infer_request_t>,
}

impl InferRequest {
    // template instead of hardcoding particular type
    pub fn get_blob(&self, name: &str) -> ArrayViewMut<f32, IxDyn> {
        unsafe {
            let name = CString::new(name).unwrap();
            let name = name.as_ptr();
            let mut ie_blob = Box::<*mut ffi::ie_blob_t>::new(mem::zeroed());

            let status = ffi::ie_infer_request_get_blob(*self.ie_infer_request,
                name, &mut *ie_blob);
            check_status(status);

            let mut byte_size = 0;
            let status = ffi::ie_blob_byte_size(*ie_blob, &mut byte_size);
            check_status(status);
            
            let mut ie_dims = ffi::dimensions_t {
                ranks: 0,
                dims: [0, 0, 0, 0, 0, 0, 0, 0],
            };
            let status = ffi::ie_blob_get_dims(*ie_blob, &mut ie_dims);
            check_status(status);
            let rank = ie_dims.ranks as usize;
            let mut dims = vec![0 as usize; rank];
            for (i, dim) in dims.iter_mut().enumerate() {
                *dim = ie_dims.dims[i] as usize;
            }

            let mut ie_blob_buffer = ffi::ie_blob_buffer {
                __bindgen_anon_1: ffi::ie_blob_buffer__bindgen_ty_1 {
                    buffer: std::ptr::null_mut(),
                }
            };
            let status = ffi::ie_blob_get_buffer(*ie_blob, &mut ie_blob_buffer);
            let buffer = ie_blob_buffer.__bindgen_anon_1.buffer;
            check_status(status);
            let data: &mut [f32] = slice::from_raw_parts_mut(buffer as *mut f32, byte_size as usize);
            ArrayViewMut::from_shape(IxDyn(&dims), data).unwrap()
        }
    }

    pub fn set_blob(&self, name: &str, blob: Array<f32, IxDyn>) {
        unsafe {
            let name = CString::new(name).unwrap();
            let name = name.as_ptr();
            let mut ie_blob = Box::<*mut ffi::ie_blob_t>::new(mem::zeroed());
            let raw_dim = blob.raw_dim();
            let ie_dims = ffi::dimensions_t {
                ranks: blob.ndim() as u64,
                dims: [raw_dim[0] as u64, raw_dim[1] as u64, raw_dim[2] as u64, raw_dim[3] as u64, 0, 0, 0, 0],
            };
            let ie_tensor_desc = ffi::tensor_desc_t {
                layout: ffi::layout_e_NCHW,
                dims: ie_dims,
                precision: ffi::precision_e_FP32,
            };
            let ie_size = (raw_dim[0] * raw_dim[1] * raw_dim[2] * raw_dim[3] * 4) as u64;
            let buffer = blob.into_raw_vec().as_mut_ptr();

            let status = ffi::ie_blob_make_memory_from_preallocated(&ie_tensor_desc, buffer as *mut core::ffi::c_void, ie_size, &mut *ie_blob);
            check_status(status);

            let status = ffi::ie_infer_request_set_blob(*self.ie_infer_request,
                name, *ie_blob);
            check_status(status);
        }
    }

    pub fn infer(&self) {
        let status = unsafe { ffi::ie_infer_request_infer(*self.ie_infer_request) };
        check_status(status);
    }
}