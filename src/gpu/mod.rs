//! Private device owners for one cuTile Jacobi step (requires `gpu`).
//!
//! All constructors and readbacks block on a private session stream. [`Gpu::step`]
//! allocates and blocks; [`Gpu::step_into`] reuses output and blocks. Only
//! [`Gpu::step_into_async`] submits lazily and returns an owned future. First use
//! can JIT-compile. All operations on one executor share its ordered stream.
//!
//! Buffers retain their session and expose no tensors, pointers, streams, clones,
//! or mutable views. Distinct constructors allocate distinct device storage.
//! An initialized halo can still be stale or mathematically incorrect.
//!
//! ```no_run
//! use gudra::{gpu::Gpu, JacobiParams, Shape2D};
//! let gpu = Gpu::new(0)?;
//! let shape = Shape2D::new(17, 19)?;
//! let source = gpu.upload_halo(shape, vec![1.0; shape.haloed_len()])?;
//! let rhs = gpu.upload_field(shape, vec![0.0; shape.interior_len()])?;
//! let answer = gpu.step(&source, &rhs, JacobiParams::new(0.5, 0.01)?)?
//!     .into_host()?;
//! assert_eq!(answer.len(), 323);
//! # Ok::<(), gudra::Error>(())
//! ```

use std::sync::Arc;

mod cutile_impl;
mod kernels;

/// One CUDA device and one private, ordered execution stream.
///
/// Buffers from separate instances cannot be mixed, even at the same ordinal.
/// Dropping this handle does not invalidate buffers or owned pending work.
pub struct Gpu {
    session: Arc<cutile_impl::Session>,
}

/// A fresh, immutable row-major device source with a one-cell halo on every side.
///
/// The physical shape is `[height + 2, width + 2]`. Halo corners are initialized
/// by the caller but unused by this stencil. This type cannot be an output.
pub struct HaloGrid2D {
    storage: cutile_impl::Buffer,
}

/// A fresh row-major interior device field, used as RHS or output.
///
/// Rust borrowing excludes using the same field for both roles in a step.
/// Storage is private and cannot be cloned or imported.
pub struct Field2D {
    storage: cutile_impl::Buffer,
}

/// The three owned buffers returned after an async step completes successfully.
pub struct StepBuffers {
    /// Unchanged haloed source.
    pub source: HaloGrid2D,
    /// Unchanged right-hand-side field.
    pub rhs: Field2D,
    /// Completed interior result.
    pub output: Field2D,
}
