// Exact dyadic analytic fixtures deliberately require exact values.
#![allow(clippy::float_cmp)]
use gudra::{reference::laplacian_into, Error, LaplacianParams, Shape2D};

#[test]
fn spacing_domain() {
    for h in [
        0.0,
        -0.0,
        -1.0,
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
        f32::from_bits(1),
    ] {
        assert!(LaplacianParams::new(h).is_err());
    }
    for h in [0.25, 1.0, 4.0, f32::MAX] {
        assert!(LaplacianParams::new(h).is_ok());
    }
}

#[test]
fn asymmetric_halo_sign_spacing_and_nonfinite_policy() {
    let shape = Shape2D::new(1, 1).unwrap();
    let mut halo = [
        f32::NAN,
        2.0,
        f32::NAN,
        3.0,
        10.0,
        5.0,
        f32::NAN,
        7.0,
        f32::NAN,
    ];
    let p = LaplacianParams::new(4.0).unwrap();
    let mut out = [0.0];
    laplacian_into(shape, &halo, &mut out, p).unwrap();
    assert_eq!(out, [-5.75]);
    halo[4] = f32::INFINITY;
    laplacian_into(shape, &halo, &mut out, p).unwrap();
    assert_eq!(out, [f32::NEG_INFINITY]);
    halo[1] = f32::INFINITY;
    laplacian_into(shape, &halo, &mut out, p).unwrap();
    assert!(out[0].is_nan());
}

#[test]
fn manufactured_quadratic_constant_and_affine_on_partial_tiles() {
    for (height, width) in [
        (1, 1),
        (1, 19),
        (19, 1),
        (2, 3),
        (16, 16),
        (17, 19),
        (63, 65),
    ] {
        let shape = Shape2D::new(height, width).unwrap();
        let stride = width + 2;
        for quadratic in [false, true] {
            let mut halo = vec![0.0; shape.haloed_len()];
            for row in 0..height + 2 {
                for col in 0..width + 2 {
                    let y = f32::from(u16::try_from(row).unwrap());
                    let x = f32::from(u16::try_from(col).unwrap());
                    halo[row * stride + col] = if quadratic {
                        x * x + 2.0 * y * y
                    } else {
                        3.0 * x - 2.0 * y + 7.0
                    };
                }
            }
            for i in [0, stride - 1, halo.len() - stride, halo.len() - 1] {
                halo[i] = f32::NAN;
            }
            let mut out = vec![0.0; shape.interior_len()];
            laplacian_into(shape, &halo, &mut out, LaplacianParams::new(0.25).unwrap()).unwrap();
            assert!(out.iter().all(|&v| v == if quadratic { 24.0 } else { 0.0 }));
            laplacian_into(
                shape,
                &vec![2.0; shape.haloed_len()],
                &mut out,
                LaplacianParams::new(1.0).unwrap(),
            )
            .unwrap();
            assert!(out.iter().all(|&v| v == 0.0));
        }
    }
}

#[test]
fn wrong_lengths_are_atomic() {
    let shape = Shape2D::new(2, 3).unwrap();
    for (source_len, out_len, role) in [
        (19, 6, "source"),
        (21, 6, "source"),
        (20, 5, "output"),
        (20, 7, "output"),
    ] {
        let mut out = vec![123.0; out_len];
        let result = laplacian_into(
            shape,
            &vec![0.0; source_len],
            &mut out,
            LaplacianParams::new(1.0).unwrap(),
        );
        assert!(matches!(result,Err(Error::LengthMismatch{buffer,..}) if buffer==role));
        assert!(out.iter().all(|&v| v == 123.0));
    }
}

#[test]
fn seeded_oracle_matches_independent_f64_windows() {
    for (height, width) in [(1, 19), (19, 1), (17, 19), (63, 65)] {
        let shape = Shape2D::new(height, width).unwrap();
        let stride = width + 2;
        let halo: Vec<_> = (0..shape.haloed_len())
            .map(|i| f32::from(u8::try_from((i * 17) % 127).unwrap()) / 16.0)
            .collect();
        for h in [0.25, 1.0, 4.0] {
            let mut out = vec![0.0; shape.interior_len()];
            laplacian_into(shape, &halo, &mut out, LaplacianParams::new(h).unwrap()).unwrap();
            let expected: Vec<_> = halo
                .windows(2 * stride + 3)
                .take(height * stride)
                .enumerate()
                .filter(|(i, _)| i % stride < width)
                .map(|(_, w)| {
                    (f64::from(w[1])
                        + f64::from(w[2 * stride + 1])
                        + f64::from(w[stride + 2])
                        + f64::from(w[stride])
                        - 4.0 * f64::from(w[stride + 1]))
                        / f64::from(h)
                })
                .collect();
            assert_eq!(out.len(), expected.len());
            for (&a, &e) in out.iter().zip(&expected) {
                assert_eq!(f64::from(a), e);
            }
        }
    }
}
