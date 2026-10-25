use super::Env;
use std::sync::OnceLock;

/// Global storage for the `Env` singleton — written once via `set`/`ensure_init`.
static ENV: OnceLock<Env> = OnceLock::new();

impl Env {
    /// Stores the Env created once in `main`; second call is ignored.
    pub fn set(env: Env) -> &'static Env {
        let _ = ENV.set(env);
        ENV.get().unwrap()
    }

    /// Returns the Env stored via `set`; panics if not yet initialized.
    pub fn global() -> &'static Env {
        ENV.get()
            .expect("env not initialized: call Env::set in main")
    }

    /// Ensure the global is initialized from the real environment; panics with
    /// the full list of missing/empty vars otherwise (devenv must provide them).
    pub fn ensure_init() {
        if ENV.get().is_none() {
            let env = Self::create().unwrap_or_else(|e| panic!("{e}"));
            let _ = ENV.set(env);
        }
    }
}
