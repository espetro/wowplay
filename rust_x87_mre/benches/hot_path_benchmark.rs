//! Criterion benchmark for the x87 hot-path MRE.

use rust_x87_mre::{hot_path_config, run_benchmark};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn bench_hot_path(c: &mut Criterion) {
    let _config = hot_path_config();
    let iterations = 100_000;

    let mut group = c.benchmark_group("hot_path_dispatch");
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(5));

    group.bench_with_input(BenchmarkId::from_parameter(iterations), &iterations, |b, _| {
        b.iter(|| run_benchmark(black_box(iterations)));
    });

    group.finish();
    let ips = run_benchmark(iterations);
    println!("Throughput: {:.0} ips", ips);
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = bench_hot_path
}
criterion_main!(benches);
