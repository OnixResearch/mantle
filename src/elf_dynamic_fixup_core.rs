// HARDENING-BACKLOG 2026-10-22: pre-existing tigerstyle findings in this file are
// recorded in .cairn/archive/2026-10-22-schedule-tigerstyle-hardening-backlog/evidence/
// tigerstyle-backlog-2026-10-22.log and scheduled for the standalone hardening
// pass. Scoped to the lint categories present at recording time.
#![allow(tigerstyle::usize_in_public_api)]

//! Pure, bounded planning of in-place ELF64 dynamic-string rewrites.
//! The caller owns file IO and publication; a plan never changes its input.

extern crate alloc;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;
use core::ops::Range;

const MAX_IMAGE_BYTES: usize = 134_217_728; // 128 MiB
const MAX_SECTIONS: usize = 4096;
const MAX_DYNAMIC_ENTRIES: usize = 4096;
const MAX_SYMBOLS: usize = 1_048_576;
const MAX_REWRITES: usize = 256;
const MAX_STRING_BYTES: usize = 8_388_608; // 8 MiB
const SECTION_BYTES: usize = 64;
const DYNAMIC_BYTES: usize = 16;
const SYMBOL_BYTES: usize = 24;
const ELF_HEADER_BYTES: usize = 64;
const PROGRAM_HEADER_BYTES: usize = 56;
const SHT_NOBITS: u32 = 8;
const SHT_DYNAMIC: u32 = 6;
const SHT_DYNSYM: u32 = 11;
const SHT_STRTAB: u32 = 3;
const DT_NEEDED: u64 = 1;
const DT_RPATH: u64 = 15;
const DT_RUNPATH: u64 = 29;
const DT_STRTAB: u64 = 5;
const DT_STRSZ: u64 = 10;
const DT_SONAME: u64 = 14;
const PT_DYNAMIC: u64 = 2;
const HASH_ALPHABET: &[u8] = b"0123456789abcdfghijklmnpqrsvwxyz";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Refusal {
    UnsupportedClass,
    UnsupportedEndianness,
    UnsupportedType,
    Truncated,
    LimitExceeded,
    InvalidLayout,
    InvalidString,
    MissingCapacity,
    SymbolTailOverlap,
    SharedDynamicString,
    DependencyMismatch,
    InvalidDependency,
}

impl core::fmt::Display for Refusal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntryKind {
    Needed,
    Runpath,
}

/// Exact existing ELF string and replacement. Padding must be part of `current`.
/// `dependency` is a full flat-store member (`<32-character hash>-<name>`).
#[derive(Clone, Debug)]
pub struct DependencyRewrite<'a> {
    pub kind: EntryKind,
    pub current: &'a str,
    pub dependency: &'a str,
    pub library_path: &'a str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StringPatch {
    pub offset: usize,
    pub original: Vec<u8>,
    pub replacement: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixupPlan {
    pub patches: Vec<StringPatch>,
}

#[derive(Clone, Copy)]
struct Section {
    kind: u32,
    offset: usize,
    address: usize,
    size: usize,
    link: usize,
    entsize: usize,
}

/// A byte window inside the image. The two bounds travel together so that a
/// call site cannot transpose them.
#[derive(Clone, Copy)]
struct ByteSpan {
    start: usize,
    size_bytes: usize,
}

impl ByteSpan {
    /// A window of `size_bytes` bytes that starts at `start`.
    const fn new(start: usize, size_bytes: usize) -> Self {
        Self { start, size_bytes }
    }
}

/// A fixed-layout ELF64 field this planner reads. The offset is relative to
/// the start of the record the field belongs to.
#[derive(Clone, Copy)]
enum Field {
    HeaderType,
    HeaderProgramOffset,
    HeaderSectionOffset,
    HeaderProgramEntrySize,
    HeaderProgramCount,
    HeaderSectionEntrySize,
    HeaderSectionCount,
    SectionType,
    SectionAddress,
    SectionOffset,
    SectionSize,
    SectionLink,
    SectionEntrySize,
    ProgramType,
    ProgramOffset,
    ProgramSize,
    DynamicTag,
    DynamicValue,
    SymbolName,
}

