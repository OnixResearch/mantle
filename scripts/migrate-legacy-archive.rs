#!/usr/bin/env -S cargo -Zscript
---
[package]
edition = "2024"
[dependencies]
blake3 = "=1.8.2"
nix-compat = { path = "../vendor/nix-compat", default-features = false, features = ["serde"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sha2 = "0.10"
postcard = { version = "1", features = ["use-std"] }
tempfile = "3"
libc = "0.2"
---
//! Standalone compatibility transport. Does not link or rebuild the Mantle binary.
//! Request identity and public verification keys are explicit operator inputs.

#[path = "legacy_archive/adapter.rs"]
mod adapter;
#[path = "legacy_archive/codec.rs"]
mod codec;
#[path = "../crates/crunch-repair-core/src/legacy_archive.rs"]
mod core;

use codec::{Frame, Header, MAGIC, PathFrame};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const MAX_ARCHIVE_BYTES: u64 = 512 * 1024 * 1024;
const MAX_REQUEST_BYTES: u64 = 64 * 1024;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    schema: String,
    expected_input_blake3: String,
    store_prefix: String,
    roots: Vec<String>,
    trusted_public_keys: Vec<String>,
}

#[derive(Serialize)]
struct Receipt {
    schema: &'static str,
    input_blake3: String,
    output_blake3: String,
    records: Vec<adapter::RecordEvidence>,
    payloads_and_signed_facts_preserved: bool,
    package_realization: bool,
}

fn main() -> std::process::ExitCode {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 3 {
        eprintln!("usage: migrate-legacy-archive.rs REQUEST.json INPUT.archive NEW-OUTPUT-DIRECTORY");
        return std::process::ExitCode::FAILURE;
    }
    match migrate(Path::new(&args[0]), Path::new(&args[1]), Path::new(&args[2])) {
        Ok(receipt) => {
            println!("{}", serde_json::to_string(&receipt).expect("receipt serializes"));
            std::process::ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("archive-compat-rejected: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn bounded_read(path: &Path, limit: u64) -> Result<Vec<u8>> {
    let mut file = File::open(path)?;
    if !file.metadata()?.is_file() {
        return Err("input must be a regular file".into());
    }
    let mut bytes = Vec::new();
    (&mut file).take(limit + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err("input byte bound exceeded".into());
    }
    Ok(bytes)
}

fn migrate(request_path: &Path, input_path: &Path, output: &Path) -> Result<Receipt> {
    let request: Request = serde_json::from_slice(&bounded_read(request_path, MAX_REQUEST_BYTES)?)?;
    adapter::validate_request(&request)?;
    // One immutable in-memory observation prevents validation/copy TOCTOU.
    let bytes = bounded_read(input_path, MAX_ARCHIVE_BYTES)?;
    let input_blake3 = blake3::hash(&bytes).to_hex().to_string();
    if input_blake3 != request.expected_input_blake3 {
        return Err("input archive identity mismatch".into());
    }
    // No output path is created until every record and the whole closure pass.
    let (header, plans, evidence) = inspect_archive(&bytes, &request)?;
    let parent = output.parent().ok_or("output parent missing")?;
    let stage = tempfile::Builder::new().prefix(".archive-compat-").tempdir_in(parent)?;
    let archive = stage.path().join("archive");
    let mut writer = OpenOptions::new().write(true).create_new(true).open(&archive)?;
    writer.write_all(MAGIC)?;
    codec::write_frame(&mut writer, &Frame::Header(header.clone()))?;
    for (frame, range) in &plans {
        codec::write_frame(&mut writer, &Frame::Path(Box::new(frame.clone())))?;
        writer.write_all(&bytes[range.clone()])?;
    }
    let payload_bytes = plans.iter().map(|(p, _)| p.payload_len).sum();
    codec::write_frame(
        &mut writer,
        &Frame::End(codec::End {
            record_count: header.record_count,
            total_payload_bytes: payload_bytes,
            payload_len: 0,
        }),
    )?;
    writer.sync_all()?;
    let mut archive_reader = File::open(&archive)?;
    let mut hash = blake3::Hasher::new();
    std::io::copy(&mut archive_reader, &mut hash)?;
    let receipt = Receipt {
        schema: "mantle-legacy-archive-migration-v1",
        input_blake3,
        output_blake3: hash.finalize().to_hex().to_string(),
        records: evidence,
        payloads_and_signed_facts_preserved: true,
        package_realization: false,
    };
    let mut receipt_file = OpenOptions::new().write(true).create_new(true).open(stage.path().join("receipt.json"))?;
    receipt_file.write_all(&serde_json::to_vec_pretty(&receipt)?)?;
    receipt_file.sync_all()?;
    publish_directory(stage.path(), output)?;
    Ok(receipt)
}

// Linux no-replace commit publishes archive + receipt together. On failure the
// TempDir owns the only cleanup; no existing destination is removed or modified.
#[cfg(target_os = "linux")]
fn publish_directory(stage: &Path, output: &Path) -> Result<()> {
    use std::os::unix::ffi::OsStrExt;
    let source = std::ffi::CString::new(stage.as_os_str().as_bytes())?;
    let destination = std::ffi::CString::new(output.as_os_str().as_bytes())?;
    File::open(stage)?.sync_all()?;
    // SAFETY: NUL-terminated names live through the synchronous syscall.
    let result = unsafe {
        libc::syscall(
            libc::SYS_renameat2,
            libc::AT_FDCWD,
            source.as_ptr(),
            libc::AT_FDCWD,
            destination.as_ptr(),
            libc::RENAME_NOREPLACE,
        )
    };
    if result != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn publish_directory(_stage: &Path, _output: &Path) -> Result<()> {
    Err("atomic archive publication requires Linux renameat2".into())
}

type Plan = (PathFrame, std::ops::Range<usize>);
fn inspect_archive(bytes: &[u8], request: &Request) -> Result<(Header, Vec<Plan>, Vec<adapter::RecordEvidence>)> {
    let mut reader = std::io::Cursor::new(bytes);
    let mut magic = vec![0; MAGIC.len()];
    reader.read_exact(&mut magic)?;
    if magic != MAGIC {
        return Err("archive magic mismatch".into());
    }
    let Frame::Header(header) = codec::read_frame(&mut reader)? else {
        return Err("header required".into());
    };
    adapter::validate_header(&header, request)?;
    let mut plans = Vec::new();
    let mut evidence = Vec::new();
    for _ in 0..header.record_count {
        let Frame::Path(mut path) = codec::read_frame(&mut reader)? else {
            return Err("path frame required".into());
        };
        if path.payload_len > 256 * 1024 * 1024 {
            return Err("NAR byte bound exceeded".into());
        }
        let start = usize::try_from(reader.position())?;
        let end = start.checked_add(usize::try_from(path.payload_len)?).ok_or("payload overflow")?;
        let payload = bytes.get(start..end).ok_or("truncated payload")?;
        evidence.push(adapter::admit_record(&mut path, payload, request)?);
        plans.push((*path, start..end));
        reader.seek(SeekFrom::Start(end as u64))?;
    }
    let Frame::End(end) = codec::read_frame(&mut reader)? else {
        return Err("end frame required".into());
    };
    if end.record_count != header.record_count
        || end.payload_len != 0
        || end.total_payload_bytes != plans.iter().map(|(p, _)| p.payload_len).sum::<u64>()
        || reader.position() != bytes.len() as u64
    {
        return Err("end counts or trailing bytes mismatch".into());
    }
    adapter::validate_closure(&header, &plans)?;
    Ok((header, plans, evidence))
}

#[path = "legacy_archive/tests.rs"]
#[cfg(test)]
mod tests;
