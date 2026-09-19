use std::{error::Error, process::ExitCode};

use gudra::{gpu::Gpu, reference::jacobi_into, JacobiParams, Shape2D};

fn run() -> Result<(), Box<dyn Error>> {
    // A rectangular, partial-tile domain. All four edges are deliberately
    // different and nonzero; this example imposes these values explicitly.
    let shape = Shape2D::new(17, 19)?;
    let stride = shape.width() + 2;
    let mut haloed = vec![0.0; shape.haloed_len()];
    for row in 0..shape.height() + 2 {
        for column in 0..shape.width() + 2 {
            let y = f32::from(u16::try_from(row)?);
            let x = f32::from(u16::try_from(column)?);
            haloed[row * stride + column] = if row == 0 {
                1.0 + 0.05 * x
            } else if row == shape.height() + 1 {
                2.0 + 0.03 * x
            } else if column == 0 {
                -3.0 + 0.02 * y
            } else if column == shape.width() + 1 {
                4.0 - 0.04 * y
            } else {
                0.01 * y * y + 0.02 * x
            };
        }
    }
    let rhs: Vec<f32> = [0.0, 0.1, -0.2, 0.3, 0.5, -0.4, 0.2]
        .into_iter()
        .cycle()
        .take(shape.interior_len())
        .collect();
    let params = JacobiParams::new(2.0 / 3.0, 0.01)?;
    let mut expected = vec![0.0; shape.interior_len()];
    jacobi_into(shape, &haloed, &rhs, &mut expected, params)?;

    let gpu = Gpu::new(0)?;
    let source = gpu.upload_halo(shape, haloed)?; // Blocking upload.
    let rhs = gpu.upload_field(shape, rhs)?; // Blocking upload.
    let actual = gpu.step(&source, &rhs, params)?.into_host()?;
    // step allocates/initializes output and waits; into_host consumes and waits.
    if actual.len() != expected.len() {
        return Err(format!(
            "readback length {}, expected {}",
            actual.len(),
            expected.len()
        )
        .into());
    }
    let mut max_error = 0.0_f32;
    for (index, (&got, &want)) in actual.iter().zip(&expected).enumerate() {
        let difference = (got - want).abs();
        if !got.is_finite() || difference > 1.0e-5 + 1.0e-5 * want.abs() {
            return Err(format!(
                "Jacobi mismatch at ({}, {}): GPU={got}, CPU={want}",
                index / shape.width(),
                index % shape.width()
            )
            .into());
        }
        max_error = max_error.max(difference);
    }
    println!(
        "result=PASS shape=17x19 cells={} max_abs_error={max_error:.8}",
        actual.len()
    );
    println!(
        "top_left={:.6} center={:.6} bottom_right={:.6}",
        actual[0],
        actual[8 * shape.width() + 9],
        actual[actual.len() - 1]
    );
    println!("One update only; the caller supplied all halo values. All transfers and the step were blocking.");
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Poisson step failed: {error}");
            eprintln!("Run ./scripts/check-gpu-env.sh and follow docs/setup.md.");
            ExitCode::from(2)
        }
    }
}
