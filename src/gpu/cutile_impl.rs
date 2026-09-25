use std::{fmt::Display, future::Future, sync::Arc};

use cutile::{
    cuda_async::device_future::DeviceFuture,
    cuda_core::Stream,
    prelude::{
        api, Device, DeviceOp, ExecutionContext, PartitionMut, Reshape, Tensor, TensorView,
        ToHostVec,
    },
};

use crate::completion::Completion;
use crate::limits::admission::{Budget, Reservation};
use crate::{error::check_len, shape::TILE, Error, JacobiParams, ResourceLimits, Result, Shape2D};

use super::{kernels::jacobi_kernel, Field2D, Gpu, HaloGrid2D, StepBuffers};

#[cfg(test)]
#[path = "verification.rs"]
mod verification;

pub(super) struct Session {
    device: Arc<Device>,
    stream: Arc<Stream>,
    budget: Arc<Budget>,
}

pub(super) struct Buffer {
    // Drop storage before our retained session. The tensor also retains its
    // backend allocation; no public API can manufacture a shared storage alias.
    tensor: Tensor<f32>,
    shape: Shape2D,
    session: Arc<Session>,
    completion: Completion,
    // Last field: release the logical charge only after storage is dropped.
    reservation: Reservation,
}

fn backend(operation: &'static str, error: impl Display) -> Error {
    Error::Backend {
        operation,
        message: error.to_string(),
    }
}

impl Session {
    fn bind(&self) -> Result<()> {
        self.device
            .bind_to_thread()
            .map_err(|e| backend("bind device to thread", e))
    }
}

impl Gpu {
    /// Creates a device context and private stream for `device_ordinal`.
    ///
    /// # Errors
    /// Returns [`Error::Backend`] if the ordinal, driver, device, or stream
    /// initialization fails. Requires the documented CUDA/cuTile environment.
    pub fn new(device_ordinal: usize) -> Result<Self> {
        Self::with_limits(device_ordinal, ResourceLimits::default())
    }

    /// Creates a private session with explicit resource admission limits.
    ///
    /// Limits count logical storage and active/admitted operations, including
    /// unpolled futures. They do not isolate tenants or include backend overhead.
    /// # Errors
    /// Returns [`Error::Backend`] when device or stream initialization fails.
    pub fn with_limits(device_ordinal: usize, limits: ResourceLimits) -> Result<Self> {
        let device = Device::new(device_ordinal).map_err(|e| backend("create device", e))?;
        let stream = device
            .new_stream()
            .map_err(|e| backend("create stream", e))?;
        Ok(Self {
            session: Arc::new(Session {
                device,
                stream,
                budget: Budget::new(limits),
            }),
        })
    }

    /// Allocates and uploads a complete haloed source, blocking until copied.
    ///
    /// Consumes `data`; no host or device storage alias is retained by the caller.
    /// Every halo value is supplied explicitly; none is padded or inferred.
    ///
    /// # Errors
    /// Returns [`Error::LengthMismatch`] before device work for the wrong length,
    /// [`Error::ResourceLimit`] on admission, or [`Error::Backend`] on backend failure.
    pub fn upload_halo(&self, shape: Shape2D, data: Vec<f32>) -> Result<HaloGrid2D> {
        check_len("source", shape.haloed_len(), data.len())?;
        Ok(HaloGrid2D {
            storage: self.upload(shape, data, &shape.haloed())?,
        })
    }

    /// Allocates and uploads an interior field, blocking until copied.
    ///
    /// # Errors
    /// Returns [`Error::LengthMismatch`] before device work for the wrong length,
    /// [`Error::ResourceLimit`] on admission, or [`Error::Backend`] on backend failure.
    pub fn upload_field(&self, shape: Shape2D, data: Vec<f32>) -> Result<Field2D> {
        check_len("field", shape.interior_len(), data.len())?;
        Ok(Field2D {
            storage: self.upload(shape, data, &shape.interior())?,
        })
    }

    fn upload(&self, shape: Shape2D, data: Vec<f32>, extent: &[usize; 2]) -> Result<Buffer> {
        let reservation = self.session.budget.buffer(data.len() * size_of::<f32>())?;
        self.upload_reserved(shape, data, extent, reservation)
    }

    fn upload_reserved(
        &self,
        shape: Shape2D,
        data: Vec<f32>,
        extent: &[usize; 2],
        reservation: Reservation,
    ) -> Result<Buffer> {
        let _operation = self.session.budget.operation()?;
        self.session.bind()?;
        let host = Arc::new(data);
        let tensor = api::copy_host_vec_to_device(&host)
            .sync_on(&self.session.stream)
            .map_err(|e| backend("upload buffer", e))?
            .reshape(extent)
            .map_err(|e| backend("reshape uploaded buffer", e))?;
        Ok(Buffer {
            tensor,
            shape,
            session: Arc::clone(&self.session),
            completion: Completion::default(),
            reservation,
        })
    }

