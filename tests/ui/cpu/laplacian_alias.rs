// error: E0502
use gudra::{reference::laplacian_into, LaplacianParams, Shape2D};
pub fn alias(shape: Shape2D, data: &mut [f32], p: LaplacianParams) {
    laplacian_into(shape, data, data, p).unwrap();
}
