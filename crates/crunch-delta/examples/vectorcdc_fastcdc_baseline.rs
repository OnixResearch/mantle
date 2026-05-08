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
use snix_castore::blobservice::chunker::FastCdcChunker;

const DEFAULT_CORPUS_MANIFEST: &str = "target/vectorcdc-corpus/manifest.json";
const DEFAULT_OUTPUT_PATH: &str = "target/vectorcdc-fastcdc-baseline.json";
const MIN_CHUNK_BYTES: u32 = 131_072;
const AVG_CHUNK_BYTES: u32 = 262_144;
const MAX_CHUNK_BYTES: u32 = 524_288;
const ZSTD_LEVEL: i32 = 3;
const BLOB_METADATA_OBJECTS_PER_ENTRY: u64 = 1;

#[derive(Debug, Deserialize)]
struct CorpusManifest {
    schema: String,
    entries: Vec<CorpusEntry>,
}

#[derive(Debug, Deserialize)]
struct CorpusEntry {
    id: String,
    kind: String,
    relative_path: String,
    mutation: String,
    bytes: usize,
    blake3: String,
}

#[derive(Debug)]
struct EntryMetrics {
    id: String,
    kind: String,
    mutation: String,
    bytes: usize,
    chunks: usize,
    unique_chunks: usize,
    min_chunk_bytes: usize,
    avg_chunk_bytes: u64,
    max_chunk_bytes: usize,
    read_nanos: u128,
    blake3_nanos: u128,
    cdc_nanos: u128,
    zstd_nanos: u128,
    zstd_bytes: usize,
    raw_digest_matches_manifest: bool,
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

    let profile = ChunkProfile::new(MIN_CHUNK_BYTES, AVG_CHUNK_BYTES, MAX_CHUNK_BYTES);
    let chunker = FastCdcChunker;
    let total_started = Instant::now();
    let mut entries = Vec::new();
    let mut global_unique_chunks = BTreeSet::new();
    let mut chunk_sizes = Vec::new();
    let mut chunk_frequency = BTreeMap::<String, u64>::new();

