use std::collections::BTreeSet;

use object::Object;
use object::ObjectSection;
use object::ObjectSymbol;

use crate::elf_local_symbol_core::ElfCanonicalizationError;
use crate::elf_local_symbol_core::canonicalize_local_elf_symbol_names;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElfCheckOutput {
    pub bytes: Vec<u8>,
    pub rewrite_count: u32,
}

const ELF_HEADER_BYTES: usize = 64;
const ELF_SECTION_HEADER_BYTES: usize = 64;
const ELF_SYMBOL_BYTES: usize = 24;
const ELF_TYPE_REL: u16 = 1;
const SHT_SYMTAB: u32 = 2;
const SHT_STRTAB: u32 = 3;
const SHT_RELA: u32 = 4;
const SHT_NOBITS: u32 = 8;
const SHT_GROUP: u32 = 17;
const SHN_XINDEX: u16 = 0xffff;
const STB_LOCAL: u8 = 0;

const RUST_SECTION_COUNT_MAX: usize = if crate::MANTLE_FUZZING_ENABLED { 16 } else { 4_096 };
const RUST_SYMBOL_COUNT_MAX: usize = if crate::MANTLE_FUZZING_ENABLED { 256 } else { 1_048_576 };
const RUST_STRING_TABLE_BYTES_MAX: usize = if crate::MANTLE_FUZZING_ENABLED {
    64 * 1_024
} else {
    64 * 1_024 * 1_024
};

unsafe extern "C" {
    fn fuzz_c_canonicalize_elf(bytes: *mut u8, file_size: u64, rewrite_count: *mut u64) -> i32;
    fn fuzz_c_canonicalize_file(path: *const std::ffi::c_char) -> i32;
}

pub fn c_canonicalize_elf(input: &[u8]) -> Option<(Vec<u8>, u64)> {
    let mut boxed: Box<[u8]> = if input.is_empty() {
        vec![0u8; 1].into_boxed_slice()
    } else {
        input.to_vec().into_boxed_slice()
    };
    let mut rewrites = 0u64;
    let rc = unsafe { fuzz_c_canonicalize_elf(boxed.as_mut_ptr(), input.len() as u64, &mut rewrites) };
    if rc == 1 {
        let out = if input.is_empty() { Vec::new() } else { boxed.into_vec() };
        Some((out, rewrites))
    } else {
        None
    }
}

pub fn c_canonicalize_file(path: &std::path::Path) -> i32 {
    let c_path = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).expect("valid C path");
    unsafe { fuzz_c_canonicalize_file(c_path.as_ptr()) }
}

pub fn check_elf_rust_invariants(input: &[u8]) -> Result<ElfCheckOutput, String> {
    let res = canonicalize_local_elf_symbol_names(input);
    if let Ok(ref canon1) = res {
        assert!(
            canon1.bytes.len() >= input.len(),
            "canonical ELF unexpectedly shrank: {} < {}",
            canon1.bytes.len(),
            input.len()
        );
        let canon2 = canonicalize_local_elf_symbol_names(&canon1.bytes)
            .expect("canonicalized ELF must succeed on second pass (idempotence)");
        assert_eq!(canon2.bytes, canon1.bytes, "canonicalize_local_elf_symbol_names is not idempotent");
        assert_eq!(canon2.rewrite_count, canon1.rewrite_count, "rewrite_count changed on second pass");
        verify_canonical_local_symbol_widths(&canon1.bytes, canon1.rewrite_count);
    }
    res.map(|c| ElfCheckOutput {
        bytes: c.bytes,
        rewrite_count: c.rewrite_count,
    })
    .map_err(|e| e.to_string())
}

fn verify_canonical_local_symbol_widths(bytes: &[u8], expected_rewrites: u32) {
    let e_shoff = read_u64(bytes, 40) as usize;
    let e_shnum = usize::from(read_u16(bytes, 60));
    let mut sym_sec = None;
    for idx in 0..e_shnum {
        let off = e_shoff + idx * ELF_SECTION_HEADER_BYTES;
        if read_u32(bytes, off + 4) == SHT_SYMTAB {
            sym_sec = Some(off);
            break;
        }
    }
    let sym_off = sym_sec.expect("symtab exists");
    let sym_file_off = read_u64(bytes, sym_off + 24) as usize;
    let sym_size = read_u64(bytes, sym_off + 32) as usize;
    let str_idx = usize::from(read_u32(bytes, sym_off + 40) as u16);
    let str_hdr = e_shoff + str_idx * ELF_SECTION_HEADER_BYTES;
    let str_file_off = read_u64(bytes, str_hdr + 24) as usize;
    let str_size = read_u64(bytes, str_hdr + 32) as usize;
    let strtab = &bytes[str_file_off..str_file_off + str_size];

    let sym_count = sym_size / ELF_SYMBOL_BYTES;
    let mut seen_target_offsets = BTreeSet::new();
    for i in 0..sym_count {
        let s = sym_file_off + i * ELF_SYMBOL_BYTES;
        let st_name = read_u32(bytes, s) as usize;
        let st_bind = bytes[s + 4] >> 4;
        let end = strtab[st_name..].iter().position(|&b| b == 0).expect("null terminated");
        let name = &strtab[st_name..st_name + end];
        if st_bind == STB_LOCAL && name.len() > 2 && name.starts_with(b"L.") && name[2..].iter().all(u8::is_ascii_digit)
        {
            assert_eq!(
                name.len(),
                12,
                "expected 10-digit canonical local symbol name, got {:?}",
                String::from_utf8_lossy(name)
            );
            let expected = format!("L.{i:010}");
            assert_eq!(name, expected.as_bytes());
            seen_target_offsets.insert(st_name);
        }
    }
    assert_eq!(seen_target_offsets.len(), expected_rewrites as usize);
}

