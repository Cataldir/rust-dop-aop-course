use async_nats::jetstream::{self,consumer::{pull,AckPolicy},stream};
use event_svc::{HeaderCarrier,apply};
use futures_util::StreamExt;
use opentelemetry::global;
use shared_kernel::EventEnvelope;
use sqlx::Row;
use std::time::Duration;
use tracing_opentelemetry::OpenTelemetrySpanExt;
#[tokio::main] async fn main()->anyhow::Result<()> {
    let provider=aspects::telemetry("course-events")?;
    let pool=sqlx::PgPool::connect(&std::env::var("COURSE_DATABASE_URL")?).await?;
    let client=async_nats::connect(std::env::var("COURSE_NATS_URL").unwrap_or_else(|_|"nats://127.0.0.1:4222".into())).await?;
    let js=jetstream::new(client);
    let orders=js.get_or_create_stream(stream::Config{name:"COURSE_ORDERS".into(),subjects:vec!["orders.created".into(),"orders.dlq".into()],max_messages:10000,max_bytes:64*1024*1024,..Default::default()}).await?;
    let mode=std::env::args().nth(1).unwrap_or_else(||"consume".into());
    if mode=="publish" {publish(&pool,&js).instrument(tracing::info_span!("publish_outbox")).await?;} else if mode=="consume" {
        let consumer=orders.get_or_create_consumer("course-fulfill",pull::Config{durable_name:Some("course-fulfill".into()),filter_subject:"orders.created".into(),ack_policy:AckPolicy::Explicit,ack_wait:Duration::from_secs(30),max_ack_pending:32,max_deliver:6,..Default::default()}).await?;
        let mut messages=consumer.messages().await?;
        loop {tokio::select! {
            _=tokio::signal::ctrl_c()=>break,
            message=messages.next()=>{let Some(message)=message else {break;};let message=message?;
                let mut carrier=HeaderCarrier(Default::default());
                if let Some(headers)=&message.headers {for key in ["traceparent","tracestate"] {if let Some(value)=headers.get(key){carrier.0.insert(key.into(),value.to_string());}}}
                let parent=global::get_text_map_propagator(|p|p.extract(&carrier));
                let span=tracing::info_span!("fulfill_order");
                // Observability failure cannot choose or suppress the domain effect.
                if span.set_parent(parent).is_err() {tracing::warn!("trace parent unavailable");}
                let result=handle(&pool,&js,&message).instrument(span).await;
                if let Err(error)=result {tracing::warn!(failure=%error,"delivery unresolved; nack");message.ack_with(jetstream::AckKind::Nak(Some(Duration::from_secs(2)))).await?;}
            }
        }}
    } else {anyhow::bail!("mode must be publish or consume");}
    pool.close().await;provider.shutdown()?;Ok(())
}
use tracing::Instrument;
async fn publish(pool:&sqlx::PgPool,js:&jetstream::Context)->anyhow::Result<()> {
    // Serial loop is deliberate: one learner dispatcher. FOR UPDATE holds only this batch.
    let mut tx=pool.begin().await?;
    let rows=sqlx::query("SELECT tenant,event_id,payload,trace_headers FROM outbox WHERE sent=false ORDER BY tenant,event_id LIMIT 32 FOR UPDATE SKIP LOCKED").fetch_all(&mut *tx).await?;
    for row in rows {
        let tenant:String=row.try_get("tenant")?;let id:String=row.try_get("event_id")?;let value:serde_json::Value=row.try_get("payload")?;
        let trace_value:serde_json::Value=row.try_get("trace_headers")?;
        let carrier=HeaderCarrier(serde_json::from_value(trace_value)?);
        let mut headers=async_nats::HeaderMap::new();for (k,v) in carrier.0 {headers.insert(k,v);}
        headers.insert("Nats-Msg-Id",format!("{tenant}.{id}"));
        // First await sends; second awaits durable publish acknowledgement.
        js.publish_with_headers("orders.created",headers,serde_json::to_vec(&value)?.into()).await?.await?;
        sqlx::query("UPDATE outbox SET sent=true WHERE tenant=$1 AND event_id=$2").bind(tenant).bind(id).execute(&mut *tx).await?;
    }
    // Crash after publish/before this commit yields redelivery; consumer DB dedup contains it.
    tx.commit().await?;Ok(())
}
async fn handle(pool:&sqlx::PgPool,js:&jetstream::Context,message:&jetstream::Message)->anyhow::Result<()> {
    let decoded=if message.payload.len()>1024*1024 {Err(anyhow::anyhow!("oversized payload"))} else {serde_json::from_slice::<EventEnvelope>(&message.payload).map_err(anyhow::Error::from)};
    match decoded {
        Ok(event) if event.validate().is_ok()=>match apply(pool,&event).await {
            Ok(_)=>{message.double_ack().await?;Ok(())}
            Err(error)=>{
                let attempts=message.info()?.delivered;
                if attempts>=5 {dead_letter(pool,js,message,&event.tenant.as_str().to_owned(),&event.event_id,"apply_failed").await?;Ok(())}
                else {Err(error)}
            }
        },
        _=>{let id=format!("invalid.{}",message.info()?.stream_sequence);dead_letter(pool,js,message,"invalid",&id,"decode_or_contract_failed").await?;Ok(())}
    }
}
async fn dead_letter(pool:&sqlx::PgPool,js:&jetstream::Context,message:&jetstream::Message,tenant:&str,id:&str,reason:&str)->anyhow::Result<()> {
    // Persist bounded evidence before DLQ ack. Do not record tokens or headers.
    if message.payload.len()>1024*1024 {anyhow::bail!("oversized poison input requires operator quarantine");}
    sqlx::query("INSERT INTO dead_letter(tenant,event_id,reason,payload) VALUES($1,$2,$3,$4) ON CONFLICT DO NOTHING").bind(tenant).bind(id).bind(reason).bind(message.payload.as_ref()).execute(pool).await?;
    let mut headers=async_nats::HeaderMap::new();headers.insert("Nats-Msg-Id",format!("dlq.{tenant}.{id}"));
    js.publish_with_headers("orders.dlq",headers,message.payload.clone()).await?.await?;
    message.double_ack().await?;Ok(())
}