impl Field {
    /// The byte offset of the field inside its own record.
    const fn start(self) -> usize {
        match self {
            Self::HeaderType => 16,
            Self::HeaderProgramOffset => 32,
            Self::HeaderSectionOffset => 40,
            Self::HeaderProgramEntrySize => 54,
            Self::HeaderProgramCount => 56,
            Self::HeaderSectionEntrySize => 58,
            Self::HeaderSectionCount => 60,
            Self::SectionType => 4,
            Self::SectionAddress => 16,
            Self::SectionOffset => 24,
            Self::SectionSize => 32,
            Self::SectionLink => 40,
            Self::SectionEntrySize => 56,
            Self::ProgramType => 0,
            Self::ProgramOffset => 8,
            Self::ProgramSize => 32,
            Self::DynamicTag => 0,
            Self::DynamicValue => 8,
            Self::SymbolName => 0,
        }
    }

    /// The width of the field in bytes.
    const fn width(self) -> usize {
        match self {
            Self::HeaderType
            | Self::HeaderProgramEntrySize
            | Self::HeaderProgramCount
            | Self::HeaderSectionEntrySize
            | Self::HeaderSectionCount => 2,
            Self::SectionType | Self::SectionLink | Self::SymbolName | Self::ProgramType => 4,
            Self::HeaderProgramOffset
            | Self::HeaderSectionOffset
            | Self::SectionAddress
            | Self::SectionOffset
            | Self::SectionSize
            | Self::SectionEntrySize
            | Self::ProgramOffset
            | Self::ProgramSize
            | Self::DynamicTag
            | Self::DynamicValue => 8,
        }
    }
}

/// The dynamic section, the string table it declares, and the section table
/// both are read from.
struct DynamicLayout<'a> {
    sections: &'a [Section],
    dynamic_index: usize,
    table_index: usize,
    table: Section,
    entries: Vec<(u64, usize)>,
}

fn checked_range(bytes: &[u8], span: ByteSpan) -> Result<Range<usize>, Refusal> {
    let end = span.start.checked_add(span.size_bytes).ok_or(Refusal::Truncated)?;
    if end > bytes.len() {
        return Err(Refusal::Truncated);
    }
    Ok(span.start..end)
}

/// Reads the little-endian value of `field` at `base` inside the image.
fn num(bytes: &[u8], base: usize, field: Field) -> Result<u64, Refusal> {
    let start = base.checked_add(field.start()).ok_or(Refusal::Truncated)?;
    let range = checked_range(bytes, ByteSpan::new(start, field.width()))?;
    let mut value = 0_u64;
    for (shift, byte) in bytes[range].iter().enumerate() {
        value |= u64::from(*byte) << shift.saturating_mul(8);
    }
    Ok(value)
}

fn index(value: u64) -> Result<usize, Refusal> {
    usize::try_from(value).map_err(|_| Refusal::LimitExceeded)
}

/// Reads the six section header fields this planner needs.
fn read_section(bytes: &[u8], base: usize) -> Result<Section, Refusal> {
    let section = Section {
        kind: u32::try_from(num(bytes, base, Field::SectionType)?).map_err(|_| Refusal::LimitExceeded)?,
        address: index(num(bytes, base, Field::SectionAddress)?)?,
        offset: index(num(bytes, base, Field::SectionOffset)?)?,
        size: index(num(bytes, base, Field::SectionSize)?)?,
        link: index(num(bytes, base, Field::SectionLink)?)?,
        entsize: index(num(bytes, base, Field::SectionEntrySize)?)?,
    };
    if section.kind != SHT_NOBITS {
        checked_range(bytes, ByteSpan::new(section.offset, section.size))?;
    }
    debug_assert!(base <= bytes.len());
    debug_assert!(section.kind == SHT_NOBITS || section.offset.saturating_add(section.size) <= bytes.len());
    Ok(section)
}

