//! Tests private production views directly; an isotropic sum cannot detect a swap.
use super::*;

#[cutile::module]
mod probe {
    // Match the production DSL prelude, including macro-generated tensor types.
    #[allow(clippy::wildcard_imports)]
    use cutile::core::*;

    #[cutile::entry()]
    fn copy_view(output: &mut Tensor<f32, { [16, 16] }>, input: &Tensor<f32, { [-1, -1] }>) {
        let values = input.load_like(output);
        output.store(values);
    }
}

#[test]
fn each_direction_reads_asymmetric_sentinels() -> Result<()> {
    let gpu = Gpu::new(0)?;
    for (height, width) in [(1_u16, 1_u16), (2, 3), (17, 19), (33, 47)] {
        let shape = Shape2D::new(usize::from(height), usize::from(width))?;
        let sentinel = |r: u16, c: u16| f32::from(r) * 4096.0 + f32::from(c) * 3.0;
        let data = (0..height + 2)
            .flat_map(|r| (0..width + 2).map(move |c| sentinel(r, c)))
            .collect();
        let source = gpu.upload_halo(shape, data)?;
        let views = directional_views(&source)?;
        // Compare every labeled direction separately against independent host
        // coordinates. Swapping even north/south fails despite identical sums.
        for (view, row_offset, col_offset) in [
            (&views.center, 1, 1),
            (&views.north, 0, 1),
            (&views.south, 2, 1),
            (&views.east, 1, 2),
            (&views.west, 1, 0),
        ] {
            let mut output = gpu.upload_field(shape, vec![f32::NAN; shape.interior_len()])?;
            {
                let _completed =
                    probe::copy_view((&mut output.storage.tensor).partition([TILE, TILE]), view)
                        .sync_on(&gpu.session.stream)
                        .map_err(|e| backend("probe directional view", e))?;
            }
            let expected: Vec<_> = (0..height)
                .flat_map(|r| (0..width).map(move |c| sentinel(r + row_offset, c + col_offset)))
                .collect();
            assert_eq!(output.into_host()?, expected);
        }
    }
    Ok(())
}

#[test]
fn invalidated_field_is_rejected_by_every_public_data_path() -> Result<()> {
    let gpu = Gpu::new(0)?;
    let shape = Shape2D::new(1, 1)?;
    let p = JacobiParams::new(0.5, 1.0)?;
    let source = gpu.upload_halo(shape, vec![1.0; 9])?;
    let rhs = gpu.zeros(shape)?;
    let mut bad = gpu.zeros(shape)?;
    let mut good = gpu.zeros(shape)?;
    assert!(bad
        .storage
        .completion
        .run(|| Err(backend("injected error", "no launch")))
        .is_err());
    assert_eq!(bad.shape(), shape);
    assert!(matches!(
        gpu.step(&source, &bad, p),
        Err(Error::InvalidBuffer { buffer: "rhs" })
    ));
    assert!(matches!(
        gpu.step_into(&source, &rhs, &mut bad, p),
        Err(Error::InvalidBuffer { buffer: "output" })
    ));
    assert!(matches!(
        gpu.step_into(&source, &bad, &mut good, p),
        Err(Error::InvalidBuffer { buffer: "rhs" })
    ));
    assert!(matches!(
        bad.into_host(),
        Err(Error::InvalidBuffer { buffer: "field" })
    ));
    assert_eq!(good.into_host()?, vec![0.0]);
    let mut bad_async = gpu.zeros(shape)?;
    assert!(bad_async
        .storage
        .completion
        .run(|| Err(backend("injected error", "no launch")))
        .is_err());
    assert!(matches!(
        gpu.step_into_async(source, rhs, bad_async, p),
        Err(Error::InvalidBuffer { buffer: "output" })
    ));
    Ok(())
}
