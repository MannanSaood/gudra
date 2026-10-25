use gudra::{gpu::{Gpu, HaloGrid2D, Field2D}, JacobiParams};

pub fn separate(gpu: &Gpu, source: &HaloGrid2D, rhs: &Field2D, output: &mut Field2D, p: JacobiParams) {
    gpu.step_into(source, rhs, output, p).unwrap();
    // Sequential reuse is legal; rejection tests must keep both borrows alive.
    gpu.step_into(source, rhs, output, p).unwrap();
}

pub fn owned(gpu: &Gpu, source: HaloGrid2D, rhs: Field2D, output: Field2D, p: JacobiParams) {
    drop(gpu.step_into_async(source, rhs, output, p).unwrap());
}

pub fn timesteps(grid: gudra::gpu::DoubleGrid, p: JacobiParams) {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<gudra::gpu::DoubleGrid>();
    drop(grid.advance_async(100, p).unwrap());
}

pub fn laplacian(gpu: &Gpu, source: &HaloGrid2D, out: &mut Field2D, p: gudra::LaplacianParams) {
    gpu.laplacian_into(source, out, p).unwrap();
}
