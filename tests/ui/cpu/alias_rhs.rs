use gudra::{reference::jacobi_into, JacobiParams, Shape2D};

pub fn alias(shape: Shape2D, halo: &[f32], mut rhs: Vec<f32>, p: JacobiParams) {
    jacobi_into(shape, halo, &rhs, &mut rhs, p).unwrap(); // error: E0502
}