pub fn check_elf_differential(input: &[u8]) {
    let rust_res = canonicalize_local_elf_symbol_names(input);
    if let Ok(ref canon1) = rust_res {
        assert!(
            canon1.bytes.len() >= input.len(),
            "canonical ELF unexpectedly shrank: {} < {}",
            canon1.bytes.len(),
            input.len()
        );
        let canon2 = canonicalize_local_elf_symbol_names(&canon1.bytes)
            .expect("canonicalized ELF must succeed on second pass (idempotence)");
        assert_eq!(canon2.bytes, canon1.bytes, "canonicalize_local_elf_symbol_names is not idempotent");
        assert_eq!(canon2.rewrite_count, canon1.rewrite_count, "rewrite_count changed on second pass");
        verify_canonical_local_symbol_widths(&canon1.bytes, canon1.rewrite_count);
    }
    let c_res = c_canonicalize_elf(input);

    match (rust_res, c_res) {
        (Ok(rust), Some((c_bytes, c_rewrites))) => {
            assert_eq!(
                c_rewrites,
                u64::from(rust.rewrite_count),
                "rewrite_count mismatch: c={c_rewrites}, rust={}",
                rust.rewrite_count
            );
            if rust.rewrite_count == 0 || all_targets_already_ten_digits(input) {
                let expected = normalize_c_padding_gap_nobits_for_rust_diff(input, &c_bytes);
                assert_eq!(expected, rust.bytes, "same-width C vs Rust output divergence");
            }
            let rust_of_c = canonicalize_local_elf_symbol_names(&c_bytes)
                .expect("Rust must accept C-canonicalized output when Rust accepted input");
            assert_eq!(rust_of_c.bytes, rust.bytes, "Rust(C(input)) != Rust(input) divergence");
            verify_elf_structural_preservation(input, &rust.bytes, true);
            verify_elf_structural_preservation(input, &c_bytes, false);
        }
        (Ok(_rust), None) => {
            assert!(
                has_c_width_overflow(input),
                "Rust accepted ELF input that C rejected without decimal-width overflow"
            );
        }
        (Err(err), Some(_)) => {
            assert!(
                is_allowed_c_accept_rust_reject(input, &err),
                "unexpected C-accept / Rust-reject divergence: err={err}"
            );
        }
        (Err(_), None) => {}
    }
}

pub(crate) fn normalize_c_padding_gap_nobits_for_rust_diff(input: &[u8], c_bytes: &[u8]) -> Vec<u8> {
    let e_shoff = read_u64(input, 40) as usize;
    let e_shnum = usize::from(read_u16(input, 60));
    let mut str_idx = None;
    for idx in 0..e_shnum {
        let hdr = e_shoff + idx * ELF_SECTION_HEADER_BYTES;
        if read_u32(input, hdr + 4) == SHT_SYMTAB {
            str_idx = Some(read_u32(input, hdr + 40) as usize);
            break;
        }
    }
    let Some(str_idx) = str_idx else {
        return c_bytes.to_vec();
    };
    let str_hdr = e_shoff + str_idx * ELF_SECTION_HEADER_BYTES;
    let str_off = read_u64(input, str_hdr + 24) as usize;
    let str_size = read_u64(input, str_hdr + 32) as usize;
    let old_string_end = str_off + str_size;
    let mut boundary = input.len();
    if e_shoff > str_off && e_shoff <= boundary {
        boundary = e_shoff;
    }
    for idx in 0..e_shnum {
        let hdr = e_shoff + idx * ELF_SECTION_HEADER_BYTES;
        let sh_type = read_u32(input, hdr + 4);
        let sh_off = read_u64(input, hdr + 24) as usize;
        let sh_size = read_u64(input, hdr + 32) as usize;
        if idx == str_idx || sh_type == SHT_NOBITS || sh_size == 0 || sh_off <= str_off {
            continue;
        }
        if sh_off < boundary {
            boundary = sh_off;
        }
    }
    let mut expected = c_bytes.to_vec();
    for idx in 0..e_shnum {
        let hdr = e_shoff + idx * ELF_SECTION_HEADER_BYTES;
        let sh_type = read_u32(expected.as_slice(), hdr + 4);
        let sh_off = read_u64(expected.as_slice(), hdr + 24) as usize;
        let sh_size = read_u64(expected.as_slice(), hdr + 32) as usize;
        if (sh_type == SHT_NOBITS || sh_size == 0) && sh_off > old_string_end && sh_off < boundary {
            expected[hdr + 24..hdr + 32].copy_from_slice(&(old_string_end as u64).to_le_bytes());
        }
    }
    expected
}

