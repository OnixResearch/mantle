const ARCHIVE_MAGIC: &[u8] = b"!<arch>\n";
const ARCHIVE_HEADER_BYTES: usize = 60;
const ARCHIVE_HEADER_TRAILER: &[u8] = b"`\n";
const ARCHIVE_NAME_BYTES: usize = 16;
const ARCHIVE_TIMESTAMP_BYTES: usize = 12;
const ARCHIVE_OWNER_BYTES: usize = 6;
const ARCHIVE_GROUP_BYTES: usize = 6;
const ARCHIVE_MODE_BYTES: usize = 8;
const ARCHIVE_SIZE_BYTES: usize = 10;
const ARCHIVE_ALIGNMENT_BYTES: usize = 2;
const ARCHIVE_MEMBER_COUNT_MAX: usize = 256;
const ARCHIVE_MEMBER_BYTES_MAX: usize = 64 * 1_024 * 1_024;
const ARCHIVE_STRING_TABLE_BYTES_MAX: usize = 1_024 * 1_024;
const ARCHIVE_OUTPUT_BYTES_MAX: usize = 512 * 1_024 * 1_024;
const ARCHIVE_MEMBER_NAME_BYTES_MAX: usize = 255;
const ARCHIVE_SHORT_NAME_BYTES_MAX: usize = 15;
const ARCHIVE_MODE: &str = "100644";
const ARCHIVE_STRING_TABLE_NAME: &str = "//";
const ARCHIVE_LONG_NAME_SUFFIX: &[u8] = b"/\n";
const ELF_MAGIC: &[u8] = b"\x7fELF";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum StagexArchiveError {
    UnsupportedFormat(&'static str),
    OutOfBounds(&'static str),
    LimitExceeded(&'static str),
    InvalidMemberName,
    ElfMember(String),
}

impl std::fmt::Display for StagexArchiveError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedFormat(message) => write!(formatter, "unsupported StageX archive format: {message}"),
            Self::OutOfBounds(message) => write!(formatter, "StageX archive range is out of bounds: {message}"),
            Self::LimitExceeded(message) => write!(formatter, "StageX archive limit exceeded: {message}"),
            Self::InvalidMemberName => write!(formatter, "StageX archive member name is invalid"),
            Self::ElfMember(message) => write!(formatter, "StageX archive ELF member is invalid: {message}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalizedStagexArchive {
    pub(crate) bytes: Vec<u8>,
    pub(crate) member_count: u32,
    pub(crate) rewritten_member_count: u32,
    pub(crate) rewritten_symbol_count: u32,
}

#[derive(Debug, Clone, Copy)]
struct ArchiveMember<'a> {
    header_name: &'a str,
    payload: &'a [u8],
}

pub(crate) fn canonicalize_stagex_archive_elf_members(
    bytes: &[u8],
) -> Result<CanonicalizedStagexArchive, StagexArchiveError> {
    let members = parse_archive(bytes)?;
    let string_table = archive_string_table(&members)?;
    let mut output = Vec::with_capacity(bytes.len());
    output.extend_from_slice(ARCHIVE_MAGIC);
    let mut member_count = 0u32;
    let mut rewritten_member_count = 0u32;
    let mut rewritten_symbol_count = 0u32;
    for member in members {
        if member.header_name == ARCHIVE_STRING_TABLE_NAME {
            append_member(&mut output, member.header_name, member.payload)?;
            continue;
        }
        validate_member_name(member.header_name, string_table)?;
        let canonical = canonicalize_member(member.payload)?;
        member_count = member_count.checked_add(1).ok_or(StagexArchiveError::LimitExceeded("member count"))?;
        rewritten_member_count = rewritten_member_count
            .checked_add(u32::from(canonical.bytes != member.payload))
            .ok_or(StagexArchiveError::LimitExceeded("rewritten member count"))?;
        rewritten_symbol_count = rewritten_symbol_count
            .checked_add(canonical.rewrite_count)
            .ok_or(StagexArchiveError::LimitExceeded("rewritten symbol count"))?;
        append_member(&mut output, member.header_name, &canonical.bytes)?;
    }
    if member_count == 0 {
        return Err(StagexArchiveError::UnsupportedFormat("archive has no object members"));
    }
    if output.len() > ARCHIVE_OUTPUT_BYTES_MAX {
        return Err(StagexArchiveError::LimitExceeded("canonical archive bytes"));
    }
    assert!(usize::try_from(member_count).is_ok_and(|count| count <= ARCHIVE_MEMBER_COUNT_MAX));
    assert!(output.starts_with(ARCHIVE_MAGIC));
    Ok(CanonicalizedStagexArchive {
        bytes: output,
        member_count,
        rewritten_member_count,
        rewritten_symbol_count,
    })
}

fn canonicalize_member(payload: &[u8]) -> Result<crate::elf_local_symbol_core::CanonicalizedElf, StagexArchiveError> {
    if payload.len() > ARCHIVE_MEMBER_BYTES_MAX {
        return Err(StagexArchiveError::LimitExceeded("member bytes"));
    }
    if !payload.starts_with(ELF_MAGIC) {
        return Err(StagexArchiveError::UnsupportedFormat("archive member is not ELF"));
    }
    let canonical = crate::elf_local_symbol_core::canonicalize_local_elf_symbol_names(payload)
        .map_err(|error| StagexArchiveError::ElfMember(error.to_string()))?;
    if canonical.bytes.len() > ARCHIVE_MEMBER_BYTES_MAX {
        return Err(StagexArchiveError::LimitExceeded("canonical member bytes"));
    }
    assert!(canonical.bytes.starts_with(ELF_MAGIC));
    assert!(!canonical.bytes.is_empty());
    Ok(canonical)
}

fn parse_archive(bytes: &[u8]) -> Result<Vec<ArchiveMember<'_>>, StagexArchiveError> {
    if !bytes.starts_with(ARCHIVE_MAGIC) {
        return Err(StagexArchiveError::UnsupportedFormat("global magic mismatch"));
    }
    if bytes.len() > ARCHIVE_OUTPUT_BYTES_MAX {
        return Err(StagexArchiveError::LimitExceeded("input archive bytes"));
    }
    let mut members = Vec::new();
    let mut cursor = ARCHIVE_MAGIC.len();
    while cursor < bytes.len() {
        if members.len() > ARCHIVE_MEMBER_COUNT_MAX {
            return Err(StagexArchiveError::LimitExceeded("archive member count"));
        }
        let header_range = checked_range(bytes.len(), cursor, ARCHIVE_HEADER_BYTES, "member header")?;
        let header = &bytes[header_range];
        validate_header_metadata(header)?;
        let header_name = parse_header_name(header)?;
        let size_start = ARCHIVE_NAME_BYTES
            + ARCHIVE_TIMESTAMP_BYTES
            + ARCHIVE_OWNER_BYTES
            + ARCHIVE_GROUP_BYTES
            + ARCHIVE_MODE_BYTES;
        let size_field = &header[size_start..ARCHIVE_HEADER_BYTES - ARCHIVE_HEADER_TRAILER.len()];
        let payload_bytes = parse_decimal_field(size_field, "member size")?;
        require_canonical_field(size_field, payload_bytes, ARCHIVE_SIZE_BYTES, "member size")?;
        if payload_bytes > ARCHIVE_MEMBER_BYTES_MAX && header_name != ARCHIVE_STRING_TABLE_NAME {
            return Err(StagexArchiveError::LimitExceeded("member bytes"));
        }
        if payload_bytes > ARCHIVE_STRING_TABLE_BYTES_MAX && header_name == ARCHIVE_STRING_TABLE_NAME {
            return Err(StagexArchiveError::LimitExceeded("string-table bytes"));
        }
        cursor = cursor.checked_add(ARCHIVE_HEADER_BYTES).ok_or(StagexArchiveError::OutOfBounds("member payload"))?;
        let payload_range = checked_range(bytes.len(), cursor, payload_bytes, "member payload")?;
        members.push(ArchiveMember {
            header_name,
            payload: &bytes[payload_range.clone()],
        });
        cursor = aligned_member_end(bytes, payload_range.end, payload_bytes)?;
    }
    if cursor != bytes.len() {
        return Err(StagexArchiveError::OutOfBounds("archive trailer"));
    }
    assert!(members.len() <= ARCHIVE_MEMBER_COUNT_MAX + 1);
    assert_eq!(cursor, bytes.len());
    Ok(members)
}

fn validate_header_metadata(header: &[u8]) -> Result<(), StagexArchiveError> {
    if header.len() != ARCHIVE_HEADER_BYTES {
        return Err(StagexArchiveError::OutOfBounds("member header"));
    }
    let timestamp_start = ARCHIVE_NAME_BYTES;
    let owner_start = timestamp_start + ARCHIVE_TIMESTAMP_BYTES;
    let group_start = owner_start + ARCHIVE_OWNER_BYTES;
    let mode_start = group_start + ARCHIVE_GROUP_BYTES;
    let size_start = mode_start + ARCHIVE_MODE_BYTES;
    let timestamp = &header[timestamp_start..owner_start];
    let owner = &header[owner_start..group_start];
    let group = &header[group_start..mode_start];
    let mode = &header[mode_start..size_start];
    require_canonical_field(timestamp, 0, ARCHIVE_TIMESTAMP_BYTES, "member timestamp")?;
    require_canonical_field(owner, 0, ARCHIVE_OWNER_BYTES, "member owner")?;
    require_canonical_field(group, 0, ARCHIVE_GROUP_BYTES, "member group")?;
    let expected_mode = format!("{ARCHIVE_MODE:<width$}", width = ARCHIVE_MODE_BYTES);
    if mode != expected_mode.as_bytes() {
        return Err(StagexArchiveError::UnsupportedFormat("member mode is not canonical"));
    }
    if &header[ARCHIVE_HEADER_BYTES - ARCHIVE_HEADER_TRAILER.len()..] != ARCHIVE_HEADER_TRAILER {
        return Err(StagexArchiveError::UnsupportedFormat("member header trailer mismatch"));
    }
    assert_eq!(size_start + ARCHIVE_SIZE_BYTES + ARCHIVE_HEADER_TRAILER.len(), ARCHIVE_HEADER_BYTES);
    assert_eq!(header.len(), ARCHIVE_HEADER_BYTES);
    Ok(())
}

fn parse_header_name(header: &[u8]) -> Result<&str, StagexArchiveError> {
    let name = parse_ascii_field(&header[..ARCHIVE_NAME_BYTES], "member name")?;
    if name.is_empty() {
        return Err(StagexArchiveError::InvalidMemberName);
    }
    if name.len() > ARCHIVE_NAME_BYTES {
        return Err(StagexArchiveError::InvalidMemberName);
    }
    assert!(!name.is_empty());
    assert!(name.len() <= ARCHIVE_NAME_BYTES);
    Ok(name)
}

fn parse_ascii_field<'a>(field: &'a [u8], label: &'static str) -> Result<&'a str, StagexArchiveError> {
    let text = std::str::from_utf8(field).map_err(|_| StagexArchiveError::UnsupportedFormat(label))?;
    let trimmed = text.trim_end_matches(' ');
    if trimmed.bytes().any(|byte| byte.is_ascii_control()) {
        return Err(StagexArchiveError::UnsupportedFormat(label));
    }
    Ok(trimmed)
}

fn parse_decimal_field(field: &[u8], label: &'static str) -> Result<usize, StagexArchiveError> {
    let text = parse_ascii_field(field, label)?;
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(StagexArchiveError::UnsupportedFormat(label));
    }
    text.parse::<usize>().map_err(|_| StagexArchiveError::LimitExceeded(label))
}

