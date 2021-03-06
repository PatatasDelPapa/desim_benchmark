use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use desim_benchmark::simulation;

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("simulation");
    for limit in [10000.0, 20000.0, 30000.0, 40000.0, 50000.0] {
            group.bench_with_input(BenchmarkId::from_parameter(limit), &limit, |b, &limit| {
                    b.iter(|| simulation(black_box(limit)));
                });
        }
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);