pub fn verify_elf_structural_preservation(original: &[u8], canonical: &[u8], expect_10_digit_locals: bool) {
    let e_shoff_o = read_u64(original, 40) as usize;
    let e_shoff_c = read_u64(canonical, 40) as usize;
    let e_shnum = usize::from(read_u16(original, 60));
    assert_eq!(e_shnum, usize::from(read_u16(canonical, 60)));
    assert_eq!(&original[..16], &canonical[..16], "e_ident changed");
    assert_eq!(read_u16(original, 16), read_u16(canonical, 16));
    assert_eq!(read_u16(original, 18), read_u16(canonical, 18));
    assert_eq!(read_u32(original, 20), read_u32(canonical, 20));
    assert_eq!(read_u32(original, 48), read_u32(canonical, 48));
    assert_eq!(read_u16(original, 52), read_u16(canonical, 52));
    assert_eq!(read_u16(original, 62), read_u16(canonical, 62));

    let mut strtab_idx = None;
    let mut symtab_idx = None;
    for idx in 0..e_shnum {
        let ho = e_shoff_o + idx * ELF_SECTION_HEADER_BYTES;
        if read_u32(original, ho + 4) == SHT_SYMTAB {
            symtab_idx = Some(idx);
            strtab_idx = Some(read_u32(original, ho + 40) as usize);
            break;
        }
    }
    for idx in 0..e_shnum {
        let ho = e_shoff_o + idx * ELF_SECTION_HEADER_BYTES;
        let hc = e_shoff_c + idx * ELF_SECTION_HEADER_BYTES;
        let sh_type = read_u32(original, ho + 4);
        assert_eq!(read_u32(original, ho), read_u32(canonical, hc), "sh_name changed");
        assert_eq!(sh_type, read_u32(canonical, hc + 4), "sh_type changed");
        assert_eq!(read_u64(original, ho + 8), read_u64(canonical, hc + 8), "sh_flags changed");
        assert_eq!(read_u64(original, ho + 16), read_u64(canonical, hc + 16), "sh_addr changed");
        assert_eq!(read_u32(original, ho + 40), read_u32(canonical, hc + 40), "sh_link changed");
        assert_eq!(read_u32(original, ho + 44), read_u32(canonical, hc + 44), "sh_info changed");
        assert_eq!(read_u64(original, ho + 48), read_u64(canonical, hc + 48), "sh_addralign changed");
        assert_eq!(read_u64(original, ho + 56), read_u64(canonical, hc + 56), "sh_entsize changed");
        if sh_type != SHT_NOBITS {
            let so_off = read_u64(original, ho + 24) as usize;
            let sc_off = read_u64(canonical, hc + 24) as usize;
            let size = read_u64(original, ho + 32) as usize;
            if Some(idx) != symtab_idx && Some(idx) != strtab_idx && size > 0 {
                assert_eq!(
                    &original[so_off..so_off + size],
                    &canonical[sc_off..sc_off + size],
                    "raw section {idx} payload mutated"
                );
            }
        }
    }
    if let Some(s_idx) = strtab_idx {
        let ho = e_shoff_o + s_idx * ELF_SECTION_HEADER_BYTES;
        let hc = e_shoff_c + s_idx * ELF_SECTION_HEADER_BYTES;
        assert_eq!(read_u32(original, ho + 4), SHT_STRTAB);
        assert_eq!(read_u32(canonical, hc + 4), SHT_STRTAB);
    }

    let orig_parsed = object::File::parse(original);
    let canon_parsed = object::File::parse(canonical);
    assert_eq!(
        orig_parsed.is_ok(),
        canon_parsed.is_ok(),
        "object::File::parse agreement changed across canonicalization"
    );
    let (Ok(obj_orig), Ok(obj_canon)) = (orig_parsed, canon_parsed) else {
        return;
    };

    assert_eq!(obj_orig.architecture(), obj_canon.architecture());
    assert_eq!(obj_orig.endianness(), obj_canon.endianness());
    assert_eq!(obj_orig.kind(), obj_canon.kind());

    let orig_sections: Vec<_> = obj_orig.sections().collect();
    let canon_sections: Vec<_> = obj_canon.sections().collect();
    assert_eq!(orig_sections.len(), canon_sections.len(), "section count changed");

    for (so, sc) in orig_sections.iter().zip(canon_sections.iter()) {
        let idx = so.index().0;
        assert_eq!(idx, sc.index().0, "section index changed");
        assert_eq!(so.name().ok(), sc.name().ok(), "section name changed at {idx}");
        if Some(idx) == strtab_idx || Some(idx) == symtab_idx {
            continue;
        }
        let ho = e_shoff_o + idx * ELF_SECTION_HEADER_BYTES;
        let sh_type = read_u32(original, ho + 4);
        if sh_type == SHT_NOBITS {
            assert_eq!(
                read_u64(original, ho + 32),
                read_u64(canonical, e_shoff_c + idx * ELF_SECTION_HEADER_BYTES + 32)
            );
            continue;
        }
        assert_eq!(
            so.data().unwrap(),
            sc.data().unwrap(),
            "non-symtab/non-strtab section {idx} ({:?}) payload mutated",
            so.name()
        );
        if sh_type == SHT_RELA || sh_type == SHT_GROUP {
            assert_eq!(so.data().unwrap(), sc.data().unwrap());
        }
    }

    let orig_syms: Vec<_> = obj_orig.symbols().collect();
    let canon_syms: Vec<_> = obj_canon.symbols().collect();
    assert_eq!(orig_syms.len(), canon_syms.len(), "symbol count changed");
    for (so, sc) in orig_syms.iter().zip(canon_syms.iter()) {
        let idx = so.index().0;
        assert_eq!(idx, sc.index().0, "symbol index changed");
        assert_eq!(so.address(), sc.address());
        assert_eq!(so.size(), sc.size());
        assert_eq!(so.kind(), sc.kind());
        assert_eq!(so.scope(), sc.scope());
        assert_eq!(so.is_local(), sc.is_local());
        assert_eq!(so.section_index(), sc.section_index());
        let name_o = so.name().unwrap_or("");
        let name_c = sc.name().unwrap_or("");
        let is_target = so.is_local()
            && name_o.len() > 2
            && name_o.starts_with("L.")
            && name_o[2..].bytes().all(|b| b.is_ascii_digit());
        if is_target {
            if expect_10_digit_locals {
                assert_eq!(name_c, format!("L.{idx:010}"));
            } else {
                let width = name_o.len() - 2;
                assert_eq!(name_c, format!("L.{idx:0width$}"));
            }
        } else {
            assert_eq!(name_o, name_c, "non-target symbol name changed at index {idx}");
        }
    }
}

