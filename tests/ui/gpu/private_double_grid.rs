// error: E0616
use gudra::gpu::DoubleGrid;
fn alias(grid: DoubleGrid) {
    let _ = grid.current;
}
