use gudra::{reference::jacobi_into, Error, JacobiParams, Shape2D};

#[test]
fn one_cell_uses_all_four_halos_and_negative_rhs_term() {
    let shape = Shape2D::new(1, 1).unwrap();
    let halo = [
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
    let mut output = [0.0];
    jacobi_into(
        shape,
        &halo,
        &[2.0],
        &mut output,
        JacobiParams::new(0.5, 4.0).unwrap(),
    )
    .unwrap();
    // (2 + 7 + 5 + 3 - 4*2)/4 = 2.25; 0.5*10 + 0.5*2.25 = 6.125.
    // These dyadic values are exactly representable; require the exact bits.
    assert_eq!(output.map(f32::to_bits), [6.125_f32].map(f32::to_bits));
}

#[test]
fn rectangular_row_major_indexing_and_weighting() {
    let shape = Shape2D::new(2, 3).unwrap();
    let halo = [
        100.0, 1.0, 2.0, 3.0, 200.0, 4.0, 10.0, 20.0, 30.0, 5.0, 6.0, 40.0, 50.0, 60.0, 7.0, 300.0,
        8.0, 9.0, 11.0, 400.0,
    ];
    let mut output = [0.0; 6];
    jacobi_into(
        shape,
        &halo,
        &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
        &mut output,
        JacobiParams::new(1.0, 2.0).unwrap(),
    )
    .unwrap();
    assert_eq!(
        output.map(f32::to_bits),
        [15.75_f32, 22.0, 20.5, 16.5, 29.75, 21.5].map(f32::to_bits)
    );
}

#[test]
fn wrong_lengths_leave_output_untouched() {
    let shape = Shape2D::new(1, 1).unwrap();
    let p = JacobiParams::new(0.5, 1.0).unwrap();
    for (halo, rhs, output_len, role) in
        [(8, 1, 1, "source"), (9, 2, 1, "rhs"), (9, 1, 2, "output")]
    {
        let mut output = vec![123.0; output_len];
        let result = jacobi_into(shape, &vec![1.0; halo], &vec![0.0; rhs], &mut output, p);
        assert!(matches!(result, Err(Error::LengthMismatch { buffer, .. }) if buffer == role));
        assert_eq!(output, vec![123.0; output_len]);
    }
}

#[test]
fn zero_weight_does_not_hide_nan() {
    let shape = Shape2D::new(1, 1).unwrap();
    let mut output = [0.0];
    jacobi_into(
        shape,
        &[1.0; 9],
        &[f32::NAN],
        &mut output,
        JacobiParams::new(0.0, 0.0).unwrap(),
    )
    .unwrap();
    assert!(output[0].is_nan());
}

#[test]
fn narrow_and_partial_domains_preserve_constant_field() {
    for (height, width) in [(1, 1), (1, 19), (19, 1), (16, 16), (17, 19)] {
        let shape = Shape2D::new(height, width).unwrap();
        let mut output = vec![0.0; shape.interior_len()];
        jacobi_into(
            shape,
            &vec![4.0; shape.haloed_len()],
            &vec![0.0; shape.interior_len()],
            &mut output,
            JacobiParams::new(0.5, 0.25).unwrap(),
        )
        .unwrap();
        assert_eq!(output, vec![4.0; shape.interior_len()]);
    }
}
