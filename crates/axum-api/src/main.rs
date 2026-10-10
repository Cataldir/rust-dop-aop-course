use axum_api::{AppState,Inner,router};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
#[tokio::main] async fn main()->anyhow::Result<()> {
    let provider=aspects::telemetry("course-api")?;
    metrics_exporter_prometheus::PrometheusBuilder::new().with_http_listener(([127,0,0,1],9000)).install()?;
    let secret=std::env::var("COURSE_JWT_SECRET")?;
    if secret.len()<32 {anyhow::bail!("COURSE_JWT_SECRET needs at least 32 bytes");}
    let url=std::env::var("COURSE_DATABASE_URL")?;
    let pool=PgPoolOptions::new().max_connections(8).connect(&url).await?;
    // Explicit learner migration command is a separate binary; serving never migrates.
    let (tail,_)=tokio::sync::broadcast::channel(128);
    let app=router(AppState(Arc::new(Inner{pool:pool.clone(),tail,secret})))?;
    // Host defaults to loopback. A local container must explicitly bind its own
    // network interface, with the host port still published only on loopback.
    let bind:std::net::SocketAddr=std::env::var("COURSE_BIND")
        .unwrap_or_else(|_| "127.0.0.1:3000".into()).parse()?;
    let listener=tokio::net::TcpListener::bind(bind).await?;
    let result=axum::serve(listener,app).with_graceful_shutdown(shutdown()).await;
    pool.close().await;provider.shutdown()?;result?;Ok(())
}
async fn shutdown() {
    #[cfg(unix)] {let mut signal=tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).expect("install SIGTERM handler");tokio::select!{_=tokio::signal::ctrl_c()=>{},_=signal.recv()=>{}}}
    #[cfg(not(unix))] {let _=tokio::signal::ctrl_c().await;}
}
