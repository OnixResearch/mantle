//! Bounded ELF inspection; no host tools or filesystem authority.
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::native::PAGE_BYTES;

/// Largest ELF image the reader accepts, in bytes (512 MiB).
pub const MAX_ELF_BYTES: usize = 536_870_912;
const MAX_SECTIONS: u64 = 2048;
const MAX_SYMBOLS: usize = 262_144;
const MAX_DEPENDENCIES: usize = 256;
const SECTION_HEADER_BYTES: u64 = 64;
const PROGRAM_HEADER_BYTES: u64 = 56;
const NOTE_HEADER_BYTES: u64 = 12;
const DYNAMIC_ENTRY_BYTES: usize = 16;
const SYMBOL_ENTRY_BYTES: usize = 24;
const MAX_DYNAMIC_TABLE_BYTES: usize = 65536;
const MAX_NOTE_TABLE_BYTES: usize = 65536;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Elf {
    pub needed: Vec<String>,
    pub soname: Option<String>,
    pub android_api: Option<u32>,
    pub exports: BTreeSet<String>,
    pub imports: BTreeSet<String>,
}

/// A byte range inside the inspected ELF image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ByteRange {
    offset_bytes: u64,
    size_bytes: u64,
}

impl ByteRange {
    const fn new(offset_bytes: u64, size_bytes: u64) -> Self {
        Self {
            offset_bytes,
            size_bytes,
        }
    }
}

/// The section headers the reader needs after scanning the section table.
struct SectionHeaders<'a> {
    dynamic: Option<&'a [u8]>,
    symbols: Option<&'a [u8]>,
}

fn bytes(data: &[u8], range: ByteRange) -> Result<&[u8], &'static str> {
    let end_bytes = range.offset_bytes.checked_add(range.size_bytes).ok_or("ELF range overflow")?;
    let start = usize::try_from(range.offset_bytes).map_err(|_| "ELF offset overflow")?;
    let end = usize::try_from(end_bytes).map_err(|_| "ELF offset overflow")?;
    data.get(start..end).ok_or("truncated ELF")
}

/// Reads a little-endian integer field of the requested width.
fn n(data: &[u8], range: ByteRange) -> Result<u64, &'static str> {
    let value = bytes(data, range)?;
    let mut word = [0; 8];
    let width = value.len();
    if width > word.len() {
        return Err("ELF integer field is too wide");
    }
    word[..width].copy_from_slice(value);
    Ok(u64::from_le_bytes(word))
}

/// Reads a little-endian 32-bit field.
fn word32(data: &[u8], range: ByteRange) -> Result<u32, &'static str> {
    let value = bytes(data, range)?;
    let mut word = [0; 4];
    let width = value.len();
    if width != word.len() {
        return Err("ELF field is not a 32-bit word");
    }
    word.copy_from_slice(value);
    Ok(u32::from_le_bytes(word))
}

/// Reads a NUL-terminated string starting at `offset_bytes`.
fn text(data: &[u8], offset_bytes: u64) -> Result<&str, &'static str> {
    let start = usize::try_from(offset_bytes).map_err(|_| "ELF string overflow")?;
    let rest = data.get(start..).ok_or("ELF string outside table")?;
    let end = rest.iter().take(1025).position(|byte| *byte == 0).ok_or("unterminated or oversized ELF string")?;
    core::str::from_utf8(&rest[..end]).map_err(|_| "non-UTF8 ELF string")
}

/// Returns the section header at `index`.
fn section_header(data: &[u8], index: u64) -> Result<&[u8], &'static str> {
    let count = n(data, ByteRange::new(60, 2))?;
    if index >= count {
        return Err("ELF section index outside table");
    }
    let stride = index.checked_mul(SECTION_HEADER_BYTES).ok_or("ELF section overflow")?;
    let offset_bytes = n(data, ByteRange::new(40, 8))?.checked_add(stride).ok_or("ELF section overflow")?;
    bytes(data, ByteRange::new(offset_bytes, SECTION_HEADER_BYTES))
}

/// Returns the program header at `index`.
fn program_header(data: &[u8], index: u64) -> Result<&[u8], &'static str> {
    let stride = index.checked_mul(PROGRAM_HEADER_BYTES).ok_or("ELF header overflow")?;
    let offset_bytes = n(data, ByteRange::new(32, 8))?.checked_add(stride).ok_or("ELF header overflow")?;
    bytes(data, ByteRange::new(offset_bytes, PROGRAM_HEADER_BYTES))
}

/// Returns the bytes a section header describes.
fn section_data<'a>(data: &'a [u8], header: &[u8]) -> Result<&'a [u8], &'static str> {
    let offset_bytes = n(header, ByteRange::new(24, 8))?;
    let size_bytes = n(header, ByteRange::new(32, 8))?;
    bytes(data, ByteRange::new(offset_bytes, size_bytes))
}

