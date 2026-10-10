//! Equal arithmetic with deliberately different layouts; no result claims.
#[derive(Clone, Copy)]
pub struct Row { pub value: f64, pub timestamp: u64, pub cold: [u8; 48] }
pub struct Columns { pub values: Vec<f64>, pub timestamps: Vec<u64>, pub cold: Vec<[u8;48]> }
pub fn fixture(n: usize) -> Vec<Row> { (0..n).map(|i| Row{ value:(i%100) as f64, timestamp:i as u64,cold:[0;48] }).collect() }
pub fn to_columns(rows: &[Row]) -> Columns { Columns { values:rows.iter().map(|r|r.value).collect(),timestamps:rows.iter().map(|r|r.timestamp).collect(),cold:rows.iter().map(|r|r.cold).collect() } }
pub fn aos_sum(rows: &[Row]) -> f64 { rows.iter().map(|r|r.value).sum() }
pub fn soa_sum(values: &[f64]) -> f64 { values.iter().sum() }
/// Stable portable vector operations with an explicit scalar remainder.
pub fn wide_sum(values: &[f64]) -> f64 {
    let mut sum=wide::f64x4::ZERO; let mut chunks=values.chunks_exact(4);
    for c in &mut chunks { sum += wide::f64x4::from([c[0],c[1],c[2],c[3]]); }
    sum.to_array().iter().sum::<f64>() + chunks.remainder().iter().sum::<f64>()
}
#[derive(Clone,Copy)] pub struct Value(pub f64);
pub fn ecs_fixture(rows: &[Row]) -> hecs::World {
    let mut world=hecs::World::new(); for r in rows { world.spawn((Value(r.value),r.timestamp,r.cold)); } world
}
pub fn ecs_sum(world: &hecs::World) -> f64 { world.query::<&Value>().iter().map(|(_,v)|v.0).sum() }
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn all_layouts_have_equal_work() { for n in [0,1,3,4,5,1025] { let rows=fixture(n);let c=to_columns(&rows);let world=ecs_fixture(&rows);let expected=aos_sum(&rows);assert_eq!(soa_sum(&c.values),expected);assert_eq!(wide_sum(&c.values),expected);assert_eq!(ecs_sum(&world),expected); } }
}