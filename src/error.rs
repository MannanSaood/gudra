use std::fmt;

use crate::Shape2D;

/// A geometry, input, or device-operation failure.
///
/// Backend diagnostics retain their operation and original display text; that
/// text is not a stable protocol. Floating-point NaN/overflow is not an error.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Resource limits must be positive and per-buffer bytes must fit the total.
    InvalidResourceLimits,
    /// A session admission budget rejected work before backend submission.
    ResourceLimit {
        /// Logical resource whose budget would be exceeded.
        resource: &'static str,
        /// Configured inclusive maximum.
        maximum: usize,
    },
    /// An interior axis was zero.
    EmptyInterior {
        /// `height` or `width`.
        axis: &'static str,
    },
    /// Shape arithmetic overflowed `usize`.
    ShapeOverflow {
        /// Arithmetic being evaluated.
        operation: &'static str,
    },
    /// A representable quantity exceeded the supported domain.
    ShapeLimit {
        /// The constrained quantity.
        quantity: &'static str,
        /// Requested value.
        actual: usize,
        /// Inclusive limit.
        maximum: usize,
    },
    /// A host slice or vector has the wrong number of elements.
    LengthMismatch {
        /// Buffer role.
        buffer: &'static str,
        /// Required length.
        expected: usize,
        /// Supplied length.
        actual: usize,
    },
    /// A field has different two-dimensional extents from the source.
    ShapeMismatch {
        /// Buffer role.
        buffer: &'static str,
        /// Source interior geometry.
        expected: Shape2D,
        /// Supplied field geometry.
        actual: Shape2D,
    },
    /// A buffer belongs to another executor, even on the same device ordinal.
    ContextMismatch {
        /// Buffer role.
        buffer: &'static str,
    },
    /// A prior backend execution did not establish completion for this field.
    /// It can be dropped or queried for shape, but never read or reused.
    InvalidBuffer {
        /// Attempted role (`rhs`, `output`, or `field` for readback).
        buffer: &'static str,
    },
    /// A coefficient is nonfinite, or `h_squared` is negative.
    InvalidParameter {
        /// Coefficient name.
        parameter: &'static str,
        /// Rejected value.
        value: f32,
    },
    /// Device initialization, allocation, copy, view, or execution failed.
    Backend {
        /// Gudra operation label.
        operation: &'static str,
        /// Original backend diagnostic, or a private metadata invariant failure.
        message: String,
    },
}

/// Result of a Gudra operation.
pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidResourceLimits => write!(
                f,
                "resource limits must be positive and buffer bytes must fit total bytes"
            ),
            Self::ResourceLimit { resource, maximum } => {
                write!(f, "{resource} budget exceeded (maximum {maximum})")
            }
            Self::InvalidBuffer { buffer } => write!(
                f,
                "{buffer} is unavailable after an incomplete backend operation; drop it"
            ),
            Self::EmptyInterior { axis } => write!(f, "interior {axis} must be positive"),
            Self::ShapeOverflow { operation } => {
                write!(f, "shape arithmetic overflow: {operation}")
            }
            Self::ShapeLimit {
                quantity,
                actual,
                maximum,
            } => {
                write!(f, "{quantity} is {actual}; supported maximum is {maximum}")
            }
            Self::LengthMismatch {
                buffer,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "{buffer} length is {actual}; expected {expected} elements"
                )
            }
            Self::ShapeMismatch {
                buffer,
                expected,
                actual,
            } => write!(
                f,
                "{buffer} shape is {}x{}; expected {}x{}",
                actual.height(),
                actual.width(),
                expected.height(),
                expected.width()
            ),
            Self::ContextMismatch { buffer } => {
                write!(
                    f,
                    "{buffer} belongs to a different Gpu; upload it with this executor"
                )
            }
            Self::InvalidParameter { parameter, value } => write!(
                f,
                "invalid {parameter}={value}; coefficients must be finite and h_squared nonnegative"
            ),
            Self::Backend { operation, message } => write!(f, "{operation}: {message}"),
        }
    }
}

impl std::error::Error for Error {}

pub(crate) fn check_len(buffer: &'static str, expected: usize, actual: usize) -> Result<()> {
    if expected != actual {
        return Err(Error::LengthMismatch {
            buffer,
            expected,
            actual,
        });
    }
    Ok(())
}
