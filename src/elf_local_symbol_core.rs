use std::collections::BTreeMap;

const ELF_HEADER_BYTES: usize = 64;
const ELF_SECTION_HEADER_BYTES: usize = 64;
const ELF_SYMBOL_BYTES: usize = 24;
const ELF_TYPE_OFFSET: usize = 16;
const ELF_PROGRAM_TABLE_OFFSET: usize = 32;
const ELF_SECTION_TABLE_OFFSET: usize = 40;
const ELF_PROGRAM_ENTRY_SIZE_OFFSET: usize = 54;
const ELF_PROGRAM_COUNT_OFFSET: usize = 56;
const ELF_SECTION_ENTRY_SIZE_OFFSET: usize = 58;
const ELF_SECTION_COUNT_OFFSET: usize = 60;
const SECTION_TYPE_OFFSET: usize = 4;
const SECTION_FILE_OFFSET_OFFSET: usize = 24;
const SECTION_SIZE_OFFSET: usize = 32;
const SECTION_LINK_OFFSET: usize = 40;
const SECTION_ALIGNMENT_OFFSET: usize = 48;
const SECTION_ENTRY_SIZE_OFFSET: usize = 56;
const SECTION_HEADER_ALIGNMENT_BYTES: usize = 8;
const SECTION_ALIGNMENT_BYTES_MAX: usize = 4_096;
const SYMBOL_NAME_OFFSET: usize = 0;
const SYMBOL_INFO_OFFSET: usize = 4;
const ELF_IDENT_CLASS_OFFSET: usize = 4;
const ELF_IDENT_DATA_OFFSET: usize = 5;
const ELF_CLASS_64: u8 = 2;
const ELF_DATA_LITTLE_ENDIAN: u8 = 1;
const ELF_TYPE_RELOCATABLE: u16 = 1;
const SECTION_TYPE_SYMBOL_TABLE: u32 = 2;
const SECTION_TYPE_STRING_TABLE: u32 = 3;
const SECTION_TYPE_NO_BITS: u32 = 8;
const SYMBOL_BIND_SHIFT: u8 = 4;
const SYMBOL_BIND_LOCAL: u8 = 0;
const SECTION_COUNT_MAX: usize = 4_096;
const SYMBOL_COUNT_MAX: usize = 1_048_576;
const STRING_TABLE_BYTES_MAX: usize = 64 * 1_024 * 1_024;
const LOCAL_SYMBOL_PREFIX: &[u8] = b"L.";
const LOCAL_SYMBOL_DECIMAL_DIGITS: usize = 10;
const DECIMAL_RADIX: usize = 10;
const ELF_MAGIC: &[u8] = b"\x7fELF";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ElfCanonicalizationError {
    UnsupportedFormat(&'static str),
    OutOfBounds(&'static str),
    LimitExceeded(&'static str),
    ConflictingSymbolName,
    SymbolIndexTooWide,
}

impl std::fmt::Display for ElfCanonicalizationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedFormat(message) => write!(formatter, "unsupported ELF format: {message}"),
            Self::OutOfBounds(message) => write!(formatter, "ELF range is out of bounds: {message}"),
            Self::LimitExceeded(message) => write!(formatter, "ELF limit exceeded: {message}"),
            Self::ConflictingSymbolName => write!(formatter, "ELF symbol name is shared by conflicting symbols"),
            Self::SymbolIndexTooWide => write!(formatter, "ELF local-symbol index exceeds the canonical decimal width"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalizedElf {
    pub(crate) bytes: Vec<u8>,
    pub(crate) rewrite_count: u32,
}

#[derive(Debug, Clone, Copy)]
struct SectionHeader {
    section_type: u32,
    file_offset: usize,
    size_bytes: usize,
    linked_section_index: usize,
    alignment_bytes: usize,
    entry_size_bytes: usize,
}

#[derive(Debug, Clone, Copy)]
struct StringEntry {
    old_offset: usize,
    start: usize,
    length: usize,
}

#[derive(Debug, Clone, Copy)]
struct SymbolReference {
    file_offset: usize,
    old_name_offset: usize,
    binding: u8,
    symbol_index: usize,
}

pub(crate) fn canonicalize_local_elf_symbol_names(bytes: &[u8]) -> Result<CanonicalizedElf, ElfCanonicalizationError> {
    let (section_table_offset, section_count) = validate_header(bytes)?;
    let sections = read_sections(bytes, section_table_offset, section_count)?;
    let (symbol_section_index, symbols) = sole_symbol_table(&sections)?;
    if symbols.linked_section_index >= section_count {
        return Err(ElfCanonicalizationError::OutOfBounds("linked string-table section"));
    }
    let string_section_index = symbols.linked_section_index;
    let strings = sections[string_section_index];
    if strings.section_type != SECTION_TYPE_STRING_TABLE {
        return Err(ElfCanonicalizationError::UnsupportedFormat("symbol table does not link to a string table"));
    }
    validate_section_layout(bytes, section_table_offset, &sections, string_section_index)?;
    let entries = read_string_entries(bytes, strings)?;
    let symbols = read_symbols(bytes, symbols, &entries)?;
    let targets = collect_canonical_targets(bytes, strings, &symbols)?;
    let (canonical_strings, name_offsets) = rebuild_string_table(bytes, strings, &entries, &targets)?;
    let transformed = rebuild_elf(
        bytes,
        section_table_offset,
        &sections,
        symbol_section_index,
        string_section_index,
        &symbols,
        &name_offsets,
        &canonical_strings,
    )?;
    let rewrite_count =
        u32::try_from(targets.len()).map_err(|_| ElfCanonicalizationError::LimitExceeded("rewrite count"))?;
    assert!(section_count <= SECTION_COUNT_MAX);
    assert!(targets.len() <= SYMBOL_COUNT_MAX);
    Ok(CanonicalizedElf {
        bytes: transformed,
        rewrite_count,
    })
}

fn validate_header(bytes: &[u8]) -> Result<(usize, usize), ElfCanonicalizationError> {
    if bytes.len() < ELF_HEADER_BYTES {
        return Err(ElfCanonicalizationError::UnsupportedFormat("file is smaller than an ELF64 header"));
    }
    if bytes.get(..ELF_MAGIC.len()) != Some(ELF_MAGIC) {
        return Err(ElfCanonicalizationError::UnsupportedFormat("ELF magic mismatch"));
    }
    if bytes[ELF_IDENT_CLASS_OFFSET] != ELF_CLASS_64 || bytes[ELF_IDENT_DATA_OFFSET] != ELF_DATA_LITTLE_ENDIAN {
        return Err(ElfCanonicalizationError::UnsupportedFormat("ELF must be 64-bit little-endian"));
    }
    if read_u16(bytes, ELF_TYPE_OFFSET, "ELF type")? != ELF_TYPE_RELOCATABLE {
        return Err(ElfCanonicalizationError::UnsupportedFormat("ELF must be relocatable"));
    }
    let program_table_offset = read_usize_from_u64(bytes, ELF_PROGRAM_TABLE_OFFSET, "program table offset")?;
    let program_entry_size = read_u16(bytes, ELF_PROGRAM_ENTRY_SIZE_OFFSET, "program entry size")?;
    let program_count = read_u16(bytes, ELF_PROGRAM_COUNT_OFFSET, "program count")?;
    if program_table_offset != 0 || program_entry_size != 0 || program_count != 0 {
        return Err(ElfCanonicalizationError::UnsupportedFormat("relocatable ELF must not contain program headers"));
    }
    let section_table_offset = read_usize_from_u64(bytes, ELF_SECTION_TABLE_OFFSET, "section table offset")?;
    let section_entry_size = usize::from(read_u16(bytes, ELF_SECTION_ENTRY_SIZE_OFFSET, "section entry size")?);
    let section_count = usize::from(read_u16(bytes, ELF_SECTION_COUNT_OFFSET, "section count")?);
    if section_entry_size != ELF_SECTION_HEADER_BYTES || section_count == 0 {
        return Err(ElfCanonicalizationError::UnsupportedFormat("invalid section table shape"));
    }
    if section_count > SECTION_COUNT_MAX {
        return Err(ElfCanonicalizationError::LimitExceeded("section count"));
    }
    let section_table_bytes = section_count
        .checked_mul(section_entry_size)
        .ok_or(ElfCanonicalizationError::OutOfBounds("section table size"))?;
    checked_range(bytes.len(), section_table_offset, section_table_bytes, "section table")?;
    assert!(ELF_HEADER_BYTES <= bytes.len());
    debug_assert_eq!(ELF_MAGIC.len(), ELF_IDENT_CLASS_OFFSET);
    Ok((section_table_offset, section_count))
}

fn read_sections(
    bytes: &[u8],
    section_table_offset: usize,
    section_count: usize,
) -> Result<Vec<SectionHeader>, ElfCanonicalizationError> {
    let mut sections = Vec::with_capacity(section_count);
    for section_index in 0..section_count {
        sections.push(read_section(bytes, section_table_offset, section_index)?);
    }
    assert_eq!(sections.len(), section_count);
    debug_assert!(sections.len() <= SECTION_COUNT_MAX);
    Ok(sections)
}

fn sole_symbol_table(sections: &[SectionHeader]) -> Result<(usize, SectionHeader), ElfCanonicalizationError> {
    let mut symbol_table = None;
    for (section_index, section) in sections.iter().copied().enumerate() {
        if section.section_type != SECTION_TYPE_SYMBOL_TABLE {
            continue;
        }
        if symbol_table.is_some() {
            return Err(ElfCanonicalizationError::UnsupportedFormat("multiple symbol tables are not supported"));
        }
        symbol_table = Some((section_index, section));
    }
    let result = symbol_table.ok_or(ElfCanonicalizationError::UnsupportedFormat("ELF has no symbol table"))?;
    assert!(result.0 < sections.len());
    debug_assert_eq!(result.1.section_type, SECTION_TYPE_SYMBOL_TABLE);
    Ok(result)
}

fn validate_section_layout(
    bytes: &[u8],
    section_table_offset: usize,
    sections: &[SectionHeader],
    string_section_index: usize,
) -> Result<(), ElfCanonicalizationError> {
    let strings = sections[string_section_index];
    let string_range = checked_range(bytes.len(), strings.file_offset, strings.size_bytes, "string table")?;
    let section_table_bytes = sections
        .len()
        .checked_mul(ELF_SECTION_HEADER_BYTES)
        .ok_or(ElfCanonicalizationError::OutOfBounds("section table"))?;
    let section_table_range = checked_range(bytes.len(), section_table_offset, section_table_bytes, "section table")?;
    if ranges_overlap(&string_range, &section_table_range) {
        return Err(ElfCanonicalizationError::UnsupportedFormat("string table overlaps section headers"));
    }
    for (section_index, section) in sections.iter().copied().enumerate() {
        if section.section_type == SECTION_TYPE_NO_BITS || section.size_bytes == 0 {
            continue;
        }
        let range = checked_range(bytes.len(), section.file_offset, section.size_bytes, "section payload")?;
        if section_index != string_section_index && ranges_overlap(&string_range, &range) {
            return Err(ElfCanonicalizationError::UnsupportedFormat("string table overlaps another section"));
        }
    }
    assert!(string_range.end <= bytes.len());
    debug_assert!(string_section_index < sections.len());
    Ok(())
}

fn read_string_entries(bytes: &[u8], strings: SectionHeader) -> Result<Vec<StringEntry>, ElfCanonicalizationError> {
    let range = checked_range(bytes.len(), strings.file_offset, strings.size_bytes, "string table")?;
    let mut entries = Vec::new();
    let mut old_offset = 0usize;
    while old_offset < strings.size_bytes {
        if entries.len() > SYMBOL_COUNT_MAX {
            return Err(ElfCanonicalizationError::LimitExceeded("string-table entry count"));
        }
        let start = range
            .start
            .checked_add(old_offset)
            .ok_or(ElfCanonicalizationError::OutOfBounds("string-table entry"))?;
        let capacity = strings.size_bytes - old_offset;
        let length = bounded_string_length(bytes, start, capacity)?;
        entries.push(StringEntry {
            old_offset,
            start,
            length,
        });
        old_offset = old_offset
            .checked_add(length)
            .and_then(|offset| offset.checked_add(1))
            .ok_or(ElfCanonicalizationError::OutOfBounds("string-table entry"))?;
    }
    if old_offset != strings.size_bytes || entries.first().is_none_or(|entry| entry.length != 0) {
        return Err(ElfCanonicalizationError::UnsupportedFormat(
            "string table is not a complete null-prefixed string sequence",
        ));
    }
    assert!(!entries.is_empty());
    assert_eq!(old_offset, strings.size_bytes);
    Ok(entries)
}

fn read_symbols(
    bytes: &[u8],
    symbols: SectionHeader,
    entries: &[StringEntry],
) -> Result<Vec<SymbolReference>, ElfCanonicalizationError> {
    if symbols.entry_size_bytes != ELF_SYMBOL_BYTES || !symbols.size_bytes.is_multiple_of(ELF_SYMBOL_BYTES) {
        return Err(ElfCanonicalizationError::UnsupportedFormat("invalid ELF64 symbol-table entry size"));
    }
    checked_range(bytes.len(), symbols.file_offset, symbols.size_bytes, "symbol table")?;
    let symbol_count = symbols.size_bytes / ELF_SYMBOL_BYTES;
    if symbol_count > SYMBOL_COUNT_MAX {
        return Err(ElfCanonicalizationError::LimitExceeded("symbol count"));
    }
    let entry_offsets = entries.iter().map(|entry| (entry.old_offset, ())).collect::<BTreeMap<_, _>>();
    let mut references = Vec::with_capacity(symbol_count);
    for symbol_index in 0..symbol_count {
        let relative_offset = symbol_index
            .checked_mul(ELF_SYMBOL_BYTES)
            .ok_or(ElfCanonicalizationError::OutOfBounds("symbol offset"))?;
        let file_offset = symbols
            .file_offset
            .checked_add(relative_offset)
            .ok_or(ElfCanonicalizationError::OutOfBounds("symbol offset"))?;
        let old_name_offset = usize::try_from(read_u32(bytes, file_offset + SYMBOL_NAME_OFFSET, "symbol name")?)
            .map_err(|_| ElfCanonicalizationError::OutOfBounds("symbol name"))?;
        if !entry_offsets.contains_key(&old_name_offset) {
            return Err(ElfCanonicalizationError::UnsupportedFormat(
                "symbol name does not start at a string-table entry",
            ));
        }
        references.push(SymbolReference {
            file_offset,
            old_name_offset,
            binding: bytes[file_offset + SYMBOL_INFO_OFFSET] >> SYMBOL_BIND_SHIFT,
            symbol_index,
        });
    }
    assert_eq!(references.len(), symbol_count);
    debug_assert!(references.len() <= SYMBOL_COUNT_MAX);
    Ok(references)
}

fn collect_canonical_targets(
    bytes: &[u8],
    strings: SectionHeader,
    symbols: &[SymbolReference],
) -> Result<BTreeMap<usize, usize>, ElfCanonicalizationError> {
    let mut targets = BTreeMap::new();
    for symbol in symbols {
        let name_start = strings
            .file_offset
            .checked_add(symbol.old_name_offset)
            .ok_or(ElfCanonicalizationError::OutOfBounds("symbol name"))?;
        let name_capacity = strings.size_bytes - symbol.old_name_offset;
        let name_length = bounded_string_length(bytes, name_start, name_capacity)?;
        let name_end =
            name_start.checked_add(name_length).ok_or(ElfCanonicalizationError::OutOfBounds("symbol name"))?;
        let name = &bytes[name_start..name_end];
        if !is_decimal_local_symbol(name) || symbol.binding != SYMBOL_BIND_LOCAL {
            continue;
        }
        if name.len() - LOCAL_SYMBOL_PREFIX.len() > LOCAL_SYMBOL_DECIMAL_DIGITS {
            return Err(ElfCanonicalizationError::UnsupportedFormat(
                "local symbol decimal suffix is wider than the canonical form",
            ));
        }
        if let Some(existing) = targets.insert(symbol.old_name_offset, symbol.symbol_index)
            && existing != symbol.symbol_index
        {
            return Err(ElfCanonicalizationError::ConflictingSymbolName);
        }
    }
    for symbol in symbols {
        if targets.contains_key(&symbol.old_name_offset) && symbol.binding != SYMBOL_BIND_LOCAL {
            return Err(ElfCanonicalizationError::ConflictingSymbolName);
        }
    }
    assert!(targets.len() <= symbols.len());
    debug_assert!(targets.values().all(|index| *index < SYMBOL_COUNT_MAX));
    Ok(targets)
}

fn rebuild_string_table(
    bytes: &[u8],
    strings: SectionHeader,
    entries: &[StringEntry],
    targets: &BTreeMap<usize, usize>,
) -> Result<(Vec<u8>, BTreeMap<usize, usize>), ElfCanonicalizationError> {
    let mut output = Vec::with_capacity(strings.size_bytes);
    let mut offsets = BTreeMap::new();
    for entry in entries {
        let new_offset = output.len();
        offsets.insert(entry.old_offset, new_offset);
        if let Some(symbol_index) = targets.get(&entry.old_offset) {
            append_canonical_symbol_name(&mut output, *symbol_index)?;
        } else {
            let end = entry
                .start
                .checked_add(entry.length)
                .ok_or(ElfCanonicalizationError::OutOfBounds("string-table entry"))?;
            output.extend_from_slice(&bytes[entry.start..end]);
        }
        output.push(0);
        if output.len() > STRING_TABLE_BYTES_MAX {
            return Err(ElfCanonicalizationError::LimitExceeded("canonical string table bytes"));
        }
    }
    if output.len() < strings.size_bytes {
        return Err(ElfCanonicalizationError::UnsupportedFormat("canonical string table unexpectedly shrank"));
    }
    assert_eq!(offsets.len(), entries.len());
    debug_assert_eq!(output.first(), Some(&0));
    Ok((output, offsets))
}

fn append_canonical_symbol_name(output: &mut Vec<u8>, symbol_index: usize) -> Result<(), ElfCanonicalizationError> {
    let mut digits = [b'0'; LOCAL_SYMBOL_DECIMAL_DIGITS];
    let mut value = symbol_index;
    let mut cursor = digits.len();
    loop {
        if cursor == 0 {
            return Err(ElfCanonicalizationError::SymbolIndexTooWide);
        }
        cursor -= 1;
        let digit = u8::try_from(value % DECIMAL_RADIX).map_err(|_| ElfCanonicalizationError::SymbolIndexTooWide)?;
        digits[cursor] = b'0' + digit;
        value /= DECIMAL_RADIX;
        if value == 0 {
            break;
        }
    }
    output.extend_from_slice(LOCAL_SYMBOL_PREFIX);
    output.extend_from_slice(&digits);
    assert_eq!(digits.len(), LOCAL_SYMBOL_DECIMAL_DIGITS);
    debug_assert_eq!(value, 0);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn rebuild_elf(
    bytes: &[u8],
    section_table_offset: usize,
    sections: &[SectionHeader],
    symbol_section_index: usize,
    string_section_index: usize,
    symbols: &[SymbolReference],
    name_offsets: &BTreeMap<usize, usize>,
    canonical_strings: &[u8],
) -> Result<Vec<u8>, ElfCanonicalizationError> {
    let strings = sections[string_section_index];
    let old_string_end = strings
        .file_offset
        .checked_add(strings.size_bytes)
        .ok_or(ElfCanonicalizationError::OutOfBounds("string table"))?;
    let (old_replacement_end, following_alignment_bytes) =
        following_file_boundary(bytes.len(), section_table_offset, sections, string_section_index, old_string_end)?;
    let expected_old_replacement_end = align_up(old_string_end, following_alignment_bytes)?;
    if old_replacement_end != expected_old_replacement_end
        || bytes[old_string_end..old_replacement_end].iter().any(|byte| *byte != 0)
    {
        return Err(ElfCanonicalizationError::UnsupportedFormat("string table has non-canonical following padding"));
    }
    let mut replacement = canonical_strings.to_vec();
    let canonical_string_end = strings
        .file_offset
        .checked_add(canonical_strings.len())
        .ok_or(ElfCanonicalizationError::OutOfBounds("canonical string table"))?;
    let canonical_replacement_end = align_up(canonical_string_end, following_alignment_bytes)?;
    let canonical_padding_bytes = canonical_replacement_end - canonical_string_end;
    let replacement_bytes = replacement
        .len()
        .checked_add(canonical_padding_bytes)
        .ok_or(ElfCanonicalizationError::LimitExceeded("canonical ELF padding"))?;
    replacement.resize(replacement_bytes, 0);
    let old_replacement_bytes = old_replacement_end - strings.file_offset;
    let growth = replacement
        .len()
        .checked_sub(old_replacement_bytes)
        .ok_or(ElfCanonicalizationError::UnsupportedFormat("canonical string-table region unexpectedly shrank"))?;

    let mut output = bytes.to_vec();
    for symbol in symbols {
        let new_name_offset = *name_offsets
            .get(&symbol.old_name_offset)
            .ok_or(ElfCanonicalizationError::UnsupportedFormat("canonical string table lacks a symbol name"))?;
        let new_name_offset = u32::try_from(new_name_offset)
            .map_err(|_| ElfCanonicalizationError::LimitExceeded("symbol name offset"))?;
        write_u32(&mut output, symbol.file_offset + SYMBOL_NAME_OFFSET, new_name_offset, "symbol name")?;
    }
    output.splice(strings.file_offset..old_replacement_end, replacement.iter().copied());
    let expected_output_bytes = bytes
        .len()
        .checked_add(growth)
        .ok_or(ElfCanonicalizationError::LimitExceeded("canonical ELF bytes"))?;
    if output.len() != expected_output_bytes {
        return Err(ElfCanonicalizationError::OutOfBounds("canonical ELF size"));
    }

    let new_section_table_offset =
        shift_offset(section_table_offset, strings.file_offset, old_replacement_end, growth)?;
    write_u64(
        &mut output,
        ELF_SECTION_TABLE_OFFSET,
        u64::try_from(new_section_table_offset)
            .map_err(|_| ElfCanonicalizationError::LimitExceeded("section table offset"))?,
        "section table offset",
    )?;
    for (section_index, section) in sections.iter().copied().enumerate() {
        let header_relative_offset = section_index
            .checked_mul(ELF_SECTION_HEADER_BYTES)
            .ok_or(ElfCanonicalizationError::OutOfBounds("section header"))?;
        let header_offset = new_section_table_offset
            .checked_add(header_relative_offset)
            .ok_or(ElfCanonicalizationError::OutOfBounds("section header"))?;
        checked_range(output.len(), header_offset, ELF_SECTION_HEADER_BYTES, "section header")?;
        let new_file_offset = if section.file_offset == 0 || section_index == string_section_index {
            section.file_offset
        } else {
            shift_offset(section.file_offset, strings.file_offset, old_replacement_end, growth)?
        };
        let new_size_bytes = if section_index == string_section_index {
            canonical_strings.len()
        } else {
            section.size_bytes
        };
        write_u64(
            &mut output,
            header_offset + SECTION_FILE_OFFSET_OFFSET,
            u64::try_from(new_file_offset)
                .map_err(|_| ElfCanonicalizationError::LimitExceeded("section file offset"))?,
            "section file offset",
        )?;
        write_u64(
            &mut output,
            header_offset + SECTION_SIZE_OFFSET,
            u64::try_from(new_size_bytes).map_err(|_| ElfCanonicalizationError::LimitExceeded("section size"))?,
            "section size",
        )?;
    }
    let new_symbol_offset = sections[symbol_section_index].file_offset;
    assert!(new_symbol_offset < output.len());
    assert_eq!(canonical_replacement_end, strings.file_offset + replacement.len());
    debug_assert_eq!(output.len(), expected_output_bytes);
    Ok(output)
}

fn following_file_boundary(
    file_bytes: usize,
    section_table_offset: usize,
    sections: &[SectionHeader],
    string_section_index: usize,
    old_string_end: usize,
) -> Result<(usize, usize), ElfCanonicalizationError> {
    let string_start = sections[string_section_index].file_offset;
    let mut boundary = file_bytes;
    let mut alignment_bytes = 1usize;
    if section_table_offset > string_start && section_table_offset <= boundary {
        boundary = section_table_offset;
        alignment_bytes = SECTION_HEADER_ALIGNMENT_BYTES;
    }
    for (section_index, section) in sections.iter().copied().enumerate() {
        if section_index == string_section_index
            || section.section_type == SECTION_TYPE_NO_BITS
            || section.size_bytes == 0
            || section.file_offset <= string_start
        {
            continue;
        }
        validate_alignment(section.alignment_bytes)?;
        if section.file_offset < boundary {
            boundary = section.file_offset;
            alignment_bytes = section.alignment_bytes.max(1);
        } else if section.file_offset == boundary {
            alignment_bytes = alignment_bytes.max(section.alignment_bytes.max(1));
        }
    }
    if boundary < old_string_end {
        return Err(ElfCanonicalizationError::UnsupportedFormat("following ELF boundary overlaps string table"));
    }
    validate_alignment(alignment_bytes)?;
    assert!(boundary <= file_bytes);
    debug_assert!(alignment_bytes > 0);
    Ok((boundary, alignment_bytes))
}

fn validate_alignment(alignment_bytes: usize) -> Result<(), ElfCanonicalizationError> {
    if alignment_bytes == 0 {
        return Ok(());
    }
    if !alignment_bytes.is_power_of_two() || alignment_bytes > SECTION_ALIGNMENT_BYTES_MAX {
        return Err(ElfCanonicalizationError::UnsupportedFormat("invalid section alignment"));
    }
    Ok(())
}

fn align_up(value: usize, alignment_bytes: usize) -> Result<usize, ElfCanonicalizationError> {
    validate_alignment(alignment_bytes)?;
    let alignment_bytes = alignment_bytes.max(1);
    let mask = alignment_bytes - 1;
    value
        .checked_add(mask)
        .map(|aligned| aligned & !mask)
        .ok_or(ElfCanonicalizationError::OutOfBounds("aligned ELF offset"))
}

fn shift_offset(
    old_offset: usize,
    replaced_start: usize,
    replaced_end: usize,
    growth: usize,
) -> Result<usize, ElfCanonicalizationError> {
    if old_offset < replaced_start {
        return Ok(old_offset);
    }
    if old_offset < replaced_end {
        return Err(ElfCanonicalizationError::UnsupportedFormat("ELF offset points inside the replaced string table"));
    }
    old_offset.checked_add(growth).ok_or(ElfCanonicalizationError::OutOfBounds("shifted ELF offset"))
}

fn read_section(
    bytes: &[u8],
    section_table_offset: usize,
    section_index: usize,
) -> Result<SectionHeader, ElfCanonicalizationError> {
    if section_index >= SECTION_COUNT_MAX {
        return Err(ElfCanonicalizationError::LimitExceeded("section index"));
    }
    let relative_offset = section_index
        .checked_mul(ELF_SECTION_HEADER_BYTES)
        .ok_or(ElfCanonicalizationError::OutOfBounds("section header"))?;
    let offset = section_table_offset
        .checked_add(relative_offset)
        .ok_or(ElfCanonicalizationError::OutOfBounds("section header"))?;
    checked_range(bytes.len(), offset, ELF_SECTION_HEADER_BYTES, "section header")?;
    let section = SectionHeader {
        section_type: read_u32(bytes, offset + SECTION_TYPE_OFFSET, "section type")?,
        file_offset: read_usize_from_u64(bytes, offset + SECTION_FILE_OFFSET_OFFSET, "section file offset")?,
        size_bytes: read_usize_from_u64(bytes, offset + SECTION_SIZE_OFFSET, "section size")?,
        linked_section_index: usize::try_from(read_u32(bytes, offset + SECTION_LINK_OFFSET, "section link")?)
            .map_err(|_| ElfCanonicalizationError::OutOfBounds("section link"))?,
        alignment_bytes: read_usize_from_u64(bytes, offset + SECTION_ALIGNMENT_OFFSET, "section alignment")?,
        entry_size_bytes: read_usize_from_u64(bytes, offset + SECTION_ENTRY_SIZE_OFFSET, "section entry size")?,
    };
    assert!(section_index < SECTION_COUNT_MAX);
    debug_assert!(offset >= section_table_offset);
    Ok(section)
}

fn bounded_string_length(bytes: &[u8], start: usize, capacity: usize) -> Result<usize, ElfCanonicalizationError> {
    let range = checked_range(bytes.len(), start, capacity, "string-table entry")?;
    bytes[range]
        .iter()
        .position(|byte| *byte == 0)
        .ok_or(ElfCanonicalizationError::UnsupportedFormat("unterminated string-table entry"))
}

fn is_decimal_local_symbol(name: &[u8]) -> bool {
    if name.len() <= LOCAL_SYMBOL_PREFIX.len() || !name.starts_with(LOCAL_SYMBOL_PREFIX) {
        return false;
    }
    name[LOCAL_SYMBOL_PREFIX.len()..].iter().all(u8::is_ascii_digit)
}

fn ranges_overlap(left: &std::ops::Range<usize>, right: &std::ops::Range<usize>) -> bool {
    left.start < right.end && right.start < left.end
}

fn read_u16(bytes: &[u8], offset: usize, label: &'static str) -> Result<u16, ElfCanonicalizationError> {
    let range = checked_range(bytes.len(), offset, size_of::<u16>(), label)?;
    let raw: [u8; size_of::<u16>()] =
        bytes[range].try_into().map_err(|_| ElfCanonicalizationError::OutOfBounds(label))?;
    Ok(u16::from_le_bytes(raw))
}

fn read_u32(bytes: &[u8], offset: usize, label: &'static str) -> Result<u32, ElfCanonicalizationError> {
    let range = checked_range(bytes.len(), offset, size_of::<u32>(), label)?;
    let raw: [u8; size_of::<u32>()] =
        bytes[range].try_into().map_err(|_| ElfCanonicalizationError::OutOfBounds(label))?;
    Ok(u32::from_le_bytes(raw))
}

fn read_usize_from_u64(bytes: &[u8], offset: usize, label: &'static str) -> Result<usize, ElfCanonicalizationError> {
    let range = checked_range(bytes.len(), offset, size_of::<u64>(), label)?;
    let raw: [u8; size_of::<u64>()] =
        bytes[range].try_into().map_err(|_| ElfCanonicalizationError::OutOfBounds(label))?;
    usize::try_from(u64::from_le_bytes(raw)).map_err(|_| ElfCanonicalizationError::OutOfBounds(label))
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32, label: &'static str) -> Result<(), ElfCanonicalizationError> {
    let range = checked_range(bytes.len(), offset, size_of::<u32>(), label)?;
    bytes[range].copy_from_slice(&value.to_le_bytes());
    Ok(())
}

fn write_u64(bytes: &mut [u8], offset: usize, value: u64, label: &'static str) -> Result<(), ElfCanonicalizationError> {
    let range = checked_range(bytes.len(), offset, size_of::<u64>(), label)?;
    bytes[range].copy_from_slice(&value.to_le_bytes());
    Ok(())
}

fn checked_range(
    total_bytes: usize,
    offset: usize,
    length: usize,
    label: &'static str,
) -> Result<std::ops::Range<usize>, ElfCanonicalizationError> {
    let end = offset.checked_add(length).ok_or(ElfCanonicalizationError::OutOfBounds(label))?;
    if end > total_bytes {
        return Err(ElfCanonicalizationError::OutOfBounds(label));
    }
    Ok(offset..end)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SECTION_COUNT: u16 = 3;
    const TEST_SECTION_COUNT_USIZE: usize = 3;
    const TEST_SYMBOL_COUNT: usize = 2;
    const TEST_STRING_TABLE_SECTION_INDEX: u32 = 2;
    const TEST_SYMBOL_BIND_GLOBAL: u8 = 1;
    const TEST_SECTION_TABLE_OFFSET: usize = ELF_HEADER_BYTES;
    const TEST_SYMBOL_TABLE_OFFSET: usize =
        TEST_SECTION_TABLE_OFFSET + TEST_SECTION_COUNT_USIZE * ELF_SECTION_HEADER_BYTES;
    const TEST_STRING_TABLE_OFFSET: usize = TEST_SYMBOL_TABLE_OFFSET + TEST_SYMBOL_COUNT * ELF_SYMBOL_BYTES;
    const TEST_SHORT_NAME: &[u8] = b"L.1";
    const TEST_LONG_NAME: &[u8] = b"L.521018500";
    const EXTERNAL_VARIANT_A_ENV: &str = "MANTLE_ELF_LOCAL_SYMBOL_VARIANT_A";
    const EXTERNAL_VARIANT_B_ENV: &str = "MANTLE_ELF_LOCAL_SYMBOL_VARIANT_B";

    #[test]
    fn canonicalizes_different_decimal_widths_to_one_idempotent_elf() {
        let short = canonicalize_local_elf_symbol_names(&synthetic_elf(TEST_SHORT_NAME)).unwrap();
        let long = canonicalize_local_elf_symbol_names(&synthetic_elf(TEST_LONG_NAME)).unwrap();
        let second = canonicalize_local_elf_symbol_names(&short.bytes).unwrap();

        assert_eq!(short.rewrite_count, 1);
        assert_eq!(long.rewrite_count, 1);
        assert_eq!(second.rewrite_count, 1);
        assert_eq!(short.bytes, long.bytes);
        assert_eq!(short.bytes, second.bytes);
        assert!(short.bytes.windows(b"L.0000000001".len()).any(|window| window == b"L.0000000001"));
    }

    #[test]
    #[ignore = "requires two retained ELF objects whose decimal local-symbol widths differ"]
    fn retained_decimal_width_variants_converge() {
        let variant_a = std::fs::read(std::env::var_os(EXTERNAL_VARIANT_A_ENV).unwrap()).unwrap();
        let variant_b = std::fs::read(std::env::var_os(EXTERNAL_VARIANT_B_ENV).unwrap()).unwrap();

        let canonical_a = canonicalize_local_elf_symbol_names(&variant_a).unwrap();
        let canonical_b = canonicalize_local_elf_symbol_names(&variant_b).unwrap();
        let digest = blake3::hash(&canonical_a.bytes).to_hex().to_string();

        assert_ne!(variant_a, variant_b);
        assert_eq!(canonical_a.bytes, canonical_b.bytes);
        assert!(canonical_a.rewrite_count > 0);
        assert!(canonical_b.rewrite_count > 0);
        println!("canonical-elf-blake3={digest}");
    }

    #[test]
    fn rejects_malformed_elf_and_preserves_names_outside_the_exact_match() {
        let malformed_error = canonicalize_local_elf_symbol_names(&[0u8; ELF_HEADER_BYTES]).unwrap_err();
        let mut non_decimal = synthetic_elf(TEST_LONG_NAME);
        let name_start = TEST_STRING_TABLE_OFFSET + 1;
        non_decimal[name_start..name_start + LOCAL_SYMBOL_PREFIX.len()].copy_from_slice(b"X.");
        let non_decimal_result = canonicalize_local_elf_symbol_names(&non_decimal).unwrap();

        let mut non_local = synthetic_elf(TEST_LONG_NAME);
        let local_symbol = TEST_SYMBOL_TABLE_OFFSET + ELF_SYMBOL_BYTES;
        non_local[local_symbol + SYMBOL_INFO_OFFSET] = TEST_SYMBOL_BIND_GLOBAL << SYMBOL_BIND_SHIFT;
        let non_local_result = canonicalize_local_elf_symbol_names(&non_local).unwrap();

        assert!(malformed_error.to_string().contains("magic"));
        assert_eq!(non_decimal_result.rewrite_count, 0);
        assert_eq!(non_decimal_result.bytes, non_decimal);
        assert_eq!(non_local_result.rewrite_count, 0);
        assert_eq!(non_local_result.bytes, non_local);
    }

    #[test]
    fn rejects_out_of_bounds_symbol_table_without_an_output() {
        let mut elf = synthetic_elf(TEST_LONG_NAME);
        let symbols_section = TEST_SECTION_TABLE_OFFSET + ELF_SECTION_HEADER_BYTES;
        let invalid_offset = u64::try_from(elf.len() + ELF_SYMBOL_BYTES).unwrap();
        write_test_u64(&mut elf, symbols_section + SECTION_FILE_OFFSET_OFFSET, invalid_offset);

        let error = canonicalize_local_elf_symbol_names(&elf).unwrap_err();

        assert!(matches!(error, ElfCanonicalizationError::OutOfBounds("section payload")));
        assert!(elf.windows(TEST_LONG_NAME.len()).any(|window| window == TEST_LONG_NAME));
    }

    fn synthetic_elf(local_name: &[u8]) -> Vec<u8> {
        let string_table = [b"\0".as_slice(), local_name, b"\0global\0".as_slice()].concat();
        let file_bytes = TEST_STRING_TABLE_OFFSET + string_table.len();
        let mut bytes = vec![0u8; file_bytes];
        bytes[..ELF_MAGIC.len()].copy_from_slice(ELF_MAGIC);
        bytes[ELF_IDENT_CLASS_OFFSET] = ELF_CLASS_64;
        bytes[ELF_IDENT_DATA_OFFSET] = ELF_DATA_LITTLE_ENDIAN;
        write_test_u16(&mut bytes, ELF_TYPE_OFFSET, ELF_TYPE_RELOCATABLE);
        write_test_u64(&mut bytes, ELF_SECTION_TABLE_OFFSET, u64::try_from(TEST_SECTION_TABLE_OFFSET).unwrap());
        write_test_u16(&mut bytes, ELF_SECTION_ENTRY_SIZE_OFFSET, u16::try_from(ELF_SECTION_HEADER_BYTES).unwrap());
        write_test_u16(&mut bytes, ELF_SECTION_COUNT_OFFSET, TEST_SECTION_COUNT);

        let symbols_section = TEST_SECTION_TABLE_OFFSET + ELF_SECTION_HEADER_BYTES;
        write_test_u32(&mut bytes, symbols_section + SECTION_TYPE_OFFSET, SECTION_TYPE_SYMBOL_TABLE);
        write_test_u64(
            &mut bytes,
            symbols_section + SECTION_FILE_OFFSET_OFFSET,
            u64::try_from(TEST_SYMBOL_TABLE_OFFSET).unwrap(),
        );
        write_test_u64(
            &mut bytes,
            symbols_section + SECTION_SIZE_OFFSET,
            u64::try_from(TEST_SYMBOL_COUNT * ELF_SYMBOL_BYTES).unwrap(),
        );
        write_test_u32(&mut bytes, symbols_section + SECTION_LINK_OFFSET, TEST_STRING_TABLE_SECTION_INDEX);
        write_test_u64(
            &mut bytes,
            symbols_section + SECTION_ENTRY_SIZE_OFFSET,
            u64::try_from(ELF_SYMBOL_BYTES).unwrap(),
        );

        let strings_section = symbols_section + ELF_SECTION_HEADER_BYTES;
        write_test_u32(&mut bytes, strings_section + SECTION_TYPE_OFFSET, SECTION_TYPE_STRING_TABLE);
        write_test_u64(
            &mut bytes,
            strings_section + SECTION_FILE_OFFSET_OFFSET,
            u64::try_from(TEST_STRING_TABLE_OFFSET).unwrap(),
        );
        write_test_u64(&mut bytes, strings_section + SECTION_SIZE_OFFSET, u64::try_from(string_table.len()).unwrap());

        let local_symbol = TEST_SYMBOL_TABLE_OFFSET + ELF_SYMBOL_BYTES;
        write_test_u32(&mut bytes, local_symbol + SYMBOL_NAME_OFFSET, 1);
        bytes[local_symbol + SYMBOL_INFO_OFFSET] = SYMBOL_BIND_LOCAL << SYMBOL_BIND_SHIFT;
        bytes[TEST_STRING_TABLE_OFFSET..].copy_from_slice(&string_table);
        bytes
    }

    fn write_test_u16(bytes: &mut [u8], offset: usize, value: u16) {
        bytes[offset..offset + size_of::<u16>()].copy_from_slice(&value.to_le_bytes());
    }

    fn write_test_u32(bytes: &mut [u8], offset: usize, value: u32) {
        bytes[offset..offset + size_of::<u32>()].copy_from_slice(&value.to_le_bytes());
    }

    fn write_test_u64(bytes: &mut [u8], offset: usize, value: u64) {
        bytes[offset..offset + size_of::<u64>()].copy_from_slice(&value.to_le_bytes());
    }
}
