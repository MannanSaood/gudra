use gudra::{gpu::{Gpu, HaloGrid2D, Field2D}, JacobiParams};

pub fn in_place(gpu: &Gpu, mut source: HaloGrid2D, rhs: &Field2D, p: JacobiParams) {
    gpu.step_into(&source, rhs, &mut source, p).unwrap(); // error: E0308
}
