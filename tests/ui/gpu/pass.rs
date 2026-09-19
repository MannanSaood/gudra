use gudra::{gpu::{Gpu, HaloGrid2D, Field2D}, JacobiParams};

pub fn separate(gpu: &Gpu, source: &HaloGrid2D, rhs: &Field2D, output: &mut Field2D, p: JacobiParams) {
    gpu.step_into(source, rhs, output, p).unwrap();
    // Sequential reuse is legal; rejection tests must keep both borrows alive.
    gpu.step_into(source, rhs, output, p).unwrap();
}

pub fn owned(gpu: &Gpu, source: HaloGrid2D, rhs: Field2D, output: Field2D, p: JacobiParams) {
    drop(gpu.step_into_async(source, rhs, output, p).unwrap());
}
