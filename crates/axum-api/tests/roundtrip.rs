use axum::{body::{Body, to_bytes}, http::{Request, StatusCode}};
use axum_api::{AppState, Claims, Inner, router};
use jsonwebtoken::{EncodingKey, Header};
use shared_kernel::Tenant;
use std::{sync::Arc, time::{SystemTime, UNIX_EPOCH}};
use tower::ServiceExt;

#[tokio::test]
#[ignore = "requires isolated migrated learner PostgreSQL; no execution in authoring"]
async fn authenticated_batch_roundtrip_and_foreign_tenant_refusal() {
    let url = std::env::var("COURSE_DATABASE_URL").unwrap();
    let options: sqlx::postgres::PgConnectOptions = url.parse().unwrap();
    assert_eq!(options.get_database(), Some("course_local"));
    assert!(matches!(options.get_host(), "127.0.0.1" | "localhost" | "::1"));
    let pool = sqlx::postgres::PgPoolOptions::new().connect_with(options).await.unwrap();
    let secret = "synthetic-roundtrip-secret-over-32-bytes".to_owned();
    let tenant = Tenant::new("roundtrip-a".into()).unwrap();
    let (tail, _) = tokio::sync::broadcast::channel(128);
    let state = AppState(Arc::new(Inner { pool: pool.clone(), tail, secret: secret.clone() }));
    let app = router(state).unwrap();
    let token = jsonwebtoken::encode(&Header::default(), &Claims {
        sub: "test".into(), tenant,
        exp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize + 60,
        iss: "course-local".into(), aud: "course-api".into(),
    }, &EncodingKey::from_secret(secret.as_bytes())).unwrap();
    let make_ingest = |tenant: &str| Request::builder().method("POST").uri("/ingest")
        .header("authorization", format!("Bearer {token}")).header("content-type", "application/json")
        .body(Body::from(format!(
            "{{\"points\":[{{\"tenant\":\"{tenant}\",\"metric\":\"cpu\",\"timestamp\":0,\"value\":2.0,\"tags\":{{}}}},{{\"tenant\":\"{tenant}\",\"metric\":\"cpu\",\"timestamp\":500,\"value\":4.0,\"tags\":{{}}}}]}}"
        ))).unwrap();
    let response = app.clone().oneshot(make_ingest("roundtrip-a")).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    let response = app.clone().oneshot(make_ingest("roundtrip-b")).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let response = app.oneshot(Request::builder()
        .uri("/query?metric=cpu&from=0&to=1000&width=1000")
        .header("authorization", format!("Bearer {token}")).body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), 8192).await.unwrap();
    let windows: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(windows[0]["count"], 2);
    assert_eq!(windows[0]["mean"], 3.0);
    let foreign: i64 = sqlx::query_scalar("SELECT count(*) FROM metric_point WHERE tenant='roundtrip-b'")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(foreign, 0);
    pool.close().await;
}