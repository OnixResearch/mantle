#[cfg(not(feature = "experimental-vectorcdc"))]
compile_error!("run with --features experimental-vectorcdc to enable the gated candidate");

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::time::Instant;

use serde::Deserialize;
use serde_json::json;
use snix_castore::blobservice::chunker::ChunkProfile;
use snix_castore::blobservice::chunker::Chunker;
use snix_castore::blobservice::chunker::ExperimentalVectorCdcChunker;
use snix_castore::blobservice::chunker::FastCdcChunker;

const DEFAULT_CORPUS_MANIFEST: &str = "target/vectorcdc-corpus/manifest.json";
const DEFAULT_OUTPUT_PATH: &str = "target/vectorcdc-candidate-comparison.json";
const MIN_CHUNK_BYTES: u32 = 131_072;
const AVG_CHUNK_BYTES: u32 = 262_144;
const MAX_CHUNK_BYTES: u32 = 524_288;
const ZSTD_LEVEL: i32 = 3;
const BLOB_METADATA_OBJECTS_PER_ENTRY: u64 = 1;
const PROMOTION_SPEEDUP_PPM: u64 = 1_200_000;
const PROMOTION_REUSE_FLOOR_PPM: u64 = 900_000;

#[derive(Debug, Deserialize)]
struct CorpusManifest {
    schema: String,
    entries: Vec<CorpusEntry>,
}

#[derive(Debug, Deserialize)]
struct CorpusEntry {
    id: String,
    relative_path: String,
    bytes: usize,
    blake3: String,
}

#[derive(Debug)]
struct CorpusBytes {
    id: String,
    bytes: Vec<u8>,
    blake3: String,
}

#[derive(Debug)]
struct AlgorithmMetrics {
    algorithm: &'static str,
    entries: usize,
    input_bytes: usize,
    chunk_count: usize,
    unique_chunk_count: usize,
    reused_chunk_count: usize,
    estimated_object_count: u64,
    compressed_chunk_bytes_zstd: usize,
    all_raw_digests_match_manifest: bool,
    chunk_sizes: Vec<usize>,
    timing_nanos: TimingNanos,
}

#[derive(Debug, Default)]
struct TimingNanos {
    blake3_raw: u128,
    cdc: u128,
    zstd_chunk_compress: u128,
    total_ingest_wall: u128,
}

fn main() -> io::Result<()> {
    let manifest_path = env::args().nth(1).unwrap_or_else(|| DEFAULT_CORPUS_MANIFEST.to_string());
    let output_path = env::args().nth(2).unwrap_or_else(|| DEFAULT_OUTPUT_PATH.to_string());

    let manifest_path = repo_root().join(manifest_path);
    let output_path = repo_root().join(output_path);
    let manifest_dir = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let manifest: CorpusManifest = serde_json::from_slice(&fs::read(&manifest_path)?)?;
    if manifest.schema != "crunch.vectorcdc-corpus.v1" {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "unexpected corpus manifest schema"));
    }

    let mut corpus = Vec::with_capacity(manifest.entries.len());
    for entry in &manifest.entries {
        let input_path = manifest_dir.join(&entry.relative_path);
        let bytes = fs::read(&input_path)?;
        if bytes.len() != entry.bytes {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("{} byte count mismatch", entry.id)));
        }
        corpus.push(CorpusBytes {
            id: entry.id.clone(),
            bytes,
            blake3: entry.blake3.clone(),
        });
    }

    let profile = ChunkProfile::new(MIN_CHUNK_BYTES, AVG_CHUNK_BYTES, MAX_CHUNK_BYTES);
    let fastcdc = collect_metrics("FastCDC", FastCdcChunker, profile, &corpus)?;
    let candidate = collect_metrics("ExperimentalVectorCDCScalar", ExperimentalVectorCdcChunker, profile, &corpus)?;
    let decision = decision_json(&fastcdc, &candidate);

    let report = json!({
        "schema": "crunch.vectorcdc-candidate-comparison.v1",
        "corpus_manifest": manifest_path.display().to_string(),
        "identity_invariant": "Blob identity remains BLAKE3(raw bytes); candidate chunking is physical-storage evidence only.",
        "profile": {
            "min_chunk_bytes": MIN_CHUNK_BYTES,
            "avg_chunk_bytes": AVG_CHUNK_BYTES,
            "max_chunk_bytes": MAX_CHUNK_BYTES,
            "zstd_level": ZSTD_LEVEL,
        },
        "algorithms": [metrics_json(&fastcdc), metrics_json(&candidate)],
        "relative_to_fastcdc": relative_json(&fastcdc, &candidate),
        "adoption_decision": decision,
    });

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&output_path, serde_json::to_string_pretty(&report)? + "\n")?;
    println!("wrote candidate comparison to {}", output_path.display());
    println!(
        "fastcdc_bps={} candidate_bps={} candidate_chunks={} candidate_unique={}",
        bytes_per_second(fastcdc.input_bytes, fastcdc.timing_nanos.cdc),
        bytes_per_second(candidate.input_bytes, candidate.timing_nanos.cdc),
        candidate.chunk_count,
        candidate.unique_chunk_count,
    );
    Ok(())
}