fn sections(bytes: &[u8]) -> Result<Vec<Section>, Refusal> {
    let start = index(num(bytes, 0, Field::HeaderSectionOffset)?)?;
    let section_count = index(num(bytes, 0, Field::HeaderSectionCount)?)?;
    let entry_bytes = index(num(bytes, 0, Field::HeaderSectionEntrySize)?)?;
    let is_bounded_count = section_count > 0 && section_count <= MAX_SECTIONS;
    if !is_bounded_count || entry_bytes != SECTION_BYTES {
        return Err(Refusal::InvalidLayout);
    }
    let table_bytes = section_count.checked_mul(SECTION_BYTES).ok_or(Refusal::LimitExceeded)?;
    checked_range(bytes, ByteSpan::new(start, table_bytes))?;
    let mut result = Vec::with_capacity(section_count);
    for section_index in 0..section_count {
        let offset_bytes = section_index.checked_mul(SECTION_BYTES).ok_or(Refusal::LimitExceeded)?;
        let base = start.checked_add(offset_bytes).ok_or(Refusal::LimitExceeded)?;
        result.push(read_section(bytes, base)?);
    }
    debug_assert_eq!(result.len(), section_count);
    debug_assert!(start.saturating_add(table_bytes) <= bytes.len());
    Ok(result)
}

/// Rejects a string table that overlaps another section or the section table.
fn validate_table_placement(bytes: &[u8], sections: &[Section], table_index: usize) -> Result<(), Refusal> {
    let table = sections[table_index];
    let table_range = checked_range(bytes, ByteSpan::new(table.offset, table.size))?;
    let section_start = index(num(bytes, 0, Field::HeaderSectionOffset)?)?;
    let section_bytes = sections.len().checked_mul(SECTION_BYTES).ok_or(Refusal::LimitExceeded)?;
    let header_range = checked_range(bytes, ByteSpan::new(section_start, section_bytes))?;
    let is_header_overlapped = table_range.start < header_range.end && header_range.start < table_range.end;
    if is_header_overlapped || table.offset < ELF_HEADER_BYTES {
        return Err(Refusal::InvalidLayout);
    }
    for (section_index, section) in sections.iter().enumerate() {
        if section_index == table_index || section.kind == SHT_NOBITS || section.size == 0 {
            continue;
        }
        let other = checked_range(bytes, ByteSpan::new(section.offset, section.size))?;
        if table_range.start < other.end && other.start < table_range.end {
            return Err(Refusal::InvalidLayout);
        }
    }
    debug_assert!(table_range.start <= table_range.end);
    debug_assert_eq!(sections[table_index].kind, SHT_STRTAB);
    Ok(())
}

/// Requires exactly one `PT_DYNAMIC` program header covering the dynamic
/// section, and a program header table that fits inside the image.
fn validate_dynamic_segment(bytes: &[u8], dynamic: Section) -> Result<(), Refusal> {
    let program_start = index(num(bytes, 0, Field::HeaderProgramOffset)?)?;
    let program_count = index(num(bytes, 0, Field::HeaderProgramCount)?)?;
    let entry_bytes = index(num(bytes, 0, Field::HeaderProgramEntrySize)?)?;
    let is_bounded_count = program_count > 0 && program_count <= MAX_SECTIONS;
    if !is_bounded_count || entry_bytes != PROGRAM_HEADER_BYTES {
        return Err(Refusal::InvalidLayout);
    }
    let table_bytes = program_count.checked_mul(PROGRAM_HEADER_BYTES).ok_or(Refusal::LimitExceeded)?;
    checked_range(bytes, ByteSpan::new(program_start, table_bytes))?;
    let mut dynamic_segment_count = 0;
    for header in 0..program_count {
        let offset_bytes = header.checked_mul(PROGRAM_HEADER_BYTES).ok_or(Refusal::LimitExceeded)?;
        let base = program_start.checked_add(offset_bytes).ok_or(Refusal::LimitExceeded)?;
        if num(bytes, base, Field::ProgramType)? == PT_DYNAMIC {
            dynamic_segment_count += 1;
            let segment_offset_bytes = index(num(bytes, base, Field::ProgramOffset)?)?;
            let segment_size_bytes = index(num(bytes, base, Field::ProgramSize)?)?;
            if segment_offset_bytes != dynamic.offset || segment_size_bytes != dynamic.size {
                return Err(Refusal::InvalidLayout);
            }
        }
    }
    if dynamic_segment_count != 1 {
        return Err(Refusal::InvalidLayout);
    }
    debug_assert_eq!(dynamic_segment_count, 1);
    debug_assert!(program_start.saturating_add(table_bytes) <= bytes.len());
    Ok(())
}

