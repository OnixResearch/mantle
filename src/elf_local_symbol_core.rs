const ELF_HEADER_BYTES: usize = 64;
const ELF_SECTION_HEADER_BYTES: usize = 64;
const ELF_SYMBOL_BYTES: usize = 24;
const ELF_IDENT_CLASS_OFFSET: usize = 4;
const ELF_IDENT_DATA_OFFSET: usize = 5;
const ELF_SECTION_TABLE_OFFSET: usize = 40;
const ELF_SECTION_ENTRY_SIZE_OFFSET: usize = 58;
const ELF_SECTION_COUNT_OFFSET: usize = 60;
const SECTION_TYPE_OFFSET: usize = 4;
const SECTION_FILE_OFFSET_OFFSET: usize = 24;
const SECTION_SIZE_OFFSET: usize = 32;
const SECTION_LINK_OFFSET: usize = 40;
const SECTION_ENTRY_SIZE_OFFSET: usize = 56;
const SYMBOL_NAME_OFFSET: usize = 0;
const SYMBOL_INFO_OFFSET: usize = 4;
const ELF_CLASS_64: u8 = 2;
const ELF_DATA_LITTLE_ENDIAN: u8 = 1;
const SECTION_TYPE_SYMBOL_TABLE: u32 = 2;
const SECTION_TYPE_STRING_TABLE: u32 = 3;
const SYMBOL_BIND_SHIFT: u8 = 4;
const SYMBOL_BIND_LOCAL: u8 = 0;
const SECTION_COUNT_MAX: usize = 4_096;
const SYMBOL_COUNT_MAX: usize = 1_048_576;
const LOCAL_SYMBOL_PREFIX: &[u8] = b"L.";
const DECIMAL_RADIX: usize = 10;
const ELF_MAGIC: &[u8] = b"\x7fELF";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ElfCanonicalizationError {
    UnsupportedFormat(&'static str),
    OutOfBounds(&'static str),
    LimitExceeded(&'static str),
    SymbolIndexTooWide,
}

impl std::fmt::Display for ElfCanonicalizationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedFormat(message) => write!(formatter, "unsupported ELF format: {message}"),
            Self::OutOfBounds(message) => write!(formatter, "ELF range is out of bounds: {message}"),
            Self::LimitExceeded(message) => write!(formatter, "ELF limit exceeded: {message}"),
            Self::SymbolIndexTooWide => {
                write!(formatter, "ELF local-symbol index does not fit the existing name width")
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct SectionHeader {
    section_type: u32,
    file_offset: usize,
    size_bytes: usize,
    linked_section_index: usize,
    entry_size_bytes: usize,
}

pub(crate) fn canonicalize_local_elf_symbol_names(bytes: &mut [u8]) -> Result<u32, ElfCanonicalizationError> {
    validate_header(bytes)?;
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

    let mut rewrite_count = 0u32;
    for section_index in 0..section_count {
        let symbols = read_section(bytes, section_table_offset, section_index)?;
        if symbols.section_type != SECTION_TYPE_SYMBOL_TABLE {
            continue;
        }
        let rewritten = canonicalize_symbol_table(bytes, section_table_offset, section_count, symbols)?;
        rewrite_count = rewrite_count
            .checked_add(rewritten)
            .ok_or(ElfCanonicalizationError::LimitExceeded("rewrite count"))?;
    }
    assert!(section_count <= SECTION_COUNT_MAX);
    debug_assert_eq!(bytes.get(..ELF_MAGIC.len()), Some(ELF_MAGIC));
    Ok(rewrite_count)
}

fn validate_header(bytes: &[u8]) -> Result<(), ElfCanonicalizationError> {
    if bytes.len() < ELF_HEADER_BYTES {
        return Err(ElfCanonicalizationError::UnsupportedFormat("file is smaller than an ELF64 header"));
    }
    if bytes.get(..ELF_MAGIC.len()) != Some(ELF_MAGIC) {
        return Err(ElfCanonicalizationError::UnsupportedFormat("ELF magic mismatch"));
    }
    if bytes[ELF_IDENT_CLASS_OFFSET] != ELF_CLASS_64 || bytes[ELF_IDENT_DATA_OFFSET] != ELF_DATA_LITTLE_ENDIAN {
        return Err(ElfCanonicalizationError::UnsupportedFormat("ELF must be 64-bit little-endian"));
    }
    assert!(ELF_HEADER_BYTES <= bytes.len());
    debug_assert_eq!(ELF_MAGIC.len(), ELF_IDENT_CLASS_OFFSET);
    Ok(())
}

fn canonicalize_symbol_table(
    bytes: &mut [u8],
    section_table_offset: usize,
    section_count: usize,
    symbols: SectionHeader,
) -> Result<u32, ElfCanonicalizationError> {
    if symbols.linked_section_index >= section_count {
        return Err(ElfCanonicalizationError::OutOfBounds("linked string-table section"));
    }
    if symbols.entry_size_bytes != ELF_SYMBOL_BYTES || !symbols.size_bytes.is_multiple_of(ELF_SYMBOL_BYTES) {
        return Err(ElfCanonicalizationError::UnsupportedFormat("invalid ELF64 symbol-table entry size"));
    }
    let strings = read_section(bytes, section_table_offset, symbols.linked_section_index)?;
    if strings.section_type != SECTION_TYPE_STRING_TABLE {
        return Err(ElfCanonicalizationError::UnsupportedFormat("symbol table does not link to a string table"));
    }
    checked_range(bytes.len(), symbols.file_offset, symbols.size_bytes, "symbol table")?;
    checked_range(bytes.len(), strings.file_offset, strings.size_bytes, "string table")?;
    let symbol_count = symbols.size_bytes / ELF_SYMBOL_BYTES;
    if symbol_count > SYMBOL_COUNT_MAX {
        return Err(ElfCanonicalizationError::LimitExceeded("symbol count"));
    }

    let mut rewrite_count = 0u32;
    for symbol_index in 0..symbol_count {
        let symbol_relative_offset = symbol_index
            .checked_mul(ELF_SYMBOL_BYTES)
            .ok_or(ElfCanonicalizationError::OutOfBounds("symbol offset"))?;
        let symbol_offset = symbols
            .file_offset
            .checked_add(symbol_relative_offset)
            .ok_or(ElfCanonicalizationError::OutOfBounds("symbol offset"))?;
        let name_offset = usize::try_from(read_u32(bytes, symbol_offset + SYMBOL_NAME_OFFSET, "symbol name")?)
            .map_err(|_| ElfCanonicalizationError::OutOfBounds("symbol name"))?;
        let binding = bytes[symbol_offset + SYMBOL_INFO_OFFSET] >> SYMBOL_BIND_SHIFT;
        if binding != SYMBOL_BIND_LOCAL || name_offset >= strings.size_bytes {
            continue;
        }
        let name_start = strings
            .file_offset
            .checked_add(name_offset)
            .ok_or(ElfCanonicalizationError::OutOfBounds("symbol name"))?;
        let name_capacity = strings.size_bytes - name_offset;
        let name_length = bounded_string_length(bytes, name_start, name_capacity)?;
        let name_end =
            name_start.checked_add(name_length).ok_or(ElfCanonicalizationError::OutOfBounds("symbol name"))?;
        if !is_decimal_local_symbol(&bytes[name_start..name_end]) {
            continue;
        }
        write_decimal_symbol_index(&mut bytes[name_start..name_end], symbol_index)?;
        rewrite_count = rewrite_count.checked_add(1).ok_or(ElfCanonicalizationError::LimitExceeded("rewrite count"))?;
    }
    assert!(symbol_count <= SYMBOL_COUNT_MAX);
    debug_assert!(symbols.size_bytes.is_multiple_of(ELF_SYMBOL_BYTES));
    Ok(rewrite_count)
}

fn read_section(
    bytes: &[u8],
    section_table_offset: usize,
    section_index: usize,
) -> Result<SectionHeader, ElfCanonicalizationError> {
    if section_index > SECTION_COUNT_MAX {
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
        entry_size_bytes: read_usize_from_u64(bytes, offset + SECTION_ENTRY_SIZE_OFFSET, "section entry size")?,
    };
    assert!(section_index <= SECTION_COUNT_MAX);
    debug_assert!(offset >= section_table_offset);
    Ok(section)
}

fn bounded_string_length(bytes: &[u8], start: usize, capacity: usize) -> Result<usize, ElfCanonicalizationError> {
    checked_range(bytes.len(), start, capacity, "string-table entry")?;
    bytes[start..start + capacity]
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

fn write_decimal_symbol_index(name: &mut [u8], symbol_index: usize) -> Result<(), ElfCanonicalizationError> {
    let digit_count = name.len() - LOCAL_SYMBOL_PREFIX.len();
    let mut cursor = name.len();
    let mut value = symbol_index;
    name[LOCAL_SYMBOL_PREFIX.len()..].fill(b'0');
    loop {
        if cursor <= LOCAL_SYMBOL_PREFIX.len() {
            return Err(ElfCanonicalizationError::SymbolIndexTooWide);
        }
        cursor -= 1;
        let digit = u8::try_from(value % DECIMAL_RADIX).map_err(|_| ElfCanonicalizationError::SymbolIndexTooWide)?;
        name[cursor] = b'0' + digit;
        value /= DECIMAL_RADIX;
        if value == 0 {
            break;
        }
    }
    assert!(digit_count > 0);
    debug_assert!(name.starts_with(LOCAL_SYMBOL_PREFIX));
    Ok(())
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
    const TEST_NAME: &[u8] = b"\0L.521018500\0global\0";
    const TEST_FILE_BYTES: usize = TEST_STRING_TABLE_OFFSET + TEST_NAME.len();

    #[test]
    fn canonicalizes_decimal_local_symbol_and_is_idempotent() {
        let mut elf = synthetic_elf();

        let first = canonicalize_local_elf_symbol_names(&mut elf).unwrap();
        let first_bytes = elf.clone();
        let second = canonicalize_local_elf_symbol_names(&mut elf).unwrap();

        assert_eq!(first, 1);
        assert_eq!(second, 1);
        assert_eq!(elf, first_bytes);
        assert!(elf.windows(b"L.000000001".len()).any(|window| window == b"L.000000001"));
    }

    #[test]
    fn rejects_malformed_elf_and_preserves_names_outside_the_exact_match() {
        let malformed_error = canonicalize_local_elf_symbol_names(&mut [0u8; ELF_HEADER_BYTES]).unwrap_err();
        let mut non_decimal = synthetic_elf();
        let name_start = TEST_STRING_TABLE_OFFSET + 1;
        non_decimal[name_start..name_start + LOCAL_SYMBOL_PREFIX.len()].copy_from_slice(b"X.");
        let non_decimal_unchanged = non_decimal.clone();
        let non_decimal_count = canonicalize_local_elf_symbol_names(&mut non_decimal).unwrap();

        let mut non_local = synthetic_elf();
        let local_symbol = TEST_SYMBOL_TABLE_OFFSET + ELF_SYMBOL_BYTES;
        non_local[local_symbol + SYMBOL_INFO_OFFSET] = TEST_SYMBOL_BIND_GLOBAL << SYMBOL_BIND_SHIFT;
        let non_local_unchanged = non_local.clone();
        let non_local_count = canonicalize_local_elf_symbol_names(&mut non_local).unwrap();

        assert!(malformed_error.to_string().contains("magic"));
        assert_eq!(non_decimal_count, 0);
        assert_eq!(non_decimal, non_decimal_unchanged);
        assert_eq!(non_local_count, 0);
        assert_eq!(non_local, non_local_unchanged);
    }

    #[test]
    fn rejects_out_of_bounds_symbol_table_before_rewriting() {
        let mut elf = synthetic_elf();
        let symbols_section = TEST_SECTION_TABLE_OFFSET + ELF_SECTION_HEADER_BYTES;
        let invalid_offset = u64::try_from(TEST_FILE_BYTES + ELF_SYMBOL_BYTES).unwrap();
        write_u64(&mut elf, symbols_section + SECTION_FILE_OFFSET_OFFSET, invalid_offset);
        let unchanged = elf.clone();

        let error = canonicalize_local_elf_symbol_names(&mut elf).unwrap_err();

        assert!(matches!(error, ElfCanonicalizationError::OutOfBounds("symbol table")));
        assert_eq!(elf, unchanged);
    }

    fn synthetic_elf() -> Vec<u8> {
        let mut bytes = vec![0u8; TEST_FILE_BYTES];
        bytes[..ELF_MAGIC.len()].copy_from_slice(ELF_MAGIC);
        bytes[ELF_IDENT_CLASS_OFFSET] = ELF_CLASS_64;
        bytes[ELF_IDENT_DATA_OFFSET] = ELF_DATA_LITTLE_ENDIAN;
        write_u64(&mut bytes, ELF_SECTION_TABLE_OFFSET, u64::try_from(TEST_SECTION_TABLE_OFFSET).unwrap());
        write_u16(&mut bytes, ELF_SECTION_ENTRY_SIZE_OFFSET, u16::try_from(ELF_SECTION_HEADER_BYTES).unwrap());
        write_u16(&mut bytes, ELF_SECTION_COUNT_OFFSET, TEST_SECTION_COUNT);

        let symbols_section = TEST_SECTION_TABLE_OFFSET + ELF_SECTION_HEADER_BYTES;
        write_u32(&mut bytes, symbols_section + SECTION_TYPE_OFFSET, SECTION_TYPE_SYMBOL_TABLE);
        write_u64(
            &mut bytes,
            symbols_section + SECTION_FILE_OFFSET_OFFSET,
            u64::try_from(TEST_SYMBOL_TABLE_OFFSET).unwrap(),
        );
        write_u64(
            &mut bytes,
            symbols_section + SECTION_SIZE_OFFSET,
            u64::try_from(TEST_SYMBOL_COUNT * ELF_SYMBOL_BYTES).unwrap(),
        );
        write_u32(&mut bytes, symbols_section + SECTION_LINK_OFFSET, TEST_STRING_TABLE_SECTION_INDEX);
        write_u64(&mut bytes, symbols_section + SECTION_ENTRY_SIZE_OFFSET, u64::try_from(ELF_SYMBOL_BYTES).unwrap());

        let strings_section = symbols_section + ELF_SECTION_HEADER_BYTES;
        write_u32(&mut bytes, strings_section + SECTION_TYPE_OFFSET, SECTION_TYPE_STRING_TABLE);
        write_u64(
            &mut bytes,
            strings_section + SECTION_FILE_OFFSET_OFFSET,
            u64::try_from(TEST_STRING_TABLE_OFFSET).unwrap(),
        );
        write_u64(&mut bytes, strings_section + SECTION_SIZE_OFFSET, u64::try_from(TEST_NAME.len()).unwrap());

        let local_symbol = TEST_SYMBOL_TABLE_OFFSET + ELF_SYMBOL_BYTES;
        write_u32(&mut bytes, local_symbol + SYMBOL_NAME_OFFSET, 1);
        bytes[local_symbol + SYMBOL_INFO_OFFSET] = SYMBOL_BIND_LOCAL << SYMBOL_BIND_SHIFT;
        bytes[TEST_STRING_TABLE_OFFSET..].copy_from_slice(TEST_NAME);
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