fn require_canonical_field(
    field: &[u8],
    value: usize,
    width: usize,
    label: &'static str,
) -> Result<(), StagexArchiveError> {
    let expected = format!("{value:<width$}");
    if field != expected.as_bytes() {
        return Err(StagexArchiveError::UnsupportedFormat(label));
    }
    assert_eq!(field.len(), width);
    assert_eq!(expected.len(), width);
    Ok(())
}

fn aligned_member_end(bytes: &[u8], payload_end: usize, payload_bytes: usize) -> Result<usize, StagexArchiveError> {
    if payload_bytes.is_multiple_of(ARCHIVE_ALIGNMENT_BYTES) {
        return Ok(payload_end);
    }
    let padding_range = checked_range(bytes.len(), payload_end, 1, "member alignment padding")?;
    if bytes[padding_range.clone()] != *b"\n" {
        return Err(StagexArchiveError::UnsupportedFormat("member alignment padding mismatch"));
    }
    let result = padding_range.end;
    assert_eq!(result, payload_end + 1);
    assert_eq!(result % ARCHIVE_ALIGNMENT_BYTES, 0);
    Ok(result)
}

fn archive_string_table<'a>(members: &[ArchiveMember<'a>]) -> Result<Option<&'a [u8]>, StagexArchiveError> {
    let mut table = None;
    for (index, member) in members.iter().enumerate() {
        if member.header_name != ARCHIVE_STRING_TABLE_NAME {
            continue;
        }
        if index != 0 || table.is_some() {
            return Err(StagexArchiveError::UnsupportedFormat(
                "string table must be the first and only special member",
            ));
        }
        validate_string_table(member.payload)?;
        table = Some(member.payload);
    }
    assert!(table.is_none_or(|bytes| bytes.len() <= ARCHIVE_STRING_TABLE_BYTES_MAX));
    assert!(members.len() <= ARCHIVE_MEMBER_COUNT_MAX + 1);
    Ok(table)
}