fn validate_dynamic_layout(
    bytes: &[u8],
    sections: &[Section],
    dynamic: Section,
    table_index: usize,
) -> Result<(), Refusal> {
    validate_table_placement(bytes, sections, table_index)?;
    validate_dynamic_segment(bytes, dynamic)
}

fn dynamic_entries(bytes: &[u8], section: Section) -> Result<Vec<(u64, usize)>, Refusal> {
    if section.entsize != DYNAMIC_BYTES || !section.size.is_multiple_of(DYNAMIC_BYTES) {
        return Err(Refusal::InvalidLayout);
    }
    let entry_count = section.size / DYNAMIC_BYTES;
    if entry_count > MAX_DYNAMIC_ENTRIES {
        return Err(Refusal::LimitExceeded);
    }
    let end = section.offset.checked_add(section.size).ok_or(Refusal::Truncated)?;
    let mut entries = Vec::with_capacity(entry_count);
    let mut is_terminated = false;
    for offset_bytes in (section.offset..end).step_by(DYNAMIC_BYTES) {
        let tag = num(bytes, offset_bytes, Field::DynamicTag)?;
        if tag == 0 {
            is_terminated = true;
            break;
        }
        entries.push((tag, index(num(bytes, offset_bytes, Field::DynamicValue)?)?));
    }
    if !is_terminated {
        return Err(Refusal::InvalidLayout);
    }
    debug_assert!(entries.len() <= entry_count);
    debug_assert!(is_terminated);
    Ok(entries)
}

fn string_at(bytes: &[u8], table: Section, offset_bytes: usize) -> Result<(&[u8], usize), Refusal> {
    if offset_bytes >= table.size {
        return Err(Refusal::InvalidString);
    }
    let start = table.offset.checked_add(offset_bytes).ok_or(Refusal::InvalidString)?;
    let end = table.offset.checked_add(table.size).ok_or(Refusal::InvalidString)?;
    let length_bytes = bytes[start..end].iter().position(|b| *b == 0).ok_or(Refusal::InvalidString)?;
    if length_bytes == 0 {
        return Err(Refusal::InvalidString);
    }
    let text_end = start.checked_add(length_bytes).ok_or(Refusal::InvalidString)?;
    debug_assert!(start < end);
    debug_assert!(length_bytes > 0);
    Ok((&bytes[start..text_end], start))
}

fn replacement(rewrite: &DependencyRewrite<'_>, depth: usize) -> Result<String, Refusal> {
    let member = rewrite.dependency.as_bytes();
    if member.len() < 34 || member[32] != b'-' || !member[..32].iter().all(|b| HASH_ALPHABET.contains(b)) {
        return Err(Refusal::InvalidDependency);
    }
    if !member[33..].iter().all(|b| b.is_ascii_alphanumeric() || b"+._-".contains(b)) {
        return Err(Refusal::InvalidDependency);
    }
    let library_path = rewrite.library_path;
    let is_relative_path = !library_path.is_empty() && !library_path.starts_with('/');
    if !is_relative_path {
        return Err(Refusal::InvalidDependency);
    }
    let has_unsafe_component = library_path.split('/').any(|part| part.is_empty() || part == "." || part == "..");
    let has_unsafe_character = library_path.bytes().any(|byte| matches!(byte, 0 | b':' | b';' | b'$'));
    if has_unsafe_component || has_unsafe_character {
        return Err(Refusal::InvalidDependency);
    }
    if depth > 32 {
        return Err(Refusal::LimitExceeded);
    }
    let mut path = String::from("$ORIGIN/");
    for _ in 0..=depth {
        path.push_str("../");
    }
    path.push_str(rewrite.dependency);
    path.push('/');
    path.push_str(rewrite.library_path);
    debug_assert!(member.len() >= 34);
    debug_assert!(depth <= 32);
    Ok(path)
}