/// Confirms the identification bytes describe a little-endian AArch64 ELF64.
fn has_expected_ident(data: &[u8]) -> Result<bool, &'static str> {
    if bytes(data, ByteRange::new(0, 7))? != b"\x7fELF\x02\x01\x01" {
        return Ok(false);
    }
    if n(data, ByteRange::new(16, 2))? != 3 {
        return Ok(false);
    }
    if n(data, ByteRange::new(18, 2))? != 183 {
        return Ok(false);
    }
    if n(data, ByteRange::new(20, 4))? != 1 {
        return Ok(false);
    }
    Ok(true)
}

/// Confirms the header declares the fixed 64-bit AArch64 layout.
fn has_expected_header_layout(data: &[u8]) -> Result<bool, &'static str> {
    if n(data, ByteRange::new(52, 2))? != 64 {
        return Ok(false);
    }
    if n(data, ByteRange::new(54, 2))? != 56 {
        return Ok(false);
    }
    if n(data, ByteRange::new(58, 2))? != 64 {
        return Ok(false);
    }
    Ok(true)
}

/// A load segment's placement, in the terms the loader checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LoadSegment {
    offset_bytes: u64,
    address: u64,
    alignment: u64,
}

/// Confirms a load segment honours its declared power-of-two alignment.
fn has_valid_alignment(segment: LoadSegment) -> Result<bool, &'static str> {
    if !segment.alignment.is_power_of_two() {
        return Ok(false);
    }
    // A power-of-two alignment is at least one, so the mask is exact.
    let mask = segment.alignment.checked_sub(1).ok_or("invalid ELF load alignment")?;
    Ok((segment.offset_bytes & mask) == (segment.address & mask))
}

/// Confirms a load segment honours the Android 16 KiB page alignment.
fn has_page_alignment(segment: LoadSegment) -> bool {
    let is_page_aligned = segment.alignment >= PAGE_BYTES;
    let is_same_page = (segment.offset_bytes % PAGE_BYTES) == (segment.address % PAGE_BYTES);
    is_page_aligned && is_same_page
}

pub fn inspect(data: &[u8], require_16k: bool) -> Result<Elf, &'static str> {
    if data.len() > MAX_ELF_BYTES {
        return Err("expected AArch64 little-endian ELF64 shared object");
    }
    if !has_expected_ident(data)? {
        return Err("expected AArch64 little-endian ELF64 shared object");
    }
    if !has_expected_header_layout(data)? {
        return Err("unsupported ELF header layout");
    }
    let dynamic_segment = inspect_segments(data, require_16k)?;
    let mut result = Elf {
        needed: Vec::new(),
        soname: None,
        android_api: None,
        exports: BTreeSet::new(),
        imports: BTreeSet::new(),
    };
    let headers = locate_sections(data, &mut result)?;
    let dynamic_header = headers.dynamic.ok_or("missing ELF dynamic section")?;
    if dynamic_segment != (n(dynamic_header, ByteRange::new(24, 8))?, n(dynamic_header, ByteRange::new(32, 8))?) {
        return Err("ELF dynamic section differs from loader segment");
    }
    let symbols_header = headers.symbols.ok_or("missing ELF dynamic symbols")?;
    if n(dynamic_header, ByteRange::new(40, 4))? != n(symbols_header, ByteRange::new(40, 4))? {
        return Err("ELF dynamic strings differ from symbol strings");
    }
    dynamic(data, dynamic_header, &mut result)?;
    symbols(data, symbols_header, &mut result)?;
    check_tables(data, dynamic_header, symbols_header)?;
    debug_assert!(result.needed.len() <= MAX_DEPENDENCIES, "the dependency count stayed bounded");
    debug_assert!(!result.soname.as_deref().is_some_and(str::is_empty), "the SONAME is never empty");
    Ok(result)
}

