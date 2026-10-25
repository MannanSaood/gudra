use gudra::{reference::jacobi_into, JacobiParams, Shape2D};

fn main() -> gudra::Result<()> {
    let shape = Shape2D::new(1, 1)?;
    // Rows run north to south; center=4, N=1, S=3, E=2, W=6.
    let source = [0.0, 1.0, 0.0, 6.0, 4.0, 2.0, 0.0, 3.0, 0.0];
    let rhs = [0.0];
    let mut output = [0.0];
    jacobi_into(
        shape,
        &source,
        &rhs,
        &mut output,
        JacobiParams::new(0.5, 1.0)?,
    )?;
    // This dyadic fixture has an exactly representable expected result.
    assert_eq!(output[0].to_bits(), 3.5_f32.to_bits());
    println!("old center={} new center={}", source[4], output[0]);
    Ok(())
}