fn symbol_tail_check(
    bytes: &[u8],
    sections: &[Section],
    table_index: usize,
    target: Range<usize>,
) -> Result<(), Refusal> {
    let table = sections[table_index];
    let prefix_end = table.offset.checked_add(target.start).ok_or(Refusal::InvalidString)?;
    // Only names after the previous terminator can extend into the target.
    // Scan that prefix once, not the whole string table for every symbol.
    let containing_start = bytes[table.offset..prefix_end]
        .iter()
        .rposition(|byte| *byte == 0)
        .map_or(0, |offset| offset.saturating_add(1));
    for symtab in sections.iter().filter(|s| s.kind == SHT_DYNSYM && s.link == table_index) {
        if symtab.entsize != SYMBOL_BYTES || symtab.size % SYMBOL_BYTES != 0 {
            return Err(Refusal::InvalidLayout);
        }
        if symtab.size / SYMBOL_BYTES > MAX_SYMBOLS {
            return Err(Refusal::LimitExceeded);
        }
        let end = symtab.offset.checked_add(symtab.size).ok_or(Refusal::Truncated)?;
        for offset_bytes in (symtab.offset..end).step_by(SYMBOL_BYTES) {
            let name = index(num(bytes, offset_bytes, Field::SymbolName)?)?;
            if name == 0 {
                continue;
            }
            if name >= table.size {
                return Err(Refusal::InvalidString);
            }
            if name >= containing_start && name < target.end {
                return Err(Refusal::SymbolTailOverlap);
            }
        }
        debug_assert_eq!(symtab.entsize, SYMBOL_BYTES);
        debug_assert_eq!(symtab.size % SYMBOL_BYTES, 0);
    }
    Ok(())
}

/// Rejects a declared rewrite that does not match exactly one live dynamic
/// entry, and returns the offset of that entry.
fn match_unique_entry(
    bytes: &[u8],
    layout: &DynamicLayout<'_>,
    wanted_tag: u64,
    rewrite: &DependencyRewrite<'_>,
) -> Result<usize, Refusal> {
    let mut candidates = layout.entries.iter().filter(|(tag, offset_bytes)| {
        *tag == wanted_tag
            && string_at(bytes, layout.table, *offset_bytes).is_ok_and(|(text, _)| text == rewrite.current.as_bytes())
    });
    let matched = candidates.next().ok_or(Refusal::DependencyMismatch)?;
    if candidates.next().is_some() {
        return Err(Refusal::DependencyMismatch);
    }
    Ok(matched.1)
}

/// Rejects any other dynamic string whose bytes or terminator overlap a target.
fn check_shared_string(
    bytes: &[u8],
    layout: &DynamicLayout<'_>,
    wanted_tag: u64,
    string_span: core::ops::RangeInclusive<usize>,
) -> Result<(), Refusal> {
    let offset_bytes = *string_span.start();
    assert!(offset_bytes <= *string_span.end(), "a string span includes its terminator");
    let prefix_end = layout.table.offset.checked_add(offset_bytes).ok_or(Refusal::InvalidString)?;
    let containing_start = bytes[layout.table.offset..prefix_end]
        .iter()
        .rposition(|byte| *byte == 0)
        .map_or(0, |offset| offset.saturating_add(1));
    assert!(containing_start <= offset_bytes, "the containing string starts at or before the target");
    for (tag, other) in &layout.entries {
        let is_shared_tag = matches!(*tag, DT_NEEDED | DT_RPATH | DT_RUNPATH | DT_SONAME);
        if !is_shared_tag || (*tag == wanted_tag && *other == offset_bytes) {
            continue;
        }
        if *other >= layout.table.size {
            return Err(Refusal::InvalidString);
        }
        if *other >= containing_start && *other <= *string_span.end() {
            return Err(Refusal::SharedDynamicString);
        }
    }
    Ok(())
}

