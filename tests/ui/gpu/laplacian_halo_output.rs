// error: E0308
use gudra::{gpu::{Gpu, HaloGrid2D}, LaplacianParams};
pub fn bad(gpu: &Gpu, source: &HaloGrid2D, output: &mut HaloGrid2D, p: LaplacianParams) {
    gpu.laplacian_into(source, output, p).unwrap();
}
