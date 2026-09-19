//! Scalar CPU oracle for verifying a device step; no automatic CPU fallback.

use crate::{error::check_len, JacobiParams, Result, Shape2D};

/// Writes one weighted Jacobi update into a row-major interior output.
///
/// `haloed` must contain `(height + 2) * (width + 2)` initialized values.
/// RHS/output contain `height * width` values. Edge interior cells read the
/// caller's halo, and the four halo corners are unused. No boundary policy is
/// inferred. Arithmetic is left-associated `north + south + east + west`, then
/// `candidate = 0.25 * (sum - h_squared * rhs)` and
/// `output = (1 - omega) * center + omega * candidate`.
///
/// NaNs and infinities propagate normally, including when a coefficient is zero.
/// GPU contraction/rounding can differ; small finite fixtures use the test
/// tolerance `abs(gpu - cpu) <= 1e-5 + 1e-5 * abs(cpu)`.
///
/// # Errors
///
/// Returns [`crate::Error::LengthMismatch`] before writing any output if any
/// length is incorrect.
///
/// # Example
///
/// ```
/// use gudra::{reference::jacobi_into, JacobiParams, Shape2D};
/// let shape = Shape2D::new(1, 1)?;
/// let mut output = [0.0];
/// jacobi_into(shape, &[1.0; 9], &[0.0], &mut output,
///             JacobiParams::new(0.5, 1.0)?)?;
/// assert_eq!(output, [1.0]);
/// # Ok::<(), gudra::Error>(())
/// ```
pub fn jacobi_into(
    shape: Shape2D,
    haloed: &[f32],
    rhs: &[f32],
    output: &mut [f32],
    params: JacobiParams,
) -> Result<()> {
    check_len("source", shape.haloed_len(), haloed.len())?;
    check_len("rhs", shape.interior_len(), rhs.len())?;
    check_len("output", shape.interior_len(), output.len())?;
    let stride = shape.width() + 2;
    for row in 0..shape.height() {
        for column in 0..shape.width() {
            let index = (row + 1) * stride + column + 1;
            let cell = row * shape.width() + column;
            let candidate = 0.25
                * (haloed[index - stride]
                    + haloed[index + stride]
                    + haloed[index + 1]
                    + haloed[index - 1]
                    - params.h_squared * rhs[cell]);
            output[cell] = (1.0 - params.omega) * haloed[index] + params.omega * candidate;
        }
    }
    Ok(())
}
