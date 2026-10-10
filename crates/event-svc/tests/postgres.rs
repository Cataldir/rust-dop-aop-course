use event_svc::{apply,ApplyResult};
use shared_kernel::*;
#[tokio::test] #[ignore="requires isolated migrated learner PostgreSQL"]
async fn replay_and_key_conflict_preserve_single_effect() {
    let url=std::env::var("COURSE_DATABASE_URL").unwrap();
    let options:sqlx::postgres::PgConnectOptions=url.parse().unwrap();
    assert_eq!(options.get_database(),Some("course_local"));
    assert!(matches!(options.get_host(),"127.0.0.1"|"localhost"|"::1"));
    let pool=sqlx::postgres::PgPoolOptions::new().connect_with(options).await.unwrap();
    let tenant=Tenant::new("replay-test".into()).unwrap();
    // Fixed private test IDs. Run in a throwaway learner database, never shared production.
    sqlx::query("INSERT INTO course_order VALUES($1,'o1',2,'accepted') ON CONFLICT DO NOTHING").bind(tenant.as_str()).execute(&pool).await.unwrap();
    let event=EventEnvelope{version:1,event_id:"replay-e1".into(),tenant,payload:OrderCreated{order_id:"o1".into(),units:2}};
    assert!(matches!(apply(&pool,&event).await.unwrap(),ApplyResult::Applied|ApplyResult::Duplicate));
    assert_eq!(apply(&pool,&event).await.unwrap(),ApplyResult::Duplicate);
    let mut changed=event.clone();changed.payload.units=3;assert!(apply(&pool,&changed).await.is_err());
    let count:i64=sqlx::query_scalar("SELECT count(*) FROM processed_event WHERE tenant=$1 AND event_id=$2").bind(event.tenant.as_str()).bind(&event.event_id).fetch_one(&pool).await.unwrap();assert_eq!(count,1);
    pool.close().await;
}