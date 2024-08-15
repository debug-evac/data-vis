use std::ffi::{c_int, c_float, c_void};

extern "C" {
    fn gen_tornado(xs: c_int, ys: c_int, zs: c_int, time: c_int, tornado: *mut c_float) -> c_void;
}

pub struct CartesianDataGrid {
    data: Vec<f32>,
    xs: usize, ys: usize, zs: usize
}

impl CartesianDataGrid {
    fn new(xs: usize, ys: usize, zs: usize) -> (Self, usize) {
        let array_size = xs * ys * zs * 3;

        let mut data = vec![0.0; array_size];
        data.shrink_to_fit();

        (CartesianDataGrid {
            data,
            xs, ys, zs
        }, array_size)
    }

    pub fn get_xs(&self) -> usize { self.xs }

    pub fn get_ys(&self) -> usize { self.ys }

    pub fn get_zs(&self) -> usize { self.zs }

    fn get_ptr(&mut self) -> *mut c_float { self.data.as_mut_ptr() }

    pub fn print_horizontal_values(&self, y: usize, ic: usize) {
        assert!(y <= self.ys);
        debug_assert!(ic < 3);

        let y_level = y * self.zs * 3 + ic;

        for xi in 0..self.xs {
            let row = y_level + xi * self.ys * self.zs * 3;

            for zi in 0..self.zs {
                print!("{}\t", self.data.get(row + zi * 3).expect("Did not get value!"));
            }
            
            println!("");
        }
    }
}

pub struct FlowDataSource {
    array_size: usize,
    cartesian_data_grid_rs: CartesianDataGrid,
}

impl FlowDataSource {
    fn new(xs: usize, ys: usize, zs: usize) -> Self {
        let (cartesian_data_grid_rs, array_size) = CartesianDataGrid::new(xs, ys, zs);

        let mut c_array: Vec<c_float> = vec![0.0; array_size];
        c_array.shrink_to_fit();

        FlowDataSource {
            array_size,
            cartesian_data_grid_rs
        }
    }

    fn gen_tornado(&mut self, time: i32) {
        unsafe { gen_tornado(
            self.cartesian_data_grid_rs.get_xs() as i32,
            self.cartesian_data_grid_rs.get_ys() as i32,
            self.cartesian_data_grid_rs.get_zs() as i32, time,
            self.cartesian_data_grid_rs.get_ptr()); 
        }
    }

    fn get_values(&self) -> &CartesianDataGrid {
        &self.cartesian_data_grid_rs
    }
}