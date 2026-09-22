//! Admission budgets for service deployments. These are not GPU isolation.

use crate::{Error, Result, Shape2D};

/// Logical device-storage and operation limits for one GPU session.
///
/// These do not include CUDA/JIT overhead or host vectors. An application must
/// also bound input files, process memory, request rate, session count and time.
#[derive(Debug, Clone, Copy)]
pub struct ResourceLimits {
    pub(crate) buffer_bytes: usize,
    pub(crate) total_bytes: usize,
    pub(crate) buffers: usize,
    pub(crate) operations: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            buffer_bytes: 256 * 1024 * 1024,
            total_bytes: 1024 * 1024 * 1024,
            buffers: 64,
            operations: 8,
        }
    }
}

impl ResourceLimits {
    /// Maximum simultaneously admitted operations, including unpolled jobs.
    #[must_use]
    pub fn max_operations(self) -> usize {
        self.operations
    }

    /// Sets positive per-buffer bytes, total bytes, live buffers and operations.
    ///
    /// # Errors
    /// Rejects zero limits and a per-buffer limit larger than the total.
    pub fn new(
        buffer_bytes: usize,
        total_bytes: usize,
        buffers: usize,
        operations: usize,
    ) -> Result<Self> {
        if buffer_bytes == 0
            || total_bytes == 0
            || buffers == 0
            || operations == 0
            || buffer_bytes > total_bytes
        {
            return Err(Error::InvalidResourceLimits);
        }
        Ok(Self {
            buffer_bytes,
            total_bytes,
            buffers,
            operations,
        })
    }

    /// Checks one source and two interior fields before allocating host input.
    ///
    /// This is a static request check; live session usage is checked separately.
    /// # Errors
    /// Returns [`Error::ResourceLimit`] when a buffer or the three-buffer step
    /// would exceed these limits, including arithmetic overflow.
    pub fn check_shape(self, shape: Shape2D) -> Result<()> {
        check("buffer bytes", shape.haloed_bytes(), self.buffer_bytes)?;
        let total = shape
            .interior_bytes()
            .checked_mul(2)
            .and_then(|n| n.checked_add(shape.haloed_bytes()))
            .ok_or(Error::ResourceLimit {
                resource: "device bytes",
                maximum: self.total_bytes,
            })?;
        check("device bytes", total, self.total_bytes)?;
        check("live buffers", 3, self.buffers)
    }
}

fn check(resource: &'static str, amount: usize, maximum: usize) -> Result<()> {
    if amount > maximum {
        Err(Error::ResourceLimit { resource, maximum })
    } else {
        Ok(())
    }
}

#[cfg(any(feature = "gpu", test))]
pub(crate) mod admission {
    use super::{check, ResourceLimits};
    use crate::{Error, Result};
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct Usage {
        bytes: usize,
        buffers: usize,
        operations: usize,
    }

    pub(crate) struct Budget {
        limits: ResourceLimits,
        usage: Mutex<Usage>,
    }

    // No Clone: exactly one guard releases each reservation. Forgetting a guard
    // retains the charge, just as forgetting its associated owner leaks storage.
    pub(crate) struct Reservation {
        budget: Arc<Budget>,
        bytes: usize,
        buffer: bool,
    }

    impl Budget {
        pub(crate) fn new(limits: ResourceLimits) -> Arc<Self> {
            Arc::new(Self {
                limits,
                usage: Mutex::new(Usage::default()),
            })
        }

        pub(crate) fn buffer(self: &Arc<Self>, bytes: usize) -> Result<Reservation> {
            check("buffer bytes", bytes, self.limits.buffer_bytes)?;
            let mut usage = self
                .usage
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let total = usage.bytes.checked_add(bytes).ok_or(Error::ResourceLimit {
                resource: "device bytes",
                maximum: self.limits.total_bytes,
            })?;
            check("device bytes", total, self.limits.total_bytes)?;
            // Checking before increment also excludes usize overflow.
            if usage.buffers >= self.limits.buffers {
                return Err(Error::ResourceLimit {
                    resource: "live buffers",
                    maximum: self.limits.buffers,
                });
            }
            usage.bytes = total;
            usage.buffers += 1;
            Ok(Reservation {
                budget: Arc::clone(self),
                bytes,
                buffer: true,
            })
        }

