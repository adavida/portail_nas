pub mod domain;
pub mod http;

#[cfg(test)]
pub(crate) mod test_helpers;

pub use http::router as app;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::app::test_app;
    use axum::{body::Body, extract::Request, http::StatusCode};
    use http_body_util::BodyExt;
    use portail_directory::domain::ldap::{Email, Name, Password};
    use portail_directory::domain::users::NewUser;
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_returns_ok() {
        let app = app();
        let req = Request::builder()
            .uri("/api/health")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();

        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "GET /api/health should return 200 OK"
        );

        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(v["status"], "ok", "health status should be ok");
    }

    #[tokio::test]
    async fn unknown_returns_404() {
        let app = test_app();
        let req = Request::builder()
            .uri("/api/unknown")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();

        assert_eq!(
            resp.status(),
            StatusCode::NOT_FOUND,
            "unknown route should return 404"
        );
    }

    #[tokio::test]
    async fn users_returns_array() {
        let app = test_app();
        let req = Request::builder()
            .uri("/api/users")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();

        if resp.status() == StatusCode::INTERNAL_SERVER_ERROR {
            let body = resp.into_body().collect().await.unwrap().to_bytes();
            let v: serde_json::Value = serde_json::from_slice(&body).unwrap();

            assert!(v.get("error").is_some(), "500 should return {{error}}");
            return;
        }

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or(""),
            "application/json"
        );

        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert!(v.is_array(), "GET /api/users should return []");
        if let Some(first) = v.as_array().unwrap().first() {
            assert!(first.get("uid").is_some(), "User.uid missing");
            assert!(first.get("name").is_some(), "User.name missing");
            assert!(first.get("email").is_some(), "User.email missing");
        }
    }

    #[tokio::test]
    async fn users_lists_seeded_user() {
        for uid in ["apitest1", "apitest2"] {
            let _ = portail_directory::repository::delete_user(&uid_string(uid)).await;
        }
        for (uid, name, email) in [
            ("apitest1", "Api Test1", "apitest1@example.com"),
            ("apitest2", "Api Test2", "apitest2@example.com"),
        ] {
            let new = NewUser {
                uid: uid_string(uid),
                name: Name::try_new(name.into()).unwrap(),
                email: Email::try_new(email.into()).unwrap(),
                password: Password::try_new("secret123".into()).unwrap(),
            };
            portail_directory::repository::create_user(new)
                .await
                .unwrap();
        }

        let app = test_app();
        let req = Request::builder()
            .uri("/api/users")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();

        assert_eq!(resp.status(), StatusCode::OK);

        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let arr = v.as_array().unwrap();

        let uids: Vec<_> = arr
            .iter()
            .filter_map(|o| o.get("uid").and_then(|x| x.as_str()))
            .collect();

        let p1 = uids
            .iter()
            .position(|&x| x == "apitest1")
            .expect("apitest1 should be listed");
        let p2 = uids
            .iter()
            .position(|&x| x == "apitest2")
            .expect("apitest2 should be listed");

        assert!(p1 < p2, "uid sort asc expected");

        let apitest1 = arr.iter().find(|o| o["uid"] == "apitest1").unwrap();

        assert_eq!(apitest1["name"], "Api Test1");
        assert_eq!(apitest1["email"], "apitest1@example.com");

        for uid in ["apitest1", "apitest2"] {
            let _ = portail_directory::repository::delete_user(&uid_string(uid)).await;
        }
    }

    fn uid_string(uid: &str) -> portail_directory::domain::ldap::Uid {
        portail_directory::domain::ldap::Uid::try_new(uid.into()).unwrap()
    }
}
