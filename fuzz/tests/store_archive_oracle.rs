use std::path::Path;

use crunch_store::list_store_archive;
use mantle_fuzz::ChunkedSliceReader;
use mantle_fuzz::MANTLE_FUZZING_ENABLED;
use mantle_fuzz::check_store_archive_list_invariants;
use proptest::prelude::*;

fn replay_dir(dir: &Path, expect_ok: bool) {
    assert!(dir.exists(), "expected seed directory {}", dir.display());
    let mut entries: Vec<_> =
        std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()).filter(|p| p.is_file()).collect();
    entries.sort();
    assert!(!entries.is_empty(), "expected seed files in {}", dir.display());
    for path in entries {
        let bytes = std::fs::read(&path).unwrap();
        let res = check_store_archive_list_invariants(&bytes);
        if expect_ok {
            assert!(res.is_ok(), "expected corpus seed {} to succeed: {:?}", path.display(), res);
        } else {
            assert!(res.is_err(), "expected regression seed {} to fail", path.display());
        }
    }
}

#[test]
fn replays_store_archive_corpus_and_regression_seeds() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    replay_dir(&manifest_dir.join("corpus/store_archive"), true);
    replay_dir(&manifest_dir.join("regressions/store_archive"), false);
}

#[test]
fn store_archive_rejects_one_million_record_oom_seed() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let oom_seed = manifest_dir.join("regressions/store_archive/oom_record_count_header.mstore");
    let bytes = std::fs::read(&oom_seed).unwrap();
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    rt.block_on(async move {
        let err = list_store_archive(&mut std::io::Cursor::new(&bytes)).await.unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("reading archive frame length") || msg.contains("archive record count exceeds"),
            "unexpected error for OOM header seed: {msg}"
        );
    });
}

const FUZZ_MAX_ARCHIVE_RECORDS: usize = 16;
const SEED_MAGIC_LEN: usize = 24;
const FRAME_LEN_BYTES: usize = 4;
const SEED_NAR_PAYLOAD_LEN: usize = 120;

fn push_framed_json(out: &mut Vec<u8>, json: &str) {
    let frame_len = u32::try_from(json.len()).expect("synthetic frame fits u32");
    out.extend_from_slice(&frame_len.to_le_bytes());
    out.extend_from_slice(json.as_bytes());
}

fn read_seed_frame(seed: &[u8], offset: usize) -> (&str, usize) {
    let len_bytes: [u8; FRAME_LEN_BYTES] =
        seed[offset..offset + FRAME_LEN_BYTES].try_into().expect("frame length bytes");
    let frame_len = usize::try_from(u32::from_le_bytes(len_bytes)).expect("frame length fits usize");
    let start = offset + FRAME_LEN_BYTES;
    let end = start + frame_len;
    let json = std::str::from_utf8(&seed[start..end]).expect("valid UTF-8 frame JSON");
    (json, end)
}

fn build_synthetic_mstore(seed: &[u8], header_count: u32, path_count: usize, end_count: u32) -> Vec<u8> {
    assert!(seed.starts_with(b"mantle-store-archive-v1\n"));
    let (header_tpl, path_off) = read_seed_frame(seed, SEED_MAGIC_LEN);
    let (path_tpl, nar_off) = read_seed_frame(seed, path_off);
    let nar_payload = &seed[nar_off..nar_off + SEED_NAR_PAYLOAD_LEN];
    let (end_tpl, total_end) = read_seed_frame(seed, nar_off + SEED_NAR_PAYLOAD_LEN);
    assert_eq!(total_end, seed.len());

    let mut out = Vec::from(&seed[..SEED_MAGIC_LEN]);
    let header_json = header_tpl.replacen("\"record_count\":1", &format!("\"record_count\":{header_count}"), 1);
    assert_ne!(header_json, header_tpl);
    push_framed_json(&mut out, &header_json);

    for idx in 0..path_count {
        let path_json = path_tpl.replacen("-seed\"", &format!("-seed-{idx:02}\""), 1);
        assert_ne!(path_json, path_tpl);
        push_framed_json(&mut out, &path_json);
        out.extend_from_slice(nar_payload);
    }

    let total_payload_bytes =
        u64::try_from(path_count).expect("path count") * u64::try_from(SEED_NAR_PAYLOAD_LEN).expect("nar len");
    let end_json = end_tpl.replacen("\"record_count\":1", &format!("\"record_count\":{end_count}"), 1).replacen(
        "\"total_payload_bytes\":120",
        &format!("\"total_payload_bytes\":{total_payload_bytes}"),
        1,
    );
    assert_ne!(end_json, end_tpl);
    push_framed_json(&mut out, &end_json);
    out
}

