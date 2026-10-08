//! Pure, bounded planning of in-place ELF64 dynamic-string rewrites.
//! The caller owns file IO and publication; a plan never changes its input.

extern crate alloc;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;
use core::ops::Range;

const MAX_IMAGE_BYTES: usize = 128 * 1024 * 1024;
const MAX_SECTIONS: usize = 4096;
const MAX_DYNAMIC_ENTRIES: usize = 4096;
const MAX_SYMBOLS: usize = 1_048_576;
const MAX_REWRITES: usize = 256;
const MAX_STRING_BYTES: usize = 8 * 1024 * 1024;
const SECTION_BYTES: usize = 64;
const DYNAMIC_BYTES: usize = 16;
const SYMBOL_BYTES: usize = 24;
const SHT_DYNAMIC: u32 = 6;
const SHT_DYNSYM: u32 = 11;
const SHT_STRTAB: u32 = 3;
const DT_NEEDED: u64 = 1;
const DT_RPATH: u64 = 15;
const DT_RUNPATH: u64 = 29;
const DT_STRTAB: u64 = 5;
const DT_STRSZ: u64 = 10;
const DT_SONAME: u64 = 14;
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

fn checked_range(bytes: &[u8], start: usize, size: usize) -> Result<Range<usize>, Refusal> {
    let end = start.checked_add(size).ok_or(Refusal::Truncated)?;
    if end > bytes.len() {
        return Err(Refusal::Truncated);
    }
    Ok(start..end)
}

fn num(bytes: &[u8], offset: usize, width: usize) -> Result<u64, Refusal> {
    let range = checked_range(bytes, offset, width)?;
    let mut value = 0_u64;
    for (shift, byte) in bytes[range].iter().enumerate() {
        value |= u64::from(*byte) << (shift * 8);
    }
    Ok(value)
}

fn index(value: u64) -> Result<usize, Refusal> {
    usize::try_from(value).map_err(|_| Refusal::LimitExceeded)
}

fn sections(bytes: &[u8]) -> Result<Vec<Section>, Refusal> {
    let start = index(num(bytes, 40, 8)?)?;
    let count = index(num(bytes, 60, 2)?)?;
    if count == 0 || count > MAX_SECTIONS || num(bytes, 58, 2)? != SECTION_BYTES as u64 {
        return Err(Refusal::InvalidLayout);
    }
    checked_range(bytes, start, count.checked_mul(SECTION_BYTES).ok_or(Refusal::LimitExceeded)?)?;
    let mut result = Vec::with_capacity(count);
    for section_index in 0..count {
        let base = start + section_index * SECTION_BYTES;
        let section = Section {
            kind: num(bytes, base + 4, 4)? as u32,
            address: index(num(bytes, base + 16, 8)?)?,
            offset: index(num(bytes, base + 24, 8)?)?,
            size: index(num(bytes, base + 32, 8)?)?,
            link: index(num(bytes, base + 40, 4)?)?,
            entsize: index(num(bytes, base + 56, 8)?)?,
        };
        if section.kind != 8 {
            checked_range(bytes, section.offset, section.size)?;
        }
        result.push(section);
    }
    Ok(result)
}

fn validate_dynamic_layout(
    bytes: &[u8],
    sections: &[Section],
    dynamic: Section,
    table_index: usize,
) -> Result<(), Refusal> {
    let table = sections[table_index];
    let table_range = checked_range(bytes, table.offset, table.size)?;
    let section_start = index(num(bytes, 40, 8)?)?;
    let section_bytes = sections.len().checked_mul(SECTION_BYTES).ok_or(Refusal::LimitExceeded)?;
    let header_range = checked_range(bytes, section_start, section_bytes)?;
    if table_range.start < header_range.end && header_range.start < table_range.end {
        return Err(Refusal::InvalidLayout);
    }
    if table.offset < 64 {
        return Err(Refusal::InvalidLayout);
    }
    for (section_index, section) in sections.iter().enumerate() {
        if section_index == table_index || section.kind == 8 || section.size == 0 {
            continue;
        }
        let other = checked_range(bytes, section.offset, section.size)?;
        if table_range.start < other.end && other.start < table_range.end {
            return Err(Refusal::InvalidLayout);
        }
    }
    let program_start = index(num(bytes, 32, 8)?)?;
    let program_count = index(num(bytes, 56, 2)?)?;
    if program_count == 0 || program_count > MAX_SECTIONS || num(bytes, 54, 2)? != 56 {
        return Err(Refusal::InvalidLayout);
    }
    checked_range(bytes, program_start, program_count.checked_mul(56).ok_or(Refusal::LimitExceeded)?)?;
    let mut dynamic_segment_count = 0;
    for header in 0..program_count {
        let base = program_start + header * 56;
        if num(bytes, base, 4)? == 2 {
            dynamic_segment_count += 1;
            if index(num(bytes, base + 8, 8)?)? != dynamic.offset || index(num(bytes, base + 32, 8)?)? != dynamic.size {
                return Err(Refusal::InvalidLayout);
            }
        }
    }
    if dynamic_segment_count != 1 {
        return Err(Refusal::InvalidLayout);
    }
    Ok(())
}

