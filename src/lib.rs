//! One alias-safe, weighted five-point Jacobi step on a haloed 2D grid.
//!
//! Rows increase southward; columns increase eastward. The caller supplies an
//! initialized one-cell halo. The update is
//! `candidate = 0.25 * (north + south + east + west - h_squared * rhs)`, then
//! `output = (1 - omega) * center + omega * candidate`.
//!
//! [`Shape2D`], [`JacobiParams`], and [`mod@reference`] work without CUDA. Enable
//! `gpu` for private device owners and blocking/owned-async execution. Safe
//! callers cannot alias read inputs with mutable output or overlap output tiles.
//! This does not establish halo freshness, convergence, or correctness of the
//! underlying compiler, driver, or hardware. There is no CPU fallback or solver.

#![deny(missing_docs)]

mod error;
mod params;
mod shape;

pub mod reference;

#[cfg(feature = "gpu")]
pub mod gpu;

pub use error::{Error, Result};
pub use params::JacobiParams;
pub use shape::Shape2D;

/// Computes the CPU reference for an elementwise vector addition.
///
/// Returns `None` when the inputs have different lengths.
#[must_use]
pub fn vector_add_reference(left: &[f32], right: &[f32]) -> Option<Vec<f32>> {
    (left.len() == right.len()).then(|| {
        left.iter()
            .zip(right)
            .map(|(left_value, right_value)| left_value + right_value)
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::vector_add_reference;

    #[test]
    fn vector_adds_equal_length_inputs() {
        assert_eq!(
            vector_add_reference(&[1.0, 2.0], &[3.0, 4.0]),
            Some(vec![4.0, 6.0])
        );
    }

    #[test]
    fn rejects_mismatched_lengths() {
        assert_eq!(vector_add_reference(&[1.0], &[2.0, 3.0]), None);
    }
}
