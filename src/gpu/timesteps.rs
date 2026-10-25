use super::{backend, Buffer, Gpu};
use crate::gpu::{DoubleGrid, Field2D, HaloGrid2D};
use crate::{error::check_len, JacobiParams, Result, Shape2D};
use cutile::{
    cuda_async::device_future::DeviceFuture,
    prelude::{DeviceOp, ExecutionContext, PartitionMut, ToHostVec},
};
use std::{future::Future, sync::Arc};

impl Gpu {
    /// Uploads two independent copies of a haloed grid and one interior RHS.
    /// The initial halo is preserved bit-for-bit at every timestep, including
    /// corners. This implements fixed Dirichlet boundaries only.
    /// Each haloed axis must be at most 65,535 for the scalar-tile baseline.
    ///
    /// # Errors
    /// Rejects lengths, resource limits, and backend failures. Partial
    /// construction releases already completed allocations.
    pub fn double_grid(
        &self,
        shape: Shape2D,
        initial: Vec<f32>,
        rhs: Vec<f32>,
    ) -> Result<DoubleGrid> {
        check_len("source", shape.haloed_len(), initial.len())?;
        check_len("rhs", shape.interior_len(), rhs.len())?;
        for (quantity, actual) in [
            ("timestep row blocks", shape.haloed()[0]),
            ("timestep column blocks", shape.haloed()[1]),
        ] {
            if actual > 65_535 {
                return Err(crate::Error::ShapeLimit {
                    quantity,
                    actual,
                    maximum: 65_535,
                });
            }
        }
        // Reserve all device storage before duplicating grid-sized host data.
        let first = self
            .session
            .budget
            .buffer(shape.haloed_len() * size_of::<f32>())?;
        let second = self
            .session
            .budget
            .buffer(shape.haloed_len() * size_of::<f32>())?;
        let forcing = self.session.budget.buffer(shape.interior_bytes())?;
        let current = HaloGrid2D {
            storage: self.upload_reserved(shape, initial.clone(), &shape.haloed(), first)?,
        };
        let next = HaloGrid2D {
            storage: self.upload_reserved(shape, initial, &shape.haloed(), second)?,
        };
        let rhs = Field2D {
            storage: self.upload_reserved(shape, rhs, &shape.interior(), forcing)?,
        };
        Ok(DoubleGrid {
            current,
            next,
            rhs,
            completed: 0,
        })
    }
}

impl DoubleGrid {
    /// Returns the interior dimensions without accessing device memory.
    #[must_use]
    pub fn shape(&self) -> Shape2D {
        self.current.shape()
    }

    /// Number of successfully completed timesteps since construction.
    #[must_use]
    pub fn completed_steps(&self) -> usize {
        self.completed
    }

    /// Advances using direct blocking launches and returns the next ready state.
    /// No device allocation or host transfer occurs between timesteps.
    ///
    /// # Errors
    /// Admission, counter overflow, or backend failure consumes the entire state;
    /// partially updated storage is never returned. Unprovable completion aborts
    /// under the backend's existing fail-stop policy.
    pub fn advance(mut self, steps: usize, params: JacobiParams) -> Result<Self> {
        self.check_count(steps)?;
        let session = Arc::clone(&self.current.storage.session);
        let _operation = session.budget.operation()?;
        session.bind()?;
        for _ in 0..steps {
            {
                let _completed = self
                    .operation(params)
                    .sync_on(&session.stream)
                    .map_err(|e| backend("execute timestep", e))?;
            }
            self.swap_completed();
        }
        Ok(self)
    }

