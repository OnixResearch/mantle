//! Bounded ELF inspection; no host tools or filesystem authority.
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::native::PAGE_BYTES;

pub const MAX_ELF_BYTES: usize = 512 * 1024 * 1024;
const MAX_SECTIONS: u64 = 2048;
const MAX_SYMBOLS: u64 = 262_144;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Elf {
    pub needed: Vec<String>,
    pub soname: Option<String>,
    pub android_api: Option<u32>,
    pub exports: BTreeSet<String>,
    pub imports: BTreeSet<String>,
}

fn bytes(data: &[u8], offset: u64, size: u64) -> Result<&[u8], &'static str> {
    let end = offset.checked_add(size).ok_or("ELF range overflow")?;
    let start = usize::try_from(offset).map_err(|_| "ELF offset overflow")?;
    let end = usize::try_from(end).map_err(|_| "ELF offset overflow")?;
    data.get(start..end).ok_or("truncated ELF")
}
fn n(data: &[u8], offset: u64, size: u64) -> Result<u64, &'static str> {
    let mut word = [0; 8];
    let value = bytes(data, offset, size)?;
    word[..value.len()].copy_from_slice(value);
    Ok(u64::from_le_bytes(word))
}
fn text(data: &[u8], offset: u64) -> Result<&str, &'static str> {
    let start = usize::try_from(offset).map_err(|_| "ELF string overflow")?;
    let rest = data.get(start..).ok_or("ELF string outside table")?;
    let end = rest.iter().take(1025).position(|byte| *byte == 0).ok_or("unterminated or oversized ELF string")?;
    core::str::from_utf8(&rest[..end]).map_err(|_| "non-UTF8 ELF string")
}
fn section(data: &[u8], index: u64) -> Result<&[u8], &'static str> {
    let count = n(data, 60, 2)?;
    if index >= count {
        return Err("ELF section index outside table");
    }
    bytes(data, n(data, 40, 8)?.checked_add(index * 64).ok_or("ELF section overflow")?, 64)
}
fn section_data<'a>(data: &'a [u8], header: &[u8]) -> Result<&'a [u8], &'static str> {
    bytes(data, n(header, 24, 8)?, n(header, 32, 8)?)
}

pub fn inspect(data: &[u8], require_16k: bool) -> Result<Elf, &'static str> {
    if data.len() > MAX_ELF_BYTES
        || bytes(data, 0, 7)? != b"\x7fELF\x02\x01\x01"
        || n(data, 16, 2)? != 3
        || n(data, 18, 2)? != 183
        || n(data, 20, 4)? != 1
    {
        return Err("expected AArch64 little-endian ELF64 shared object");
    }
    if n(data, 52, 2)? != 64 || n(data, 54, 2)? != 56 || n(data, 58, 2)? != 64 {
        return Err("unsupported ELF header layout");
    }
    let dynamic_segment = inspect_segments(data, require_16k)?;
    let sections = n(data, 60, 2)?;
    if sections == 0 || sections > MAX_SECTIONS {
        return Err("missing or excessive ELF sections");
    }
    let mut symbols_header = None;
    let mut dynamic_header = None;
    let mut result = Elf {
        needed: Vec::new(),
        soname: None,
        android_api: None,
        exports: BTreeSet::new(),
        imports: BTreeSet::new(),
    };
    for index in 0..sections {
        let header = section(data, index)?;
        match n(header, 4, 4)? {
            6 if dynamic_header.replace(header).is_some() => return Err("duplicate ELF dynamic section"),
            7 => android_notes(data, header, &mut result)?,
            11 if symbols_header.replace(header).is_some() => return Err("duplicate ELF symbol section"),
            _ => {}
        }
    }
    let dynamic_header = dynamic_header.ok_or("missing ELF dynamic section")?;
    if dynamic_segment != (n(dynamic_header, 24, 8)?, n(dynamic_header, 32, 8)?) {
        return Err("ELF dynamic section differs from loader segment");
    }
    let symbols_header = symbols_header.ok_or("missing ELF dynamic symbols")?;
    if n(dynamic_header, 40, 4)? != n(symbols_header, 40, 4)? {
        return Err("ELF dynamic strings differ from symbol strings");
    }
    dynamic(data, dynamic_header, &mut result)?;
    symbols(data, symbols_header, &mut result)?;
    check_tables(data, dynamic_header, symbols_header)?;
    Ok(result)
}

fn inspect_segments(data: &[u8], require_16k: bool) -> Result<(u64, u64), &'static str> {
    let count = n(data, 56, 2)?;
    if count == 0 || count > 128 {
        return Err("invalid ELF program header count");
    }
    let mut loads = 0;
    let mut dynamic_segment = None;
    for index in 0..count {
        let header = bytes(data, n(data, 32, 8)?.checked_add(index * 56).ok_or("ELF header overflow")?, 56)?;
        let kind = n(header, 0, 4)?;
        if kind == 3 {
            return Err("Android library must not request an ELF interpreter");
        }
        if kind == 2 && dynamic_segment.replace((n(header, 8, 8)?, n(header, 32, 8)?)).is_some() {
            return Err("duplicate ELF dynamic segment");
        }
        if kind != 1 {
            continue;
        }
        loads += 1;
        let offset = n(header, 8, 8)?;
        let address = n(header, 16, 8)?;
        let file_size = n(header, 32, 8)?;
        if file_size > n(header, 40, 8)? {
            return Err("ELF load size exceeds memory size");
        }
        bytes(data, offset, file_size)?;
        let alignment = n(header, 48, 8)?;
        if !alignment.is_power_of_two() || offset % alignment != address % alignment {
            return Err("invalid ELF load alignment");
        }
        if require_16k && (alignment < PAGE_BYTES || offset % PAGE_BYTES != address % PAGE_BYTES) {
            return Err("Android ELF load segment is not 16KiB aligned");
        }
    }
    if loads == 0 {
        return Err("ELF has no load segments");
    }
    dynamic_segment.ok_or("ELF has no dynamic segment")
}

