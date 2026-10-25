#[allow(dead_code)] // Other fixtures are exercised by gpu_jacobi.
mod support;
use gudra::{gpu::Gpu, reference::laplacian_into, Error, LaplacianParams, Shape2D};

#[test]
fn matches_oracle_across_shapes_and_spacing() {
    let gpu = Gpu::new(0).unwrap();
    for &(height, width) in support::SHAPES {
        let shape = Shape2D::new(height, width).unwrap();
        let (halo, _) = support::fixture(shape, 42);
        let source = gpu.upload_halo(shape, halo.clone()).unwrap();
        for h in [0.25, 1.0, 4.0] {
            let p = LaplacianParams::new(h).unwrap();
            let mut expected = vec![0.0; shape.interior_len()];
            laplacian_into(shape, &halo, &mut expected, p).unwrap();
            let mut out = gpu.zeros(shape).unwrap();
            gpu.laplacian_into(&source, &mut out, p).unwrap();
            support::assert_close(&out.into_host().unwrap(), &expected);
        }
    }
}

#[test]
fn validation_leaves_output_unchanged() {
    let gpu = Gpu::new(0).unwrap();
    let other = Gpu::new(0).unwrap();
    let shape = Shape2D::new(2, 3).unwrap();
    let source = gpu
        .upload_halo(shape, vec![1.0; shape.haloed_len()])
        .unwrap();
    let p = LaplacianParams::new(1.0).unwrap();
    let mut out = gpu
        .upload_field(Shape2D::new(3, 2).unwrap(), vec![123.0; 6])
        .unwrap();
    assert!(matches!(
        gpu.laplacian_into(&source, &mut out, p),
        Err(Error::ShapeMismatch { .. })
    ));
    assert!(out
        .into_host()
        .unwrap()
        .iter()
        .all(|v| v.to_bits() == 123.0_f32.to_bits()));
    let mut out = other.upload_field(shape, vec![123.0; 6]).unwrap();
    assert!(matches!(
        gpu.laplacian_into(&source, &mut out, p),
        Err(Error::ContextMismatch { .. })
    ));
    assert!(out
        .into_host()
        .unwrap()
        .iter()
        .all(|v| v.to_bits() == 123.0_f32.to_bits()));
}

#[test]
fn nonfinite_classification_matches_oracle() {
    let gpu = Gpu::new(0).unwrap();
    let shape = Shape2D::new(1, 1).unwrap();
    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        for index in [1, 3, 4, 5, 7] {
            let mut halo = vec![1.0; 9];
            halo[index] = value;
            let p = LaplacianParams::new(1.0).unwrap();
            let mut expected = [0.0];
            laplacian_into(shape, &halo, &mut expected, p).unwrap();
            let source = gpu.upload_halo(shape, halo).unwrap();
            let mut out = gpu.zeros(shape).unwrap();
            gpu.laplacian_into(&source, &mut out, p).unwrap();
            support::assert_close(&out.into_host().unwrap(), &expected);
        }
    }
}
