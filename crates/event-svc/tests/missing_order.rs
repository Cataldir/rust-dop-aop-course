use event_svc::apply;
use shared_kernel::{EventEnvelope, OrderCreated, Tenant};

#[tokio::test]
#[ignore = "requires isolated migrated learner PostgreSQL; no execution in authoring"]
async fn failed_effect_does_not_consume_idempotency_key() {
    let url = std::env::var("COURSE_DATABASE_URL").expect("isolated learner URL");
    let options: sqlx::postgres::PgConnectOptions = url.parse().unwrap();
    assert_eq!(options.get_database(), Some("course_local"));
    assert!(matches!(options.get_host(), "127.0.0.1" | "localhost" | "::1"));
    let pool = sqlx::postgres::PgPoolOptions::new().connect_with(options).await.unwrap();
    let tenant = Tenant::new("missing-order-test".into()).unwrap();
    let event = EventEnvelope {
        version: 1,
        event_id: "missing-order-e1".into(),
        tenant,
        payload: OrderCreated { order_id: "missing-order-o1".into(), units: 2 },
    };
    // Only this test's private namespace is reset. Never point at a shared DB.
    sqlx::query("DELETE FROM processed_event WHERE tenant=$1 AND event_id=$2")
        .bind(event.tenant.as_str()).bind(&event.event_id).execute(&pool).await.unwrap();
    sqlx::query("DELETE FROM course_order WHERE tenant=$1 AND order_id=$2")
        .bind(event.tenant.as_str()).bind(&event.payload.order_id).execute(&pool).await.unwrap();
    assert!(apply(&pool, &event).await.is_err());
    let count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM processed_event WHERE tenant=$1 AND event_id=$2")
        .bind(event.tenant.as_str()).bind(&event.event_id).fetch_one(&pool).await.unwrap();
    assert_eq!(count, 0, "dedup insertion must roll back with missing domain effect");
    sqlx::query("INSERT INTO course_order(tenant,order_id,units,status) VALUES($1,$2,$3,'accepted')")
        .bind(event.tenant.as_str()).bind(&event.payload.order_id).bind(2_i32)
        .execute(&pool).await.unwrap();
    assert_eq!(apply(&pool, &event).await.unwrap(), event_svc::ApplyResult::Applied);
    assert_eq!(apply(&pool, &event).await.unwrap(), event_svc::ApplyResult::Duplicate);
    pool.close().await;
}