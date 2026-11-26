use std::collections::BTreeSet;
use std::pin::Pin;
use std::sync::OnceLock;
use std::task::Context;
use std::task::Poll;

use tokio::io::AsyncRead;
use tokio::io::ReadBuf;

#[path = "../../src/elf_local_symbol_core.rs"]
pub mod elf_local_symbol_core;
#[path = "../../src/stagex_archive_core.rs"]
pub mod stagex_archive_core;

pub const MANTLE_FUZZING_ENABLED: bool = matches!(
    option_env!("MANTLE_FUZZING"),
    Some(value) if !value.is_empty() && !matches!(value.as_bytes(), b"0")
);

pub mod archive_oracle;
pub mod elf_oracle;

static STORE_RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

fn runtime() -> &'static tokio::runtime::Runtime {
    STORE_RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("create fuzz tokio runtime")
    })
}

pub struct ChunkedSliceReader<'a> {
    bytes: &'a [u8],
    pos: usize,
    chunk_bytes: usize,
}

impl<'a> ChunkedSliceReader<'a> {
    pub fn new(bytes: &'a [u8], chunk_bytes: usize) -> Self {
        assert!(chunk_bytes > 0);
        Self {
            bytes,
            pos: 0,
            chunk_bytes,
        }
    }
}

impl AsyncRead for ChunkedSliceReader<'_> {
    fn poll_read(mut self: Pin<&mut Self>, _cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<std::io::Result<()>> {
        if self.pos >= self.bytes.len() {
            return Poll::Ready(Ok(()));
        }
        let remaining = self.bytes.len() - self.pos;
        let writable = buf.remaining().min(self.chunk_bytes).min(remaining);
        let start = self.pos;
        let end = start + writable;
        buf.put_slice(&self.bytes[start..end]);
        self.pos = end;
        Poll::Ready(Ok(()))
    }
}

pub fn check_store_archive_list_invariants(data: &[u8]) -> Result<crunch_store::ArchiveListReport, String> {
    let chunk_size = 1 + (data.first().copied().unwrap_or(0) as usize % 64);
    runtime().block_on(async move {
        let direct = crunch_store::list_store_archive(&mut std::io::Cursor::new(data)).await;
        let chunked = crunch_store::list_store_archive(&mut ChunkedSliceReader::new(data, chunk_size)).await;
        assert_eq!(direct.is_ok(), chunked.is_ok(), "chunk-size invariance violated");
        if let (Ok(r1), Ok(r2)) = (&direct, &chunked) {
            assert_eq!(r1, r2, "chunked report differed from cursor report");
            assert_eq!(r1.record_count as usize, r1.paths.len());
            assert!(r1.store_prefix.starts_with('/'));
            let mut uniq = BTreeSet::new();
            let mut sum_nar = 0u64;
            for p in &r1.paths {
                assert!(uniq.insert(&p.store_path), "duplicate store_path in report");
                assert_eq!(p.payload_blake3.len(), 64);
                sum_nar = sum_nar.saturating_add(p.nar_size);
            }
            assert_eq!(sum_nar, r1.total_payload_bytes);
        }
        direct.map_err(|err| err.to_string())
    })
}

/// # Safety
///
/// `data` must be valid for `size` readable bytes whenever `size > 0`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fuzz_elf_rust(data: *const u8, size: usize) {
    let slice = if size == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(data, size) }
    };
    let _ = elf_oracle::check_elf_rust_invariants(slice);
}

/// # Safety
///
/// `data` must be valid for `size` readable bytes whenever `size > 0`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fuzz_archive_rust(data: *const u8, size: usize) {
    let slice = if size == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(data, size) }
    };
    let _ = archive_oracle::check_archive_invariants(slice);
}

/// # Safety
///
/// `data` must be valid for `size` readable bytes whenever `size > 0`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fuzz_elf_diff(data: *const u8, size: usize) {
    let slice = if size == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(data, size) }
    };
    elf_oracle::check_elf_differential(slice);
}

/// # Safety
///
/// `data` must be valid for `size` readable bytes whenever `size > 0`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fuzz_store_archive_list(data: *const u8, size: usize) {
    let slice = if size == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(data, size) }
    };
    let _ = check_store_archive_list_invariants(slice);
}