    /// Allocates a fresh interior field and blocks until zero initialization ends.
    ///
    /// # Errors
    /// Returns [`Error::ResourceLimit`] on admission, or [`Error::Backend`] on backend failure.
    pub fn zeros(&self, shape: Shape2D) -> Result<Field2D> {
        let reservation = self.session.budget.buffer(shape.interior_bytes())?;
        // Use the retained host-upload path instead of cuTile's composed
        // allocate-then-fill operation. A later composed-stage error could
        // otherwise drop its intermediate tensor before the terminal drains.
        let data = vec![0.0; shape.interior_len()];
        Ok(Field2D {
            storage: self.upload_reserved(shape, data, &shape.interior(), reservation)?,
        })
    }

    /// Validates, allocates a zeroed output, and blocks until one step completes.
    ///
    /// Allocation/initialization and execution can wait on the stream separately.
    /// This is one update, with no convergence test or halo refresh.
    ///
    /// # Errors
    /// Shape/session/metadata failures precede output allocation. Backend errors
    /// include admission, allocation, JIT, launch and completion; no partial field returns.
    pub fn step(
        &self,
        source: &HaloGrid2D,
        rhs: &Field2D,
        params: JacobiParams,
    ) -> Result<Field2D> {
        validate_step(&self.session, source, rhs, None)?;
        let mut output = self.zeros(source.shape())?;
        self.step_into(source, rhs, &mut output, params)?;
        Ok(output)
    }

    /// Overwrites a supplied output and blocks until the session stream completes.
    ///
    /// Reuses the device allocation; performs no grid-sized allocation or host
    /// transfer. View metadata, JIT, and backend bookkeeping may allocate. The
    /// complete operation retains all borrows through synchronization.
    ///
    /// # Errors
    /// Shape/session/metadata/admission rejection leaves output untouched. A backend
    /// execution error invalidates output: readback and reuse return
    /// [`Error::InvalidBuffer`]. No rollback or retry is attempted. If completion
    /// cannot be proven, the process aborts before allocation owners are released.
    ///
    /// The same RHS cannot also be the mutable output:
    ///
    /// ```compile_fail,E0502
    /// use gudra::{gpu::{Gpu, HaloGrid2D, Field2D}, JacobiParams};
    /// fn alias(gpu: &Gpu, source: &HaloGrid2D, mut rhs: Field2D, p: JacobiParams) {
    ///     gpu.step_into(source, &rhs, &mut rhs, p).unwrap();
    /// }
    /// ```
    pub fn step_into(
        &self,
        source: &HaloGrid2D,
        rhs: &Field2D,
        output: &mut Field2D,
        params: JacobiParams,
    ) -> Result<()> {
        validate_step(&self.session, source, rhs, Some(output))?;
        let _operation = self.session.budget.operation()?;
        self.session.bind()?;
        let views = directional_views(source)?;
        let Buffer {
            tensor, completion, ..
        } = &mut output.storage;
        completion.run(|| {
            // Keep the recovered argument tuple until the sync terminal ends.
            let _completed = jacobi_kernel::jacobi(
                (&mut *tensor).partition([TILE, TILE]),
                &views.center,
                &views.north,
                &views.south,
                &views.east,
                &views.west,
                &rhs.storage.tensor,
                params.omega,
                params.h_squared,
            )
            .sync_on(&self.session.stream)
            .map_err(|e| backend("execute Jacobi step", e))?;
            Ok(())
        })
    }

    /// Validates immediately, then returns a lazy future owning all three buffers.
    ///
    /// No kernel submits until the first poll, which may spend host time on JIT.
    /// Success returns all owners after completion. Dropping an unpolled future
    /// submits nothing. Dropping submitted work can block to drain the shared
    /// session stream, including other work; it does not cancel the GPU kernel.
    /// Forgetting the future retains/leaks its buffers and session.
    ///
    /// # Errors
    /// Validation/admission consumes the buffers even on `Err`. Execution failure or
    /// cancellation does not return them. Use [`Self::step_into`] to retain inputs
    /// on failure. Driver/context fault recovery is unsupported. A failed drain,
    /// panic, or otherwise uncertain completion aborts the process before retained
    /// owners can be released. Run GPU work in an isolated disposable worker.
    pub fn step_into_async(
        &self,
        source: HaloGrid2D,
        rhs: Field2D,
        mut output: Field2D,
        params: JacobiParams,
    ) -> Result<impl Future<Output = Result<StepBuffers>> + Send + 'static> {
        validate_step(&self.session, &source, &rhs, Some(&output))?;
        let operation = self.session.budget.operation()?;
        let session = Arc::clone(&self.session);
        Ok(async move {
            // Bind the guard inside the frame so unpolled jobs retain admission.
            let _operation = operation;
            session.bind()?;
            {
                let views = directional_views(&source)?;
                let op = jacobi_kernel::jacobi(
                    (&mut output.storage.tensor).partition([TILE, TILE]),
                    &views.center,
                    &views.north,
                    &views.south,
                    &views.east,
                    &views.west,
                    &rhs.storage.tensor,
                    params.omega,
                    params.h_squared,
                );
                let _completed =
                    DeviceFuture::scheduled(op, ExecutionContext::new(Arc::clone(&session.stream)))
                        .await
                        .map_err(|e| backend("execute async Jacobi step", e))?;
                // End child result/view/partition borrows before moving owners.
            }
            Ok(StepBuffers {
                source,
                rhs,
                output,
            })
        })
    }
}

