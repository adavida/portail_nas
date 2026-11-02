use portail_core::config::Config;
use std::collections::HashMap;
use std::io::Read;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct AuthFlow {
    pub client_id: String,
    pub redirect_uri: String,
    pub state: Option<String>,
    pub nonce: Option<String>,
    pub scopes: Vec<String>,
    pub challenge: Option<String>,
    pub sub: String,
    pub groups: Vec<String>,
    pub email: Option<String>,
}

#[derive(Default)]
struct Store {
    pending: HashMap<String, (AuthFlow, Instant)>,
    codes: HashMap<String, (AuthFlow, Instant)>,
    sessions: HashMap<String, (String, Instant)>,
    login_failures: HashMap<String, (u32, Instant)>,
}

// shortcut: sessions in memory, single-process — restart = re-login, upgrade when multi-process
static STORE: OnceLock<Mutex<Store>> = OnceLock::new();

fn store() -> &'static Mutex<Store> {
    STORE.get_or_init(|| Mutex::new(Store::default()))
}

pub(crate) fn random_id() -> Result<String, String> {
    let mut buf = vec![0u8; Config::global().oidc_id_bytes];
    let mut file = std::fs::File::open("/dev/urandom")
        .map_err(|e| format!("cannot read /dev/urandom: {e}"))?;
    file.read_exact(&mut buf)
        .map_err(|e| format!("cannot read /dev/urandom: {e}"))?;
    Ok(buf.iter().map(|b| format!("{b:02x}")).collect())
}

fn prune<T>(map: &mut HashMap<String, (T, Instant)>, ttl: Duration) {
    let now = Instant::now();
    map.retain(|_, (_, born)| now.duration_since(*born) < ttl);
}

pub fn save_pending(flow: AuthFlow) -> Result<String, String> {
    let id = random_id()?;
    let mut guard = store().lock().map_err(|e| e.to_string())?;
    prune(&mut guard.pending, Config::global().oidc_pending_ttl);
    guard.pending.insert(id.clone(), (flow, Instant::now()));
    Ok(id)
}

pub fn take_pending(id: &str) -> Option<AuthFlow> {
    let mut guard = store().lock().ok()?;
    prune(&mut guard.pending, Config::global().oidc_pending_ttl);
    let flow = guard.pending.remove(id).map(|(f, _)| f);
    if flow.is_some() {
        guard.login_failures.remove(id);
    }
    flow
}

pub fn record_login_failure(req_id: &str) {
    let Ok(mut guard) = store().lock() else {
        return;
    };
    prune(&mut guard.login_failures, Config::global().oidc_pending_ttl);
    guard
        .login_failures
        .entry(req_id.to_string())
        .and_modify(|(count, _)| *count += 1)
        .or_insert((1, Instant::now()));
}

pub fn login_attempts_exceeded(req_id: &str) -> bool {
    let Ok(guard) = store().lock() else {
        return false;
    };
    let max_attempts = Config::global().login_max_attempts;
    guard
        .login_failures
        .get(req_id)
        .is_some_and(|(count, born)| {
            *count >= max_attempts && born.elapsed() < Config::global().oidc_pending_ttl
        })
}

pub fn has_pending(id: &str) -> bool {
    let Ok(guard) = store().lock() else {
        return false;
    };
    guard
        .pending
        .get(id)
        .is_some_and(|(_, born)| born.elapsed() < Config::global().oidc_pending_ttl)
}

pub fn save_code(flow: AuthFlow) -> Result<String, String> {
    let id = random_id()?;
    let mut guard = store().lock().map_err(|e| e.to_string())?;
    prune(&mut guard.codes, Config::global().oidc_code_ttl);
    guard.codes.insert(id.clone(), (flow, Instant::now()));
    Ok(id)
}

pub fn take_code(code: &str) -> Option<AuthFlow> {
    let mut guard = store().lock().ok()?;
    prune(&mut guard.codes, Config::global().oidc_code_ttl);
    guard.codes.remove(code).map(|(f, _)| f)
}

pub fn create_session(sub: &str) -> Result<String, String> {
    let id = random_id()?;
    let mut guard = store().lock().map_err(|e| e.to_string())?;
    prune(&mut guard.sessions, Config::global().oidc_session_ttl);
    guard
        .sessions
        .insert(id.clone(), (sub.to_string(), Instant::now()));
    Ok(id)
}

pub fn session_sub(sid: &str) -> Option<String> {
    let mut guard = store().lock().ok()?;
    prune(&mut guard.sessions, Config::global().oidc_session_ttl);
    guard.sessions.get(sid).map(|(sub, _)| sub.clone())
}

pub fn destroy_session(sid: &str) {
    if let Ok(mut guard) = store().lock() {
        guard.sessions.remove(sid);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flow(sub: &str) -> AuthFlow {
        AuthFlow {
            client_id: "app1".into(),
            redirect_uri: "http://localhost:5173/callback".into(),
            state: Some("st".into()),
            nonce: None,
            scopes: vec!["openid".into()],
            challenge: Some("chal".into()),
            sub: sub.into(),
            groups: vec!["user".into()],
            email: Some(format!("{sub}@example.com")),
        }
    }

    #[test]
    fn login_failures_block_after_max_attempts() {
        let req = "fail-counter-req";
        let exceeded_before = login_attempts_exceeded(req);

        for _ in 0..Config::global().login_max_attempts {
            record_login_failure(req);
        }
        let exceeded_after = login_attempts_exceeded(req);

        assert!(!exceeded_before, "unknown req must not be blocked");
        assert!(exceeded_after, "req at the configured cap must be blocked");
    }

    #[test]
    fn take_pending_clears_login_failures() {
        let req = save_pending(flow("erin")).unwrap();
        for _ in 0..Config::global().login_max_attempts {
            record_login_failure(&req);
        }

        let taken = take_pending(&req);
        let blocked = login_attempts_exceeded(&req);

        assert!(taken.is_some(), "pending should be retrievable");
        assert!(
            !blocked,
            "consumed pending must also clear its failure counter"
        );
    }

    #[test]
    fn code_is_single_use() {
        let code = save_code(flow("alice")).unwrap();

        let taken = take_code(&code);
        let again = take_code(&code);

        assert!(taken.is_some(), "first take should succeed");
        assert!(again.is_none(), "code must be consumed once");
    }

    #[test]
    fn session_roundtrip_and_destroy() {
        let sid = create_session("bob").unwrap();

        let sub = session_sub(&sid);
        destroy_session(&sid);
        let after = session_sub(&sid);

        assert_eq!(sub.as_deref(), Some("bob"), "session should map to sub");
        assert_eq!(after, None, "destroyed session must be gone");
    }

    #[test]
    fn pending_take_is_single_use() {
        let id = save_pending(flow("carol")).unwrap();

        let first = take_pending(&id);
        let second = take_pending(&id);

        assert!(first.is_some(), "pending request should be retrievable");
        assert!(second.is_none(), "pending request must be consumed once");
    }

    #[test]
    fn has_pending_checks_without_consuming() {
        let id = save_pending(flow("dave")).unwrap();

        let before = has_pending(&id);
        let taken = take_pending(&id);
        let after = has_pending(&id);

        assert!(before, "stored pending id should be visible");
        assert!(taken.is_some(), "take after has_pending must still succeed");
        assert!(!after, "consumed pending id must be gone");
        assert!(!has_pending("unknown-id"), "unknown id must be false");
    }
}