pub fn all_targets_already_ten_digits(bytes: &[u8]) -> bool {
    if bytes.len() < ELF_HEADER_BYTES {
        return false;
    }
    let e_shoff = read_u64(bytes, 40) as usize;
    let e_shnum = usize::from(read_u16(bytes, 60));
    for idx in 0..e_shnum {
        let off = e_shoff + idx * ELF_SECTION_HEADER_BYTES;
        if off + ELF_SECTION_HEADER_BYTES > bytes.len() {
            return false;
        }
        if read_u32(bytes, off + 4) != SHT_SYMTAB {
            continue;
        }
        let sym_off = read_u64(bytes, off + 24) as usize;
        let sym_size = read_u64(bytes, off + 32) as usize;
        let str_idx = usize::from(read_u32(bytes, off + 40) as u16);
        let str_hdr = e_shoff + str_idx * ELF_SECTION_HEADER_BYTES;
        let str_off = read_u64(bytes, str_hdr + 24) as usize;
        let str_size = read_u64(bytes, str_hdr + 32) as usize;
        let strtab = &bytes[str_off..str_off + str_size];
        for i in 0..(sym_size / ELF_SYMBOL_BYTES) {
            let s = sym_off + i * ELF_SYMBOL_BYTES;
            let st_name = read_u32(bytes, s) as usize;
            let st_bind = bytes[s + 4] >> 4;
            if st_bind != STB_LOCAL || st_name >= str_size {
                continue;
            }
            let end = strtab[st_name..].iter().position(|&b| b == 0).unwrap_or(strtab.len() - st_name);
            let name = &strtab[st_name..st_name + end];
            if name.len() > 2 && name.starts_with(b"L.") && name[2..].iter().all(u8::is_ascii_digit) && name.len() != 12
            {
                return false;
            }
        }
    }
    true
}

