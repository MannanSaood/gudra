use gudra::{gpu::{Gpu, HaloGrid2D, Field2D}, JacobiParams};

pub fn overlap(gpu: &Gpu, source: &HaloGrid2D, rhs: &Field2D, mut out: Field2D, p: JacobiParams) {
    let first = &mut out;
    let second = &mut out; // error: E0499
    gpu.step_into(source, rhs, first, p).unwrap();
    gpu.step_into(source, rhs, second, p).unwrap();
}
