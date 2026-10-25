use gudra::{LaplacianParams, Shape2D};
pub fn apply(
    shape: Shape2D,
    halo: &[f32],
    out: &mut [f32],
    p: LaplacianParams,
) -> gudra::Result<()> {
    gudra::reference::laplacian_into(shape, halo, out, p)?;
    #[cfg(feature = "gpu")]
    {
        let gpu = gudra::gpu::Gpu::new(0)?;
        let source = gpu.upload_halo(shape, halo.to_vec())?;
        let mut field = gpu.zeros(shape)?;
        gpu.laplacian_into(&source, &mut field, p)?;
        let actual = field.into_host()?;
        for (&a, &e) in actual.iter().zip(out.iter()) {
            assert!(a.is_finite() && (a - e).abs() <= 1e-5 + 1e-5 * e.abs());
        }
    }
    Ok(())
}
