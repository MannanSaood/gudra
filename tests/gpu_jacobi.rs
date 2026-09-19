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

mod support;
use support::{assert_close, fixture, nonfinite_cases, SHAPES};

// Numerical assertions alone did not catch early deallocation. Run this test
// under memcheck with stream-ordered race tracking as well as normally.
#[test]
fn readback_retains_allocation_until_copy_completes() -> gudra::Result<()> {
    for (height, width) in [(1, 1), (2, 3), (17, 19), (33, 47), (257, 263)] {
        let gpu = Gpu::new(0)?;
        let shape = Shape2D::new(height, width)?;
        for seed in 0..16 {
            let (_, expected) = fixture(shape, seed);
            let field = gpu.upload_field(shape, expected.clone())?;
            assert_eq!(field.into_host()?, expected);
            assert_eq!(
                gpu.zeros(shape)?.into_host()?,
                vec![0.0; shape.interior_len()]
            );
        }
        let (_, expected) = fixture(shape, 123);
        let field = gpu.upload_field(shape, expected.clone())?;
        drop(gpu);
        let actual = std::thread::spawn(move || field.into_host())
            .join()
            .expect("readback worker panicked")?;
        assert_eq!(actual, expected);
    }
    Ok(())
}

#[test]
fn blocking_steps_match_reference() -> gudra::Result<()> {
    let gpu = Gpu::new(0)?;
    let p = JacobiParams::new(2.0 / 3.0, 0.25)?;
    for &(height, width) in SHAPES {
        let shape = Shape2D::new(height, width)?;
        let (halo, host_rhs) = fixture(shape, 0x5eed_0005);
        let mut expected = vec![0.0; shape.interior_len()];
        jacobi_into(shape, &halo, &host_rhs, &mut expected, p)?;
        let source = gpu.upload_halo(shape, halo)?;
        let rhs = gpu.upload_field(shape, host_rhs.clone())?;
        // Fresh poison detects unwritten edge cells; each repetition uses the
        // CPU oracle, never a possibly wrong first GPU result.
        for _ in 0..3 {
            assert_close(&gpu.step(&source, &rhs, p)?.into_host()?, &expected);
            let mut output = gpu.upload_field(shape, vec![f32::NAN; shape.interior_len()])?;
            for _ in 0..3 {
                gpu.step_into(&source, &rhs, &mut output, p)?;
            }
            assert_close(&output.into_host()?, &expected);
        }
        assert_eq!(rhs.into_host()?, host_rhs);
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
    let (halo, host_rhs) = fixture(shape, 0x5eed_0005);
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
    for poll_once in [false, true].into_iter().cycle().take(10) {
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

#[test]
fn unusual_finite_coefficients_match_oracle() -> gudra::Result<()> {
    let gpu = Gpu::new(0)?;
    let shape = Shape2D::new(17, 19)?;
    for (omega, h2) in [
        (-2.0, 0.25),
        (0.0, 0.0),
        (1.0, 0.0),
        (2.0, 0.5),
        (-1.0, 1.0),
    ] {
        let (halo, rhs_host) = fixture(shape, 123);
        let p = JacobiParams::new(omega, h2)?;
        let mut expected = vec![f32::NAN; shape.interior_len()];
        jacobi_into(shape, &halo, &rhs_host, &mut expected, p)?;
        let source = gpu.upload_halo(shape, halo)?;
        let rhs = gpu.upload_field(shape, rhs_host)?;
        assert_close(&gpu.step(&source, &rhs, p)?.into_host()?, &expected);
    }
    Ok(())
}

#[test]
fn nonfinite_policy_matches_oracle_and_stated_classification() -> gudra::Result<()> {
    let gpu = Gpu::new(0)?;
    let shape = Shape2D::new(1, 1)?;
    for (halo, rhs_value, omega, h2, expected) in nonfinite_cases() {
        let p = JacobiParams::new(omega, h2)?;
        let mut cpu = [0.0];
        jacobi_into(shape, &halo, &[rhs_value], &mut cpu, p)?;
        assert_close(&cpu, &[expected]);
        let source = gpu.upload_halo(shape, halo.to_vec())?;
        let rhs = gpu.upload_field(shape, vec![rhs_value])?;
        assert_close(&gpu.step(&source, &rhs, p)?.into_host()?, &cpu);
    }
    Ok(())
}

#[test]
fn every_validation_role_leaves_output_unchanged_and_session_usable() -> gudra::Result<()> {
    let gpu = Gpu::new(0)?;
    let other = Gpu::new(0)?;
    let shape = Shape2D::new(2, 3)?;
    let transposed = Shape2D::new(3, 2)?;
    let p = JacobiParams::new(0.5, 1.0)?;
    assert!(matches!(
        gpu.upload_halo(shape, vec![0.0; 19]),
        Err(Error::LengthMismatch {
            buffer: "source",
            ..
        })
    ));
    assert!(matches!(
        gpu.upload_field(shape, vec![0.0; 7]),
        Err(Error::LengthMismatch {
            buffer: "field",
            ..
        })
    ));
    let source = gpu.upload_halo(shape, vec![4.0; shape.haloed_len()])?;
    let rhs = gpu.zeros(shape)?;
    let foreign_source = other.upload_halo(shape, vec![4.0; shape.haloed_len()])?;
    let foreign_rhs = other.zeros(shape)?;
    for role in ["source", "rhs", "output", "shape"] {
        let mut output = if role == "output" {
            other.upload_field(shape, vec![123.0; shape.interior_len()])?
        } else {
            gpu.upload_field(
                if role == "shape" { transposed } else { shape },
                vec![123.0; shape.interior_len()],
            )?
        };
        let result = gpu.step_into(
            if role == "source" {
                &foreign_source
            } else {
                &source
            },
            if role == "rhs" { &foreign_rhs } else { &rhs },
            &mut output,
            p,
        );
        if role == "shape" {
            assert!(matches!(
                result,
                Err(Error::ShapeMismatch {
                    buffer: "output",
                    ..
                })
            ));
        } else {
            assert!(matches!(result, Err(Error::ContextMismatch { buffer }) if buffer == role));
        }
        assert_eq!(output.into_host()?, vec![123.0; shape.interior_len()]);
        assert_eq!(
            gpu.step(&source, &rhs, p)?.into_host()?,
            vec![4.0; shape.interior_len()]
        );
    }
    let bad_rhs = gpu.zeros(transposed)?;
    assert!(matches!(
        gpu.step(&source, &bad_rhs, p),
        Err(Error::ShapeMismatch { buffer: "rhs", .. })
    ));
    let output = gpu.zeros(shape)?;
    assert!(matches!(
        gpu.step_into_async(source, bad_rhs, output, p),
        Err(Error::ShapeMismatch { buffer: "rhs", .. })
    ));
    assert_eq!(rhs.into_host()?, vec![0.0; shape.interior_len()]);
    assert_eq!(
        gpu.zeros(shape)?.into_host()?,
        vec![0.0; shape.interior_len()]
    );
    Ok(())
}
