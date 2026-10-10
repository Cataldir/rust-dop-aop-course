use rdkafka::{ClientConfig,Message,consumer::{Consumer,StreamConsumer,CommitMode}};
#[tokio::main] async fn main()->anyhow::Result<()> {
    let consumer:StreamConsumer=ClientConfig::new().set("bootstrap.servers","127.0.0.1:9092")
        .set("group.id","course-fulfill").set("enable.auto.commit","false").set("enable.auto.offset.store","false")
        .set("auto.offset.reset","earliest").create()?;
    consumer.subscribe(&["orders.created"])?;
    let pool=sqlx::PgPool::connect(&std::env::var("COURSE_DATABASE_URL")?).await?;
    loop {tokio::select! {
        _=tokio::signal::ctrl_c()=>break,
        received=consumer.recv()=>{
            let message=received?;let payload=message.payload().ok_or_else(||anyhow::anyhow!("empty Kafka record"))?;
            if payload.len()>1024*1024 {anyhow::bail!("oversized record: leave offset uncommitted");}
            let event:shared_kernel::EventEnvelope=serde_json::from_slice(payload)?;
            event_svc::apply(&pool,&event).await?;
            consumer.commit_message(&message,CommitMode::Sync)?;
        }
    }}
    consumer.unsubscribe();pool.close().await;Ok(())
}