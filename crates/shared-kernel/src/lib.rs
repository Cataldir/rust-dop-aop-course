//! Typed wire contracts and deterministic transforms; no network or storage effects.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum ContractError {
    #[error("invalid {0}")]
    Invalid(&'static str),
    #[error("tenant mismatch")]
    TenantMismatch,
}
/// A validated identity. HTTP and message adapters must still authenticate it.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Tenant(String);
impl Tenant {
    pub fn new(value: String) -> Result<Self, ContractError> {
        if valid_id(&value) { Ok(Self(value)) } else { Err(ContractError::Invalid("tenant")) }
    }
    pub fn as_str(&self) -> &str { &self.0 }
}
impl TryFrom<String> for Tenant {
    type Error = ContractError;
    fn try_from(value: String) -> Result<Self, Self::Error> { Self::new(value) }
}
impl From<Tenant> for String { fn from(t: Tenant) -> Self { t.0 } }
/// Bounded ASCII identifiers avoid unbounded keys and control characters.
pub fn valid_id(s: &str) -> bool {
    !s.is_empty() && s.len() <= 64 && s.bytes().all(|c| c.is_ascii_alphanumeric() || b"_-.".contains(&c))
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Point {
    pub tenant: Tenant,
    pub metric: String,
    pub timestamp: i64,
    pub value: f64,
    pub tags: BTreeMap<String, String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IngestBatch { pub points: Vec<Point> }
/// Private columns preserve equal lengths; construct once at the input boundary.
#[derive(Clone, Debug, Default)]
pub struct MetricColumns {
    metric: Vec<String>, timestamp: Vec<i64>, value: Vec<f64>, tags: Vec<BTreeMap<String, String>>,
}
impl MetricColumns {
    pub fn from_batch(batch: IngestBatch, tenant: &Tenant) -> Result<Self, ContractError> {
        if batch.points.is_empty() || batch.points.len() > 4096 { return Err(ContractError::Invalid("batch size")); }
        let mut columns = Self::default();
        for p in batch.points {
            if &p.tenant != tenant { return Err(ContractError::TenantMismatch); }
            if !valid_id(&p.metric) || p.timestamp < 0 || !p.value.is_finite() || p.tags.len() > 8
                || p.tags.iter().any(|(k,v)| !valid_id(k) || v.len() > 128 || v.chars().any(char::is_control)) {
                return Err(ContractError::Invalid("point"));
            }
            columns.metric.push(p.metric); columns.timestamp.push(p.timestamp);
            columns.value.push(p.value); columns.tags.push(p.tags);
        }
        Ok(columns)
    }
    pub fn len(&self) -> usize { self.value.len() }
    pub fn is_empty(&self) -> bool { self.value.is_empty() }
    pub fn metrics(&self) -> &[String] { &self.metric }
    pub fn timestamps(&self) -> &[i64] { &self.timestamp }
    pub fn values(&self) -> &[f64] { &self.value }
    pub fn tags(&self) -> &[BTreeMap<String, String>] { &self.tags }
    pub fn into_points(self, tenant: Tenant) -> Vec<Point> {
        self.metric.into_iter().zip(self.timestamp).zip(self.value).zip(self.tags)
            .map(|(((metric,timestamp),value),tags)| Point { tenant: tenant.clone(), metric,timestamp,value,tags }).collect()
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Window { pub start: i64, pub mean: f64, pub count: usize }
/// Deterministic mean over nonnegative epoch milliseconds; no implicit wall clock.
pub fn downsample(points: &[(i64, f64)], width: i64) -> Result<Vec<Window>, ContractError> {
    if width <= 0 || points.iter().any(|(t,v)| *t < 0 || !v.is_finite()) { return Err(ContractError::Invalid("window")); }
    let mut groups = BTreeMap::<i64, (f64, usize)>::new();
    for &(timestamp, value) in points {
        let entry = groups.entry(timestamp / width * width).or_default();
        entry.0 += value; entry.1 += 1;
        if !entry.0.is_finite() { return Err(ContractError::Invalid("aggregate overflow")); }
    }
    Ok(groups.into_iter().map(|(start,(sum,count))| Window { start,mean:sum/count as f64,count }).collect())
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct OrderCreated { pub order_id: String, pub units: u32 }
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EventEnvelope {
    pub version: u16, pub event_id: String, pub tenant: Tenant, pub payload: OrderCreated,
}
impl EventEnvelope {
    pub fn validate(&self) -> Result<(), ContractError> {
        if self.version != 1 || !valid_id(&self.event_id) || !valid_id(&self.payload.order_id)
            || self.payload.units == 0 || self.payload.units > 10000 { return Err(ContractError::Invalid("event")); }
        Ok(())
    }
}
/// Single batch duplicate check; durable deduplication belongs to the database adapter.
pub fn unique_event_keys(events: &[EventEnvelope]) -> Result<BTreeSet<(String,String)>, ContractError> {
    let mut keys = BTreeSet::new();
    for e in events { e.validate()?; keys.insert((e.tenant.as_str().to_owned(),e.event_id.clone())); }
    Ok(keys)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn point(t: &str, v: f64) -> Point { Point { tenant:Tenant::new(t.into()).unwrap(),metric:"cpu".into(),timestamp:1000,value:v,tags:BTreeMap::new() } }
    #[test] fn tenant_cannot_be_empty_or_controlled() { assert!(Tenant::new("".into()).is_err()); assert!(Tenant::new("a\nb".into()).is_err()); }
    #[test] fn columns_reject_cross_tenant_and_nan_before_exposing_data() {
        let a=Tenant::new("a".into()).unwrap();
        assert_eq!(MetricColumns::from_batch(IngestBatch { points:vec![point("b",1.0)] },&a).unwrap_err(),ContractError::TenantMismatch);
        assert!(MetricColumns::from_batch(IngestBatch { points:vec![point("a",f64::NAN)] },&a).is_err());
    }
    #[test] fn column_roundtrip_preserves_tags_order_and_values() {
        let a=Tenant::new("a".into()).unwrap(); let points=vec![point("a",3.0),point("a",9.0)];
        let c=MetricColumns::from_batch(IngestBatch { points:points.clone() },&a).unwrap(); assert_eq!(c.len(),2); assert_eq!(c.into_points(a),points);
    }
    #[test] fn windows_have_explicit_boundaries_and_empty_is_valid() {
        let actual=downsample(&[(1999,2.0),(1000,4.0),(2000,8.0)],1000).unwrap();
        assert_eq!(actual,vec![Window{start:1000,mean:3.0,count:2},Window{start:2000,mean:8.0,count:1}]);
        assert!(downsample(&[],1000).unwrap().is_empty()); assert!(downsample(&[],0).is_err());
    }
    #[test] fn huge_values_cannot_make_nonfinite_response() { assert!(downsample(&[(0,f64::MAX),(0,f64::MAX)],1).is_err()); }
}