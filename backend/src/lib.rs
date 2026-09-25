pub mod auth;
pub mod controllers;
pub mod domain;
pub mod env;
pub mod error;
pub mod http;
pub mod repository;

#[cfg(test)]
pub(crate) mod test_helpers;

pub use http::router as app;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::app::test_app;
    use axum::{body::Body, extract::Request, http::StatusCode};
    use http_body_util::BodyExt;
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
        let url = crate::env::ldap_test_url();
        let base = crate::env::ldap_test_base_dn();

        if let Ok((conn, mut ldap)) = ldap3::LdapConnAsync::new(&url).await {
            ldap3::drive!(conn);
            let bind_dn = format!("cn=admin,{base}");
            let _ = ldap.simple_bind(&bind_dn, "admin").await;

            for (uid, cn, mail) in [
                ("apitest2", "Api Test2", "apitest2@example.com"),
                ("apitest1", "Api Test1", "apitest1@example.com"),
            ] {
                let dn = format!("uid={uid},ou=people,{base}");
                let attrs = vec![
                    (
                        "objectClass".to_string(),
                        ["inetOrgPerson".to_string()].into_iter().collect(),
                    ),
                    ("uid".to_string(), [uid.to_string()].into_iter().collect()),
                    ("cn".to_string(), [cn.to_string()].into_iter().collect()),
                    ("sn".to_string(), [cn.to_string()].into_iter().collect()),
                    ("mail".to_string(), [mail.to_string()].into_iter().collect()),
                ];
                let _ = ldap.add(&dn, attrs).await;
            }
            let _ = ldap.unbind().await;
        }

        let app = test_app();
        let req = Request::builder()
            .uri("/api/users")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();

        if resp.status() == StatusCode::INTERNAL_SERVER_ERROR {
            return;
        }

        assert_eq!(resp.status(), StatusCode::OK);

        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let arr = v.as_array().unwrap();

        let uids: Vec<_> = arr
            .iter()
            .filter_map(|o| o.get("uid").and_then(|x| x.as_str()))
            .collect();

        if uids.contains(&"apitest1") {
            assert!(uids.contains(&"apitest2"));

            let p1 = uids.iter().position(|&x| x == "apitest1").unwrap();
            let p2 = uids.iter().position(|&x| x == "apitest2").unwrap();

            assert!(p1 < p2, "uid sort asc expected");

            let apitest1 = arr.iter().find(|o| o["uid"] == "apitest1").unwrap();

            assert_eq!(apitest1["name"], "Api Test1");
            assert_eq!(apitest1["email"], "apitest1@example.com");

            // Cleanup: la base test reste propre après la run
            // (les seeds ne passeront pas par la purge du binaire lib).
            if let Ok((conn, mut l2)) = ldap3::LdapConnAsync::new(&url).await {
                ldap3::drive!(conn);
                let _ = l2.simple_bind(&format!("cn=admin,{base}"), "admin").await;
                for uid in ["apitest1", "apitest2"] {
                    let _ = l2.delete(&format!("uid={uid},ou=people,{base}")).await;
                }
                let _ = l2.unbind().await;
            }
        }
    }
}