fn validate_string_table(bytes: &[u8]) -> Result<(), StagexArchiveError> {
    if bytes.is_empty() || bytes.len() > ARCHIVE_STRING_TABLE_BYTES_MAX {
        return Err(StagexArchiveError::UnsupportedFormat("invalid archive string table size"));
    }
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        let suffix = bytes[cursor..]
            .windows(ARCHIVE_LONG_NAME_SUFFIX.len())
            .position(|window| window == ARCHIVE_LONG_NAME_SUFFIX)
            .ok_or(StagexArchiveError::UnsupportedFormat("unterminated long member name"))?;
        let name_end = cursor.checked_add(suffix).ok_or(StagexArchiveError::OutOfBounds("long member name"))?;
        validate_plain_member_name(&bytes[cursor..name_end])?;
        cursor = name_end
            .checked_add(ARCHIVE_LONG_NAME_SUFFIX.len())
            .ok_or(StagexArchiveError::OutOfBounds("long member name"))?;
    }
    assert_eq!(cursor, bytes.len());
    assert!(!bytes.is_empty());
    Ok(())
}

fn validate_member_name(header_name: &str, string_table: Option<&[u8]>) -> Result<(), StagexArchiveError> {
    if let Some(offset_text) = header_name.strip_prefix('/') {
        if offset_text.is_empty() || !offset_text.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(StagexArchiveError::InvalidMemberName);
        }
        let offset = offset_text.parse::<usize>().map_err(|_| StagexArchiveError::InvalidMemberName)?;
        if offset_text != offset.to_string() {
            return Err(StagexArchiveError::InvalidMemberName);
        }
        let table = string_table.ok_or(StagexArchiveError::InvalidMemberName)?;
        validate_long_name_offset(table, offset)?;
        return Ok(());
    }
    let name = header_name.strip_suffix('/').ok_or(StagexArchiveError::InvalidMemberName)?;
    if name.len() > ARCHIVE_SHORT_NAME_BYTES_MAX {
        return Err(StagexArchiveError::InvalidMemberName);
    }
    validate_plain_member_name(name.as_bytes())
}

