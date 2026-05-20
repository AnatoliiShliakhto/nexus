#![allow(unused_crate_dependencies)]

use criterion::{Criterion, criterion_group, criterion_main};
use nx_logger::{LevelFilter, Logger};
use std::hint::black_box;
use tokio::runtime::Runtime;

fn bench_concurrent_logging(c: &mut Criterion) {
    let _ = Logger::builder()
        .name("bench-service")
        .console(false)
        .json(true)
        .level(LevelFilter::INFO)
        .init();

    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("concurrent_logging");

    for workers in &[1, 4, 8] {
        group.bench_with_input(format!("workers_{workers}"), workers, |b, &w| {
            b.to_async(&rt).iter(|| async move {
                let mut tasks = Vec::new();
                for _ in 0..w {
                    tasks.push(tokio::spawn(async move {
                        for i in 0..10 {
                            tracing::info!(
                                request_id = black_box(i),
                                message = "processing request"
                            );
                        }
                    }));
                }
                for task in tasks {
                    task.await.unwrap();
                }
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_concurrent_logging);
criterion_main!(benches);
