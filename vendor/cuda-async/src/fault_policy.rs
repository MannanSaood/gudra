/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Fail-stop policy for operations whose device completion is not yet proven.

use std::cell::Cell;
use std::fmt::Display;
use std::io::{self, Write};

thread_local! {
    static UNCERTAIN_EXECUTIONS: Cell<usize> = const { Cell::new(0) };
}

/// Keeps device-allocation destructors quarantined while an operation may have
/// submitted work but has not reached a successful stream terminal.
pub(crate) struct UncertainExecution {
    armed: bool,
    operation: &'static str,
}

impl UncertainExecution {
    pub(crate) fn new(operation: &'static str) -> Self {
        UNCERTAIN_EXECUTIONS.with(|depth| {
            let Some(next) = depth.get().checked_add(1) else {
                fail_stop(operation, "uncertain-execution depth overflow");
            };
            depth.set(next);
        });
        Self {
            armed: true,
            operation,
        }
    }

    /// Records that the associated stream reached a successful completion
    /// terminal, allowing ordinary owner destruction again.
    pub(crate) fn completion_proven(mut self) {
        UNCERTAIN_EXECUTIONS.with(|depth| {
            let current = depth.get();
            if current == 0 {
                fail_stop(self.operation, "uncertain-execution depth underflow");
            }
            depth.set(current - 1);
            if current == 1 {
                crate::device_buffer::take_quarantined_resources().release_after_completion();
            }
        });
        self.armed = false;
    }

    /// Moves resources quarantined during a successful asynchronous execute
    /// into the future that will prove stream completion.
    pub(crate) fn handoff_to_terminal(mut self) -> crate::device_buffer::QuarantinedResources {
        let resources = UNCERTAIN_EXECUTIONS.with(|depth| {
            if depth.get() != 1 {
                fail_stop(
                    self.operation,
                    "asynchronous quarantine handoff was not outermost",
                );
            }
            depth.set(0);
            crate::device_buffer::take_quarantined_resources()
        });
        self.armed = false;
        resources
    }
}

impl Drop for UncertainExecution {
    fn drop(&mut self) {
        if self.armed {
            fail_stop(
                self.operation,
                "host unwound before device completion was proven",
            );
        }
    }
}

/// Whether an owned device allocation may be submitted to the deallocator.
pub(crate) fn allocation_release_allowed() -> bool {
    UNCERTAIN_EXECUTIONS.with(|depth| depth.get() == 0)
}

/// Guard for a dependency-internal host transfer destination. It aborts if
/// execution errors or unwinds before the destination is handed to the outer
/// cuda-async terminal, which owns the final completion proof.
#[doc(hidden)]
pub struct ExternalTransferGuard(Option<UncertainExecution>);

impl ExternalTransferGuard {
    /// Arms fail-stop protection around a dependency-internal transfer.
    ///
    /// # Safety
    /// The caller must already be executing inside a cuda-async terminal that
    /// will retain the returned owner until device completion is proven.
    pub unsafe fn new(operation: &'static str) -> Self {
        Self(Some(UncertainExecution::new(operation)))
    }

    /// Transfers responsibility to the surrounding cuda-async terminal after
    /// the operation has successfully returned its destination owner.
    pub fn handoff_to_terminal(mut self) {
        self.0
            .take()
            .expect("external transfer guard is handed off once")
            .completion_proven();
    }
}

/// Terminates the worker before uncertain device work can outlive or reuse an
/// allocation owner. This intentionally does not run Rust destructors.
pub(crate) fn fail_stop(operation: &str, error: impl Display) -> ! {
    let mut stderr = io::stderr().lock();
    let _ = writeln!(
        stderr,
        "cuda-async: aborting worker because {operation} did not prove device completion: {error}"
    );
    std::process::abort()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    struct TestOwner(Arc<AtomicUsize>);

    impl Drop for TestOwner {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }

    unsafe impl crate::device_buffer::DeviceAllocation for TestOwner {
        fn device_ptr(&self) -> cuda_core::sys::CUdeviceptr {
            1
        }

        fn len_bytes(&self) -> usize {
            4
        }

        fn device_id(&self) -> usize {
            0
        }
    }

    #[test]
    fn completion_controls_allocation_quarantine() {
        assert!(allocation_release_allowed());
        let outer = UncertainExecution::new("outer test");
        assert!(!allocation_release_allowed());
        let inner = UncertainExecution::new("inner test");
        assert!(!allocation_release_allowed());
        inner.completion_proven();
        assert!(!allocation_release_allowed());
        outer.completion_proven();
        assert!(allocation_release_allowed());
    }

    #[test]
    fn asynchronous_handoff_retains_foreign_owner_until_terminal() {
        let drops = Arc::new(AtomicUsize::new(0));
        let guard = UncertainExecution::new("async handoff test");
        drop(crate::device_buffer::DeviceBuffer::foreign(
            Arc::new(TestOwner(Arc::clone(&drops))),
            4,
        ));
        assert_eq!(drops.load(Ordering::Relaxed), 0);

        let resources = guard.handoff_to_terminal();
        assert!(allocation_release_allowed());
        assert_eq!(drops.load(Ordering::Relaxed), 0);

        resources.release_after_completion();
        assert_eq!(drops.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn abort_helper() {
        if std::env::var_os("CUDA_ASYNC_ABORT_HELPER").is_some() {
            fail_stop("test operation", "injected uncertain completion");
        }
    }

    #[test]
    fn armed_unwind_helper() {
        if std::env::var_os("CUDA_ASYNC_UNWIND_HELPER").is_some() {
            let _guard = UncertainExecution::new("panicking test operation");
            panic!("injected panic after possible submission");
        }
    }

    fn assert_helper_aborts(test: &str, variable: &str) {
        let status = Command::new(std::env::current_exe().expect("current test binary"))
            .args(["--exact", test, "--nocapture"])
            .env(variable, "1")
            .status()
            .expect("launch abort helper");
        assert!(!status.success(), "fail-stop helper unexpectedly succeeded");
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            assert_eq!(status.signal(), Some(6), "expected SIGABRT: {status:?}");
        }
    }

    #[test]
    fn explicit_fail_stop_aborts_subprocess() {
        assert_helper_aborts(
            "fault_policy::tests::abort_helper",
            "CUDA_ASYNC_ABORT_HELPER",
        );
    }

    #[test]
    fn unwind_while_armed_aborts_subprocess() {
        assert_helper_aborts(
            "fault_policy::tests::armed_unwind_helper",
            "CUDA_ASYNC_UNWIND_HELPER",
        );
    }
}
