use axum::{body::{Body,to_bytes},http::{Request,StatusCode}};
use axum_api::*;
use std::{sync::Arc,time::{SystemTime,UNIX_EPOCH}};
use tower::ServiceExt;
fn state()->AppState {let pool=sqlx::postgres::PgPoolOptions::new().connect_lazy("postgres://unused:unused@127.0.0.1:1/course_local").unwrap();let(tail,_)=tokio::sync::broadcast::channel(2);AppState(Arc::new(Inner{pool,tail,secret:"test-only-secret-at-least-32-bytes-long".into()}))}
fn token(s:&AppState,tenant:&str,expiry:usize)->String {
    jsonwebtoken::encode(&jsonwebtoken::Header::default(),&Claims{sub:"test".into(),tenant:shared_kernel::Tenant::new(tenant.into()).unwrap(),exp:expiry,iss:"course-local".into(),aud:"course-api".into()},&jsonwebtoken::EncodingKey::from_secret(s.0.secret.as_bytes())).unwrap()
}
#[tokio::test] async fn unauthenticated_post_never_contacts_database() {
    let response=router(state()).unwrap().oneshot(Request::post("/ingest").header("content-type","application/json").body(Body::from(r#"{"points":[]}"#)).unwrap()).await.unwrap();
    assert_eq!(response.status(),StatusCode::UNAUTHORIZED);assert!(response.headers().contains_key("x-request-id"));
}
#[tokio::test] async fn authenticated_mismatched_payload_is_forbidden_before_storage() {
    let state=state();let expiry=SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize+60;let bearer=token(&state,"a",expiry);
    let response=router(state).unwrap().oneshot(Request::post("/ingest").header("authorization",format!("Bearer {bearer}")).header("content-type","application/json").body(Body::from(r#"{"points":[{"tenant":"b","metric":"cpu","timestamp":0,"value":1.0,"tags":{}}]}"#)).unwrap()).await.unwrap();
    assert_eq!(response.status(),StatusCode::FORBIDDEN);assert!(to_bytes(response.into_body(),1024).await.unwrap().len()>0);
}
#[tokio::test] async fn expired_or_modified_token_is_refused() {
    let state=state();let expired=token(&state,"a",1);assert_eq!(decode_tenant(&expired,&state.0.secret),Err(StatusCode::UNAUTHORIZED));assert_eq!(decode_tenant("not-a-token",&state.0.secret),Err(StatusCode::UNAUTHORIZED));
}
#[tokio::test] async fn tenant_quota_isolated_and_auth_precedes_key_extraction() {
    let state=state();let now=SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;let a=token(&state,"a",now+60);let b=token(&state,"b",now+60);let app=router(state).unwrap();
    let req=|token:&str|Request::post("/ingest").header("authorization",format!("Bearer {token}")).header("content-type","application/json").body(Body::from(r#"{"points":[]}"#)).unwrap();
    for _ in 0..20 {assert_eq!(app.clone().oneshot(req(&a)).await.unwrap().status(),StatusCode::UNPROCESSABLE_ENTITY);}
    assert_eq!(app.clone().oneshot(req(&a)).await.unwrap().status(),StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(app.oneshot(req(&b)).await.unwrap().status(),StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn valid_syntax_with_modified_signature_wrong_issuer_or_audience_is_refused() {
    let state=state();
    let now=SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize;
    let valid=token(&state,"a",now+60);
    let mut bytes=valid.into_bytes();
    let signature_start=bytes.iter().rposition(|b|*b==b'.').unwrap()+1;
    bytes[signature_start]=if bytes[signature_start]==b'A' {b'B'} else {b'A'};
    let tampered=String::from_utf8(bytes).unwrap();
    assert_eq!(decode_tenant(&tampered,&state.0.secret),Err(StatusCode::UNAUTHORIZED));
    for (issuer,audience) in [("wrong","course-api"),("course-local","wrong")] {
        let encoded=jsonwebtoken::encode(&jsonwebtoken::Header::default(),&Claims {
            sub:"test".into(),tenant:shared_kernel::Tenant::new("a".into()).unwrap(),
            exp:now+60,iss:issuer.into(),aud:audience.into(),
        },&jsonwebtoken::EncodingKey::from_secret(state.0.secret.as_bytes())).unwrap();
        assert_eq!(decode_tenant(&encoded,&state.0.secret),Err(StatusCode::UNAUTHORIZED));
    }
}