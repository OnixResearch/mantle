//! Read-only, bounded contextual inspection using the same Snix reference scanner
//! that runs during output ingestion. No bytes are rewritten or admitted here.
use snix_castore::Node;
use snix_castore::refscan::ReferencePattern;
use snix_castore::refscan::ReferenceScanner;
use tokio::io::AsyncReadExt;

use crate::Error;
use crate::StoreHandle;

const MAX_SCAN_NODES: usize = 200_000;
const MAX_SCAN_DEPTH: u32 = 128;
const MAX_SCAN_HITS: usize = 128;
const MAX_SCAN_BYTES: u64 = 17_179_869_184; // 16 GiB.
const MAX_EXCERPT_BYTES: usize = 160;
const MAX_CANDIDATE_BYTES: usize = 4096;
const READ_BYTES: usize = 8192;

struct FileToScan<'a> {
    store: &'a StoreHandle,
    digest: &'a snix_castore::B3Digest,
    path: &'a str,
}

struct ScanPatterns<'a> {
    patterns: &'a [ReferencePattern<Vec<u8>>],
    candidates: &'a [String],
}

struct HitIdentity<'a> {
    path: &'a str,
    candidate: &'a str,
}
#[derive(Debug, Clone)]
pub struct ContextualReferenceHit {
    pub file: String,
    pub reference: String,
    pub excerpt: String,
}

pub async fn scan_output_references(
    store: &StoreHandle,
    output: &Node,
    candidates: &[String],
) -> Result<Vec<ContextualReferenceHit>, Error> {
    if candidates.len() > 129
        || candidates.iter().any(|candidate| candidate.len() < 2 || candidate.len() > MAX_CANDIDATE_BYTES)
    {
        return Err(Error::Store("reference finish gate candidates exceed bounds".into()));
    }
    if candidates.is_empty() {
        return Ok(Vec::new());
    }
    let patterns = candidates
        .iter()
        .map(|candidate| ReferencePattern::new(vec![candidate.as_bytes().to_vec()]))
        .collect::<Vec<_>>();
    let mut stack = vec![(String::new(), output.clone(), 0u32)];
    let scan_patterns = ScanPatterns {
        patterns: &patterns,
        candidates,
    };
    let mut hits = Vec::new();
    let mut total_bytes = 0u64;
    let mut visited = 0usize;
    while let Some((path, node, depth)) = stack.pop() {
        visited = visited.saturating_add(1);
        if visited > MAX_SCAN_NODES || depth > MAX_SCAN_DEPTH {
            return Err(Error::Store("reference finish gate tree exceeds node or depth bound".into()));
        }
        match node {
            Node::Directory { digest, .. } => {
                let directory = store
                    .directory_service()
                    .get(&digest)
                    .await
                    .map_err(|error| Error::DirectoryService(format!("reading finish-gate directory: {error}")))?
                    .ok_or_else(|| Error::DirectoryService("missing finish-gate output directory".into()))?;
                for (name, child) in directory.nodes() {
                    let name = std::str::from_utf8(name.as_ref())
                        .map_err(|_| Error::Store("non-UTF8 output filename in reference gate".into()))?;
                    stack.push((format!("{path}/{name}"), child.clone(), depth.saturating_add(1)));
                }
            }
            Node::File { digest, .. } => {
                scan_file(
                    FileToScan {
                        store,
                        digest: &digest,
                        path: &path,
                    },
                    &scan_patterns,
                    &mut hits,
                    &mut total_bytes,
                )
                .await?;
            }
            Node::Symlink { target } => {
                scan_bytes(target.as_ref(), &path, &scan_patterns, &mut hits)?;
            }
        }
    }
    assert!(hits.len() <= MAX_SCAN_HITS, "reference scan hit bound");
    assert!(total_bytes <= MAX_SCAN_BYTES, "reference scan byte bound");
    Ok(hits)
}

