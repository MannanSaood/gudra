//! Fixed-Dirichlet Poisson iteration, periodic observation, and a launch comparison.
use gudra::{gpu::Gpu, reference::jacobi_into, JacobiParams, Shape2D};
use std::{
    future::Future,
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
    time::{Duration, Instant},
};

struct ThreadWake(std::thread::Thread);
impl Wake for ThreadWake {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.0.unpark();
    }
}
fn complete<T>(future: impl Future<Output = gudra::Result<T>>) -> gudra::Result<T> {
    let waker = Waker::from(Arc::new(ThreadWake(std::thread::current())));
    let mut cx = Context::from_waker(&waker);
    let mut future = std::pin::pin!(future);
    let deadline = Instant::now() + Duration::from_secs(120);
    loop {
        if let Poll::Ready(result) = future.as_mut().poll(&mut cx) {
            return result;
        }
        assert!(Instant::now() < deadline, "GPU completion timeout");
        std::thread::park_timeout(Duration::from_millis(10));
    }
}
fn main() -> gudra::Result<()> {
    let gpu = Gpu::new(0)?;
    let shape = Shape2D::new(17, 19)?;
    let p = JacobiParams::new(0.5, 0.01)?;
    let mut initial = vec![1.0; shape.haloed_len()];
    for row in 0..shape.height() {
        let start = (row + 1) * (shape.width() + 2) + 1;
        initial[start..start + shape.width()].fill(0.0);
    }
    let rhs = vec![-1.0; shape.interior_len()];
    let mut oracle = initial.clone();
    let mut output = vec![0.0; shape.interior_len()];
    let mut previous = initial.clone();
    let mut grid = gpu.double_grid(shape, initial.clone(), rhs.clone())?;
    for _ in 0..5 {
        for _ in 0..25 {
            jacobi_into(shape, &oracle, &rhs, &mut output, p)?;
            for row in 0..shape.height() {
                let start = (row + 1) * (shape.width() + 2) + 1;
                oracle[start..start + shape.width()]
                    .copy_from_slice(&output[row * shape.width()..(row + 1) * shape.width()]);
            }
        }
        let (ready, snapshot) = complete(grid.advance_async(25, p)?)?.observe()?;
        let mut change = 0.0_f32;
        for ((actual, expected), old) in snapshot.iter().zip(&oracle).zip(&previous) {
            assert!(
                actual.is_finite() && (actual - expected).abs() <= 1e-5 + 1e-5 * expected.abs()
            );
            change = change.max((actual - old).abs());
        }
        println!(
            "step={} max_change_over_25_steps={change:.6}",
            ready.completed_steps()
        );
        previous = snapshot;
        grid = ready;
    }
    drop(grid);
    // Same kernel, shape, parameters, number of completions, and allocation
    // lifecycle. Exclude construction, first-launch JIT, and observation.
    let mut direct = Vec::new();
    let mut asynchronous = Vec::new();
    for round in 0..5 {
        for mode in 0..2 {
            let use_async = (round + mode) % 2 == 0;
            let ready = gpu
                .double_grid(shape, initial.clone(), rhs.clone())?
                .advance(1, p)?;
            let start = Instant::now();
            let ready = if use_async {
                complete(ready.advance_async(125, p)?)?
            } else {
                ready.advance(125, p)?
            };
            let elapsed = start.elapsed();
            if use_async {
                asynchronous.push(elapsed);
            } else {
                direct.push(elapsed);
            }
            assert_eq!(ready.completed_steps(), 126);
        }
    }
    direct.sort_unstable();
    asynchronous.sort_unstable();
    println!("125 steps, median of 5, 17x19 interior, scalar tiles");
    println!(
        "direct_blocking_us={} owned_async_us={} async_over_direct={:.3}",
        direct[2].as_micros(),
        asynchronous[2].as_micros(),
        asynchronous[2].as_secs_f64() / direct[2].as_secs_f64()
    );
    Ok(())
}