fn dynamic_entries(bytes: &[u8], section: Section) -> Result<Vec<(u64, usize)>, Refusal> {
    if section.entsize != DYNAMIC_BYTES || !section.size.is_multiple_of(DYNAMIC_BYTES) {
        return Err(Refusal::InvalidLayout);
    }
    if section.size / DYNAMIC_BYTES > MAX_DYNAMIC_ENTRIES {
        return Err(Refusal::LimitExceeded);
    }
    let mut entries = Vec::new();
    let mut terminated = false;
    for offset in (section.offset..section.offset + section.size).step_by(DYNAMIC_BYTES) {
        let tag = num(bytes, offset, 8)?;
        if tag == 0 {
            terminated = true;
            break;
        }
        entries.push((tag, index(num(bytes, offset + 8, 8)?)?));
    }
    if !terminated {
        return Err(Refusal::InvalidLayout);
    }
    Ok(entries)
}

fn string_at(bytes: &[u8], table: Section, offset: usize) -> Result<(&[u8], usize), Refusal> {
    if offset >= table.size {
        return Err(Refusal::InvalidString);
    }
    let start = table.offset + offset;
    let end = table.offset + table.size;
    let length = bytes[start..end].iter().position(|b| *b == 0).ok_or(Refusal::InvalidString)?;
    if length == 0 {
        return Err(Refusal::InvalidString);
    }
    Ok((&bytes[start..start + length], start))
}

