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

const PENDING_TTL: Duration = Duration::from_secs(300);
const CODE_TTL: Duration = Duration::from_secs(60);
const SESSION_TTL: Duration = Duration::from_secs(12 * 3600);

#[derive(Default)]
struct Store {
    pending: HashMap<String, (AuthFlow, Instant)>,
    codes: HashMap<String, (AuthFlow, Instant)>,
    sessions: HashMap<String, (String, Instant)>,
}

// shortcut: sessions in memory, single-process — restart = re-login, upgrade when multi-process
static STORE: OnceLock<Mutex<Store>> = OnceLock::new();

fn store() -> &'static Mutex<Store> {
    STORE.get_or_init(|| Mutex::new(Store::default()))
}

pub(crate) fn random_id() -> Result<String, String> {
    let mut buf = [0u8; 16];
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
    prune(&mut guard.pending, PENDING_TTL);
    guard.pending.insert(id.clone(), (flow, Instant::now()));
    Ok(id)
}

pub fn take_pending(id: &str) -> Option<AuthFlow> {
    let mut guard = store().lock().ok()?;
    prune(&mut guard.pending, PENDING_TTL);
    guard.pending.remove(id).map(|(f, _)| f)
}

pub fn save_code(flow: AuthFlow) -> Result<String, String> {
    let id = random_id()?;
    let mut guard = store().lock().map_err(|e| e.to_string())?;
    prune(&mut guard.codes, CODE_TTL);
    guard.codes.insert(id.clone(), (flow, Instant::now()));
    Ok(id)
}

pub fn take_code(code: &str) -> Option<AuthFlow> {
    let mut guard = store().lock().ok()?;
    prune(&mut guard.codes, CODE_TTL);
    guard.codes.remove(code).map(|(f, _)| f)
}

pub fn create_session(sub: &str) -> Result<String, String> {
    let id = random_id()?;
    let mut guard = store().lock().map_err(|e| e.to_string())?;
    prune(&mut guard.sessions, SESSION_TTL);
    guard
        .sessions
        .insert(id.clone(), (sub.to_string(), Instant::now()));
    Ok(id)
}

pub fn session_sub(sid: &str) -> Option<String> {
    let mut guard = store().lock().ok()?;
    prune(&mut guard.sessions, SESSION_TTL);
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
}