impl HaloGrid2D {
    /// Returns the validated interior shape; does not synchronize or expose data.
    #[must_use]
    pub fn shape(&self) -> Shape2D {
        self.storage.shape
    }
}

impl Field2D {
    /// Returns the validated interior shape; does not synchronize or expose data.
    #[must_use]
    pub fn shape(&self) -> Shape2D {
        self.storage.shape
    }

    /// Consumes this field and blocks until a newly allocated host vector is filled.
    ///
    /// There is no additional device tensor copy. The retained session remains
    /// alive through readback, even if the original [`Gpu`] has been dropped.
    ///
    /// # Errors
    /// Returns [`Error::InvalidBuffer`] before device work for invalidated fields,
    /// [`Error::ResourceLimit`] on admission, or [`Error::Backend`] on backend failure.
    pub fn into_host(self) -> Result<Vec<f32>> {
        self.storage.completion.ensure_ready("field")?;
        let _operation = self.storage.session.budget.operation()?;
        let Buffer {
            tensor,
            shape: _,
            session,
            completion: _,
            reservation: _reservation,
        } = self.storage;
        session.bind()?;
        // cuTile 0.3.1 consumes the copy operation (and its tensor Arc) inside
        // execute(), before sync_on synchronizes. Keep a separate private owner
        // so its deallocator stream cannot free storage during the copy.
        let retained = Arc::new(tensor);
        let result = (&retained).to_host_vec().sync_on(&session.stream);
        drop(retained);
        result.map_err(|e| backend("read back field", e))
    }
}

fn validate_step(
    session: &Arc<Session>,
    source: &HaloGrid2D,
    rhs: &Field2D,
    output: Option<&Field2D>,
) -> Result<()> {
    let shape = source.shape();
    for (role, buffer) in [("rhs", Some(rhs)), ("output", output)] {
        if let Some(buffer) = buffer {
            if buffer.shape() != shape {
                return Err(Error::ShapeMismatch {
                    buffer: role,
                    expected: shape,
                    actual: buffer.shape(),
                });
            }
        }
    }
    for (role, buffer) in [
        ("source", Some(&source.storage)),
        ("rhs", Some(&rhs.storage)),
        ("output", output.map(|field| &field.storage)),
    ] {
        if let Some(buffer) = buffer {
            buffer.completion.ensure_ready(role)?;
            if !Arc::ptr_eq(session, &buffer.session) {
                return Err(Error::ContextMismatch { buffer: role });
            }
            let extent = if role == "source" {
                shape.haloed()
            } else {
                shape.interior()
            };
            if buffer.tensor.device_id() != session.device.ordinal()
                || !matches_metadata(buffer.tensor.shape(), extent)
                || !matches_metadata(buffer.tensor.strides(), [extent[1], 1])
            {
                return Err(backend(
                    "validate tensor metadata",
                    format!("{role} shape/stride/device invariant failed"),
                ));
            }
        }
    }
    Ok(())
}

fn matches_metadata(actual: &[i32], expected: [usize; 2]) -> bool {
    actual.len() == 2
        && actual
            .iter()
            .zip(expected)
            .all(|(&a, e)| usize::try_from(a) == Ok(e))
}

struct ReadViews<'a> {
    center: TensorView<'a, f32>,
    north: TensorView<'a, f32>,
    south: TensorView<'a, f32>,
    east: TensorView<'a, f32>,
    west: TensorView<'a, f32>,
}

// Tensor::slice accepts Range<usize>, not RangeInclusive<usize>.
#[allow(clippy::range_plus_one)]
fn directional_views(source: &HaloGrid2D) -> Result<ReadViews<'_>> {
    let shape = source.shape();
    let [height, width] = shape.interior();
    let [halo_height, halo_width] = shape.haloed();
    let tensor = &source.storage.tensor;
    let views = ReadViews {
        center: tensor
            .slice(&[1..height + 1, 1..width + 1])
            .map_err(|e| backend("slice center", e))?,
        north: tensor
            .slice(&[0..height, 1..width + 1])
            .map_err(|e| backend("slice north", e))?,
        south: tensor
            .slice(&[2..halo_height, 1..width + 1])
            .map_err(|e| backend("slice south", e))?,
        east: tensor
            .slice(&[1..height + 1, 2..halo_width])
            .map_err(|e| backend("slice east", e))?,
        west: tensor
            .slice(&[1..height + 1, 0..width])
            .map_err(|e| backend("slice west", e))?,
    };
    // Zero-copy views preserve the physical row stride, including the halo.
    for view in [
        &views.center,
        &views.north,
        &views.south,
        &views.east,
        &views.west,
    ] {
        if !matches_metadata(view.shape(), shape.interior())
            || !matches_metadata(view.strides(), [halo_width, 1])
        {
            return Err(backend(
                "validate directional view",
                "shape/stride invariant failed",
            ));
        }
    }
    Ok(views)
}