fn validate_long_name_offset(table: &[u8], offset: usize) -> Result<(), StagexArchiveError> {
    if offset >= table.len() {
        return Err(StagexArchiveError::InvalidMemberName);
    }
    if offset > 0 && table[offset - 1] != b'\n' {
        return Err(StagexArchiveError::InvalidMemberName);
    }
    let suffix = table[offset..]
        .windows(ARCHIVE_LONG_NAME_SUFFIX.len())
        .position(|window| window == ARCHIVE_LONG_NAME_SUFFIX)
        .ok_or(StagexArchiveError::InvalidMemberName)?;
    let end = offset.checked_add(suffix).ok_or(StagexArchiveError::InvalidMemberName)?;
    validate_plain_member_name(&table[offset..end])?;
    assert!(end <= table.len());
    assert!(offset < table.len());
    Ok(())
}

fn validate_plain_member_name(name: &[u8]) -> Result<(), StagexArchiveError> {
    if name.is_empty() || name.len() > ARCHIVE_MEMBER_NAME_BYTES_MAX {
        return Err(StagexArchiveError::InvalidMemberName);
    }
    if !name.iter().all(|byte| byte.is_ascii_alphanumeric() || matches!(*byte, b'.' | b'_' | b'+' | b'-')) {
        return Err(StagexArchiveError::InvalidMemberName);
    }
    assert!(!name.is_empty());
    assert!(name.len() <= ARCHIVE_MEMBER_NAME_BYTES_MAX);
    Ok(())
}