fn check_tables(data: &[u8], dynamic_header: &[u8], symbols_header: &[u8]) -> Result<(), &'static str> {
    let strings_header = section(data, n(symbols_header, 40, 4)?)?;
    let table = section_data(data, dynamic_header)?;
    for header in [dynamic_header, symbols_header, strings_header] {
        mapped_section(data, header)?;
    }
    for (tag, expected) in [
        (5, n(strings_header, 16, 8)?),
        (6, n(symbols_header, 16, 8)?),
        (10, n(strings_header, 32, 8)?),
        (11, 24),
    ] {
        let mut values = table
            .chunks_exact(16)
            .take_while(|entry| n(entry, 0, 8) != Ok(0))
            .filter(|entry| n(entry, 0, 8) == Ok(tag));
        if values.next().map(|entry| n(entry, 8, 8)) != Some(Ok(expected)) || values.next().is_some() {
            return Err("ELF dynamic address/size differs from symbol sections");
        }
    }
    Ok(())
}

fn mapped_section(data: &[u8], section: &[u8]) -> Result<(), &'static str> {
    let offset = n(section, 24, 8)?;
    let end = offset.checked_add(n(section, 32, 8)?).ok_or("ELF section range overflow")?;
    for index in 0..n(data, 56, 2)? {
        let header = bytes(data, n(data, 32, 8)?.checked_add(index * 56).ok_or("ELF header overflow")?, 56)?;
        if n(header, 0, 4)? != 1 {
            continue;
        }
        let start = n(header, 8, 8)?;
        let limit = start.checked_add(n(header, 32, 8)?).ok_or("ELF load range overflow")?;
        if offset >= start && end <= limit && n(header, 16, 8)?.checked_add(offset - start) == Some(n(section, 16, 8)?)
        {
            return Ok(());
        }
    }
    Err("ELF dynamic table is not mapped at its declared address")
}

fn dynamic(data: &[u8], header: &[u8], result: &mut Elf) -> Result<(), &'static str> {
    let table = section_data(data, header)?;
    let strings = section_data(data, section(data, n(header, 40, 4)?)?)?;
    if n(header, 56, 8)? != 16 || table.len() % 16 != 0 || table.len() > 65536 {
        return Err("invalid ELF dynamic table");
    }
    for entry in table.chunks_exact(16) {
        let tag = n(entry, 0, 8)?;
        if tag == 0 {
            return Ok(());
        }
        if matches!(tag, 15 | 22 | 29) || (tag == 30 && n(entry, 8, 8)? & 4 != 0) {
            return Err("ELF RPATH/RUNPATH/text relocation is forbidden");
        }
        if tag != 1 && tag != 14 {
            continue;
        }
        let name = text(strings, n(entry, 8, 8)?)?;
        if name.is_empty() || name.contains('/') || name.contains('\\') || name == "." || name == ".." {
            return Err("ELF dependency is not a library basename");
        }
        if tag == 1 {
            if result.needed.len() >= 256 {
                return Err("too many ELF dependencies");
            }
            result.needed.push(name.to_string());
        } else if result.soname.replace(name.to_string()).is_some() {
            return Err("duplicate ELF SONAME");
        }
    }
    Err("unterminated ELF dynamic table")
}

fn symbols(data: &[u8], header: &[u8], result: &mut Elf) -> Result<(), &'static str> {
    let table = section_data(data, header)?;
    let strings = section_data(data, section(data, n(header, 40, 4)?)?)?;
    if n(header, 56, 8)? != 24 || table.len() % 24 != 0 || table.len() as u64 / 24 > MAX_SYMBOLS {
        return Err("invalid ELF dynamic symbol table");
    }
    for entry in table.chunks_exact(24) {
        let binding = entry[4] >> 4;
        let defined = n(entry, 6, 2)? != 0;
        let name = text(strings, n(entry, 0, 4)?)?;
        if name.is_empty() {
            continue;
        }
        if defined && matches!(binding, 1 | 2) && matches!(entry[5] & 3, 0 | 3) {
            result.exports.insert(name.to_string());
        } else if !defined && binding == 1 {
            result.imports.insert(name.to_string());
        }
    }
    Ok(())
}

fn android_notes(data: &[u8], header: &[u8], result: &mut Elf) -> Result<(), &'static str> {
    let table = section_data(data, header)?;
    if table.len() > 65536 {
        return Err("oversized ELF note table");
    }
    let mut cursor = 0;
    while cursor < table.len() as u64 {
        let name_size = n(table, cursor, 4)?;
        let value_size = n(table, cursor + 4, 4)?;
        let kind = n(table, cursor + 8, 4)?;
        cursor += 12;
        let name = bytes(table, cursor, name_size)?;
        cursor += (name_size + 3) & !3;
        let value = bytes(table, cursor, value_size)?;
        cursor += (value_size + 3) & !3;
        if kind == 1 && name == b"Android\0" {
            mapped_section(data, header)?;
            let api = n(value, 0, 4)? as u32;
            if result.android_api.replace(api).is_some() {
                return Err("duplicate Android API note");
            }
        }
    }
    if cursor != table.len() as u64 {
        return Err("truncated ELF note padding");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    include!("elf_tests.rs");
}