/// Admits one declared rewrite and returns the patch it produces.
fn admit_rewrite(
    bytes: &[u8],
    layout: &DynamicLayout<'_>,
    rewrite: &DependencyRewrite<'_>,
    depth: usize,
    seen: &mut BTreeSet<usize>,
) -> Result<StringPatch, Refusal> {
    if rewrite.current.is_empty() || rewrite.current.as_bytes().contains(&0) {
        return Err(Refusal::InvalidDependency);
    }
    // A declared target must already retain the same scanner-visible hash.
    // Short un-hashed SONAMEs cannot be safely upgraded by this fixup.
    if !rewrite.current.split('/').any(|component| component == rewrite.dependency) {
        return Err(Refusal::DependencyMismatch);
    }
    let wanted_tag = match rewrite.kind {
        EntryKind::Needed => DT_NEEDED,
        EntryKind::Runpath => DT_RUNPATH,
    };
    let offset_bytes = match_unique_entry(bytes, layout, wanted_tag, rewrite)?;
    if !seen.insert(offset_bytes) {
        return Err(Refusal::SharedDynamicString);
    }
    let (current, start) = string_at(bytes, layout.table, offset_bytes)?;
    let end = offset_bytes.checked_add(current.len()).ok_or(Refusal::InvalidString)?;
    check_shared_string(bytes, layout, wanted_tag, offset_bytes..=end)?;
    let tail_end = end.checked_add(1).ok_or(Refusal::InvalidString)?;
    symbol_tail_check(bytes, layout.sections, layout.table_index, offset_bytes..tail_end)?;
    let new = replacement(rewrite, depth)?;
    if new.len() > current.len() {
        return Err(Refusal::MissingCapacity);
    }
    debug_assert!(!current.is_empty());
    debug_assert!(new.len() <= current.len());
    Ok(StringPatch {
        offset: start,
        original: current.to_vec(),
        replacement: new,
    })
}

/// Requires the declared string table to match the dynamic entries that
/// describe it.
fn check_table_entries(entries: &[(u64, usize)], table: Section) -> Result<(), Refusal> {
    let strsz = entries.iter().find(|(tag, _)| *tag == DT_STRSZ).ok_or(Refusal::InvalidLayout)?.1;
    let strtab = entries.iter().find(|(tag, _)| *tag == DT_STRTAB).ok_or(Refusal::InvalidLayout)?.1;
    if strsz != table.size || strtab != table.address {
        return Err(Refusal::InvalidLayout);
    }
    Ok(())
}

/// Locates the dynamic section, its string table, and the entries it declares.
fn read_dynamic_layout<'a>(bytes: &[u8], sections: &'a [Section]) -> Result<DynamicLayout<'a>, Refusal> {
    let (dynamic_index, dynamic) = sections
        .iter()
        .copied()
        .enumerate()
        .find(|(_, s)| s.kind == SHT_DYNAMIC)
        .ok_or(Refusal::InvalidLayout)?;
    let dynamic_section_count = sections.iter().filter(|section| section.kind == SHT_DYNAMIC).count();
    if dynamic_section_count != 1 {
        return Err(Refusal::InvalidLayout);
    }
    if dynamic.link >= sections.len() || sections[dynamic.link].kind != SHT_STRTAB {
        return Err(Refusal::InvalidLayout);
    }
    let table_index = dynamic.link;
    let table = sections[table_index];
    if table.size == 0 || table.size > MAX_STRING_BYTES {
        return Err(Refusal::LimitExceeded);
    }
    validate_dynamic_layout(bytes, sections, dynamic, table_index)?;
    let entries = dynamic_entries(bytes, dynamic)?;
    check_table_entries(&entries, table)?;
    debug_assert_eq!(dynamic_section_count, 1);
    debug_assert!(table_index < sections.len());
    Ok(DynamicLayout {
        sections,
        dynamic_index,
        table_index,
        table,
        entries,
    })
}