        pub(crate) fn operation(self: &Arc<Self>) -> Result<Reservation> {
            let mut usage = self
                .usage
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if usage.operations >= self.limits.operations {
                return Err(Error::ResourceLimit {
                    resource: "operations",
                    maximum: self.limits.operations,
                });
            }
            usage.operations += 1;
            Ok(Reservation {
                budget: Arc::clone(self),
                bytes: 0,
                buffer: false,
            })
        }
    }

    impl Drop for Reservation {
        fn drop(&mut self) {
            let mut usage = self
                .budget
                .usage
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if self.buffer {
                usage.bytes -= self.bytes;
                usage.buffers -= 1;
            } else {
                usage.operations -= 1;
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn rejects_huge_requests_without_allocating_and_releases_reservations() {
            let budget = Budget::new(ResourceLimits::new(16, 24, 3, 1).unwrap());
            assert!(budget.buffer(usize::MAX).is_err());
            let first = budget.buffer(16).unwrap();
            assert!(budget.buffer(16).is_err());
            let second = budget.buffer(8).unwrap();
            drop(first);
            let third = budget.buffer(16).unwrap();
            drop((second, third));
            let job = budget.operation().unwrap();
            assert!(budget.operation().is_err());
            drop(job);
            assert!(budget.operation().is_ok());
        }

        #[test]
        fn count_limit_and_overflow_fail_closed() {
            let budget = Budget::new(ResourceLimits::new(usize::MAX, usize::MAX, 1, 1).unwrap());
            let held = budget.buffer(usize::MAX).unwrap();
            assert!(budget.buffer(1).is_err());
            drop(held);
            let held = budget.buffer(1).unwrap();
            assert!(matches!(
                budget.buffer(1),
                Err(Error::ResourceLimit {
                    resource: "live buffers",
                    ..
                })
            ));
            drop(held);
        }

        #[test]
        fn concurrent_admission_is_atomic() {
            let budget = Budget::new(ResourceLimits::new(8, 8, 1, 1).unwrap());
            let barrier = Arc::new(std::sync::Barrier::new(8));
            let threads: Vec<_> = (0..8)
                .map(|_| {
                    let budget = Arc::clone(&budget);
                    let barrier = Arc::clone(&barrier);
                    std::thread::spawn(move || {
                        barrier.wait();
                        let reservation = budget.buffer(8);
                        barrier.wait();
                        reservation.is_ok()
                    })
                })
                .collect();
            assert_eq!(
                threads
                    .into_iter()
                    .map(|t| usize::from(t.join().unwrap()))
                    .sum::<usize>(),
                1
            );
            assert!(budget.buffer(8).is_ok());
        }

        #[test]
        fn unwind_releases_and_forget_retains_charge() {
            let budget = Budget::new(ResourceLimits::new(8, 8, 1, 1).unwrap());
            let _ = std::panic::catch_unwind(|| {
                let _guard = budget.operation().unwrap();
                panic!("injected failure before submission");
            });
            assert!(budget.operation().is_ok());
            std::mem::forget(budget.operation().unwrap());
            assert!(budget.operation().is_err());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_zero_limits_and_large_shape_without_allocation() {
        for args in [
            (0, 1, 1, 1),
            (1, 0, 1, 1),
            (1, 1, 0, 1),
            (1, 1, 1, 0),
            (2, 1, 1, 1),
        ] {
            assert!(ResourceLimits::new(args.0, args.1, args.2, args.3).is_err());
        }
        assert!(ResourceLimits::default()
            .check_shape(Shape2D::new(32768, 32768).unwrap())
            .is_err());
        ResourceLimits::default()
            .check_shape(Shape2D::new(17, 19).unwrap())
            .unwrap();
    }
}
