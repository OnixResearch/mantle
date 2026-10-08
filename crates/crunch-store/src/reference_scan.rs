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
const MAX_SCAN_BYTES: u64 = 16 * 1024 * 1024 * 1024;
const MAX_EXCERPT_BYTES: usize = 160;
const READ_BYTES: usize = 8192;

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
    if candidates.len() > 129 || candidates.iter().any(|candidate| candidate.len() < 2 || candidate.len() > 4096) {
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
                scan_file(store, &digest, &path, &patterns, candidates, &mut hits, &mut total_bytes).await?;
            }
            Node::Symlink { target } => {
                scan_bytes(target.as_ref(), &path, &patterns, candidates, &mut hits)?;
            }
        }
    }
    Ok(hits)
}

async fn scan_file(
    store: &StoreHandle,
    digest: &snix_castore::B3Digest,
    path: &str,
    patterns: &[ReferencePattern<Vec<u8>>],
    candidates: &[String],
    hits: &mut Vec<ContextualReferenceHit>,
    total_bytes: &mut u64,
) -> Result<(), Error> {
    let mut reader = store
        .blob_service()
        .open_read(digest)
        .await
        .map_err(|error| Error::BlobService(format!("opening finish-gate blob: {error}")))?
        .ok_or_else(|| Error::BlobService("missing finish-gate blob".into()))?;
    let mut buffer = [0u8; READ_BYTES];
    let mut window = Vec::with_capacity(READ_BYTES + 4096 + MAX_EXCERPT_BYTES);
    let overlap = candidates.iter().map(String::len).max().unwrap_or(0).saturating_add(MAX_EXCERPT_BYTES);
    let scanners = patterns.iter().cloned().map(ReferenceScanner::new).collect::<Vec<_>>();
    let mut recorded = vec![false; candidates.len()];
    loop {
        let count = reader
            .read(&mut buffer)
            .await
            .map_err(|error| Error::BlobService(format!("reading finish-gate blob: {error}")))?;
        if count == 0 {
            break;
        }
        *total_bytes = total_bytes.saturating_add(u64::try_from(count).unwrap_or(u64::MAX));
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
                record_hit(&window, path, &candidates[index], hits)?;
                recorded[index] = true;
            }
        }
        let start = window.len().saturating_sub(overlap);
        let keep = window.len() - start;
        window.copy_within(start.., 0);
        window.truncate(keep);
    }
    Ok(())
}

fn scan_bytes(
    bytes: &[u8],
    path: &str,
    patterns: &[ReferencePattern<Vec<u8>>],
    candidates: &[String],
    hits: &mut Vec<ContextualReferenceHit>,
) -> Result<(), Error> {
    for (pattern, candidate) in patterns.iter().zip(candidates) {
        let scanner = ReferenceScanner::new(pattern.clone());
        scanner.scan(bytes);
        if scanner.candidate_matches().next().is_some() {
            record_hit(bytes, path, candidate, hits)?;
        }
    }
    Ok(())
}

fn record_hit(bytes: &[u8], path: &str, candidate: &str, hits: &mut Vec<ContextualReferenceHit>) -> Result<(), Error> {
    if hits.len() >= MAX_SCAN_HITS {
        return Err(Error::Store("reference finish gate exceeds 128 findings".into()));
    }
    let offset = bytes
        .windows(candidate.len())
        .position(|window| window == candidate.as_bytes())
        .ok_or_else(|| Error::Store("reference scanner match had no contextual bytes".into()))?;
    let start = offset.saturating_sub(32);
    let end = bytes.len().min(start.saturating_add(MAX_EXCERPT_BYTES));
    let excerpt = String::from_utf8_lossy(&bytes[start..end]).into_owned();
    hits.push(ContextualReferenceHit {
        file: path.to_string(),
        reference: candidate.to_string(),
        excerpt,
    });
    Ok(())
}
