// SPDX-License-Identifier: AGPL-3.0-or-later
// Original Mantle resource benchmark. No nixbench source is copied.
// Fixed-size seeded writes and fanout are reviewed operating patterns only.
use std::fs::{File, create_dir};
use std::io::{BufWriter, Write};
use std::path::Path;

const MAX_BYTES: u64 = 64 * 1024 * 1024;
const MAX_FILES: u32 = 32;
const CHUNK_BYTES: usize = 16 * 1024;

fn integer(value: Option<String>) -> Result<u64, &'static str> {
    value.ok_or("argument-missing")?.parse().map_err(|_| "integer-invalid")
}
fn emit(path: &Path, bytes: u64, seed: u64) -> Result<(), Box<dyn std::error::Error>> {
    if bytes == 0 || bytes > MAX_BYTES { return Err("byte-bound".into()); }
    let mut output = BufWriter::with_capacity(CHUNK_BYTES, File::create_new(path)?);
    let mut state = seed;
    let mut buffer = [0_u8; CHUNK_BYTES];
    let mut remaining = bytes;
    while remaining > 0 {
        let take = remaining.min(CHUNK_BYTES as u64) as usize;
        for (index, byte) in buffer[..take].iter_mut().enumerate() {
            // Half the payload is compressible; fixed-seed bytes are stable
            // across platforms and do not read host entropy or environment.
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            *byte = if index % 2 == 0 { 0 } else { (state >> 56) as u8 };
        }
        output.write_all(&buffer[..take])?;
        remaining -= take as u64;
    }
    output.flush()?;
    output.get_ref().sync_all()?;
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let kind = args.next().ok_or("workload-missing")?;
    let output = args.next().ok_or("output-missing")?;
    let bytes = integer(args.next())?;
    let seed = integer(args.next())?;
    let count = integer(args.next())?;
    if args.next().is_some() || bytes == 0 || bytes > MAX_BYTES || count == 0 || count > u64::from(MAX_FILES) {
        return Err("input-bound".into());
    }
    match kind.as_str() {
        "single" if count == 1 => emit(Path::new(&output), bytes, seed)?,
        "fanout" if bytes.checked_mul(count).is_some_and(|total| total <= MAX_BYTES) => {
            create_dir(&output)?;
            for index in 0..count { emit(&Path::new(&output).join(format!("object-{index:02}")), bytes, seed.wrapping_add(index))?; }
        }
        "memory" if count == 1 => {
            let mut buffer = vec![0_u8; bytes as usize];
            let mut checksum = seed;
            for pass in 0..8_u64 {
                for (index, byte) in buffer.iter_mut().enumerate() {
                    *byte = (index as u64).wrapping_add(seed).wrapping_add(pass) as u8;
                    checksum = checksum.wrapping_add(u64::from(*std::hint::black_box(byte)));
                }
            }
            let mut output = File::create_new(output)?;
            output.write_all(&checksum.to_le_bytes())?;
            output.sync_all()?;
        }
        _ => return Err("workload-invalid".into()),
    }
    // Only declared public observations. Process peak memory, CPU time, NAR
    // bytes and resource units must come from the real authenticated collector.
    println!("{{\"schema\":\"mantle-resource-workload-v1\",\"kind\":\"{kind}\",\"declared_bytes\":{bytes},\"files\":{count},\"seed\":{seed}}}");
    Ok(())
}
