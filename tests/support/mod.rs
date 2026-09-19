use gudra::Shape2D;

pub const SHAPES: &[(usize, usize)] = &[
    (1, 1),
    (1, 19),
    (19, 1),
    (2, 3),
    (3, 2),
    (15, 17),
    (16, 16),
    (17, 19),
    (32, 48),
    (63, 65),
    (129, 257),
    (257, 263),
];

// Fixed LCG, not a platform RNG. Bounded dyadic inputs make failures replayable.
pub fn fixture(shape: Shape2D, seed: u32) -> (Vec<f32>, Vec<f32>) {
    let mut state = seed;
    let mut value = || {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let bytes = state.to_le_bytes();
        f32::from(i16::from_le_bytes([bytes[2], bytes[3]])) / 8192.0
    };
    let mut halo: Vec<_> = (0..shape.haloed_len()).map(|_| value()).collect();
    let rhs = (0..shape.interior_len()).map(|_| value()).collect();
    // Corners must never contribute, even in partial tiles.
    let stride = shape.width() + 2;
    for i in [0, stride - 1, halo.len() - stride, halo.len() - 1] {
        halo[i] = f32::NAN;
    }
    (halo, rhs)
}

pub fn assert_close(actual: &[f32], expected: &[f32]) {
    assert_eq!(actual.len(), expected.len());
    for (i, (&a, &e)) in actual.iter().zip(expected).enumerate() {
        let agrees = if e.is_nan() {
            a.is_nan() // Payload/sign are not promised.
        } else if e.is_infinite() {
            a.to_bits() == e.to_bits()
        } else {
            a.is_finite() && (a - e).abs() <= 1.0e-5 + 1.0e-5 * e.abs()
        };
        assert!(agrees, "cell {i}: actual={a:?}, expected={e:?}");
    }
}

// Explicit policy fixtures with independently stated expected classifications.
pub fn nonfinite_cases() -> Vec<([f32; 9], f32, f32, f32, f32)> {
    let mut cases = Vec::new();
    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        for index in [1, 3, 4, 5, 7] {
            let mut halo = [1.0; 9];
            halo[index] = value;
            cases.push((halo, 0.0, 0.5, 1.0, value));
        }
        cases.push(([1.0; 9], value, 0.5, 1.0, -value));
        cases.push(([1.0; 9], value, 0.0, 0.0, f32::NAN));
    }
    let mut opposite = [1.0; 9];
    opposite[1] = f32::INFINITY;
    opposite[7] = f32::NEG_INFINITY;
    cases.push((opposite, 0.0, 0.5, 1.0, f32::NAN));
    cases.push(([f32::MAX; 9], 0.0, 0.5, 1.0, f32::INFINITY));
    cases.push(([0.0; 9], f32::MAX, 0.5, f32::MAX, f32::NEG_INFINITY));
    for omega in [-f32::MAX, f32::MAX] {
        cases.push(([0.0; 9], 0.0, omega, f32::MAX, 0.0));
    }
    cases
}
