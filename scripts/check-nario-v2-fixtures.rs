use std::env;
use std::fs;
use std::ops::Range;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

const FIXTURE_DIR: &str = "fixtures/nario-v2";
const POSITIVE: &str = "positive-single.nario";
const WIRE_WORD_BYTES: usize = 8;
const MAGIC_AND_MARKER_BYTES: usize = WIRE_WORD_BYTES * 2;
const END_MARKER_BYTES: usize = WIRE_WORD_BYTES;
const TRUNCATION_BYTES: usize = END_MARKER_BYTES + 1;
const OVERSIZED_STRING_BYTES: u64 = 1_048_577;
const WRITE_FLAG: &str = "--write";
const SELF_TEST_FLAG: &str = "--self-test";
const NEGATIVE_FIXTURE_COUNT: usize = 7;

struct MetadataLayout {
    ca: Range<usize>,
    nar_start: usize,
    nar_size: usize,
}

fn main() -> ExitCode {
    match run() {
        Ok(message) => {
            println!("{message}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<String, String> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.as_slice() == [SELF_TEST_FLAG] {
        return self_test();
    }
    if args.len() > 1 || args.first().is_some_and(|arg| arg != WRITE_FLAG) {
        return Err("usage: check-nario-v2-fixtures.rs [--write|--self-test]".to_string());
    }
    let root = PathBuf::from(FIXTURE_DIR);
    let positive = fs::read(root.join(POSITIVE)).map_err(|error| format!("reading positive fixture: {error}"))?;
    let corpus = negative_corpus(&positive)?;
    if args.first().is_some() {
        for (name, bytes) in &corpus {
            fs::write(root.join(name), bytes).map_err(|error| format!("writing {name}: {error}"))?;
        }
    } else {
        for (name, expected) in &corpus {
            let actual = fs::read(root.join(name)).map_err(|error| format!("reading {name}: {error}"))?;
            if &actual != expected {
                return Err(format!("stale Nario negative fixture: {name}"));
            }
        }
    }
    Ok(format!(
        "Nario v2 fixture corpus valid: positive_bytes={} negatives={}",
        positive.len(),
        corpus.len()
    ))
}

fn negative_corpus(positive: &[u8]) -> Result<Vec<(&'static str, Vec<u8>)>, String> {
    if positive.len() <= MAGIC_AND_MARKER_BYTES + TRUNCATION_BYTES {
        return Err("positive fixture is too short".to_string());
    }
    let layout = parse_metadata_layout(positive)?;
    let mut wrong_magic = positive.to_vec();
    wrong_magic[..WIRE_WORD_BYTES].fill(0);
    let truncated = positive[..positive.len() - TRUNCATION_BYTES].to_vec();
    let mut trailing = positive.to_vec();
    trailing.push(0xff);
    let record = &positive[WIRE_WORD_BYTES..positive.len() - END_MARKER_BYTES];
    let mut duplicate = positive[..WIRE_WORD_BYTES].to_vec();
    duplicate.extend_from_slice(record);
    duplicate.extend_from_slice(record);
    duplicate.extend_from_slice(&[0u8; END_MARKER_BYTES]);
    let mut oversized = positive[..MAGIC_AND_MARKER_BYTES].to_vec();
    oversized.extend_from_slice(&OVERSIZED_STRING_BYTES.to_le_bytes());
    let mut unsupported_ca = positive.to_vec();
    if layout.ca.is_empty() {
        return Err("positive fixture must carry CA metadata".to_string());
    }
    unsupported_ca[layout.ca.start] = b'x';
    let mut hash_mismatch = positive.to_vec();
    let nar_end = layout.nar_start.checked_add(layout.nar_size).ok_or("NAR range overflow")?;
    if nar_end > positive.len() - END_MARKER_BYTES {
        return Err("NAR range exceeds positive fixture".to_string());
    }
    hash_mismatch[layout.nar_start] ^= 0x01;
    Ok(vec![
        ("negative-wrong-magic.nario", wrong_magic),
        ("negative-truncated.nario", truncated),
        ("negative-trailing-data.nario", trailing),
        ("negative-duplicate-path.nario", duplicate),
        ("negative-oversized-path.nario", oversized),
        ("negative-unsupported-ca.nario", unsupported_ca),
        ("negative-hash-mismatch.nario", hash_mismatch),
    ])
}

fn parse_metadata_layout(bytes: &[u8]) -> Result<MetadataLayout, String> {
    let mut cursor = MAGIC_AND_MARKER_BYTES;
    read_string(bytes, &mut cursor)?; // path
    read_string(bytes, &mut cursor)?; // deriver
    read_string(bytes, &mut cursor)?; // NAR hash
    let references = read_u64(bytes, &mut cursor)?;
    for _ in 0..references {
        read_string(bytes, &mut cursor)?;
    }
    read_u64(bytes, &mut cursor)?; // registration time
    let nar_size = usize::try_from(read_u64(bytes, &mut cursor)?).map_err(|_| "NAR size does not fit usize")?;
    read_u64(bytes, &mut cursor)?; // ultimate
    let signatures = read_u64(bytes, &mut cursor)?;
    for _ in 0..signatures {
        read_string(bytes, &mut cursor)?;
    }
    let ca = read_string(bytes, &mut cursor)?;
    Ok(MetadataLayout {
        ca,
        nar_start: cursor,
        nar_size,
    })
}

fn read_u64(bytes: &[u8], cursor: &mut usize) -> Result<u64, String> {
    let end = cursor.checked_add(WIRE_WORD_BYTES).ok_or("wire cursor overflow")?;
    let word = bytes.get(*cursor..end).ok_or("truncated wire word")?;
    *cursor = end;
    Ok(u64::from_le_bytes(word.try_into().map_err(|_| "invalid wire word")?))
}

fn read_string(bytes: &[u8], cursor: &mut usize) -> Result<Range<usize>, String> {
    let length = usize::try_from(read_u64(bytes, cursor)?).map_err(|_| "string length does not fit usize")?;
    let start = *cursor;
    let end = start.checked_add(length).ok_or("string range overflow")?;
    bytes.get(start..end).ok_or("truncated wire string")?;
    let padding = (WIRE_WORD_BYTES - (length % WIRE_WORD_BYTES)) % WIRE_WORD_BYTES;
    *cursor = end.checked_add(padding).ok_or("string padding overflow")?;
    bytes.get(end..*cursor).ok_or("truncated wire padding")?;
    Ok(start..end)
}

fn self_test() -> Result<String, String> {
    let positive = fs::read(Path::new(FIXTURE_DIR).join(POSITIVE))
        .map_err(|error| format!("reading positive fixture: {error}"))?;
    let corpus = negative_corpus(&positive)?;
    if corpus.len() != NEGATIVE_FIXTURE_COUNT {
        return Err(format!("expected {NEGATIVE_FIXTURE_COUNT} negative fixtures, got {}", corpus.len()));
    }
    if corpus.iter().any(|(_, bytes)| bytes == &positive) {
        return Err("negative fixture equals positive fixture".to_string());
    }
    Ok("Nario v2 fixture checker self-test passed".to_string())
}