fn replacement(rewrite: &DependencyRewrite<'_>, depth: usize) -> Result<String, Refusal> {
    let member = rewrite.dependency.as_bytes();
    if member.len() < 34 || member[32] != b'-' || !member[..32].iter().all(|b| HASH_ALPHABET.contains(b)) {
        return Err(Refusal::InvalidDependency);
    }
    if !member[33..].iter().all(|b| b.is_ascii_alphanumeric() || b"+._-".contains(b)) {
        return Err(Refusal::InvalidDependency);
    }
    if rewrite.library_path.is_empty()
        || rewrite.library_path.starts_with('/')
        || rewrite.library_path.split('/').any(|part| part.is_empty() || part == "." || part == "..")
    {
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
    Ok(path)
}

fn symbol_tail_check(
    bytes: &[u8],
    sections: &[Section],
    table_index: usize,
    target: Range<usize>,
) -> Result<(), Refusal> {
    for symtab in sections.iter().filter(|s| s.kind == SHT_DYNSYM && s.link == table_index) {
        if symtab.entsize != SYMBOL_BYTES || symtab.size % SYMBOL_BYTES != 0 {
            return Err(Refusal::InvalidLayout);
        }
        if symtab.size / SYMBOL_BYTES > MAX_SYMBOLS {
            return Err(Refusal::LimitExceeded);
        }
        for offset in (symtab.offset..symtab.offset + symtab.size).step_by(SYMBOL_BYTES) {
            let name = index(num(bytes, offset, 4)?)?;
            if name != 0 && target.contains(&name) {
                return Err(Refusal::SymbolTailOverlap);
            }
        }
    }
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
    if bytes.len() < 64 {
        return Err(Refusal::Truncated);
    }
    if bytes.len() > MAX_IMAGE_BYTES || rewrites.len() > MAX_REWRITES {
        return Err(Refusal::LimitExceeded);
    }
    if bytes.get(..4) != Some(b"\x7fELF") || bytes[4] != 2 {
        return Err(Refusal::UnsupportedClass);
    }
    if bytes[5] != 1 {
        return Err(Refusal::UnsupportedEndianness);
    }
    if !matches!(num(bytes, 16, 2)?, 2 | 3) {
        return Err(Refusal::UnsupportedType);
    }
    let sections = sections(bytes)?;
    let (dynamic_index, dynamic) = sections
        .iter()
        .copied()
        .enumerate()
        .find(|(_, s)| s.kind == SHT_DYNAMIC)
        .ok_or(Refusal::InvalidLayout)?;
    if sections.iter().filter(|section| section.kind == SHT_DYNAMIC).count() != 1 {
        return Err(Refusal::InvalidLayout);
    }
    if dynamic.link >= sections.len() || sections[dynamic.link].kind != SHT_STRTAB {
        return Err(Refusal::InvalidLayout);
    }
    let table = sections[dynamic.link];
    if table.size == 0 || table.size > MAX_STRING_BYTES {
        return Err(Refusal::LimitExceeded);
    }
    validate_dynamic_layout(bytes, &sections, dynamic, dynamic.link)?;
    let entries = dynamic_entries(bytes, dynamic)?;
    let strsz = entries.iter().find(|(tag, _)| *tag == DT_STRSZ).ok_or(Refusal::InvalidLayout)?.1;
    let strtab = entries.iter().find(|(tag, _)| *tag == DT_STRTAB).ok_or(Refusal::InvalidLayout)?.1;
    if strsz != table.size || strtab != table.address {
        return Err(Refusal::InvalidLayout);
    }
    let mut patches = Vec::with_capacity(rewrites.len());
    let mut seen = BTreeSet::new();
    for rewrite in rewrites {
        if rewrite.current.is_empty() || rewrite.current.as_bytes().contains(&0) {
            return Err(Refusal::InvalidDependency);
        }
        // A declared target must already retain the same scanner-visible hash.
        // Short un-hashed SONAMEs cannot be safely upgraded by this fixup.
        if !rewrite.current.contains(rewrite.dependency) {
            return Err(Refusal::DependencyMismatch);
        }
        let wanted_tag = match rewrite.kind {
            EntryKind::Needed => DT_NEEDED,
            EntryKind::Runpath => DT_RUNPATH,
        };
        let mut candidates = entries.iter().filter(|(tag, offset)| {
            *tag == wanted_tag
                && string_at(bytes, table, *offset).is_ok_and(|(text, _)| text == rewrite.current.as_bytes())
        });
        let matched = candidates.next().ok_or(Refusal::DependencyMismatch)?;
        if candidates.next().is_some() {
            return Err(Refusal::DependencyMismatch);
        }
        let (_, offset) = *matched;
        if !seen.insert(offset) {
            return Err(Refusal::SharedDynamicString);
        }
        let (current, start) = string_at(bytes, table, offset)?;
        let end = offset + current.len();
        for (tag, other) in &entries {
            if matches!(*tag, DT_NEEDED | DT_RPATH | DT_RUNPATH | DT_SONAME)
                && *other != offset
                && (offset..=end).contains(other)
            {
                return Err(Refusal::SharedDynamicString);
            }
        }
        symbol_tail_check(bytes, &sections, dynamic.link, offset..end + 1)?;
        let new = replacement(rewrite, depth)?;
        if new.len() > current.len() {
            return Err(Refusal::MissingCapacity);
        }
        patches.push(StringPatch {
            offset: start,
            original: current.to_vec(),
            replacement: new,
        });
    }
    if patches.is_empty() || dynamic_index >= sections.len() {
        return Err(Refusal::DependencyMismatch);
    }
    Ok(FixupPlan { patches })
}

/// Applies a previously admitted plan only to the identical source image.
pub fn apply_plan(bytes: &[u8], plan: &FixupPlan) -> Result<Vec<u8>, Refusal> {
    let mut result = bytes.to_vec();
    for patch in &plan.patches {
        let range = checked_range(bytes, patch.offset, patch.original.len() + 1)?;
        if bytes[range.start..range.end - 1] != patch.original
            || bytes[range.end - 1] != 0
            || patch.replacement.len() > patch.original.len()
        {
            return Err(Refusal::InvalidLayout);
        }
    }
    for patch in &plan.patches {
        result[patch.offset..patch.offset + patch.original.len()].fill(0);
        result[patch.offset..patch.offset + patch.replacement.len()].copy_from_slice(patch.replacement.as_bytes());
    }
    Ok(result)
}
