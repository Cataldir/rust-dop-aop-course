//! Intentional pure policy blank; the reference's real effect/ack joins live in event-svc.
#[derive(Debug,PartialEq)]pub enum Effect {Committed,Duplicate,TransientFailure,Invalid}
#[derive(Debug,PartialEq)]pub enum Decision {Ack,Retry,DeadLetter}
pub fn decide(effect:Effect,attempt:u32)->Result<Decision,&'static str>{let _=(effect,attempt);Err("exercise not implemented")}
#[cfg(test)]mod tests {
    use super::*;
    #[test]fn committed_and_replay_ack(){assert_eq!(decide(Effect::Committed,1),Ok(Decision::Ack));assert_eq!(decide(Effect::Duplicate,5),Ok(Decision::Ack));}
    #[test]fn exhaustion_is_bounded(){assert_eq!(decide(Effect::TransientFailure,1),Ok(Decision::Retry));assert_eq!(decide(Effect::TransientFailure,5),Ok(Decision::DeadLetter));assert_eq!(decide(Effect::Invalid,1),Ok(Decision::DeadLetter));assert!(decide(Effect::TransientFailure,0).is_err());}
}