fn has_c_width_overflow(bytes: &[u8]) -> bool {
    if bytes.len() < ELF_HEADER_BYTES {
        return false;
    }
    let e_shoff = read_u64(bytes, 40) as usize;
    let e_shnum = usize::from(read_u16(bytes, 60));
    for idx in 0..e_shnum {
        let off = e_shoff + idx * ELF_SECTION_HEADER_BYTES;
        if off + ELF_SECTION_HEADER_BYTES > bytes.len() {
            return false;
        }
        if read_u32(bytes, off + 4) != SHT_SYMTAB {
            continue;
        }
        let sym_off = read_u64(bytes, off + 24) as usize;
        let sym_size = read_u64(bytes, off + 32) as usize;
        let str_idx = usize::from(read_u32(bytes, off + 40) as u16);
        let str_hdr = e_shoff + str_idx * ELF_SECTION_HEADER_BYTES;
        let str_off = read_u64(bytes, str_hdr + 24) as usize;
        let str_size = read_u64(bytes, str_hdr + 32) as usize;
        let strtab = &bytes[str_off..str_off + str_size];
        let sym_count = sym_size / ELF_SYMBOL_BYTES;
        for i in 0..sym_count {
            let s = sym_off + i * ELF_SYMBOL_BYTES;
            let st_name = read_u32(bytes, s) as usize;
            let st_bind = bytes[s + 4] >> 4;
            if st_bind != STB_LOCAL || st_name >= str_size {
                continue;
            }
            let end = strtab[st_name..].iter().position(|&b| b == 0).unwrap_or(strtab.len() - st_name);
            let name = &strtab[st_name..st_name + end];
            if name.len() > 2 && name.starts_with(b"L.") && name[2..].iter().all(u8::is_ascii_digit) {
                let width = name.len() - 2;
                let needed = if i == 0 { 1 } else { i.ilog10() as usize + 1 };
                if needed > width {
                    return true;
                }
            }
        }
    }
    false
}