async fn scan_file(
    file: FileToScan<'_>,
    scan_patterns: &ScanPatterns<'_>,
    hits: &mut Vec<ContextualReferenceHit>,
    total_bytes: &mut u64,
) -> Result<(), Error> {
    let FileToScan { store, digest, path } = file;
    let mut reader = store
        .blob_service()
        .open_read(digest)
        .await
        .map_err(|error| Error::BlobService(format!("opening finish-gate blob: {error}")))?
        .ok_or_else(|| Error::BlobService("missing finish-gate blob".into()))?;
    let mut buffer = [0u8; READ_BYTES];
    let mut window =
        Vec::with_capacity(READ_BYTES.saturating_add(MAX_CANDIDATE_BYTES).saturating_add(MAX_EXCERPT_BYTES));
    let overlap = scan_patterns
        .candidates
        .iter()
        .map(String::len)
        .max()
        .unwrap_or(0)
        .saturating_add(MAX_EXCERPT_BYTES);
    assert_eq!(scan_patterns.patterns.len(), scan_patterns.candidates.len(), "reference scan pattern count");
    assert!(*total_bytes <= MAX_SCAN_BYTES, "reference scan starting byte bound");
    let scanners = scan_patterns.patterns.iter().cloned().map(ReferenceScanner::new).collect::<Vec<_>>();
    let mut recorded = vec![false; scan_patterns.candidates.len()];
    let reads_max = MAX_SCAN_BYTES.saturating_sub(*total_bytes).saturating_add(1);
    for _ in 0..reads_max {
        let count = reader
            .read(&mut buffer)
            .await
            .map_err(|error| Error::BlobService(format!("reading finish-gate blob: {error}")))?;
        if count == 0 {
            return Ok(());
        }
        let count_bytes =
            u64::try_from(count).map_err(|_| Error::Store("reference finish gate read count exceeds u64".into()))?;
        *total_bytes = total_bytes.saturating_add(count_bytes);
        if *total_bytes > MAX_SCAN_BYTES {
            return Err(Error::Store("reference finish gate exceeds scanned-byte bound".into()));
        }
        window.extend_from_slice(&buffer[..count]);
        for (index, scanner) in scanners.iter().enumerate() {
            if recorded[index] {
                continue;
            }
            scanner.scan(&window);
            if scanner.candidate_matches().next().is_some() {
                record_hit(
                    &window,
                    HitIdentity {
                        path,
                        candidate: &scan_patterns.candidates[index],
                    },
                    hits,
                )?;
                recorded[index] = true;
            }
        }
        let start = window.len().saturating_sub(overlap);
        let keep = window.len().saturating_sub(start);
        window.copy_within(start.., 0);
        window.truncate(keep);
    }
    Err(Error::Store("reference finish gate exceeds scanned-byte bound".into()))
}

fn scan_bytes(
    bytes: &[u8],
    path: &str,
    scan_patterns: &ScanPatterns<'_>,
    hits: &mut Vec<ContextualReferenceHit>,
) -> Result<(), Error> {
    for (pattern, candidate) in scan_patterns.patterns.iter().zip(scan_patterns.candidates) {
        let scanner = ReferenceScanner::new(pattern.clone());
        scanner.scan(bytes);
        if scanner.candidate_matches().next().is_some() {
            record_hit(bytes, HitIdentity { path, candidate }, hits)?;
        }
    }
    Ok(())
}

