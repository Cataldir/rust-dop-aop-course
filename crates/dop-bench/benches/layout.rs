use criterion::{criterion_group,criterion_main,BenchmarkId,Criterion,Throughput};
use dop_bench::*;
use std::hint::black_box;
fn benches(c: &mut Criterion) {
    let mut group=c.benchmark_group("sum_value");
    for n in [1024usize,65536,1048576] {
        let rows=fixture(n);let columns=to_columns(&rows);let world=ecs_fixture(&rows);
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::new("aos",n),&rows,|b,r|b.iter(||aos_sum(black_box(r))));
        group.bench_with_input(BenchmarkId::new("soa",n),&columns.values,|b,v|b.iter(||soa_sum(black_box(v))));
        group.bench_with_input(BenchmarkId::new("wide",n),&columns.values,|b,v|b.iter(||wide_sum(black_box(v))));
        group.bench_function(BenchmarkId::new("ecs",n),|b|b.iter(||ecs_sum(black_box(&world))));
        group.bench_with_input(BenchmarkId::new("row_to_column",n),&rows,|b,r|b.iter(||to_columns(black_box(r))));
    }
    group.finish();
}
criterion_group!(layout,benches);criterion_main!(layout);