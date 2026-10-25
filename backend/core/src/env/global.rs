use super::Env;
use std::sync::OnceLock;

static ENV: OnceLock<Env> = OnceLock::new();

impl Env {
    pub fn set(env: Env) -> &'static Env {
        let _ = ENV.set(env);
        ENV.get().unwrap()
    }

    pub fn global() -> &'static Env {
        ENV.get()
            .expect("env not initialized: call Env::set in main")
    }

    pub fn ensure_init() {
        if ENV.get().is_none() {
            let env = Self::create().unwrap_or_else(|e| panic!("{e}"));
            let _ = ENV.set(env);
        }
    }
}
