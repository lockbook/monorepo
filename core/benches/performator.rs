use cpuprofiler::PROFILER;
use criterion::profiler::Profiler;
use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use lockbook_core::repo::file_metadata_repo::FileMetadataRepo;
use lockbook_core::service::account_service::AccountService;
use lockbook_core::service::file_service::FileService;
use lockbook_core::service::sync_service::SyncService;
use lockbook_core::{
    connect_to_db, DefaultAccountService, DefaultFileMetadataRepo, DefaultFileService,
    DefaultSyncService,
};
use lockbook_models::crypto::DecryptedValue;
use lockbook_models::file_metadata::FileType::Document;
use lockbook_models::state::Config;
use rand::distributions::Alphanumeric;
use rand::{self, Rng};
use std::env;
use std::path::Path;
use uuid::Uuid;

struct CpuProfiler;
impl Profiler for CpuProfiler {
    fn start_profiling(&mut self, benchmark_id: &str, _benchmark_dir: &Path) {
        let profile_name = format!("./{}.profile", benchmark_id.to_string().replace("/", "-"));

        PROFILER.lock().unwrap().start(profile_name).unwrap();
    }

    fn stop_profiling(&mut self, _benchmark_id: &str, _benchmark_dir: &Path) {
        PROFILER.lock().unwrap().stop().unwrap();
    }
}

pub fn bench_performator(c: &mut Criterion) {
    let config = &Config {
        writeable_path: format!("/tmp/perf-{}", Uuid::new_v4().to_string()),
    };

    let db = &connect_to_db(config).unwrap();
    let _ = DefaultAccountService::create_account(
        db,
        "performator",
        env::var("API_URL").unwrap().as_str(),
    )
    .unwrap();
    let _ = DefaultSyncService::sync(db).unwrap();
    let root = DefaultFileMetadataRepo::get_root(db).unwrap().unwrap();

    let mut group = c.benchmark_group("simple");

    let bytes = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(1000)
        .collect::<String>();

    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_function("create_write_read", |b| {
        b.iter(|| {
            let file =
                DefaultFileService::create(db, &Uuid::new_v4().to_string(), root.id, Document)
                    .unwrap();

            let _ = DefaultFileService::write_document(
                db,
                file.id,
                &DecryptedValue::from(bytes.clone()),
            )
            .unwrap();

            let _ = DefaultFileService::read_document(db, file.id).unwrap();
        });
    });

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(CpuProfiler);
    targets = bench_performator
}
criterion_main!(benches);