fn append_member(output: &mut Vec<u8>, header_name: &str, payload: &[u8]) -> Result<(), StagexArchiveError> {
    if header_name.len() > ARCHIVE_NAME_BYTES || payload.len() > ARCHIVE_MEMBER_BYTES_MAX {
        return Err(StagexArchiveError::LimitExceeded("canonical archive member"));
    }
    let header = format!("{header_name:<16}{:<12}{:<6}{:<6}{:<8}{:<10}`\n", 0, 0, 0, ARCHIVE_MODE, payload.len());
    if header.len() != ARCHIVE_HEADER_BYTES {
        return Err(StagexArchiveError::UnsupportedFormat("canonical member header width"));
    }
    let projected = output
        .len()
        .checked_add(header.len())
        .and_then(|size| size.checked_add(payload.len()))
        .and_then(|size| size.checked_add(payload.len() % ARCHIVE_ALIGNMENT_BYTES))
        .ok_or(StagexArchiveError::LimitExceeded("canonical archive bytes"))?;
    if projected > ARCHIVE_OUTPUT_BYTES_MAX {
        return Err(StagexArchiveError::LimitExceeded("canonical archive bytes"));
    }
    output.extend_from_slice(header.as_bytes());
    output.extend_from_slice(payload);
    if !payload.len().is_multiple_of(ARCHIVE_ALIGNMENT_BYTES) {
        output.push(b'\n');
    }
    assert_eq!(output.len(), projected);
    assert!(output.starts_with(ARCHIVE_MAGIC));
    Ok(())
}

