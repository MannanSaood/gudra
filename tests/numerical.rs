use gudra::{reference::jacobi_into, Error, JacobiParams, Shape2D};

mod support;
use support::{assert_close, fixture, nonfinite_cases, SHAPES};

#[test]
fn oracle_matches_independent_f64_windows_over_shape_and_parameter_matrix() {
    for &(height, width) in SHAPES {
        let shape = Shape2D::new(height, width).unwrap();
        let (halo, rhs) = fixture(shape, 0x5eed_0005);
        for (omega, h2) in [
            (-2.0, 0.25),
            (0.0, 0.0),
            (0.5, 1.0),
            (1.0, 0.25),
            (2.0, 0.5),
        ] {
            let mut output = vec![f32::NAN; shape.interior_len()];
            jacobi_into(
                shape,
                &halo,
                &rhs,
                &mut output,
                JacobiParams::new(omega, h2).unwrap(),
            )
            .unwrap();
            // Different indexing formulation and precision from src/reference.rs.
            let rows: Vec<_> = halo.chunks_exact(width + 2).collect();
            let mut expected = Vec::new();
            for (r, triple) in rows.windows(3).enumerate() {
                for (c, middle) in triple[1].windows(3).enumerate() {
                    let sum = f64::from(triple[0][c + 1])
                        + f64::from(triple[2][c + 1])
                        + f64::from(middle[2])
                        + f64::from(middle[0]);
                    let candidate = (sum - f64::from(h2) * f64::from(rhs[r * width + c])) / 4.0;
                    let value = (1.0 - f64::from(omega)) * f64::from(middle[1])
                        + f64::from(omega) * candidate;
                    #[allow(clippy::cast_possible_truncation)]
                    // Deliberate f64 oracle -> f32 comparison.
                    expected.push(value as f32);
                }
            }
            assert_close(&output, &expected);
        }
    }
}

#[test]
fn deterministic_properties_cover_tile_boundaries_and_quadratic_solution() {
    // All 1..=65 axes: includes both sides of four tile boundaries.
    for height in 1_u16..=65 {
        for width in 1_u16..=65 {
            let shape = Shape2D::new(usize::from(height), usize::from(width)).unwrap();
            assert_eq!(
                shape.interior_len(),
                usize::from(height) * usize::from(width)
            );
            assert_eq!(
                shape.haloed_len(),
                usize::from(height + 2) * usize::from(width + 2)
            );
        }
        let width = (height * 37) % 65 + 1;
        let shape = Shape2D::new(usize::from(height), usize::from(width)).unwrap();
        // u(r,c)=r^2+c^2 has neighbor sum 4*u+4, so RHS=4 preserves u.
        let halo: Vec<_> = (0..height + 2)
            .flat_map(|r| (0..width + 2).map(move |c| f32::from(r * r + c * c)))
            .collect();
        let expected: Vec<_> = (1..=height)
            .flat_map(|r| (1..=width).map(move |c| f32::from(r * r + c * c)))
            .collect();
        let mut output = vec![f32::NAN; shape.interior_len()];
        jacobi_into(
            shape,
            &halo,
            &vec![4.0; shape.interior_len()],
            &mut output,
            JacobiParams::new(-2.0, 1.0).unwrap(),
        )
        .unwrap();
        assert_eq!(output, expected);
    }
}

#[test]
fn seeded_zero_weight_preserves_every_finite_center() {
    for seed in 0..64 {
        let shape = Shape2D::new(1 + seed as usize, 1 + (seed * 17 % 67) as usize).unwrap();
        let (halo, rhs) = fixture(shape, seed);
        let mut output = vec![f32::NAN; shape.interior_len()];
        jacobi_into(
            shape,
            &halo,
            &rhs,
            &mut output,
            JacobiParams::new(0.0, 0.25).unwrap(),
        )
        .unwrap();
        for (r, row) in output.chunks_exact(shape.width()).enumerate() {
            assert_eq!(
                row,
                &halo[(r + 1) * (shape.width() + 2) + 1
                    ..(r + 1) * (shape.width() + 2) + 1 + shape.width()]
            );
        }
    }
}

#[test]
fn nonfinite_data_policy_is_explicit_and_zero_does_not_short_circuit() {
    let shape = Shape2D::new(1, 1).unwrap();
    for (halo, rhs, omega, h2, expected) in nonfinite_cases() {
        let mut output = [0.0];
        jacobi_into(
            shape,
            &halo,
            &[rhs],
            &mut output,
            JacobiParams::new(omega, h2).unwrap(),
        )
        .unwrap();
        assert_close(&output, &[expected]);
    }
}

#[test]
fn coefficients_report_the_rejected_parameter() {
    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert!(matches!(
            JacobiParams::new(value, 1.0),
            Err(Error::InvalidParameter {
                parameter: "omega",
                ..
            })
        ));
        assert!(matches!(
            JacobiParams::new(1.0, value),
            Err(Error::InvalidParameter {
                parameter: "h_squared",
                ..
            })
        ));
    }
    assert!(matches!(
        JacobiParams::new(1.0, -f32::MIN_POSITIVE),
        Err(Error::InvalidParameter {
            parameter: "h_squared",
            ..
        })
    ));
    for omega in [-f32::MAX, -2.0, -0.0, 0.0, 1.0, 2.0, f32::MAX] {
        assert!(JacobiParams::new(omega, -0.0).is_ok());
    }
}

#[test]
fn every_checked_arithmetic_stage_rejects_without_allocation() {
    let max = usize::MAX;
    for (height, width, operation) in [
        (max, 1, "height + 2"),
        (1, max, "width + 2"),
        (max / 2, 3, "height * width"),
        (max / 3, 1, "halo height * halo width"),
        (max / 8, 3, "interior bytes"),
        (max / 16, 2, "haloed bytes"),
    ] {
        assert!(
            matches!(Shape2D::new(height, width), Err(Error::ShapeOverflow { operation: actual }) if actual == operation),
            "{height}x{width}: expected {operation}"
        );
    }
    assert!(matches!(
        Shape2D::new(max / 16, 1),
        Err(Error::ShapeLimit {
            quantity: "haloed bytes",
            ..
        })
    ));
    for (h, w, axis) in [(0, 0, "height"), (0, max, "height"), (max, 0, "width")] {
        assert!(
            matches!(Shape2D::new(h, w), Err(Error::EmptyInterior { axis: actual }) if actual == axis)
        );
    }
}

#[test]
fn short_and_long_buffers_fail_atomically() {
    let shape = Shape2D::new(2, 3).unwrap();
    for (s, r, o, role) in [
        (19, 6, 6, "source"),
        (21, 6, 6, "source"),
        (20, 5, 6, "rhs"),
        (20, 7, 6, "rhs"),
        (20, 6, 5, "output"),
        (20, 6, 7, "output"),
    ] {
        let mut output = vec![123.0; o];
        assert!(
            matches!(jacobi_into(shape, &vec![1.0; s], &vec![0.0; r], &mut output, JacobiParams::new(0.5, 1.0).unwrap()), Err(Error::LengthMismatch { buffer, .. }) if buffer == role)
        );
        assert_eq!(output, vec![123.0; o]);
    }
}