#[test]
fn store_archive_chunked_invariance_on_valid_zero_record_and_limits() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let zero_seed = std::fs::read(manifest_dir.join("corpus/store_archive/valid_zero_record.mstore")).unwrap();
    let single_seed = std::fs::read(manifest_dir.join("corpus/store_archive/valid_single_path.mstore")).unwrap();

    let exact_16 = build_synthetic_mstore(&single_seed, 16, FUZZ_MAX_ARCHIVE_RECORDS, 16);
    let smuggled_17 = build_synthetic_mstore(&single_seed, 16, FUZZ_MAX_ARCHIVE_RECORDS + 1, 16);
    let consistent_17 = build_synthetic_mstore(&single_seed, 17, FUZZ_MAX_ARCHIVE_RECORDS + 1, 17);

    let exact_16_report = check_store_archive_list_invariants(&exact_16).unwrap();
    assert_eq!(exact_16_report.record_count, 16);
    assert_eq!(exact_16_report.paths.len(), FUZZ_MAX_ARCHIVE_RECORDS);

    let smuggled_17_err = check_store_archive_list_invariants(&smuggled_17).unwrap_err();
    if MANTLE_FUZZING_ENABLED {
        assert!(
            smuggled_17_err.contains("archive record count exceeded 16"),
            "unexpected smuggled 17th frame error in fuzz mode: {smuggled_17_err}"
        );
    } else {
        assert!(
            smuggled_17_err.contains("archive end record count mismatch"),
            "unexpected smuggled 17th frame error in default mode: {smuggled_17_err}"
        );
    }

    let consistent_17_res = check_store_archive_list_invariants(&consistent_17);
    let consistent_17_report = if MANTLE_FUZZING_ENABLED {
        let err = consistent_17_res.unwrap_err();
        assert!(
            err.contains("archive record count exceeds 16"),
            "unexpected consistent 17-frame error in fuzz mode: {err}"
        );
        None
    } else {
        let report = consistent_17_res.unwrap();
        assert_eq!(report.record_count, 17);
        assert_eq!(report.paths.len(), FUZZ_MAX_ARCHIVE_RECORDS + 1);
        Some(report)
    };

    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    rt.block_on(async move {
        let direct = list_store_archive(&mut std::io::Cursor::new(&zero_seed)).await.unwrap();
        assert_eq!(direct.record_count, 0);
        assert!(direct.paths.is_empty());
        for chunk in [1usize, 3, 7, 64, 4096] {
            let chunked = list_store_archive(&mut ChunkedSliceReader::new(&zero_seed, chunk)).await.unwrap();
            assert_eq!(direct, chunked);

            let chunked_exact_16 = list_store_archive(&mut ChunkedSliceReader::new(&exact_16, chunk)).await.unwrap();
            assert_eq!(exact_16_report, chunked_exact_16);

            let err =
                list_store_archive(&mut ChunkedSliceReader::new(&smuggled_17, chunk)).await.unwrap_err().to_string();
            if MANTLE_FUZZING_ENABLED {
                assert!(
                    err.contains("archive record count exceeded 16"),
                    "unexpected chunk({chunk}) smuggled 17th error in fuzz mode: {err}"
                );
            } else {
                assert!(
                    err.contains("archive end record count mismatch"),
                    "unexpected chunk({chunk}) smuggled 17th error in default mode: {err}"
                );
            }

            let chunked_consistent_17 = list_store_archive(&mut ChunkedSliceReader::new(&consistent_17, chunk)).await;
            if let Some(expected) = &consistent_17_report {
                let report = chunked_consistent_17.unwrap();
                assert_eq!(expected, &report);
            } else {
                let err = chunked_consistent_17.unwrap_err().to_string();
                assert!(
                    err.contains("archive record count exceeds 16"),
                    "unexpected chunk({chunk}) consistent 17-frame error in fuzz mode: {err}"
                );
            }
        }
    });
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    #[test]
    fn prop_store_archive_mutated_seeds_never_panic_or_diverge_on_chunking(
        mutations in prop::collection::vec((0usize..1024, any::<u8>()), 1..6),
    ) {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut seed = std::fs::read(manifest_dir.join("corpus/store_archive/valid_single_path.mstore")).unwrap();
        for (off, val) in mutations {
            let idx = off % seed.len();
            seed[idx] ^= val;
        }
        let _ = check_store_archive_list_invariants(&seed);
    }

    #[test]
    fn prop_store_archive_arbitrary_bytes_safe(bytes in prop::collection::vec(any::<u8>(), 0..512)) {
        let _ = check_store_archive_list_invariants(&bytes);
    }
}
