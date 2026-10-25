use crate::{Error, Result};

/// Finite relaxation factor and finite, nonnegative squared spacing.
///
/// No convergence interval is imposed on `omega`; zero is allowed for both
/// parameters. Tensor values may be any `f32`, including NaN and infinity.
#[derive(Debug, Clone, Copy)]
pub struct JacobiParams {
    pub(crate) omega: f32,
    pub(crate) h_squared: f32,
}

impl JacobiParams {
    /// Validates coefficients without allocating or submitting device work.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidParameter`] for nonfinite coefficients or a
    /// negative `h_squared`. This validation does not prove numerical stability.
    pub fn new(omega: f32, h_squared: f32) -> Result<Self> {
        if !omega.is_finite() {
            return Err(Error::InvalidParameter {
                parameter: "omega",
                value: omega,
            });
        }
        if !h_squared.is_finite() || h_squared < 0.0 {
            return Err(Error::InvalidParameter {
                parameter: "h_squared",
                value: h_squared,
            });
        }
        Ok(Self { omega, h_squared })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coefficient_domain() {
        for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(JacobiParams::new(invalid, 1.0).is_err());
            assert!(JacobiParams::new(1.0, invalid).is_err());
        }
        assert!(JacobiParams::new(1.0, -1.0).is_err());
        assert!(JacobiParams::new(-2.0, 0.0).is_ok());
        assert!(JacobiParams::new(0.0, -0.0).is_ok());
        assert!(JacobiParams::new(2.0, f32::MAX).is_ok());
    }
}

/// Positive finite isotropic squared spacing for the five-point Laplacian.
/// The reciprocal must also be finite; no numerical accuracy bound is implied.
#[derive(Debug, Clone, Copy)]
pub struct LaplacianParams {
    pub(crate) inverse_h_squared: f32,
}

impl LaplacianParams {
    /// Constructs spacing from `h_squared`, in physical length squared units.
    ///
    /// # Errors
    /// Returns [`Error::InvalidParameter`] for nonpositive/nonfinite spacing or
    /// an overflowing reciprocal. Field values may be nonfinite.
    pub fn new(h_squared: f32) -> Result<Self> {
        let inverse_h_squared = 1.0 / h_squared;
        if !h_squared.is_finite() || h_squared <= 0.0 || !inverse_h_squared.is_finite() {
            return Err(Error::InvalidParameter {
                parameter: "laplacian h_squared",
                value: h_squared,
            });
        }
        Ok(Self { inverse_h_squared })
    }
}