    for corpus_entry in &manifest.entries {
        let input_path = manifest_dir.join(&corpus_entry.relative_path);
        let read_started = Instant::now();
        let bytes = fs::read(&input_path)?;
        let read_nanos = read_started.elapsed().as_nanos();
        if bytes.len() != corpus_entry.bytes {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("{} byte count mismatch", corpus_entry.id)));
        }

        let blake3_started = Instant::now();
        let raw_digest = blake3::hash(&bytes).to_hex().to_string();
        let blake3_nanos = blake3_started.elapsed().as_nanos();

        let cdc_started = Instant::now();
        let boundaries = chunker
            .chunk_boundaries(&bytes, profile)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))?;
        let cdc_nanos = cdc_started.elapsed().as_nanos();

        let mut entry_unique_chunks = BTreeSet::new();
        let mut entry_chunk_sizes = Vec::with_capacity(boundaries.len());
        let mut zstd_nanos = 0u128;
        let mut zstd_bytes = 0usize;
        for boundary in &boundaries {
            let end = boundary
                .end()
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "chunk boundary overflow"))?;
            let chunk = &bytes[boundary.offset..end];
            let chunk_digest = blake3::hash(chunk).to_hex().to_string();
            *chunk_frequency.entry(chunk_digest.clone()).or_insert(0) += 1;
            global_unique_chunks.insert(chunk_digest.clone());
            entry_unique_chunks.insert(chunk_digest);
            entry_chunk_sizes.push(boundary.length);
            chunk_sizes.push(boundary.length);

            let zstd_started = Instant::now();
            let compressed = zstd::bulk::compress(chunk, ZSTD_LEVEL)?;
            zstd_nanos = zstd_nanos.saturating_add(zstd_started.elapsed().as_nanos());
            zstd_bytes = zstd_bytes.saturating_add(compressed.len());
        }

        entries.push(EntryMetrics {
            id: corpus_entry.id.clone(),
            kind: corpus_entry.kind.clone(),
            mutation: corpus_entry.mutation.clone(),
            bytes: bytes.len(),
            chunks: boundaries.len(),
            unique_chunks: entry_unique_chunks.len(),
            min_chunk_bytes: *entry_chunk_sizes.iter().min().unwrap_or(&0),
            avg_chunk_bytes: average_usize(&entry_chunk_sizes),
            max_chunk_bytes: *entry_chunk_sizes.iter().max().unwrap_or(&0),
            read_nanos,
            blake3_nanos,
            cdc_nanos,
            zstd_nanos,
            zstd_bytes,
            raw_digest_matches_manifest: raw_digest == corpus_entry.blake3,
        });
    }

    let total_nanos = total_started.elapsed().as_nanos();
    let total_input_bytes: usize = entries.iter().map(|entry| entry.bytes).sum();
    let total_chunks: usize = entries.iter().map(|entry| entry.chunks).sum();
    let total_zstd_bytes: usize = entries.iter().map(|entry| entry.zstd_bytes).sum();
    let read_nanos: u128 = entries.iter().map(|entry| entry.read_nanos).sum();
    let blake3_nanos: u128 = entries.iter().map(|entry| entry.blake3_nanos).sum();
    let cdc_nanos: u128 = entries.iter().map(|entry| entry.cdc_nanos).sum();
    let zstd_nanos: u128 = entries.iter().map(|entry| entry.zstd_nanos).sum();
    let reused_chunks = total_chunks.saturating_sub(global_unique_chunks.len());
    let object_count = global_unique_chunks.len() as u64 + entries.len() as u64 * BLOB_METADATA_OBJECTS_PER_ENTRY;
    chunk_sizes.sort_unstable();

    let report = json!({
        "schema": "crunch.vectorcdc-fastcdc-baseline.v1",
        "corpus_manifest": manifest_path.display().to_string(),
        "algorithm": "FastCDC",
        "identity_invariant": "Blob identity remains BLAKE3(raw bytes); chunk metrics are physical-storage evidence only.",
        "profile": {
            "min_chunk_bytes": MIN_CHUNK_BYTES,
            "avg_chunk_bytes": AVG_CHUNK_BYTES,
            "max_chunk_bytes": MAX_CHUNK_BYTES,
            "zstd_level": ZSTD_LEVEL,
        },
        "totals": {
            "entries": entries.len(),
            "input_bytes": total_input_bytes,
            "chunk_count": total_chunks,
            "unique_chunk_count": global_unique_chunks.len(),
            "reused_chunk_count": reused_chunks,
            "dedup_reuse_ratio_ppm": ratio_ppm(reused_chunks, total_chunks),
            "estimated_object_count": object_count,
            "compressed_chunk_bytes_zstd": total_zstd_bytes,
            "compression_ratio_ppm": ratio_ppm(total_zstd_bytes, total_input_bytes),
            "all_raw_digests_match_manifest": entries.iter().all(|entry| entry.raw_digest_matches_manifest),
        },
        "chunk_size_distribution": {
            "min": chunk_sizes.first().copied().unwrap_or(0),
            "avg": average_usize(&chunk_sizes),
            "p50": percentile(&chunk_sizes, 50),
            "p95": percentile(&chunk_sizes, 95),
            "max": chunk_sizes.last().copied().unwrap_or(0),
        },
        "timing_nanos": {
            "total_ingest_wall": total_nanos,
            "read": read_nanos,
            "blake3_raw": blake3_nanos,
            "fastcdc": cdc_nanos,
            "zstd_chunk_compress": zstd_nanos,
        },
        "throughput_bytes_per_second": {
            "fastcdc": bytes_per_second(total_input_bytes, cdc_nanos),
            "blake3_raw": bytes_per_second(total_input_bytes, blake3_nanos),
            "zstd_chunk_compress": bytes_per_second(total_input_bytes, zstd_nanos),
            "total_ingest_wall": bytes_per_second(total_input_bytes, total_nanos),
        },
        "entries": entries.iter().map(entry_json).collect::<Vec<_>>(),
        "notes": [
            "Object-store cost is estimated as unique physical chunk objects plus one blob metadata object per corpus entry; no remote object-store latency is included.",
            "Chunk reuse is measured by identical BLAKE3(chunk bytes) across corpus entries under the v1 FastCDC profile."
        ],
    });

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&output_path, serde_json::to_string_pretty(&report)? + "\n")?;
    println!("wrote FastCDC baseline to {}", output_path.display());
    println!(
        "entries={} bytes={} chunks={} unique_chunks={} reused_chunks={}",
        entries.len(),
        total_input_bytes,
        total_chunks,
        global_unique_chunks.len(),
        reused_chunks
    );
    println!(
        "fastcdc_bps={} total_ingest_bps={}",
        bytes_per_second(total_input_bytes, cdc_nanos),
        bytes_per_second(total_input_bytes, total_nanos)
    );
    Ok(())
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn entry_json(entry: &EntryMetrics) -> serde_json::Value {
    json!({
        "id": entry.id,
        "kind": entry.kind,
        "mutation": entry.mutation,
        "bytes": entry.bytes,
        "chunks": entry.chunks,
        "unique_chunks": entry.unique_chunks,
        "min_chunk_bytes": entry.min_chunk_bytes,
        "avg_chunk_bytes": entry.avg_chunk_bytes,
        "max_chunk_bytes": entry.max_chunk_bytes,
        "zstd_bytes": entry.zstd_bytes,
        "raw_digest_matches_manifest": entry.raw_digest_matches_manifest,
        "timing_nanos": {
            "read": entry.read_nanos,
            "blake3_raw": entry.blake3_nanos,
            "fastcdc": entry.cdc_nanos,
            "zstd_chunk_compress": entry.zstd_nanos,
        }
    })
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
