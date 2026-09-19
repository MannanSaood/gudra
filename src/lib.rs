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

#[cfg(any(feature = "gpu", test))]
mod completion;
mod error;
mod limits;
mod params;
mod shape;

pub mod reference;

#[cfg(feature = "gpu")]
pub mod gpu;

pub use error::{Error, Result};
pub use limits::ResourceLimits;
pub use params::JacobiParams;
pub use shape::Shape2D;
