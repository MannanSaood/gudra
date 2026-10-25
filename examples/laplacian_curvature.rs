mod support;
use gudra::{LaplacianParams, Shape2D};
fn main() -> gudra::Result<()> {
    let shape = Shape2D::new(17, 19)?;
    let mut halo = vec![0.0; shape.haloed_len()];
    let stride = shape.width() + 2;
    for row in 0..shape.height() + 2 {
        for col in 0..shape.width() + 2 {
            let x = f32::from(u16::try_from(col).unwrap());
            let y = f32::from(u16::try_from(row).unwrap());
            halo[row * stride + col] = x * x + 2.0 * y * y;
        }
    }
    let mut out = vec![0.0; shape.interior_len()];
    support::apply(shape, &halo, &mut out, LaplacianParams::new(1.0)?)?;
    assert!(out.iter().all(|&v| v.to_bits() == 6.0_f32.to_bits()));
    println!("quadratic curvature: all {} cells equal 6", out.len());
    Ok(())
}
