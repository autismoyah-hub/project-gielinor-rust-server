use gielinor_server::js5::{Provider, Worker};
use std::{
    fs,
    path::PathBuf,
    sync::{Arc, atomic::{AtomicU64, Ordering}},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        let parent = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../work/tmp")
            .canonicalize()
            .unwrap();
        let path = self.0.canonicalize().unwrap();
        assert!(path.starts_with(parent));
        fs::remove_dir_all(path).unwrap();
    }
}
fn fixture() -> Scratch {
    // Windows clock resolution can give concurrent tests the same timestamp.
    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../work/tmp")
        .join(format!(
            "js5-worker-{}-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
    fs::create_dir_all(&path).unwrap();
    // An empty index 255 produces a valid empty master index. Archive 0 has
    // sixty-four single-sector groups, each with a two-byte version trailer.
    fs::write(path.join("main_file_cache.idx255"), []).unwrap();
    let mut data = vec![0; 520];
    let mut index = Vec::new();
    for group in 0..64u16 {
        index.extend([0, 0, 5, 0, 0, (group + 1) as u8]);
        let mut sector = [0; 520];
        sector[..2].copy_from_slice(&group.to_be_bytes());
        sector[8..11].fill(group as u8);
        sector[11..13].copy_from_slice(&1u16.to_be_bytes());
        data.extend(sector);
    }
    fs::write(path.join("main_file_cache.dat2"), data).unwrap();
    fs::write(path.join("main_file_cache.idx0"), index).unwrap();
    Scratch(path)
}
fn response(worker: &mut Worker, archive: u8, group: u16) -> std::io::Result<Arc<[u8]>> {
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        if let Some(bytes) = worker.response(archive, group)? {
            return Ok(bytes);
        }
        assert!(
            Instant::now() < deadline,
            "worker did not finish {archive}:{group}"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}
#[test]
fn worker_shares_cached_groups_reports_missing_and_evicts_with_live_readers() {
    let scratch = fixture();
    let mut worker = Worker::start(scratch.0.clone(), 5, 9).unwrap();
    assert_eq!(worker.checksums, [0; 23]);
    let first = response(&mut worker, 0, 0).unwrap();
    assert_eq!(&*first, &[0, 0, 0, 0, 0, 0]);
    assert!(Arc::ptr_eq(&first, &response(&mut worker, 0, 0).unwrap()));
    assert_eq!(&*response(&mut worker, 0, 1).unwrap(), &[0, 0, 1, 1, 1, 1]);
    let reloaded = response(&mut worker, 0, 0).unwrap();
    assert!(!Arc::ptr_eq(&first, &reloaded));
    assert_eq!(first, reloaded); // Eviction must not invalidate a socket's Arc.
    assert!(response(&mut worker, 0, 100).is_err());
    assert!(response(&mut worker, 0, 100).is_err());
}
#[test]
fn dropping_worker_with_queued_requests_does_not_deadlock() {
    let scratch = fixture();
    let mut worker = Worker::start(scratch.0.clone(), 5, 9).unwrap();
    for group in 0..64 {
        let _ = worker.response(0, group).unwrap();
    }
    // Drop disconnects the completion receiver before joining the producer.
    drop(worker);
}
#[test]
fn worker_rejects_invalid_limits_and_missing_store() {
    let scratch = fixture();
    assert!(Worker::start(scratch.0.clone(), 0, 10).is_err());
    assert!(Worker::start(scratch.0.clone(), 3, 7).is_err()); // Master container is five bytes.
    assert!(Worker::start(scratch.0.clone(), 100, 100).is_err());
    assert!(Worker::start(scratch.0.join("missing"), 3, 7).is_err());
}
