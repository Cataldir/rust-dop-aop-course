//! Explicit ignored integration observation; use a disposable migrated learner DB.
use sqlx::Row;

#[tokio::test]
#[ignore = "requires isolated loopback course_local PostgreSQL"]
async fn batch_insert_is_atomic_and_tenant_query_is_scoped() {
    let url = std::env::var("COURSE_DATABASE_URL").expect("explicit learner database");
    let options: sqlx::postgres::PgConnectOptions = url.parse().unwrap();
    assert_eq!(options.get_database(), Some("course_local"));
    assert!(matches!(options.get_host(), "127.0.0.1" | "localhost" | "::1"));
    let pool = sqlx::postgres::PgPoolOptions::new().connect_with(options).await.unwrap();
    let suffix = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let a = format!("atomic-a-{suffix}");
    let b = format!("atomic-b-{suffix}");
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("INSERT INTO metric_point(tenant,metric,timestamp_ms,value,tags) VALUES($1,'cpu',0,1.0,'{}'),($2,'cpu',0,9.0,'{}')")
        .bind(&a).bind(&b).execute(&mut *tx).await.unwrap();
    let rows = sqlx::query("SELECT value FROM metric_point WHERE tenant=$1")
        .bind(&a).fetch_all(&mut *tx).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get::<f64, _>("value"), 1.0);
    tx.rollback().await.unwrap();
    let after: i64 = sqlx::query_scalar("SELECT count(*) FROM metric_point WHERE tenant=$1 OR tenant=$2")
        .bind(&a).bind(&b).fetch_one(&pool).await.unwrap();
    assert_eq!(after, 0, "rolled-back batch must leave neither tenant row");
    pool.close().await;
}