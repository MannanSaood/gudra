//! Explicit GPU lane: enabled tests fail on missing hardware, never silently skip.

use std::{
    future::Future,
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
    time::Duration,
};

use gudra::{
    gpu::{Gpu, StepBuffers},
    reference::jacobi_into,
    Error, JacobiParams, Shape2D,
};

fn fixture(shape: Shape2D) -> (Vec<f32>, Vec<f32>) {
    // Nonconstant/asymmetric values exercise view offsets, row stride and edges.
    let halo = [1.0, 7.0, -2.0, 3.0, 0.5, 9.0, -4.0, 2.0, 8.0, 5.0, -1.0]
        .into_iter()
        .cycle()
        .take(shape.haloed_len())
        .collect();
    let rhs = [0.0, 1.0, -2.0, 0.5, 3.0]
        .into_iter()
        .cycle()
        .take(shape.interior_len())
        .collect();
    (halo, rhs)
}

fn assert_close(actual: &[f32], expected: &[f32]) {
    assert_eq!(actual.len(), expected.len());
    for (i, (&a, &e)) in actual.iter().zip(expected).enumerate() {
        assert!(
            a.is_finite() && (a - e).abs() <= 1.0e-5 + 1.0e-5 * e.abs(),
            "cell {i}: GPU={a}, CPU={e}"
        );
    }
}

#[test]
fn blocking_steps_match_reference() -> gudra::Result<()> {
    let gpu = Gpu::new(0)?;
    let p = JacobiParams::new(2.0 / 3.0, 0.25)?;
    for (height, width) in [(1, 1), (1, 19), (19, 1), (16, 16), (17, 19)] {
        let shape = Shape2D::new(height, width)?;
        let (halo, host_rhs) = fixture(shape);
        let mut expected = vec![0.0; shape.interior_len()];
        jacobi_into(shape, &halo, &host_rhs, &mut expected, p)?;
        let source = gpu.upload_halo(shape, halo)?;
        let rhs = gpu.upload_field(shape, host_rhs)?;
        assert_close(&gpu.step(&source, &rhs, p)?.into_host()?, &expected);
        let mut output = gpu.upload_field(shape, vec![f32::NAN; shape.interior_len()])?;
        gpu.step_into(&source, &rhs, &mut output, p)?;
        assert_close(&output.into_host()?, &expected);
    }
    Ok(())
}

#[test]
fn validation_rejects_equal_area_wrong_extents_and_other_sessions() -> gudra::Result<()> {
    let gpu = Gpu::new(0)?;
    let shape = Shape2D::new(2, 3)?;
    let other_shape = Shape2D::new(3, 2)?;
    let source = gpu.upload_halo(shape, vec![1.0; shape.haloed_len()])?;
    let rhs = gpu.zeros(shape)?;
    let wrong_rhs = gpu.zeros(other_shape)?;
    let p = JacobiParams::new(0.5, 1.0)?;
    let mut output = gpu.upload_field(shape, vec![123.0; shape.interior_len()])?;
    assert!(matches!(
        gpu.step_into(&source, &wrong_rhs, &mut output, p),
        Err(Error::ShapeMismatch { buffer: "rhs", .. })
    ));
    assert_eq!(output.into_host()?, vec![123.0; shape.interior_len()]);
    let other_gpu = Gpu::new(0)?;
    let mut foreign_output = other_gpu.zeros(shape)?;
    assert!(matches!(
        gpu.step_into(&source, &rhs, &mut foreign_output, p),
        Err(Error::ContextMismatch { buffer: "output" })
    ));
    Ok(())
}

struct ThreadWake(std::thread::Thread);
impl Wake for ThreadWake {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.0.unpark();
    }
}

// Small test-only executor with safe pinning. A timeout fails visibly rather
// than silently accepting a lost completion wakeup. Pending Drop may block.
fn complete(
    future: impl Future<Output = gudra::Result<StepBuffers>> + Send + 'static,
) -> gudra::Result<StepBuffers> {
    let waker = Waker::from(Arc::new(ThreadWake(std::thread::current())));
    let mut cx = Context::from_waker(&waker);
    let mut future = std::pin::pin!(future);
    let deadline = std::time::Instant::now() + Duration::from_secs(120);
    loop {
        if let Poll::Ready(result) = future.as_mut().poll(&mut cx) {
            return result;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "async completion exceeded 120 seconds"
        );
        std::thread::park_timeout(Duration::from_millis(100));
    }
}

#[test]
fn owned_async_returns_usable_buffers_on_another_thread() -> gudra::Result<()> {
    let gpu = Gpu::new(0)?;
    let shape = Shape2D::new(17, 19)?;
    let p = JacobiParams::new(0.5, 0.25)?;
    let (halo, host_rhs) = fixture(shape);
    let mut expected = vec![0.0; shape.interior_len()];
    jacobi_into(shape, &halo, &host_rhs, &mut expected, p)?;
    let source = gpu.upload_halo(shape, halo)?;
    let rhs = gpu.upload_field(shape, host_rhs.clone())?;
    let output = gpu.zeros(shape)?;
    let future = gpu.step_into_async(source, rhs, output, p)?;
    // The explicit bound and thread transfer type-check Send + 'static and
    // demonstrate that Gpu itself need not outlive the job.
    drop(gpu);
    let buffers = std::thread::spawn(move || complete(future))
        .join()
        .expect("GPU worker panicked")?;
    assert_eq!(buffers.source.shape(), shape);
    assert_eq!(buffers.rhs.into_host()?, host_rhs);
    assert_close(&buffers.output.into_host()?, &expected);
    Ok(())
}

#[test]
fn owned_job_can_be_dropped_before_or_after_first_poll() -> gudra::Result<()> {
    let gpu = Gpu::new(0)?;
    let shape = Shape2D::new(17, 19)?;
    let p = JacobiParams::new(0.5, 0.25)?;
    for poll_once in [false, true] {
        let source = gpu.upload_halo(shape, vec![4.0; shape.haloed_len()])?;
        let rhs = gpu.zeros(shape)?;
        let output = gpu.zeros(shape)?;
        let mut future = Box::pin(gpu.step_into_async(source, rhs, output, p)?);
        if poll_once {
            let waker = Waker::from(Arc::new(ThreadWake(std::thread::current())));
            let mut cx = Context::from_waker(&waker);
            if let Poll::Ready(result) = future.as_mut().poll(&mut cx) {
                let buffers = result?;
                assert_eq!(buffers.output.into_host()?, vec![4.0; shape.interior_len()]);
            }
        }
        drop(future);
        assert_eq!(
            gpu.zeros(shape)?.into_host()?,
            vec![0.0; shape.interior_len()]
        );
    }
    // This is a lifecycle smoke test, not proof that the first poll was Pending
    // or that no work submitted before polling. Further verification needs instrumentation,
    // forced-pending cancellation and isolated-process forget tests.
    Ok(())
}
