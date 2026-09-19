use gudra::{gpu::{Gpu, HaloGrid2D, Field2D}, JacobiParams};

pub fn reuse(gpu: &Gpu, source: HaloGrid2D, rhs: Field2D, out: Field2D, p: JacobiParams) {
    let future = gpu.step_into_async(source, rhs, out, p).unwrap();
    let _ = out.into_host(); // error: E0382
    drop(future);
}
