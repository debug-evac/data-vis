use std::ffi::{c_int, c_float, c_void};

extern "C" {
    fn gen_tornado(xs: c_int, ys: c_int, zs: c_int, time: c_int, tornado: *mut c_float) -> c_void;
}

pub struct CartesianDataGrid {
    data: Vec<f32>,
    xs: i32, ys: i32, zs: i32
}

impl CartesianDataGrid {
    fn new(xs: i32, ys: i32, zs: i32) -> (Self, usize) {
        let array_size = (xs as usize) * (ys as usize) * (zs as usize) * 3;

        let mut data = vec![];
        data.reserve(array_size);

        (CartesianDataGrid {
            data,
            xs, ys, zs
        }, array_size)
    }

    pub fn get_xs(&self) -> i32 { self.xs }

    pub fn get_ys(&self) -> i32 { self.ys }

    pub fn get_zs(&self) -> i32 { self.zs }
}

pub struct FlowDataSource {
    array_size: usize,
    cartesian_data_grid_c: *mut c_float,
    cartesian_data_grid_rs: CartesianDataGrid,
}

impl FlowDataSource {
    fn new(xs: i32, ys: i32, zs: i32) -> Self {
        let (cartesian_data_grid_rs, array_size) = CartesianDataGrid::new(xs, ys, zs);

        let mut c_array: Vec<c_float> = vec![0.0; array_size];
        c_array.shrink_to_fit();

        let cartesian_data_grid_c = c_array.as_mut_ptr();
        std::mem::forget(c_array);

        FlowDataSource {
            array_size,
            cartesian_data_grid_c,
            cartesian_data_grid_rs
        }
    }

    fn gen_tornado(&mut self, time: i32) {
        unsafe { gen_tornado(
            self.cartesian_data_grid_rs.get_xs(),
            self.cartesian_data_grid_rs.get_ys(),
            self.cartesian_data_grid_rs.get_zs(), time,
            self.cartesian_data_grid_c); 
        }
    }

    fn get_values() -> &Vec<f32> {

    }
}