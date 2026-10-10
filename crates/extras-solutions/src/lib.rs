//! E1 reference: direct checked transform before an optimization.
pub fn bounded_sum(timestamps:&[i64],values:&[f64],from:i64,to:i64)->Result<f64,&'static str> {
    if timestamps.len()!=values.len()||from<0||to<=from||timestamps.iter().any(|t|*t<0)||values.iter().any(|v|!v.is_finite()){return Err("invalid input");}
    timestamps.iter().zip(values).filter(|(t,_)|**t>=from&&**t<to).try_fold(0.0,|sum,(_,value)|{let next=sum+value;if next.is_finite(){Ok(next)}else{Err("overflow")}})
}
#[cfg(test)]mod tests {
    use super::*;
    #[test]fn boundaries_and_remainder_are_part_of_the_work(){assert_eq!(bounded_sum(&[0,1,2,3,4],&[1.0,2.0,3.0,4.0,5.0],1,4),Ok(9.0));assert_eq!(bounded_sum(&[],&[],0,1),Ok(0.0));assert!(bounded_sum(&[0],&[],0,1).is_err());}
    #[test]fn nonfinite_and_overflow_refuse(){assert!(bounded_sum(&[0],&[f64::NAN],0,1).is_err());assert!(bounded_sum(&[0,0],&[f64::MAX,f64::MAX],0,1).is_err());}
}