/// Rejects an image that is not a bounded little-endian ELF64 object.
fn check_image_header(bytes: &[u8], rewrite_count: usize) -> Result<(), Refusal> {
    if bytes.len() < ELF_HEADER_BYTES {
        return Err(Refusal::Truncated);
    }
    if bytes.len() > MAX_IMAGE_BYTES || rewrite_count > MAX_REWRITES {
        return Err(Refusal::LimitExceeded);
    }
    if bytes.get(..4) != Some(b"\x7fELF") || bytes[4] != 2 {
        return Err(Refusal::UnsupportedClass);
    }
    if bytes[5] != 1 {
        return Err(Refusal::UnsupportedEndianness);
    }
    if !matches!(num(bytes, 0, Field::HeaderType)?, 2 | 3) {
        return Err(Refusal::UnsupportedType);
    }
    debug_assert!(bytes.len() >= ELF_HEADER_BYTES);
    debug_assert!(rewrite_count <= MAX_REWRITES);
    Ok(())
}

/// `depth` is the count of directories below the output's flat-store root.
/// Every declared rewrite must match exactly one live dynamic entry.
/// No bytes are changed until all entries have passed admission.
// r[impl mantle.relocatable_outputs.bounded_fixup_admission]
// r[impl mantle.relocatable_outputs.origin_relative_needed]
pub fn plan_dynamic_relocation(
    bytes: &[u8],
    depth: usize,
    rewrites: &[DependencyRewrite<'_>],
) -> Result<FixupPlan, Refusal> {
    check_image_header(bytes, rewrites.len())?;
    let sections = sections(bytes)?;
    let layout = read_dynamic_layout(bytes, &sections)?;
    let mut patches = Vec::with_capacity(rewrites.len());
    let mut seen = BTreeSet::new();
    for rewrite in rewrites {
        let patch = admit_rewrite(bytes, &layout, rewrite, depth, &mut seen)?;
        patches.push(patch);
    }
    if patches.is_empty() || layout.dynamic_index >= sections.len() {
        return Err(Refusal::DependencyMismatch);
    }
    debug_assert_eq!(patches.len(), rewrites.len());
    debug_assert!(layout.dynamic_index < sections.len());
    Ok(FixupPlan { patches })
}

/// Applies a previously admitted plan only to the identical source image.
pub fn apply_plan(bytes: &[u8], plan: &FixupPlan) -> Result<Vec<u8>, Refusal> {
    let mut result = bytes.to_vec();
    for patch in &plan.patches {
        let original_bytes = patch.original.len().checked_add(1).ok_or(Refusal::InvalidLayout)?;
        let range = checked_range(bytes, ByteSpan::new(patch.offset, original_bytes))?;
        let text_end = range.end.checked_sub(1).ok_or(Refusal::InvalidLayout)?;
        if bytes[range.start..text_end] != patch.original
            || bytes[text_end] != 0
            || patch.replacement.len() > patch.original.len()
        {
            return Err(Refusal::InvalidLayout);
        }
        debug_assert_eq!(result.len(), bytes.len());
        debug_assert!(patch.replacement.len() <= patch.original.len());
    }
    for patch in &plan.patches {
        let original_end = patch.offset.checked_add(patch.original.len()).ok_or(Refusal::InvalidLayout)?;
        let replacement_end = patch.offset.checked_add(patch.replacement.len()).ok_or(Refusal::InvalidLayout)?;
        result[patch.offset..original_end].fill(0);
        result[patch.offset..replacement_end].copy_from_slice(patch.replacement.as_bytes());
    }
    Ok(result)
}
