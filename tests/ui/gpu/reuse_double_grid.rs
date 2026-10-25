// error: E0382
use gudra::{gpu::DoubleGrid, JacobiParams};
fn reuse(grid: DoubleGrid, params: JacobiParams) {
    let pending = grid.advance_async(100, params).unwrap();
    let _ = grid.observe();
    drop(pending);
}