fn collect_metrics<C: Chunker>(
    algorithm: &'static str,
    chunker: C,
    profile: ChunkProfile,
    corpus: &[CorpusBytes],
) -> io::Result<AlgorithmMetrics> {
    let total_started = Instant::now();
    let mut global_unique_chunks = BTreeSet::new();
    let mut chunk_frequency = BTreeMap::<String, u64>::new();
    let mut chunk_sizes = Vec::new();
    let mut timing_nanos = TimingNanos::default();
    let mut chunk_count = 0usize;
    let mut compressed_chunk_bytes_zstd = 0usize;
    let mut all_raw_digests_match_manifest = true;

    for entry in corpus {
        let blake3_started = Instant::now();
        let raw_digest = blake3::hash(&entry.bytes).to_hex().to_string();
        timing_nanos.blake3_raw = timing_nanos.blake3_raw.saturating_add(blake3_started.elapsed().as_nanos());
        all_raw_digests_match_manifest &= raw_digest == entry.blake3;

        let cdc_started = Instant::now();
        let boundaries = chunker
            .chunk_boundaries(&entry.bytes, profile)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, format!("{}: {err}", entry.id)))?;
        timing_nanos.cdc = timing_nanos.cdc.saturating_add(cdc_started.elapsed().as_nanos());
        chunk_count = chunk_count.saturating_add(boundaries.len());

        for boundary in &boundaries {
            let end = boundary
                .end()
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "chunk boundary overflow"))?;
            let chunk = &entry.bytes[boundary.offset..end];
            let chunk_digest = blake3::hash(chunk).to_hex().to_string();
            *chunk_frequency.entry(chunk_digest.clone()).or_insert(0) += 1;
            global_unique_chunks.insert(chunk_digest);
            chunk_sizes.push(boundary.length);

            let zstd_started = Instant::now();
            let compressed = zstd::bulk::compress(chunk, ZSTD_LEVEL)?;
            timing_nanos.zstd_chunk_compress =
                timing_nanos.zstd_chunk_compress.saturating_add(zstd_started.elapsed().as_nanos());
            compressed_chunk_bytes_zstd = compressed_chunk_bytes_zstd.saturating_add(compressed.len());
        }
    }

    timing_nanos.total_ingest_wall = total_started.elapsed().as_nanos();
    let input_bytes = corpus.iter().map(|entry| entry.bytes.len()).sum::<usize>();
    let unique_chunk_count = global_unique_chunks.len();
    let reused_chunk_count = chunk_count.saturating_sub(unique_chunk_count);
    let estimated_object_count = unique_chunk_count as u64 + corpus.len() as u64 * BLOB_METADATA_OBJECTS_PER_ENTRY;
    chunk_sizes.sort_unstable();

    Ok(AlgorithmMetrics {
        algorithm,
        entries: corpus.len(),
        input_bytes,
        chunk_count,
        unique_chunk_count,
        reused_chunk_count,
        estimated_object_count,
        compressed_chunk_bytes_zstd,
        all_raw_digests_match_manifest,
        chunk_sizes,
        timing_nanos,
    })
}

fn metrics_json(metrics: &AlgorithmMetrics) -> serde_json::Value {
    json!({
        "algorithm": metrics.algorithm,
        "totals": {
            "entries": metrics.entries,
            "input_bytes": metrics.input_bytes,
            "chunk_count": metrics.chunk_count,
            "unique_chunk_count": metrics.unique_chunk_count,
            "reused_chunk_count": metrics.reused_chunk_count,
            "dedup_reuse_ratio_ppm": ratio_ppm(metrics.reused_chunk_count, metrics.chunk_count),
            "estimated_object_count": metrics.estimated_object_count,
            "compressed_chunk_bytes_zstd": metrics.compressed_chunk_bytes_zstd,
            "compression_ratio_ppm": ratio_ppm(metrics.compressed_chunk_bytes_zstd, metrics.input_bytes),
            "all_raw_digests_match_manifest": metrics.all_raw_digests_match_manifest,
        },
        "chunk_size_distribution": {
            "min": metrics.chunk_sizes.first().copied().unwrap_or(0),
            "avg": average_usize(&metrics.chunk_sizes),
            "p50": percentile(&metrics.chunk_sizes, 50),
            "p95": percentile(&metrics.chunk_sizes, 95),
            "max": metrics.chunk_sizes.last().copied().unwrap_or(0),
        },
        "timing_nanos": {
            "total_ingest_wall": metrics.timing_nanos.total_ingest_wall,
            "blake3_raw": metrics.timing_nanos.blake3_raw,
            "cdc": metrics.timing_nanos.cdc,
            "zstd_chunk_compress": metrics.timing_nanos.zstd_chunk_compress,
        },
        "throughput_bytes_per_second": {
            "cdc": bytes_per_second(metrics.input_bytes, metrics.timing_nanos.cdc),
            "blake3_raw": bytes_per_second(metrics.input_bytes, metrics.timing_nanos.blake3_raw),
            "zstd_chunk_compress": bytes_per_second(metrics.input_bytes, metrics.timing_nanos.zstd_chunk_compress),
            "total_ingest_wall": bytes_per_second(metrics.input_bytes, metrics.timing_nanos.total_ingest_wall),
        },
    })
}