fn is_allowed_c_accept_rust_reject(bytes: &[u8], err: &ElfCanonicalizationError) -> bool {
    if bytes.len() < ELF_HEADER_BYTES {
        return false;
    }
    let e_type = read_u16(bytes, 16);
    let e_phoff = read_u64(bytes, 32);
    let e_shoff = read_u64(bytes, 40) as usize;
    let e_phentsize = read_u16(bytes, 54);
    let e_phnum = read_u16(bytes, 56);
    let e_shnum = usize::from(read_u16(bytes, 60));
    let e_shstrndx = read_u16(bytes, 62);

    match err {
        ElfCanonicalizationError::ConflictingSymbolName => true,
        ElfCanonicalizationError::SymbolIndexTooWide => true,
        ElfCanonicalizationError::LimitExceeded(_) => {
            if e_shnum > RUST_SECTION_COUNT_MAX {
                return true;
            }
            for idx in 0..e_shnum {
                let off = e_shoff + idx * ELF_SECTION_HEADER_BYTES;
                if off + ELF_SECTION_HEADER_BYTES <= bytes.len() && read_u32(bytes, off + 4) == SHT_SYMTAB {
                    let sym_size = read_u64(bytes, off + 32) as usize;
                    let str_idx = read_u32(bytes, off + 40) as usize;
                    if sym_size / ELF_SYMBOL_BYTES > RUST_SYMBOL_COUNT_MAX {
                        return true;
                    }
                    if str_idx < e_shnum {
                        let str_hdr = e_shoff + str_idx * ELF_SECTION_HEADER_BYTES;
                        if str_hdr + ELF_SECTION_HEADER_BYTES <= bytes.len() {
                            let str_off = read_u64(bytes, str_hdr + 24) as usize;
                            let str_size = read_u64(bytes, str_hdr + 32) as usize;
                            if str_size > RUST_STRING_TABLE_BYTES_MAX
                                || str_size.saturating_mul(12) > RUST_STRING_TABLE_BYTES_MAX
                            {
                                return true;
                            }
                            if str_off.checked_add(str_size).is_some_and(|end| end <= bytes.len()) {
                                let strtab = &bytes[str_off..str_off + str_size];
                                let mut cursor = 0usize;
                                let mut entry_count = 0usize;
                                while cursor < str_size {
                                    entry_count += 1;
                                    if entry_count > RUST_SYMBOL_COUNT_MAX {
                                        return true;
                                    }
                                    let len =
                                        strtab[cursor..].iter().position(|&b| b == 0).unwrap_or(str_size - cursor);
                                    cursor += len + 1;
                                }
                            }
                        }
                    }
                }
            }
            bytes.len() > RUST_STRING_TABLE_BYTES_MAX
        }
        ElfCanonicalizationError::OutOfBounds(_) => {
            for idx in 0..e_shnum {
                let off = e_shoff + idx * ELF_SECTION_HEADER_BYTES;
                if off + ELF_SECTION_HEADER_BYTES > bytes.len() {
                    return true;
                }
                let sh_type = read_u32(bytes, off + 4);
                let sh_off = read_u64(bytes, off + 24);
                let sh_size = read_u64(bytes, off + 32);
                if sh_off.checked_add(bytes.len() as u64 * 12).is_none() {
                    return true;
                }
                if sh_type != SHT_NOBITS && sh_size > 0 {
                    let Some(end) = sh_off.checked_add(sh_size) else {
                        return true;
                    };
                    if end > bytes.len() as u64 {
                        return true;
                    }
                }
            }
            false
        }
        ElfCanonicalizationError::UnsupportedFormat(_) => {
            if e_type != ELF_TYPE_REL || e_phoff != 0 || e_phentsize != 0 || e_phnum != 0 || e_shstrndx == SHN_XINDEX {
                return true;
            }
            let Some(sh_table_end) =
                e_shoff.checked_add(e_shnum * ELF_SECTION_HEADER_BYTES).filter(|&end| end <= bytes.len())
            else {
                return false;
            };
            let symtab_indices: Vec<usize> = (0..e_shnum)
                .filter(|&idx| read_u32(bytes, e_shoff + idx * ELF_SECTION_HEADER_BYTES + 4) == SHT_SYMTAB)
                .collect();
            if symtab_indices.len() != 1 {
                return true;
            }
            let sym_idx = symtab_indices[0];
            let sym_hdr = e_shoff + sym_idx * ELF_SECTION_HEADER_BYTES;
            let sym_off = read_u64(bytes, sym_hdr + 24) as usize;
            let sym_size = read_u64(bytes, sym_hdr + 32) as usize;
            let sym_entsize = read_u64(bytes, sym_hdr + 56) as usize;
            let str_idx = read_u32(bytes, sym_hdr + 40) as usize;
            if sym_size == 0
                || sym_entsize != ELF_SYMBOL_BYTES
                || !sym_size.is_multiple_of(ELF_SYMBOL_BYTES)
                || str_idx >= e_shnum
                || usize::from(e_shstrndx) == str_idx
            {
                return true;
            }
            let str_hdr = e_shoff + str_idx * ELF_SECTION_HEADER_BYTES;
            if read_u32(bytes, str_hdr + 4) != SHT_STRTAB {
                return true;
            }
            let str_off = read_u64(bytes, str_hdr + 24) as usize;
            let str_size = read_u64(bytes, str_hdr + 32) as usize;
            if str_size == 0 {
                return true;
            }
            let Some(str_end) = str_off.checked_add(str_size).filter(|&end| end <= bytes.len()) else {
                return false;
            };
            if bytes[str_off] != 0 || bytes[str_end - 1] != 0 {
                return true;
            }
            let Some(sym_end) = sym_off.checked_add(sym_size).filter(|&end| end <= bytes.len()) else {
                return false;
            };
            if e_shoff < ELF_HEADER_BYTES || str_off < ELF_HEADER_BYTES {
                return true;
            }
            if str_off < sh_table_end && e_shoff < str_end {
                return true;
            }
            for idx in 0..e_shnum {
                let hdr = e_shoff + idx * ELF_SECTION_HEADER_BYTES;
                let sh_type = read_u32(bytes, hdr + 4);
                let sh_off = read_u64(bytes, hdr + 24) as usize;
                let sh_size = read_u64(bytes, hdr + 32) as usize;
                let sh_link = read_u32(bytes, hdr + 40) as usize;
                if idx != sym_idx && sh_link == str_idx {
                    return true;
                }
                if sh_type == SHT_NOBITS || sh_size == 0 {
                    if sh_off > str_off && sh_off < str_end {
                        return true;
                    }
                    continue;
                }
                let Some(sec_end) = sh_off.checked_add(sh_size).filter(|&end| end <= bytes.len()) else {
                    return false;
                };
                if sh_off < ELF_HEADER_BYTES {
                    return true;
                }
                if sh_off < sh_table_end && e_shoff < sec_end {
                    return true;
                }
                if idx != str_idx && str_off < sec_end && sh_off < str_end {
                    return true;
                }
                if idx != sym_idx && sym_off < sec_end && sh_off < sym_end {
                    return true;
                }
            }
            // Tail sharing, out-of-range/non-entry st_name, or >10-digit local symbol.
            let strtab = &bytes[str_off..str_end];
            let mut entry_starts = BTreeSet::new();
            let mut cursor = 0usize;
            while cursor < str_size {
                entry_starts.insert(cursor);
                let len = strtab[cursor..].iter().position(|&b| b == 0).unwrap_or(str_size - cursor);
                cursor += len + 1;
            }
            for s_idx in 0..(sym_size / ELF_SYMBOL_BYTES) {
                let s = sym_off + s_idx * ELF_SYMBOL_BYTES;
                let st_name = read_u32(bytes, s) as usize;
                let st_bind = bytes[s + 4] >> 4;
                if st_name >= str_size || !entry_starts.contains(&st_name) {
                    return true;
                }
                if st_bind == STB_LOCAL {
                    let len = strtab[st_name..].iter().position(|&b| b == 0).unwrap_or(0);
                    let name = &strtab[st_name..st_name + len];
                    if name.len() > 12 && name.starts_with(b"L.") && name[2..].iter().all(u8::is_ascii_digit) {
                        return true;
                    }
                }
            }
            // Following boundary alignment validity and canonical trailing zero padding after .strtab.
            let mut boundary = bytes.len();
            let mut alignment_bytes = 1usize;
            if e_shoff > str_off && e_shoff <= boundary {
                boundary = e_shoff;
                alignment_bytes = 8;
            }
            for idx in 0..e_shnum {
                let hdr = e_shoff + idx * ELF_SECTION_HEADER_BYTES;
                let sh_type = read_u32(bytes, hdr + 4);
                let sh_off = read_u64(bytes, hdr + 24) as usize;
                let sh_size = read_u64(bytes, hdr + 32) as usize;
                let sh_align = read_u64(bytes, hdr + 48) as usize;
                if idx == str_idx || sh_type == SHT_NOBITS || sh_size == 0 || sh_off <= str_off {
                    continue;
                }
                if sh_align != 0 && (!sh_align.is_power_of_two() || sh_align > 4_096) {
                    return true;
                }
                let effective_align = sh_align.max(1);
                if sh_off < boundary {
                    boundary = sh_off;
                    alignment_bytes = effective_align;
                } else if sh_off == boundary {
                    alignment_bytes = alignment_bytes.max(effective_align);
                }
            }
            if boundary < str_end {
                return true;
            }
            let mask = alignment_bytes.max(1) - 1;
            let Some(expected_replacement_end) = str_end.checked_add(mask).map(|v| v & !mask) else {
                return true;
            };
            boundary != expected_replacement_end || bytes[str_end..boundary].iter().any(|&b| b != 0)
        }
    }
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap())
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}

