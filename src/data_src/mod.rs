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

    pub fn print_horizontal_values(&self, zi: usize, ci: usize) {
        assert!(zi <= self.zs);
        debug_assert!(ci < 3);

        let xs_3 = self.xs * 3;
        let start = zi * self.ys * xs_3 + ci;
        let mut yi = self.ys;

        while yi > 0 {
            yi -= 1;
            let row = start + yi * xs_3;

            for xi in 0..self.xs {
                print!("{}\t", self.data.get(row + xi * 3).expect("Did not get value!"));
            }
            
            println!("");
        }
    }

    #[cfg(test)]
    pub(crate) fn set_vec(&mut self, vec: Vec<f32>) {
        assert!(self.xs * self.ys * self.zs * 3 == vec.len());

        self.data = vec;
    }
}

pub struct HorizontalSlice {
    data: Vec<f32>,
    xs: usize, ys: usize
}

impl HorizontalSlice {
    fn new(xs: usize, ys: usize) -> Self {
        let mut data = vec![0.0; xs * ys];
        data.shrink_to_fit();

        HorizontalSlice {
            data,
            xs, ys
        }
    }
}

pub struct FlowDataSource {
    array_size: usize,
    cartesian_data_grid_rs: CartesianDataGrid,
    slice: HorizontalSlice
}

impl FlowDataSource {
    fn new(xs: usize, ys: usize, zs: usize) -> Self {
        let (cartesian_data_grid_rs, array_size) = CartesianDataGrid::new(xs, ys, zs);

        let mut c_array: Vec<c_float> = vec![0.0; array_size];
        c_array.shrink_to_fit();

        let slice = HorizontalSlice::new(xs, ys);

        FlowDataSource {
            array_size,
            cartesian_data_grid_rs,
            slice
        }
    }

    pub fn gen_tornado(&mut self, time: i32) {
        unsafe { gen_tornado(
            self.cartesian_data_grid_rs.get_xs() as i32,
            self.cartesian_data_grid_rs.get_ys() as i32,
            self.cartesian_data_grid_rs.get_zs() as i32, time,
            self.cartesian_data_grid_rs.get_ptr()); 
        }
    }

    pub fn get_all_values(&self) -> &CartesianDataGrid {
        &self.cartesian_data_grid_rs
    }

    pub fn get_horizontal_values(&self) -> &HorizontalSlice {
        &self.slice
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_gen_tornado() {
        let mut flowsrc = FlowDataSource::new(16, 16, 16);
        flowsrc.gen_tornado(1);
        flowsrc.cartesian_data_grid_rs.print_horizontal_values(10, 0);
    }
}