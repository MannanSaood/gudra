use gudra::{reference::jacobi_into, JacobiParams, Shape2D};

pub fn separate(shape: Shape2D, halo: &[f32], rhs: &[f32], out: &mut [f32], p: JacobiParams) {
    jacobi_into(shape, halo, rhs, out, p).unwrap();
}
