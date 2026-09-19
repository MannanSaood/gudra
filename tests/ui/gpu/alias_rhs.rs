use gudra::{gpu::{Gpu, HaloGrid2D, Field2D}, JacobiParams};

pub fn alias(gpu: &Gpu, source: &HaloGrid2D, mut rhs: Field2D, p: JacobiParams) {
    // step_into internally partitions this output; the same allocation is read.
    gpu.step_into(source, &rhs, &mut rhs, p).unwrap(); // error: E0502
}
