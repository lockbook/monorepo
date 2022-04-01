use criterion::{black_box, criterion_group, BenchmarkId, Criterion, Throughput};
use lockbook_core::service::test_utils::{
    create_account, test_config, GEN_FILES_BENCH_SIZE_1, GEN_FILES_BENCH_SIZE_2,
    GEN_FILES_BENCH_SIZE_3, GEN_FILES_BENCH_SIZE_4, GEN_FILES_BENCH_SIZE_5, GEN_FILES_BENCH_SIZE_6,
};
use lockbook_models::file_metadata::FileType;
use uuid::Uuid;

fn create_file_benchmark(c: &mut Criterion) {
    let mut create_file_group = c.benchmark_group("create_file");
    for size in [
        GEN_FILES_BENCH_SIZE_1,
        GEN_FILES_BENCH_SIZE_2,
        GEN_FILES_BENCH_SIZE_3,
        GEN_FILES_BENCH_SIZE_4,
        GEN_FILES_BENCH_SIZE_5,
        GEN_FILES_BENCH_SIZE_6,
    ]
    .iter()
    {
        let db = test_config();
        let (_, root) = create_account(&db);

        create_file_group.throughput(Throughput::Elements(*size));
        create_file_group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                for _ in 0..size {
                    lockbook_core::create_file(
                        black_box(&db),
                        black_box(&Uuid::new_v4().to_string()),
                        black_box(root.id),
                        black_box(FileType::Document),
                    )
                    .unwrap();
                }
            });
        });
    }
    create_file_group.finish();
}

criterion_group!(benches, create_file_benchmark);
