use gudra::{reference::laplacian_into, LaplacianParams, Shape2D};
use std::{hint::black_box, time::Instant};
fn main() -> gudra::Result<()> {
    for (height, width) in [(17, 19), (256, 256), (257, 263)] {
        let shape = Shape2D::new(height, width)?;
        let halo: Vec<_> = (0..shape.haloed_len())
            .map(|i| f32::from(u8::try_from(i % 31).unwrap()) / 16.0)
            .collect();
        for h in [0.25, 1.0, 4.0] {
            let p = LaplacianParams::new(h)?;
            let mut out = vec![0.0; shape.interior_len()];
            laplacian_into(shape, &halo, &mut out, p)?;
            let mut samples = Vec::new();
            for _ in 0..7 {
                let start = Instant::now();
                for _ in 0..100 {
                    laplacian_into(shape, black_box(&halo), black_box(&mut out), p)?;
                }
                samples.push(start.elapsed().as_secs_f64() * 1e6 / 100.0);
            }
            samples.sort_by(f64::total_cmp);
            println!("operator=laplacian backend=cpu shape={height}x{width} h_squared={h} median_us={:.3}",samples[3]);
            #[cfg(feature = "gpu")]
            {
                let gpu = gudra::gpu::Gpu::new(0)?;
                let source = gpu.upload_halo(shape, halo.clone())?;
                let mut field = gpu.zeros(shape)?;
                gpu.laplacian_into(&source, &mut field, p)?; // warm JIT
                let mut times = Vec::new();
                for _ in 0..7 {
                    let start = Instant::now();
                    for _ in 0..100 {
                        gpu.laplacian_into(&source, &mut field, p)?;
                    }
                    times.push(start.elapsed().as_secs_f64() * 1e6 / 100.0);
                }
                times.sort_by(f64::total_cmp);
                let actual = field.into_host()?;
                for (&a, &e) in actual.iter().zip(&out) {
                    assert!(a.is_finite() && (a - e).abs() <= 1e-5 + 1e-5 * e.abs());
                }
                println!("operator=laplacian backend=gpu_blocking shape={height}x{width} h_squared={h} median_us={:.3}",times[3]);
            }
        }
    }
    Ok(())
}