#[derive(Clone, Debug)]
pub struct SymbolSpec {
    pub name: Vec<u8>,
    pub local: bool,
}

pub fn build_synthetic_elf(symbols: &[SymbolSpec], include_nobits: bool, extra_pad: usize) -> Vec<u8> {
    let mut strtab = vec![0u8];
    let mut name_offsets = Vec::with_capacity(symbols.len() + 1);
    name_offsets.push(0u32);
    for sym in symbols {
        name_offsets.push(u32::try_from(strtab.len()).unwrap());
        strtab.extend_from_slice(&sym.name);
        strtab.push(0);
    }

    let shstrtab = b"\0.text\0.rela.text\0.bss\0.symtab\0.strtab\0.shstrtab\0";
    let text_bytes = [0x90u8, 0xc3, 0x90, 0xc3, 0x00, 0x00, 0x00, 0x00];
    // One R_X86_64_PC32 relocation referencing symbol 1 (if present) or 0.
    let reloc_sym = if symbols.is_empty() { 0u64 } else { 1u64 };
    let mut rela_bytes = Vec::with_capacity(24);
    rela_bytes.extend_from_slice(&0u64.to_le_bytes()); // r_offset
    rela_bytes.extend_from_slice(&((reloc_sym << 32) | 2u64).to_le_bytes()); // r_info (R_X86_64_PC32)
    rela_bytes.extend_from_slice(&(-4i64).to_le_bytes()); // r_addend

    let mut symtab = vec![0u8; ELF_SYMBOL_BYTES]; // STN_UNDEF
    for (idx, sym) in symbols.iter().enumerate() {
        let mut entry = [0u8; ELF_SYMBOL_BYTES];
        entry[0..4].copy_from_slice(&name_offsets[idx + 1].to_le_bytes());
        entry[4] = if sym.local { 0 } else { 1 << 4 };
        entry[6..8].copy_from_slice(&1u16.to_le_bytes()); // shndx = 1 (.text)
        symtab.extend_from_slice(&entry);
    }

    let mut out = vec![0u8; ELF_HEADER_BYTES];
    out[..4].copy_from_slice(b"\x7fELF");
    out[4] = 2; // ELFCLASS64
    out[5] = 1; // ELFDATA2LSB
    out[6] = 1; // EV_CURRENT
    out[16..18].copy_from_slice(&1u16.to_le_bytes()); // ET_REL
    out[18..20].copy_from_slice(&62u16.to_le_bytes()); // EM_X86_64
    out[20..24].copy_from_slice(&1u32.to_le_bytes()); // EV_CURRENT
    out[52..54].copy_from_slice(&(ELF_HEADER_BYTES as u16).to_le_bytes());
    out[58..60].copy_from_slice(&(ELF_SECTION_HEADER_BYTES as u16).to_le_bytes());

    let text_off = out.len();
    out.extend_from_slice(&text_bytes);
    let rela_off = out.len();
    out.extend_from_slice(&rela_bytes);
    let bss_off = out.len();
    let symtab_off = out.len();
    out.extend_from_slice(&symtab);
    let shstrtab_off = out.len();
    out.extend_from_slice(shstrtab);
    let strtab_off = out.len();
    out.extend_from_slice(&strtab);
    let aligned_shoff = (out.len() + extra_pad + 7) & !7;
    out.resize(aligned_shoff, 0);

    let mut shdrs: Vec<[u8; ELF_SECTION_HEADER_BYTES]> = Vec::new();
    // 0: SHT_NULL
    shdrs.push([0u8; ELF_SECTION_HEADER_BYTES]);
    // 1: .text
    shdrs.push(make_shdr(SectionHeaderSpec {
        sh_name: 1,
        sh_type: 1,
        sh_flags: 6,
        sh_offset: text_off,
        sh_size: text_bytes.len(),
        sh_link: 0,
        sh_info: 0,
        sh_addralign: 1,
        sh_entsize: 0,
    }));
    // 2: .rela.text (sh_link = 4 or 3 depending on include_nobits, sh_info = 1)
    let sym_idx = if include_nobits { 4 } else { 3 };
    let shstr_idx = sym_idx + 1;
    let str_idx = sym_idx + 2;
    shdrs.push(make_shdr(SectionHeaderSpec {
        sh_name: 7,
        sh_type: SHT_RELA,
        sh_flags: 0x40,
        sh_offset: rela_off,
        sh_size: rela_bytes.len(),
        sh_link: sym_idx as u32,
        sh_info: 1,
        sh_addralign: 8,
        sh_entsize: 24,
    }));
    if include_nobits {
        // 3: .bss
        shdrs.push(make_shdr(SectionHeaderSpec {
            sh_name: 18,
            sh_type: SHT_NOBITS,
            sh_flags: 3,
            sh_offset: bss_off,
            sh_size: 16,
            sh_link: 0,
            sh_info: 0,
            sh_addralign: 8,
            sh_entsize: 0,
        }));
    }
    // symtab
    let first_non_local = symbols.iter().take_while(|s| s.local).count() + 1;
    shdrs.push(make_shdr(SectionHeaderSpec {
        sh_name: 23,
        sh_type: SHT_SYMTAB,
        sh_flags: 0,
        sh_offset: symtab_off,
        sh_size: symtab.len(),
        sh_link: str_idx as u32,
        sh_info: first_non_local as u32,
        sh_addralign: 8,
        sh_entsize: 24,
    }));
    // shstrtab
    shdrs.push(make_shdr(SectionHeaderSpec {
        sh_name: 39,
        sh_type: SHT_STRTAB,
        sh_flags: 0,
        sh_offset: shstrtab_off,
        sh_size: shstrtab.len(),
        sh_link: 0,
        sh_info: 0,
        sh_addralign: 1,
        sh_entsize: 0,
    }));
    // strtab
    shdrs.push(make_shdr(SectionHeaderSpec {
        sh_name: 31,
        sh_type: SHT_STRTAB,
        sh_flags: 0,
        sh_offset: strtab_off,
        sh_size: strtab.len(),
        sh_link: 0,
        sh_info: 0,
        sh_addralign: 1,
        sh_entsize: 0,
    }));

    let shoff = out.len();
    for shdr in &shdrs {
        out.extend_from_slice(shdr);
    }
    out[40..48].copy_from_slice(&(shoff as u64).to_le_bytes());
    out[60..62].copy_from_slice(&(shdrs.len() as u16).to_le_bytes());
    out[62..64].copy_from_slice(&(shstr_idx as u16).to_le_bytes());
    out
}

struct SectionHeaderSpec {
    sh_name: u32,
    sh_type: u32,
    sh_flags: u64,
    sh_offset: usize,
    sh_size: usize,
    sh_link: u32,
    sh_info: u32,
    sh_addralign: u64,
    sh_entsize: u64,
}

fn make_shdr(spec: SectionHeaderSpec) -> [u8; ELF_SECTION_HEADER_BYTES] {
    let mut s = [0u8; ELF_SECTION_HEADER_BYTES];
    s[0..4].copy_from_slice(&spec.sh_name.to_le_bytes());
    s[4..8].copy_from_slice(&spec.sh_type.to_le_bytes());
    s[8..16].copy_from_slice(&spec.sh_flags.to_le_bytes());
    s[24..32].copy_from_slice(&(spec.sh_offset as u64).to_le_bytes());
    s[32..40].copy_from_slice(&(spec.sh_size as u64).to_le_bytes());
    s[40..44].copy_from_slice(&spec.sh_link.to_le_bytes());
    s[44..48].copy_from_slice(&spec.sh_info.to_le_bytes());
    s[48..56].copy_from_slice(&spec.sh_addralign.to_le_bytes());
    s[56..64].copy_from_slice(&spec.sh_entsize.to_le_bytes());
    s
}
