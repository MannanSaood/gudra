mod support;
use gudra::{reference::jacobi_into, JacobiParams, LaplacianParams, Shape2D};
fn main() -> gudra::Result<()> {
    let shape = Shape2D::new(17, 19)?;
    let stride = shape.width() + 2;
    // Fixed zero Dirichlet samples; solve Laplacian(u)=rhs.
    let mut halo = vec![0.0; shape.haloed_len()];
    let rhs = vec![-1.0; shape.interior_len()];
    let mut out = vec![0.0; shape.interior_len()];
    for _ in 0..500 {
        jacobi_into(shape, &halo, &rhs, &mut out, JacobiParams::new(0.5, 1.0)?)?;
        for row in 0..shape.height() {
            halo[(row + 1) * stride + 1..(row + 1) * stride + 1 + shape.width()]
                .copy_from_slice(&out[row * shape.width()..(row + 1) * shape.width()]);
        }
    }
    support::apply(shape, &halo, &mut out, LaplacianParams::new(1.0)?)?;
    // Host diagnostic only: no disjoint-write claim for this reduction.
    let residual = out
        .iter()
        .zip(&rhs)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0_f32, f32::max);
    assert!(residual < 0.1);
    println!("500 Jacobi steps: max Poisson residual={residual}");
    Ok(())
}