    /// Creates a lazy, owned future for a batch of timesteps.
    ///
    /// Admission happens immediately. First poll submits the first kernel; each
    /// completion swaps current/next before submitting the following kernel.
    /// All three allocations and the session remain in the future. It is `Send`
    /// and may be polled on another thread. Each submission binds the device.
    /// Drop before polling submits nothing; drop while pending drains the stream
    /// and discards all storage. Cancellation does not undo completed work.
    /// Forgetting a future leaks its owners safely. A failed drain aborts.
    ///
    /// Use bounded batches followed by [`Self::observe`] for periodic sampling.
    /// Zero steps returns unchanged state after admission and polling.
    ///
    /// # Errors
    /// Validation/admission and execution errors consume the state. No partially
    /// updated or unknown state is returned, and there is no retry API.
    ///
    /// ```compile_fail,E0382
    /// use gudra::{gpu::DoubleGrid, JacobiParams};
    /// fn premature(grid: DoubleGrid, p: JacobiParams) {
    ///     let pending = grid.advance_async(100, p).unwrap();
    ///     let _ = grid.observe();
    ///     drop(pending);
    /// }
    /// ```
    pub fn advance_async(
        mut self,
        steps: usize,
        params: JacobiParams,
    ) -> Result<impl Future<Output = Result<Self>> + Send + 'static> {
        self.check_count(steps)?;
        let session = Arc::clone(&self.current.storage.session);
        let operation = session.budget.operation()?;
        Ok(async move {
            let _operation = operation;
            for _ in 0..steps {
                session.bind()?;
                {
                    let op = self.operation(params);
                    let _completed = DeviceFuture::scheduled(
                        op,
                        ExecutionContext::new(Arc::clone(&session.stream)),
                    )
                    .await
                    .map_err(|e| backend("execute async timestep", e))?;
                }
                self.swap_completed();
            }
            Ok(self)
        })
    }

    /// Blocks to copy the complete current haloed grid, returning it with the
    /// ready owner. No mutable access or device alias escapes. The host snapshot
    /// remains independent when later timesteps run.
    ///
    /// # Errors
    /// Admission or backend errors consume the state. Failed completion follows
    /// the same fail-stop policy as launch and readback elsewhere in the facade.
    pub fn observe(mut self) -> Result<(Self, Vec<f32>)> {
        let Buffer {
            tensor,
            shape,
            session,
            completion,
            reservation,
        } = self.current.storage;
        let _operation = session.budget.operation()?;
        session.bind()?;
        let retained = Arc::new(tensor);
        let host = (&retained)
            .to_host_vec()
            .sync_on(&session.stream)
            .map_err(|e| backend("observe timesteps", e))?;
        let tensor = Arc::try_unwrap(retained)
            .map_err(|_| backend("observe timesteps", "unexpected retained tensor alias"))?;
        self.current.storage = Buffer {
            tensor,
            shape,
            session,
            completion,
            reservation,
        };
        Ok((self, host))
    }

    fn check_count(&self, steps: usize) -> Result<()> {
        self.completed
            .checked_add(steps)
            .ok_or_else(|| backend("validate timestep count", "completed counter overflow"))?;
        Ok(())
    }

    fn swap_completed(&mut self) {
        std::mem::swap(&mut self.current, &mut self.next);
        self.completed += 1;
    }

    fn operation(&mut self, params: JacobiParams) -> impl DeviceOp + '_ {
        // Construction is the sole entry point: equal shapes/session, unique
        // storage, and initialized halos are preserved by every successful swap.
        let [height, width] = self.shape().interior();
        super::super::kernels::jacobi_kernel::timestep(
            (&mut self.next.storage.tensor).partition([1, 1]),
            &self.current.storage.tensor,
            &self.rhs.storage.tensor,
            i32::try_from(height).unwrap_or(i32::MAX),
            i32::try_from(width).unwrap_or(i32::MAX),
            params.omega,
            params.h_squared,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cutile::prelude::Reshape;

    #[test]
    fn errors_consume_state_and_release_admission() -> Result<()> {
        let gpu = Gpu::with_limits(0, crate::ResourceLimits::new(36, 76, 3, 1)?)?;
        let shape = Shape2D::new(1, 1)?;
        let p = JacobiParams::new(0.5, 1.0)?;
        let mut grid = gpu.double_grid(shape, vec![1.0; 9], vec![0.0])?;
        grid.completed = usize::MAX;
        assert!(grid.advance_async(1, p).is_err());
        let grid = gpu.double_grid(shape, vec![1.0; 9], vec![0.0])?;
        let admission = gpu.session.budget.operation()?;
        assert!(grid.advance_async(1, p).is_err());
        drop(admission);
        let mut grid = gpu
            .double_grid(shape, vec![1.0; 9], vec![0.0])?
            .advance(1, p)?;
        // Test-only invariant violation forces a real backend launch rejection
        // after a successful step; no public API exposes this mutation.
        grid.next.storage.tensor = grid
            .next
            .storage
            .tensor
            .reshape(&[9])
            .map_err(|e| backend("inject bad rank", e))?;
        assert!(grid.advance(1, p).is_err());
        drop(gpu.double_grid(shape, vec![1.0; 9], vec![0.0])?);
        Ok(())
    }
}
