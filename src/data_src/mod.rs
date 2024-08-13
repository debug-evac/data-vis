use std::ffi::{c_int, c_float, c_void};

extern "C" {
    fn gen_tornado(xs: c_int, ys: c_int, zs: c_int, time: c_int, tornado: *mut c_float) -> c_void;
}