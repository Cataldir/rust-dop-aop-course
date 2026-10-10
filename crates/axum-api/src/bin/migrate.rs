#[tokio::main] async fn main()->anyhow::Result<()> {
    let url=std::env::var("COURSE_DATABASE_URL")?;
    let expected=std::env::var("COURSE_MIGRATION_CONFIRM")?;
    let options:sqlx::postgres::PgConnectOptions=url.parse()?;
    let local=matches!(options.get_host(),"127.0.0.1"|"localhost"|"::1");
    if expected!="course_local" || options.get_database()!=Some("course_local") || !local {
        anyhow::bail!("refused: only confirmed loopback learner course_local database");
    }
    let pool=sqlx::postgres::PgPoolOptions::new().max_connections(1).connect_with(options).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;pool.close().await;Ok(())
}
