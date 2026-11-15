use axum::http::{Method, StatusCode};
use serde_json::{Value, json};

use crate::common::{
    directory::assert_available::assert_available,
    http::{req::req, send_as::send_as},
    seed::test_user::TestUser,
};

async fn put_me(user: &TestUser, attrs: Value) -> (StatusCode, Value) {
    send_as(
        &user.uid,
        Method::PUT,
        "/api/auth/me",
        Some(json!({
            "name": user.name,
            "email": user.email,
            "attrs": attrs
        })),
    )
    .await
}

async fn my_attr(user: &TestUser, label: &str) -> Option<Value> {
    let (_, body) = send_as(&user.uid, Method::GET, "/api/auth/me", None).await;
    body["attrs"]
        .as_array()?
        .iter()
        .find(|attr| attr["label"] == label)
        .cloned()
}

#[tokio::test]
async fn set_and_list_my_attrs_round_trips() {
    assert_available().await;
    let user = TestUser::new("attrrt");
    let (created, _) = user.create().await;
    if created == StatusCode::INTERNAL_SERVER_ERROR {
        return;
    }

    let (status, body) = put_me(
        &user,
        json!([
            {"label": "city", "value": "Nice"},
            {"label": "age", "value": "30"}
        ]),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "PUT /api/auth/me should 200");
    assert_eq!(
        body["attrs_saved"], true,
        "valid attrs should save, got {body}"
    );

    let city = my_attr(&user, "city").await;
    let age = my_attr(&user, "age").await;
    let city_view = city.unwrap();

    assert_eq!(
        city_view["value"], "Nice",
        "stored city should come back in me.attrs"
    );
    assert_eq!(
        city_view["names"]["fr"], "Ville",
        "the localized names from the catalog should reach the view"
    );
    assert_eq!(
        age.unwrap()["value"],
        "30",
        "stored age should come back in me.attrs"
    );

    user.delete().await;
}

#[tokio::test]
async fn empty_value_removes_the_attr() {
    assert_available().await;
    let user = TestUser::new("attrdel");
    let (created, _) = user.create().await;
    if created == StatusCode::INTERNAL_SERVER_ERROR {
        return;
    }
    put_me(&user, json!([{"label": "city", "value": "Nice"}])).await;

    let (status, body) = put_me(&user, json!([{"label": "city", "value": ""}])).await;
    assert_eq!(status, StatusCode::OK, "empty value should 200");
    assert_eq!(
        body["attrs_saved"], true,
        "empty value means deletion and should save, got {body}"
    );

    let city = my_attr(&user, "city").await;

    assert_eq!(
        city.unwrap()["value"],
        "",
        "the deleted label must show as empty in the catalog view"
    );

    user.delete().await;
}

#[tokio::test]
async fn invalid_attr_value_is_rejected() {
    assert_available().await;
    let user = TestUser::new("attrbad");
    let (created, _) = user.create().await;
    if created == StatusCode::INTERNAL_SERVER_ERROR {
        return;
    }

    let (status, body) = put_me(&user, json!([{"label": "age", "value": "not-a-number"}])).await;

    assert_eq!(
        status,
        StatusCode::OK,
        "the merge request itself should succeed"
    );
    assert_eq!(
        body["attrs_saved"], false,
        "a non-number age value should be rejected"
    );
    assert!(
        body["error"].is_string(),
        "the rejection reason should be reported, got {body}"
    );

    user.delete().await;
}

#[tokio::test]
async fn attrs_are_scoped_to_the_authenticated_user() {
    assert_available().await;
    let owner = TestUser::new("attrowner");
    let (created, _) = owner.create().await;
    if created == StatusCode::INTERNAL_SERVER_ERROR {
        return;
    }
    let viewer = TestUser::new("attrview");
    let (viewer_created, _) = viewer.create().await;
    if viewer_created == StatusCode::INTERNAL_SERVER_ERROR {
        owner.delete().await;
        return;
    }
    put_me(&owner, json!([{"label": "city", "value": "Nice"}])).await;

    let viewer_city = my_attr(&viewer, "city").await;

    assert_eq!(
        viewer_city.unwrap()["value"],
        "",
        "another user must not see the owner's attr value"
    );

    owner.delete().await;
    viewer.delete().await;
}

#[tokio::test]
async fn me_endpoint_requires_authentication() {
    assert_available().await;

    let status = req(Method::GET, "/api/auth/me", None).await;

    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "GET /api/auth/me without a token should 401"
    );
}