/// Inspects the program headers and returns the dynamic segment's file range.
fn inspect_segments(data: &[u8], require_16k: bool) -> Result<(u64, u64), &'static str> {
    let count = n(data, ByteRange::new(56, 2))?;
    if count == 0 || count > 128 {
        return Err("invalid ELF program header count");
    }
    let mut loads: u64 = 0;
    let mut dynamic_segment = None;
    for index in 0..count {
        let header = program_header(data, index)?;
        let kind = n(header, ByteRange::new(0, 4))?;
        if kind == 3 {
            return Err("Android library must not request an ELF interpreter");
        }
        if kind == 2
            && dynamic_segment
                .replace((n(header, ByteRange::new(8, 8))?, n(header, ByteRange::new(32, 8))?))
                .is_some()
        {
            return Err("duplicate ELF dynamic segment");
        }
        if kind != 1 {
            continue;
        }
        loads += 1;
        let offset_bytes = n(header, ByteRange::new(8, 8))?;
        let address = n(header, ByteRange::new(16, 8))?;
        let file_size_bytes = n(header, ByteRange::new(32, 8))?;
        if file_size_bytes > n(header, ByteRange::new(40, 8))? {
            return Err("ELF load size exceeds memory size");
        }
        bytes(data, ByteRange::new(offset_bytes, file_size_bytes))?;
        let segment = LoadSegment {
            offset_bytes,
            address,
            alignment: n(header, ByteRange::new(48, 8))?,
        };
        if !has_valid_alignment(segment)? {
            return Err("invalid ELF load alignment");
        }
        if require_16k && !has_page_alignment(segment) {
            return Err("Android ELF load segment is not 16KiB aligned");
        }
    }
    if loads == 0 {
        return Err("ELF has no load segments");
    }
    debug_assert!(loads > 0, "a load segment was counted");
    debug_assert!(count > 0, "the program header count was validated");
    dynamic_segment.ok_or("ELF has no dynamic segment")
}

/// Locates the dynamic and symbol sections and reads any Android API notes.
fn locate_sections<'a>(data: &'a [u8], result: &mut Elf) -> Result<SectionHeaders<'a>, &'static str> {
    let count = n(data, ByteRange::new(60, 2))?;
    if count == 0 || count > MAX_SECTIONS {
        return Err("missing or excessive ELF sections");
    }
    let mut dynamic = None;
    let mut symbols = None;
    for index in 0..count {
        let header = section_header(data, index)?;
        match n(header, ByteRange::new(4, 4))? {
            6 if dynamic.replace(header).is_some() => return Err("duplicate ELF dynamic section"),
            7 => android_notes(data, header, result)?,
            11 if symbols.replace(header).is_some() => return Err("duplicate ELF symbol section"),
            _ => {}
        }
    }
    debug_assert!(count <= MAX_SECTIONS, "the section count stayed bounded");
    Ok(SectionHeaders { dynamic, symbols })
}

/// Confirms the dynamic table agrees with the symbol sections it names.
fn check_tables(data: &[u8], dynamic_header: &[u8], symbols_header: &[u8]) -> Result<(), &'static str> {
    let strings_header = section_header(data, n(symbols_header, ByteRange::new(40, 4))?)?;
    let table = section_data(data, dynamic_header)?;
    debug_assert!(table.len() % DYNAMIC_ENTRY_BYTES == 0, "the dynamic table has whole entries");
    for header in [dynamic_header, symbols_header, strings_header] {
        mapped_section(data, header)?;
    }
    for (tag, expected) in [
        (5, n(strings_header, ByteRange::new(16, 8))?),
        (6, n(symbols_header, ByteRange::new(16, 8))?),
        (10, n(strings_header, ByteRange::new(32, 8))?),
        (11, 24),
    ] {
        let mut values = table
            .chunks_exact(DYNAMIC_ENTRY_BYTES)
            .take_while(|entry| n(entry, ByteRange::new(0, 8)) != Ok(0))
            .filter(|entry| n(entry, ByteRange::new(0, 8)) == Ok(tag));
        if values.next().map(|entry| n(entry, ByteRange::new(8, 8))) != Some(Ok(expected)) || values.next().is_some() {
            return Err("ELF dynamic address/size differs from symbol sections");
        }
    }
    Ok(())
}

/// Confirms a section is mapped by a load segment at its declared address.
fn mapped_section(data: &[u8], section: &[u8]) -> Result<(), &'static str> {
    let offset_bytes = n(section, ByteRange::new(24, 8))?;
    let size_bytes = n(section, ByteRange::new(32, 8))?;
    let end_bytes = offset_bytes.checked_add(size_bytes).ok_or("ELF section range overflow")?;
    let segment_count = n(data, ByteRange::new(56, 2))?;
    for index in 0..segment_count {
        let header = program_header(data, index)?;
        if n(header, ByteRange::new(0, 4))? != 1 {
            continue;
        }
        let start_bytes = n(header, ByteRange::new(8, 8))?;
        let limit_bytes =
            start_bytes.checked_add(n(header, ByteRange::new(32, 8))?).ok_or("ELF load range overflow")?;
        if offset_bytes < start_bytes || end_bytes > limit_bytes {
            continue;
        }
        let delta_bytes = offset_bytes.checked_sub(start_bytes).ok_or("ELF section range overflow")?;
        if n(header, ByteRange::new(16, 8))?.checked_add(delta_bytes) == Some(n(section, ByteRange::new(16, 8))?) {
            return Ok(());
        }
    }
    debug_assert!(end_bytes >= offset_bytes, "the section range did not wrap");
    Err("ELF dynamic table is not mapped at its declared address")
}

