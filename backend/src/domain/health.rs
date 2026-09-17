use serde::Serialize;

#[derive(Serialize)]
pub struct Health {
    pub status: String,
}

pub fn health_status() -> Health {
    Health {
        status: "ok".into(),
    }
}
