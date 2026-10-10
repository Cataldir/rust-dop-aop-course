//! Intentional boundary exercise: return validated columns, not a partially accepted batch.
use shared_kernel::{Tenant,IngestBatch,MetricColumns,ContractError};
pub fn accept(batch:IngestBatch,tenant:&Tenant)->Result<MetricColumns,ContractError> {let _=(batch,tenant);Err(ContractError::Invalid("exercise not implemented"))}
#[cfg(test)] mod tests {
    use super::*;use std::collections::BTreeMap;
    fn batch(tenant:&str)->IngestBatch {IngestBatch{points:vec![shared_kernel::Point{tenant:Tenant::new(tenant.into()).unwrap(),metric:"cpu".into(),timestamp:0,value:2.0,tags:BTreeMap::new()}]}}
    #[test] fn valid_columns_and_foreign_tenant() {let a=Tenant::new("a".into()).unwrap();assert_eq!(accept(batch("a"),&a).unwrap().len(),1);assert_eq!(accept(batch("b"),&a).unwrap_err(),ContractError::TenantMismatch);}
    #[test] fn empty_input_is_refused() {assert!(accept(IngestBatch{points:vec![]},&Tenant::new("a".into()).unwrap()).is_err());}
}