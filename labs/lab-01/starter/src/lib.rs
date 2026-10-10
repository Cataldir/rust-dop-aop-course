//! Intentional learner blank. Implement a checked column sum before benchmarking.
pub fn total(values:&[u64])->Result<u64,&'static str> {let _=values;Err("exercise not implemented")}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn empty_and_normal_batch() {assert_eq!(total(&[]),Ok(0));assert_eq!(total(&[4,8,16]),Ok(28));}
    #[test] fn overflow_is_refused() {assert_eq!(total(&[u64::MAX,1]),Err("overflow"));}
}