fn checked_range(
    total_bytes: usize,
    offset: usize,
    length: usize,
    label: &'static str,
) -> Result<std::ops::Range<usize>, StagexArchiveError> {
    let end = offset.checked_add(length).ok_or(StagexArchiveError::OutOfBounds(label))?;
    if end > total_bytes {
        return Err(StagexArchiveError::OutOfBounds(label));
    }
    Ok(offset..end)
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::*;

    const TEST_ELF_HEADER_BYTES: usize = 64;
    const TEST_ELF_SECTION_HEADER_BYTES: usize = 64;
    const TEST_ELF_SYMBOL_BYTES: usize = 24;
    const TEST_ELF_SECTION_COUNT: u16 = 3;
    const TEST_ELF_SECTION_COUNT_USIZE: usize = 3;
    const TEST_ELF_SYMBOL_COUNT: usize = 2;
    const TEST_ELF_SECTION_TABLE_OFFSET: usize = TEST_ELF_HEADER_BYTES;
    const TEST_ELF_SYMBOL_TABLE_OFFSET: usize =
        TEST_ELF_SECTION_TABLE_OFFSET + TEST_ELF_SECTION_COUNT_USIZE * TEST_ELF_SECTION_HEADER_BYTES;
    const TEST_ELF_STRING_TABLE_OFFSET: usize =
        TEST_ELF_SYMBOL_TABLE_OFFSET + TEST_ELF_SYMBOL_COUNT * TEST_ELF_SYMBOL_BYTES;
    const TEST_SHORT_NAME: &[u8] = b"L.1";
    const TEST_LONG_NAME: &[u8] = b"L.521018500";
    const EXTERNAL_ARCHIVE_VARIANT_A_ENV: &str = "MANTLE_STAGE_X_ARCHIVE_VARIANT_A";
    const EXTERNAL_ARCHIVE_VARIANT_B_ENV: &str = "MANTLE_STAGE_X_ARCHIVE_VARIANT_B";
    const EXTERNAL_ARCHIVE_INPUT_ENV: &str = "MANTLE_STAGE_X_ARCHIVE_INPUT";

    #[test]
    fn canonicalizes_width_variant_archive_members_to_one_idempotent_identity() {
        let short = archive_with_member("member.o/", &synthetic_elf(TEST_SHORT_NAME));
        let long = archive_with_member("member.o/", &synthetic_elf(TEST_LONG_NAME));

        let canonical_short = canonicalize_stagex_archive_elf_members(&short).unwrap();
        let canonical_long = canonicalize_stagex_archive_elf_members(&long).unwrap();
        let repeated = canonicalize_stagex_archive_elf_members(&canonical_short.bytes).unwrap();

        assert_ne!(short, long);
        assert_eq!(canonical_short.bytes, canonical_long.bytes);
        assert_eq!(canonical_short.bytes, repeated.bytes);
        assert_eq!(canonical_short.member_count, 1);
        assert_eq!(canonical_short.rewritten_member_count, 1);
        assert_eq!(canonical_short.rewritten_symbol_count, 1);
    }

    #[test]
    #[ignore = "requires two retained StageX archives whose local-symbol widths differ"]
    fn retained_archive_width_variants_converge() {
        let variant_a = std::fs::read(std::env::var_os(EXTERNAL_ARCHIVE_VARIANT_A_ENV).unwrap()).unwrap();
        let variant_b = std::fs::read(std::env::var_os(EXTERNAL_ARCHIVE_VARIANT_B_ENV).unwrap()).unwrap();

        let canonical_a = canonicalize_stagex_archive_elf_members(&variant_a).unwrap();
        let canonical_b = canonicalize_stagex_archive_elf_members(&variant_b).unwrap();
        let digest = blake3::hash(&canonical_a.bytes).to_hex().to_string();

        assert_ne!(variant_a, variant_b);
        assert_eq!(canonical_a.bytes, canonical_b.bytes);
        assert!(canonical_a.rewritten_member_count > 0);
        assert!(canonical_b.rewritten_member_count > 0);
        println!("canonical-stagex-archive-blake3={digest}");
    }

    #[test]
    #[ignore = "requires one retained StageX archive"]
    fn reports_canonical_retained_archive_identity() {
        let input = std::fs::read(std::env::var_os(EXTERNAL_ARCHIVE_INPUT_ENV).unwrap()).unwrap();

        let canonical = canonicalize_stagex_archive_elf_members(&input).unwrap();
        let repeated = canonicalize_stagex_archive_elf_members(&canonical.bytes).unwrap();
        let digest = blake3::hash(&canonical.bytes).to_hex().to_string();

        assert_eq!(canonical.bytes, repeated.bytes);
        assert!(canonical.member_count > 0);
        assert!(canonical.rewritten_symbol_count > 0);
        println!("canonical-stagex-archive-blake3={digest}");
    }

    #[test]
    fn accepts_the_closed_long_name_table_shape() {
        let member_name = "member-name-longer-than-fifteen.o";
        let string_table = format!("{member_name}/\n");
        let mut archive = Vec::from(ARCHIVE_MAGIC);
        append_member(&mut archive, ARCHIVE_STRING_TABLE_NAME, string_table.as_bytes()).unwrap();
        append_member(&mut archive, "/0", &synthetic_elf(TEST_SHORT_NAME)).unwrap();

        let canonical = canonicalize_stagex_archive_elf_members(&archive).unwrap();

        assert_eq!(canonical.member_count, 1);
        assert_eq!(canonical.rewritten_member_count, 1);
        assert!(canonical.bytes.windows(member_name.len()).any(|window| window == member_name.as_bytes()));
    }

    #[test]
    fn rejects_malformed_metadata_non_elf_members_and_invalid_long_offsets() {
        let mut bad_timestamp = archive_with_member("member.o/", &synthetic_elf(TEST_SHORT_NAME));
        bad_timestamp[ARCHIVE_MAGIC.len() + ARCHIVE_NAME_BYTES] = b'1';
        let non_elf = archive_with_member("member.o/", b"not-elf");
        let mut bad_long = Vec::from(ARCHIVE_MAGIC);
        append_member(&mut bad_long, ARCHIVE_STRING_TABLE_NAME, b"long-member.o/\n").unwrap();
        append_member(&mut bad_long, "/2", &synthetic_elf(TEST_SHORT_NAME)).unwrap();

        let timestamp_error = canonicalize_stagex_archive_elf_members(&bad_timestamp).unwrap_err();
        let non_elf_error = canonicalize_stagex_archive_elf_members(&non_elf).unwrap_err();
        let long_offset_error = canonicalize_stagex_archive_elf_members(&bad_long).unwrap_err();

        assert!(timestamp_error.to_string().contains("timestamp"));
        assert!(non_elf_error.to_string().contains("not ELF"));
        assert_eq!(long_offset_error, StagexArchiveError::InvalidMemberName);
    }

    fn archive_with_member(name: &str, payload: &[u8]) -> Vec<u8> {
        let mut archive = Vec::from(ARCHIVE_MAGIC);
        append_member(&mut archive, name, payload).unwrap();
        assert!(archive.len() > ARCHIVE_MAGIC.len());
        assert!(archive.starts_with(ARCHIVE_MAGIC));
        archive
    }

    fn synthetic_elf(local_name: &[u8]) -> Vec<u8> {
        let string_table = [b"\0".as_slice(), local_name, b"\0global\0".as_slice()].concat();
        let file_bytes = TEST_ELF_STRING_TABLE_OFFSET + string_table.len();
        let mut bytes = vec![0u8; file_bytes];
        bytes[..ELF_MAGIC.len()].copy_from_slice(ELF_MAGIC);
        bytes[4] = 2;
        bytes[5] = 1;
        write_u16(&mut bytes, 16, 1);
        write_u64(&mut bytes, 40, u64::try_from(TEST_ELF_SECTION_TABLE_OFFSET).unwrap());
        write_u16(&mut bytes, 58, u16::try_from(TEST_ELF_SECTION_HEADER_BYTES).unwrap());
        write_u16(&mut bytes, 60, TEST_ELF_SECTION_COUNT);

        let symbols_section = TEST_ELF_SECTION_TABLE_OFFSET + TEST_ELF_SECTION_HEADER_BYTES;
        write_u32(&mut bytes, symbols_section + 4, 2);
        write_u64(&mut bytes, symbols_section + 24, u64::try_from(TEST_ELF_SYMBOL_TABLE_OFFSET).unwrap());
        write_u64(
            &mut bytes,
            symbols_section + 32,
            u64::try_from(TEST_ELF_SYMBOL_COUNT * TEST_ELF_SYMBOL_BYTES).unwrap(),
        );
        write_u32(&mut bytes, symbols_section + 40, 2);
        write_u64(&mut bytes, symbols_section + 56, u64::try_from(TEST_ELF_SYMBOL_BYTES).unwrap());

        let strings_section = symbols_section + TEST_ELF_SECTION_HEADER_BYTES;
        write_u32(&mut bytes, strings_section + 4, 3);
        write_u64(&mut bytes, strings_section + 24, u64::try_from(TEST_ELF_STRING_TABLE_OFFSET).unwrap());
        write_u64(&mut bytes, strings_section + 32, u64::try_from(string_table.len()).unwrap());

        let local_symbol = TEST_ELF_SYMBOL_TABLE_OFFSET + TEST_ELF_SYMBOL_BYTES;
        write_u32(&mut bytes, local_symbol, 1);
        bytes[local_symbol + 4] = 0;
        bytes[TEST_ELF_STRING_TABLE_OFFSET..].copy_from_slice(&string_table);
        assert!(bytes.starts_with(ELF_MAGIC));
        assert_eq!(bytes.len(), file_bytes);
        bytes
    }

    fn write_u16(bytes: &mut [u8], offset: usize, value: u16) {
        bytes[offset..offset + size_of::<u16>()].copy_from_slice(&value.to_le_bytes());
    }

    fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
        bytes[offset..offset + size_of::<u32>()].copy_from_slice(&value.to_le_bytes());
    }

    fn write_u64(bytes: &mut [u8], offset: usize, value: u64) {
        bytes[offset..offset + size_of::<u64>()].copy_from_slice(&value.to_le_bytes());
    }
}