/// Reads the dynamic table, collecting dependencies and the SONAME.
fn dynamic(data: &[u8], header: &[u8], result: &mut Elf) -> Result<(), &'static str> {
    let table = section_data(data, header)?;
    let strings = section_data(data, section_header(data, n(header, ByteRange::new(40, 4))?)?)?;
    if n(header, ByteRange::new(56, 8))? != 16
        || table.len() % DYNAMIC_ENTRY_BYTES != 0
        || table.len() > MAX_DYNAMIC_TABLE_BYTES
    {
        return Err("invalid ELF dynamic table");
    }
    debug_assert!(table.len() % DYNAMIC_ENTRY_BYTES == 0, "the dynamic table has whole entries");
    for entry in table.chunks_exact(DYNAMIC_ENTRY_BYTES) {
        let tag = n(entry, ByteRange::new(0, 8))?;
        if tag == 0 {
            return Ok(());
        }
        if matches!(tag, 15 | 22 | 29) || (tag == 30 && n(entry, ByteRange::new(8, 8))? & 4 != 0) {
            return Err("ELF RPATH/RUNPATH/text relocation is forbidden");
        }
        if tag != 1 && tag != 14 {
            continue;
        }
        let name = text(strings, n(entry, ByteRange::new(8, 8))?)?;
        let is_empty = name.is_empty();
        let has_separator = name.contains('/') || name.contains('\\');
        let is_relative = name == "." || name == "..";
        if is_empty || has_separator || is_relative {
            return Err("ELF dependency is not a library basename");
        }
        if tag == 1 {
            if result.needed.len() >= MAX_DEPENDENCIES {
                return Err("too many ELF dependencies");
            }
            result.needed.push(name.to_string());
        } else if result.soname.replace(name.to_string()).is_some() {
            return Err("duplicate ELF SONAME");
        }
    }
    Err("unterminated ELF dynamic table")
}

/// Reads the dynamic symbol table, collecting exports and imports.
fn symbols(data: &[u8], header: &[u8], result: &mut Elf) -> Result<(), &'static str> {
    let table = section_data(data, header)?;
    let strings = section_data(data, section_header(data, n(header, ByteRange::new(40, 4))?)?)?;
    if n(header, ByteRange::new(56, 8))? != 24
        || table.len() % SYMBOL_ENTRY_BYTES != 0
        || table.len() / SYMBOL_ENTRY_BYTES > MAX_SYMBOLS
    {
        return Err("invalid ELF dynamic symbol table");
    }
    debug_assert!(table.len() % SYMBOL_ENTRY_BYTES == 0, "the symbol table has whole entries");
    for entry in table.chunks_exact(SYMBOL_ENTRY_BYTES) {
        let binding = entry[4] >> 4;
        let is_defined = n(entry, ByteRange::new(6, 2))? != 0;
        let name = text(strings, n(entry, ByteRange::new(0, 4))?)?;
        if name.is_empty() {
            continue;
        }
        if is_defined && matches!(binding, 1 | 2) && matches!(entry[5] & 3, 0 | 3) {
            result.exports.insert(name.to_string());
        } else if !is_defined && binding == 1 {
            result.imports.insert(name.to_string());
        }
    }
    Ok(())
}

/// Reads the note table, recording the Android API level note.
fn android_notes(data: &[u8], header: &[u8], result: &mut Elf) -> Result<(), &'static str> {
    let table = section_data(data, header)?;
    if table.len() > MAX_NOTE_TABLE_BYTES {
        return Err("oversized ELF note table");
    }
    debug_assert!(table.len() <= MAX_NOTE_TABLE_BYTES, "the note table stayed bounded");
    let mut cursor_bytes: u64 = 0;
    while cursor_bytes < table.len() as u64 {
        let note_header = bytes(table, ByteRange::new(cursor_bytes, NOTE_HEADER_BYTES))?;
        let name_size_bytes = n(note_header, ByteRange::new(0, 4))?;
        let value_size_bytes = n(note_header, ByteRange::new(4, 4))?;
        let kind = n(note_header, ByteRange::new(8, 4))?;
        cursor_bytes += NOTE_HEADER_BYTES;
        let name = bytes(table, ByteRange::new(cursor_bytes, name_size_bytes))?;
        cursor_bytes += (name_size_bytes.checked_add(3).ok_or("truncated ELF note padding")?) & !3;
        let value = bytes(table, ByteRange::new(cursor_bytes, value_size_bytes))?;
        cursor_bytes += (value_size_bytes.checked_add(3).ok_or("truncated ELF note padding")?) & !3;
        if kind == 1 && name == b"Android\0" {
            mapped_section(data, header)?;
            let api = word32(value, ByteRange::new(0, 4))?;
            if result.android_api.replace(api).is_some() {
                return Err("duplicate Android API note");
            }
        }
    }
    if cursor_bytes != table.len() as u64 {
        return Err("truncated ELF note padding");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    include!("elf_tests.rs");
}
