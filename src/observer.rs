// contributed by RedstoneWizard08

use argmin::core::observers::Observe;
use argmin::core::{Error, State, KV};
use tracing::debug;

/// A logger using the [`tracing`](https://crates.io/crates/tracing) crate as backend.
#[derive(Clone)]
pub struct TracingLogger;

impl TracingLogger {
    /// Create a logger.
    ///
    /// # Example
    ///
    /// ```
    /// use argmin_observer_slog::SlogLogger;
    ///
    /// let terminal_logger = TracingLogger::new();
    /// ```
    pub fn new() -> Self {
        TracingLogger
    }
}

impl<I> Observe<I> for TracingLogger
where
    I: State,
{
    /// Log basic information about the optimization after initialization.
    fn observe_init(&mut self, msg: &str, state: &I, kv: &KV) -> Result<(), Error> {
        let mut data = Vec::new();

        for (k, v) in kv.kv.iter() {
            data.push(format!("{}: {}", k, v.as_string()));
        }

        for (k, &v) in state.get_func_counts().iter() {
            data.push(format!("{}: {}", k, v));
        }

        data.push(format!("best_cost: {}", state.get_best_cost()));
        data.push(format!("cost: {}", state.get_cost()));
        data.push(format!("iter: {}", state.get_iter()));

        debug!("{} {}", msg, data.join(" | "));

        Ok(())
    }

    /// Logs information about the progress of the optimization after every iteration.
    fn observe_iter(&mut self, state: &I, kv: &KV) -> Result<(), Error> {
        let mut data = Vec::new();

        for (k, v) in kv.kv.iter() {
            data.push(format!("{}: {}", k, v.as_string()));
        }

        for (k, &v) in state.get_func_counts().iter() {
            data.push(format!("{}: {}", k, v));
        }

        data.push(format!("best_cost: {}", state.get_best_cost()));
        data.push(format!("cost: {}", state.get_cost()));
        data.push(format!("iter: {}", state.get_iter()));

        debug!("{}", data.join(" | "));

        Ok(())
    }
}
