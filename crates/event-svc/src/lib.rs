//! Durable domain effects and dedup share one transaction; broker ack is after commit.
use shared_kernel::EventEnvelope;
use sqlx::{PgPool,Row};
use sha2::{Digest,Sha256};
/// Semantic JSON hash avoids treating harmless wire whitespace as an id conflict.
pub fn fingerprint(event:&EventEnvelope)->Result<String,serde_json::Error> {
    Ok(format!("{:x}",Sha256::digest(serde_json::to_vec(event)?)))
}
#[derive(Debug,PartialEq)] pub enum ApplyResult { Applied,Duplicate }
/// Existing same key/different payload is a conflict, not a successful replay.
pub async fn apply(pool:&PgPool,event:&EventEnvelope)->anyhow::Result<ApplyResult> {
    event.validate()?;let hash=fingerprint(event)?;
    let mut tx=pool.begin().await?;
    let inserted=sqlx::query("INSERT INTO processed_event(tenant,event_id,payload_hash) VALUES($1,$2,$3) ON CONFLICT DO NOTHING")
        .bind(event.tenant.as_str()).bind(&event.event_id).bind(&hash).execute(&mut *tx).await?.rows_affected();
    if inserted==0 {
        let row=sqlx::query("SELECT payload_hash FROM processed_event WHERE tenant=$1 AND event_id=$2")
            .bind(event.tenant.as_str()).bind(&event.event_id).fetch_one(&mut *tx).await?;
        let existing:String=row.try_get("payload_hash")?;
        if existing!=hash {anyhow::bail!("idempotency key conflict");}
        tx.commit().await?;return Ok(ApplyResult::Duplicate);
    }
    let updated=sqlx::query("UPDATE course_order SET status='fulfilled' WHERE tenant=$1 AND order_id=$2 AND units=$3")
        .bind(event.tenant.as_str()).bind(&event.payload.order_id).bind(event.payload.units as i32).execute(&mut *tx).await?.rows_affected();
    if updated!=1 {anyhow::bail!("missing or conflicting order");}
    tx.commit().await?;Ok(ApplyResult::Applied)
}
/// Safe archived views over an owned aligned buffer; no unchecked/unsafe access.
#[derive(rkyv::Archive,rkyv::Serialize,rkyv::Deserialize)]
pub struct HotBatch { pub values:Vec<f64>,pub timestamps:Vec<i64> }
pub fn archived_total(batch:&HotBatch)->anyhow::Result<f64> {
    if batch.values.len()!=batch.timestamps.len()||batch.values.iter().any(|v|!v.is_finite())||batch.timestamps.iter().any(|t|*t<0) {anyhow::bail!("invalid columnar archive");}
    let bytes=rkyv::to_bytes::<rkyv::rancor::Error>(batch)?;
    let archived=rkyv::access::<ArchivedHotBatch,rkyv::rancor::Error>(&bytes)?;
    let total:f64=archived.values.iter().map(|v|v.to_native()).sum();
    if !total.is_finite() {anyhow::bail!("archive aggregate overflow");}
    Ok(total)
}
pub use aspects::HeaderCarrier;
#[cfg(test)] mod tests {
    use super::*;
    fn event(units:u32)->EventEnvelope {EventEnvelope{version:1,event_id:"e1".into(),tenant:shared_kernel::Tenant::new("a".into()).unwrap(),payload:shared_kernel::OrderCreated{order_id:"o1".into(),units}}}
    #[test] fn hash_changes_with_semantics() {assert_ne!(fingerprint(&event(1)).unwrap(),fingerprint(&event(2)).unwrap());assert_eq!(fingerprint(&event(1)).unwrap(),fingerprint(&event(1)).unwrap());}
    #[test] fn archived_columns_check_lengths_before_access() {assert!(archived_total(&HotBatch{values:vec![1.0],timestamps:vec![]}).is_err());assert_eq!(archived_total(&HotBatch{values:vec![1.0,3.0],timestamps:vec![1,2]}).unwrap(),4.0);}
    #[test] fn archived_columns_refuse_aggregate_overflow() {assert!(archived_total(&HotBatch{values:vec![f64::MAX,f64::MAX],timestamps:vec![1,2]}).is_err());}
}
