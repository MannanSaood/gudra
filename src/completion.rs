//! Conservative state for borrowed outputs crossing a backend execution call.

use crate::{Error, Result};

#[derive(Default)]
pub(crate) struct Completion {
    uncertain: bool,
}

impl Completion {
    pub(crate) fn ensure_ready(&self, buffer: &'static str) -> Result<()> {
        if self.uncertain {
            return Err(Error::InvalidBuffer { buffer });
        }
        Ok(())
    }

    pub(crate) fn run(&mut self, operation: impl FnOnce() -> Result<()>) -> Result<()> {
        // Set before invoking the backend: an error or unwind must not publish
        // a partially written field as a completed result.
        self.uncertain = true;
        operation()?;
        self.uncertain = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_success_restores_readiness() {
        let mut state = Completion::default();
        state.ensure_ready("output").unwrap();
        state.run(|| Ok(())).unwrap();
        state.ensure_ready("output").unwrap();
        let failure = state.run(|| {
            Err(Error::Backend {
                operation: "injected execution error",
                message: "no CUDA involved".into(),
            })
        });
        assert!(failure.is_err());
        for role in ["rhs", "output", "field"] {
            assert!(matches!(state.ensure_ready(role),
                Err(Error::InvalidBuffer { buffer }) if buffer == role));
        }
    }

    #[test]
    fn unwind_keeps_output_unavailable() {
        let mut state = Completion::default();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = state.run(|| panic!("injected backend unwind"));
        }));
        assert!(result.is_err());
        assert!(matches!(
            state.ensure_ready("field"),
            Err(Error::InvalidBuffer { .. })
        ));
    }
}
