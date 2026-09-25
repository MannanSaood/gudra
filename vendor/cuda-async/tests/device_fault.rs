/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! A device fault after a future has registered for completion must terminate
//! the worker before allocation owners can unwind and be reused.
//!
//! The faulting kernel is a one-instruction PTX `trap`, JIT-compiled at test
//! time. It is preceded on the same stream by ~1.2 ms of memsets so the fault
//! lands long after the inline-spin budget (20 us): the future is registered
//! with the reactor when the stream dies, which is exactly the path that used
//! to hang. A `trap` is a *sticky* error that kills the process's CUDA
//! context, so this binary holds a single test and nothing can run after it.
//! Requires a GPU.

use cuda_async::device_context::{global_policy, init_device_contexts, load_module_from_ptx};
use cuda_async::device_operation::{DeviceOp, ExecutionContext};
use cuda_async::error::DeviceError;
use cuda_async::launch::AsyncKernelLaunch;
use cuda_core::{Function, LaunchConfig};
use std::future::{Future, IntoFuture};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use std::time::{Duration, Instant};

const TRAP_PTX: &str = r#"
.version 7.0
.target sm_50
.address_size 64

.visible .entry fault_kernel()
{
    trap;
}
"#;

fn on_fresh_thread<F: FnOnce() + Send + 'static>(f: F) {
    std::thread::spawn(f).join().expect("test thread panicked");
}

fn noop_waker() -> Waker {
    fn noop(_: *const ()) {}
    fn clone(p: *const ()) -> RawWaker {
        RawWaker::new(p, &VTABLE)
    }
    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, noop, noop, noop);
    unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) }
}

fn alloc_device(bytes: usize) -> u64 {
    cuda_async::device_context::with_device(0, |device| device.bind_to_thread())
        .expect("device context")
        .expect("bind_to_thread failed");
    let mut dptr = std::mem::MaybeUninit::uninit();
    let code = unsafe { cuda_bindings::cuMemAlloc_v2(dptr.as_mut_ptr(), bytes) };
    assert_eq!(code, 0, "cuMemAlloc failed: {code}");
    unsafe { dptr.assume_init() }
}

/// Polls with a noop waker and a sleep between polls, so the test does not
/// depend on being woken at all: it checks that the future *resolves*.
fn block_on_with_deadline<F: Future + Unpin>(mut future: F, deadline: Duration) -> F::Output {
    let start = Instant::now();
    let waker = noop_waker();
    let mut cx = Context::from_waker(&waker);
    loop {
        match Pin::new(&mut future).poll(&mut cx) {
            Poll::Ready(out) => return out,
            Poll::Pending => {
                assert!(
                    start.elapsed() < deadline,
                    "future did not resolve within {deadline:?}"
                );
                std::thread::sleep(Duration::from_millis(1));
            }
        }
    }
}

/// Slow memsets, then the trapping kernel, all on the context's stream.
struct FaultingOp {
    dptr: u64,
    bytes: usize,
    passes: usize,
    trap: Arc<Function>,
}

impl DeviceOp for FaultingOp {
    type Output = ();
    unsafe fn execute(self, context: &ExecutionContext) -> Result<(), DeviceError> {
        let stream = context.get_cuda_stream().cu_stream();
        for _ in 0..self.passes {
            let code = cuda_bindings::cuMemsetD8Async(self.dptr, 0x11, self.bytes, stream);
            if code != cuda_bindings::cudaError_enum_CUDA_SUCCESS {
                return Err(DeviceError::Internal(format!(
                    "cuMemsetD8Async failed: {code}"
                )));
            }
        }
        let mut launch = AsyncKernelLaunch::new(self.trap);
        launch.set_launch_config(LaunchConfig {
            grid_dim: (1, 1, 1),
            block_dim: (1, 1, 1),
            shared_mem_bytes: 0,
        });
        launch.execute(context)
    }
}

impl IntoFuture for FaultingOp {
    type Output = Result<(), DeviceError>;
    type IntoFuture = cuda_async::device_future::DeviceFuture<(), FaultingOp>;
    fn into_future(self) -> Self::IntoFuture {
        let policy = global_policy(0).expect("global policy");
        match self.schedule(&policy) {
            Ok(future) => future,
            Err(error) => cuda_async::device_future::DeviceFuture::failed(error),
        }
    }
}

#[test]
fn device_fault_fail_stops_the_worker() {
    if std::env::var_os("CUDA_ASYNC_DEVICE_FAULT_CHILD").is_none() {
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "device_fault_fail_stops_the_worker",
                "--nocapture",
            ])
            .env("CUDA_ASYNC_DEVICE_FAULT_CHILD", "1")
            .status()
            .expect("launch isolated device-fault child");
        assert!(!status.success(), "faulting worker unexpectedly returned");
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            assert_eq!(status.signal(), Some(6), "expected SIGABRT: {status:?}");
        }
        return;
    }
    on_fresh_thread(|| {
        init_device_contexts(0, 1).expect("init failed (requires GPU)");
        let bytes = 64 << 20;
        let dptr = alloc_device(bytes);
        let module = load_module_from_ptx(TRAP_PTX, 0).expect("PTX JIT failed");
        let trap = Arc::new(module.load_function("fault_kernel").expect("fault_kernel"));

        // Fault after registration. The future observes the sticky error, and
        // its terminal must abort because stream completion is unprovable.
        let op = FaultingOp {
            dptr,
            bytes,
            passes: 32,
            trap,
        };
        let started = Instant::now();
        let _ = block_on_with_deadline(op.into_future(), Duration::from_secs(30));
        panic!(
            "faulting operation returned instead of fail-stopping after {:?}",
            started.elapsed()
        );
    });
}