fn record_hit(bytes: &[u8], identity: HitIdentity<'_>, hits: &mut Vec<ContextualReferenceHit>) -> Result<(), Error> {
    if hits.len() >= MAX_SCAN_HITS {
        return Err(Error::Store("reference finish gate exceeds 128 findings".into()));
    }
    let offset_bytes = bytes
        .windows(identity.candidate.len())
        .position(|window| window == identity.candidate.as_bytes())
        .ok_or_else(|| Error::Store("reference scanner match had no contextual bytes".into()))?;
    let start = offset_bytes.saturating_sub(32);
    let end = bytes.len().min(start.saturating_add(MAX_EXCERPT_BYTES));
    let excerpt = String::from_utf8_lossy(&bytes[start..end]).into_owned();
    hits.push(ContextualReferenceHit {
        file: identity.path.to_string(),
        reference: identity.candidate.to_string(),
        excerpt,
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use snix_castore::SymlinkTarget;
    use tokio::io::AsyncWriteExt;

    use super::*;
    use crate::StoreBackend;
    use crate::StoreConfig;
    use crate::StoreFallbackMode;

    async fn test_store(dir: &std::path::Path) -> StoreHandle {
        StoreHandle::open(StoreConfig {
            source_admission: None,
            backend: StoreBackend::Snix,
            state_dir: dir.join("state"),
            output_dir: dir.join("store"),
            remote_cache_urls: Vec::new(),
            fallback_mode: StoreFallbackMode::Practical,
            store_dir: "/nix/store".to_string(),
            base_state_dirs: Vec::new(),
        })
        .await
        .expect("open test store")
    }

    #[tokio::test]
    async fn finds_split_file_match_and_symlink_target_but_not_absent_reference() {
        let temp = tempfile::tempdir().expect("temporary store directory");
        let store = test_store(temp.path()).await;
        let mut data = vec![b'x'; READ_BYTES.saturating_sub(2)];
        data.extend_from_slice(b"needle");
        let mut writer = store.blob_service().open_write().await;
        writer.write_all(&data).await.expect("write file");
        let digest = writer.close().await.expect("close file");
        let file = Node::File {
            digest,
            size: u64::try_from(data.len()).expect("file size"),
            executable: false,
        };
        let candidates = vec!["needle".to_string(), "absent".to_string()];
        let hits = scan_output_references(&store, &file, &candidates).await.expect("scan file");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].reference, "needle");
        assert!(hits[0].excerpt.contains("needle"));

        let symlink = Node::Symlink {
            target: SymlinkTarget::try_from(b"prefix-needle-suffix".as_slice()).expect("symlink target"),
        };
        let hits = scan_output_references(&store, &symlink, &candidates).await.expect("scan symlink");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].reference, "needle");
    }

    #[tokio::test]
    async fn exact_byte_bound_reads_eof_and_extra_byte_is_rejected() {
        let temp = tempfile::tempdir().expect("temporary store directory");
        let store = test_store(temp.path()).await;
        let mut writer = store.blob_service().open_write().await;
        writer.write_all(b"ab").await.expect("write file");
        let digest = writer.close().await.expect("close file");
        let candidates = vec!["ab".to_string()];
        let patterns = vec![ReferencePattern::new(vec![b"ab".to_vec()])];
        let scan_patterns = ScanPatterns {
            patterns: &patterns,
            candidates: &candidates,
        };
        let mut hits = Vec::new();
        let mut total_bytes = MAX_SCAN_BYTES.saturating_sub(2);
        scan_file(
            FileToScan {
                store: &store,
                digest: &digest,
                path: "/at-limit",
            },
            &scan_patterns,
            &mut hits,
            &mut total_bytes,
        )
        .await
        .expect("exact bound must consume EOF");
        assert_eq!(total_bytes, MAX_SCAN_BYTES);
        assert_eq!(hits[0].reference, "ab");

        let mut hits = Vec::new();
        let mut total_bytes = MAX_SCAN_BYTES.saturating_sub(1);
        let error = scan_file(
            FileToScan {
                store: &store,
                digest: &digest,
                path: "/over-limit",
            },
            &scan_patterns,
            &mut hits,
            &mut total_bytes,
        )
        .await
        .expect_err("byte beyond bound must be rejected");
        assert!(matches!(error, Error::Store(message) if message.contains("scanned-byte bound")));
    }
}