fn relative_json(fastcdc: &AlgorithmMetrics, candidate: &AlgorithmMetrics) -> serde_json::Value {
    let fastcdc_cdc_bps = bytes_per_second(fastcdc.input_bytes, fastcdc.timing_nanos.cdc);
    let candidate_cdc_bps = bytes_per_second(candidate.input_bytes, candidate.timing_nanos.cdc);
    json!({
        "candidate_cdc_speed_ratio_ppm": ratio_ppm(candidate_cdc_bps as usize, fastcdc_cdc_bps as usize),
        "candidate_total_ingest_ratio_ppm": ratio_ppm(
            bytes_per_second(candidate.input_bytes, candidate.timing_nanos.total_ingest_wall) as usize,
            bytes_per_second(fastcdc.input_bytes, fastcdc.timing_nanos.total_ingest_wall) as usize,
        ),
        "candidate_chunk_count_ratio_ppm": ratio_ppm(candidate.chunk_count, fastcdc.chunk_count),
        "candidate_unique_chunk_ratio_ppm": ratio_ppm(candidate.unique_chunk_count, fastcdc.unique_chunk_count),
        "candidate_object_count_ratio_ppm": ratio_ppm(candidate.estimated_object_count as usize, fastcdc.estimated_object_count as usize),
        "candidate_reuse_ratio_vs_fastcdc_ppm": ratio_ppm(
            ratio_ppm(candidate.reused_chunk_count, candidate.chunk_count) as usize,
            ratio_ppm(fastcdc.reused_chunk_count, fastcdc.chunk_count) as usize,
        ),
        "candidate_zstd_bytes_ratio_ppm": ratio_ppm(candidate.compressed_chunk_bytes_zstd, fastcdc.compressed_chunk_bytes_zstd),
    })
}

fn decision_json(fastcdc: &AlgorithmMetrics, candidate: &AlgorithmMetrics) -> serde_json::Value {
    let fastcdc_cdc_bps = bytes_per_second(fastcdc.input_bytes, fastcdc.timing_nanos.cdc);
    let candidate_cdc_bps = bytes_per_second(candidate.input_bytes, candidate.timing_nanos.cdc);
    let speed_ratio = ratio_ppm(candidate_cdc_bps as usize, fastcdc_cdc_bps as usize);
    let reuse_ratio = ratio_ppm(
        ratio_ppm(candidate.reused_chunk_count, candidate.chunk_count) as usize,
        ratio_ppm(fastcdc.reused_chunk_count, fastcdc.chunk_count) as usize,
    );
    let object_ratio = ratio_ppm(candidate.estimated_object_count as usize, fastcdc.estimated_object_count as usize);
    let decision = if speed_ratio >= PROMOTION_SPEEDUP_PPM
        && reuse_ratio >= PROMOTION_REUSE_FLOOR_PPM
        && object_ratio <= 1_100_000
    {
        "open-promotion-openspec"
    } else {
        "continue-research"
    };

    json!({
        "decision": decision,
        "rationale": "The scalar VectorCDC-style prototype is useful as an opt-in research seam, but this one-run corpus comparison is not enough to promote a new default or protocol profile.",
        "thresholds": {
            "promotion_speedup_ppm": PROMOTION_SPEEDUP_PPM,
            "promotion_reuse_floor_ppm": PROMOTION_REUSE_FLOOR_PPM,
            "promotion_object_count_ceiling_ppm": 1_100_000,
        },
        "observed": {
            "candidate_cdc_speed_ratio_ppm": speed_ratio,
            "candidate_reuse_ratio_vs_fastcdc_ppm": reuse_ratio,
            "candidate_object_count_ratio_ppm": object_ratio,
        },
        "next": "Keep FastCDC as default/protocol-v1 profile; continue research/tuning behind explicit feature gates before considering promotion.",
    })
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn average_usize(values: &[usize]) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let sum = values.iter().copied().map(|value| value as u128).sum::<u128>();
    (sum / values.len() as u128).min(u128::from(u64::MAX)) as u64
}

fn percentile(sorted_values: &[usize], percentile: usize) -> usize {
    if sorted_values.is_empty() {
        return 0;
    }
    let capped = percentile.min(100);
    let index = ((sorted_values.len() - 1) * capped) / 100;
    sorted_values[index]
}

fn ratio_ppm(numerator: usize, denominator: usize) -> u64 {
    if denominator == 0 {
        return 0;
    }
    ((numerator as u128) * 1_000_000u128 / denominator as u128).min(u128::from(u64::MAX)) as u64
}

fn bytes_per_second(bytes: usize, nanos: u128) -> u64 {
    if nanos == 0 {
        return 0;
    }
    ((bytes as u128) * 1_000_000_000u128 / nanos).min(u128::from(u64::MAX)) as u64
}
