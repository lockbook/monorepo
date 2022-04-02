use criterion::{black_box, criterion_group, BenchmarkId, Criterion, Throughput};
use lockbook_core::service::test_utils::{create_account, test_config};
use lockbook_models::file_metadata::FileType;
use uuid::Uuid;

const BYTES_LEN_1: u64 = 100;
const BYTES_LEN_2: u64 = BYTES_LEN_1 * 10;
const BYTES_LEN_3: u64 = BYTES_LEN_1 * 20;
const BYTES_LEN_4: u64 = BYTES_LEN_1 * 50;
const BYTES_LEN_5: u64 = BYTES_LEN_1 * 100;
const BYTES_LEN_6: u64 = BYTES_LEN_1 * 1000;

fn write_file_benchmark(c: &mut Criterion) {
    let mut write_file_group = c.benchmark_group("write_file");
    for size in
        [BYTES_LEN_1, BYTES_LEN_2, BYTES_LEN_3, BYTES_LEN_4, BYTES_LEN_5, BYTES_LEN_6].iter()
    {
        let db = test_config();
        let (_, root) = create_account(&db);

        write_file_group.throughput(Throughput::Elements(*size));
        write_file_group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter(|| {
                let id = lockbook_core::create_file(
                    black_box(&db),
                    black_box(&Uuid::new_v4().to_string()),
                    black_box(root.id),
                    black_box(FileType::Document),
                )
                .unwrap()
                .id;

                let random_bytes: Vec<u8> = (0..*size).map(|_| rand::random::<u8>()).collect();

                lockbook_core::write_document(
                    black_box(&db),
                    black_box(id),
                    black_box(random_bytes.as_slice()),
                )
                .unwrap();
            });
        });
    }
    write_file_group.finish();
}

criterion_group!(benches, write_file_benchmark);
