use criterion::{black_box, criterion_group, criterion_main, Criterion};
use monitor_rs::collectors::{CpuCollector, MemoryCollector, MetricCollector};

fn benchmark_cpu_collection(c: &mut Criterion) {
    let mut collector = CpuCollector::new();

    c.bench_function("cpu_collect", |b| {
        b.iter(|| black_box(collector.collect().unwrap()))
    });
}

fn benchmark_memory_collection(c: &mut Criterion) {
    let mut collector = MemoryCollector::new();

    c.bench_function("memory_collect", |b| {
        b.iter(|| black_box(collector.collect().unwrap()))
    });
}

criterion_group!(benches, benchmark_cpu_collection, benchmark_memory_collection);
criterion_main!(benches);
