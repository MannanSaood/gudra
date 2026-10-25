//! Explicit GPU lane: enabled tests fail on missing hardware, never silently skip.

use std::{
    future::Future,
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
    time::Duration,
};

use gudra::{gpu::Gpu, reference::jacobi_into, Error, JacobiParams, ResourceLimits, Shape2D};

mod support;
use support::{assert_close, fixture, nonfinite_cases, SHAPES};

#[test]
fn budgets_reject_before_allocation_or_submission_and_release_on_drop() -> gudra::Result<()> {
    let gpu = Gpu::with_limits(0, ResourceLimits::new(36, 88, 6, 1)?)?;
    assert!(matches!(
        gpu.zeros(Shape2D::new(32768, 32768)?),
        Err(Error::ResourceLimit { .. })
    ));
    let shape = Shape2D::new(1, 1)?;
    let p = JacobiParams::new(0.5, 1.0)?;
    let first = (
        gpu.upload_halo(shape, vec![1.0; 9])?,
        gpu.zeros(shape)?,
        gpu.zeros(shape)?,
    );
    let second = (
        gpu.upload_halo(shape, vec![1.0; 9])?,
        gpu.zeros(shape)?,
        gpu.zeros(shape)?,
    );
    assert!(matches!(gpu.zeros(shape), Err(Error::ResourceLimit { .. })));
    let future = gpu.step_into_async(first.0, first.1, first.2, p)?;
    let (source, rhs, mut output) = second;
    assert!(matches!(
        gpu.step_into(&source, &rhs, &mut output, p),
        Err(Error::ResourceLimit {
            resource: "operations",
            ..
        })
    ));
    drop(future); // unpolled: restores admission and all three storage charges
    let replacement = (
        gpu.upload_halo(shape, vec![1.0; 9])?,
        gpu.zeros(shape)?,
        gpu.zeros(shape)?,
    );
    assert!(matches!(gpu.zeros(shape), Err(Error::ResourceLimit { .. })));
    drop(replacement);
    assert_eq!(output.into_host()?, vec![0.0]);
    let mut output = gpu.zeros(shape)?;
    gpu.step_into(&source, &rhs, &mut output, p)?;
    assert_eq!(output.into_host()?, vec![1.0]);
    let output = gpu.zeros(shape)?;
    let buffers = complete(gpu.step_into_async(source, rhs, output, p)?)?;
    assert_eq!(buffers.output.into_host()?, vec![1.0]);
    Ok(())
}

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
fn complete<T>(
    future: impl Future<Output = gudra::Result<T>> + Send + 'static,
) -> gudra::Result<T> {
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

#[test]
fn timesteps_match_oracle_and_observe_without_races() -> gudra::Result<()> {
    let gpu = Gpu::new(0)?;
    let p = JacobiParams::new(0.5, 0.01)?;
    for (height, width) in [(1, 1), (1, 19), (19, 1), (2, 3), (17, 19), (33, 47)] {
        let shape = Shape2D::new(height, width)?;
        let (initial, rhs) = fixture(shape, 81);
        let mut oracle = initial.clone();
        let mut interior = vec![0.0; shape.interior_len()];
        let mut grid = gpu.double_grid(shape, initial.clone(), rhs.clone())?;
        for chunk in 0..5 {
            for _ in 0..25 {
                jacobi_into(shape, &oracle, &rhs, &mut interior, p)?;
                for row in 0..height {
                    let start = (row + 1) * (width + 2) + 1;
                    oracle[start..start + width]
                        .copy_from_slice(&interior[row * width..(row + 1) * width]);
                }
            }
            grid = if chunk % 2 == 0 {
                complete(grid.advance_async(25, p)?)?
            } else {
                grid.advance(25, p)?
            };
            let (ready, snapshot) = grid.observe()?;
            assert_eq!(ready.completed_steps(), (chunk + 1) * 25);
            assert_close(&snapshot, &oracle);
            for row in 0..height + 2 {
                for col in 0..width + 2 {
                    if row == 0 || row == height + 1 || col == 0 || col == width + 1 {
                        let index = row * (width + 2) + col;
                        assert_eq!(snapshot[index].to_bits(), initial[index].to_bits());
                    }
                }
            }
            grid = ready;
        }
        let unchanged = complete(grid.advance_async(0, p)?)?;
        assert_eq!(unchanged.completed_steps(), 125);
    }
    Ok(())
}

#[test]
fn timestep_owners_survive_executor_drop_and_thread_migration() -> gudra::Result<()> {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<gudra::gpu::DoubleGrid>();
    send_sync::<Gpu>();
    let gpu = Gpu::new(0)?;
    let shape = Shape2D::new(2, 3)?;
    let grid = gpu.double_grid(shape, vec![1.0; shape.haloed_len()], vec![0.0; 6])?;
    let future = grid.advance_async(101, JacobiParams::new(0.5, 1.0)?)?;
    drop(gpu);
    let ready = std::thread::spawn(move || complete(future))
        .join()
        .unwrap()?;
    let (_, snapshot) = ready.observe()?;
    assert_eq!(snapshot, vec![1.0; shape.haloed_len()]);
    Ok(())
}

#[test]
fn timestep_unpolled_drop_and_admission_error_release_every_owner() -> gudra::Result<()> {
    let gpu = Gpu::with_limits(0, ResourceLimits::new(36, 76, 3, 1)?)?;
    let shape = Shape2D::new(1, 1)?;
    let p = JacobiParams::new(0.5, 1.0)?;
    for _ in 0..8 {
        let grid = gpu.double_grid(shape, vec![1.0; 9], vec![0.0])?;
        assert!(gpu.double_grid(shape, vec![1.0; 9], vec![0.0]).is_err());
        drop(grid.advance_async(100, p)?);
    }
    drop(gpu.double_grid(shape, vec![1.0; 9], vec![0.0])?);
    Ok(())
}

#[test]
fn timestep_pending_drop_drains_before_reusing_budget() -> gudra::Result<()> {
    let shape = Shape2D::new(257, 263)?;
    let bytes = shape.haloed_len() * 4;
    let gpu = Gpu::with_limits(
        0,
        ResourceLimits::new(bytes, 2 * bytes + shape.interior_len() * 4, 3, 1)?,
    )?;
    let grid = gpu.double_grid(
        shape,
        vec![1.0; shape.haloed_len()],
        vec![0.0; shape.interior_len()],
    )?;
    let mut pending = Box::pin(grid.advance_async(100, JacobiParams::new(0.5, 1.0)?)?);
    let waker = Waker::from(Arc::new(ThreadWake(std::thread::current())));
    let result = pending.as_mut().poll(&mut Context::from_waker(&waker));
    assert!(
        result.is_pending(),
        "cancellation coverage requires an actual pending operation"
    );
    std::thread::spawn(move || drop(pending)).join().unwrap();
    let replacement = gpu.double_grid(
        shape,
        vec![2.0; shape.haloed_len()],
        vec![0.0; shape.interior_len()],
    )?;
    let (_, snapshot) = replacement.observe()?;
    assert_eq!(snapshot, vec![2.0; shape.haloed_len()]);
    Ok(())
}
