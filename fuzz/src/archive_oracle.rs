use object::read::archive::ArchiveFile;

use crate::elf_local_symbol_core::canonicalize_local_elf_symbol_names;
use crate::elf_oracle::all_targets_already_ten_digits;
use crate::elf_oracle::c_canonicalize_elf;
use crate::elf_oracle::normalize_c_padding_gap_nobits_for_rust_diff;
use crate::stagex_archive_core::canonicalize_stagex_archive_elf_members;

const ARCHIVE_MAGIC: &[u8] = b"!<arch>\n";
const ARCHIVE_HEADER_BYTES: usize = 60;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveCheckOutput {
    pub bytes: Vec<u8>,
    pub member_count: u32,
    pub rewritten_member_count: u32,
    pub rewritten_symbol_count: u32,
}

#[derive(Debug, Clone)]
pub struct ParsedArMember<'a> {
    pub raw_name: &'a str,
    pub payload: &'a [u8],
}

pub fn check_archive_invariants(input: &[u8]) -> Result<ArchiveCheckOutput, String> {
    let res = canonicalize_stagex_archive_elf_members(input);
    if let Ok(ref canon1) = res {
        verify_archive_geometry(&canon1.bytes);
        let canon2 = canonicalize_stagex_archive_elf_members(&canon1.bytes)
            .expect("canonicalized archive must succeed on second pass");
        assert_eq!(canon2.bytes, canon1.bytes, "archive canonicalization is not idempotent");
        assert_eq!(canon2.member_count, canon1.member_count);
        assert_eq!(canon2.rewritten_member_count, 0);
        assert_eq!(canon2.rewritten_symbol_count, canon1.rewritten_symbol_count);

        let in_members = parse_raw_ar_members(input);
        let out_members = parse_raw_ar_members(&canon1.bytes);
        assert_eq!(in_members.len(), out_members.len());
        let mut total_rewrites = 0u32;
        for (mi, mo) in in_members.iter().zip(out_members.iter()) {
            assert_eq!(mi.raw_name, mo.raw_name);
            if mi.raw_name == "//" {
                assert_eq!(mi.payload, mo.payload);
            } else {
                let elf_canon = canonicalize_local_elf_symbol_names(mi.payload).unwrap();
                assert_eq!(elf_canon.bytes, mo.payload);
                total_rewrites += elf_canon.rewrite_count;
                if let Some((c_bytes, c_rewrites)) = c_canonicalize_elf(mi.payload) {
                    assert_eq!(c_rewrites, u64::from(elf_canon.rewrite_count));
                    if elf_canon.rewrite_count == 0 || all_targets_already_ten_digits(mi.payload) {
                        let expected = normalize_c_padding_gap_nobits_for_rust_diff(mi.payload, &c_bytes);
                        assert_eq!(expected, mo.payload);
                    }
                    let rust_of_c = canonicalize_local_elf_symbol_names(&c_bytes).unwrap();
                    assert_eq!(rust_of_c.bytes, mo.payload);
                }
            }
        }
        assert_eq!(total_rewrites, canon1.rewritten_symbol_count);
    }
    res.map(|c| ArchiveCheckOutput {
        bytes: c.bytes,
        member_count: c.member_count,
        rewritten_member_count: c.rewritten_member_count,
        rewritten_symbol_count: c.rewritten_symbol_count,
    })
    .map_err(|e| e.to_string())
}

pub fn verify_archive_geometry(bytes: &[u8]) {
    assert!(bytes.starts_with(ARCHIVE_MAGIC));
    let _parsed = ArchiveFile::parse(bytes).expect("object::read::archive parses canonical archive");
    let mut cursor = ARCHIVE_MAGIC.len();
    let mut computed_len = ARCHIVE_MAGIC.len();
    while cursor < bytes.len() {
        assert!(cursor + ARCHIVE_HEADER_BYTES <= bytes.len());
        let hdr = &bytes[cursor..cursor + ARCHIVE_HEADER_BYTES];
        assert_eq!(&hdr[16..28], b"0           ");
        assert_eq!(&hdr[28..34], b"0     ");
        assert_eq!(&hdr[34..40], b"0     ");
        assert_eq!(&hdr[40..48], b"100644  ");
        assert_eq!(&hdr[58..60], b"`\n");
        let size_str = std::str::from_utf8(&hdr[48..58]).unwrap().trim_end_matches(' ');
        let size: usize = size_str.parse().unwrap();
        cursor += ARCHIVE_HEADER_BYTES + size;
        computed_len += ARCHIVE_HEADER_BYTES + size + (size % 2);
        if size % 2 == 1 {
            assert_eq!(bytes[cursor], b'\n');
            cursor += 1;
        }
    }
    assert_eq!(cursor, bytes.len());
    assert_eq!(computed_len, bytes.len());
}

pub fn parse_raw_ar_members(bytes: &[u8]) -> Vec<ParsedArMember<'_>> {
    assert!(bytes.starts_with(ARCHIVE_MAGIC));
    let mut members = Vec::new();
    let mut cursor = ARCHIVE_MAGIC.len();
    while cursor < bytes.len() {
        let hdr = &bytes[cursor..cursor + ARCHIVE_HEADER_BYTES];
        let raw_name = std::str::from_utf8(&hdr[..16]).unwrap().trim_end_matches(' ');
        let size: usize = std::str::from_utf8(&hdr[48..58]).unwrap().trim_end_matches(' ').parse().unwrap();
        let start = cursor + ARCHIVE_HEADER_BYTES;
        let end = start + size;
        members.push(ParsedArMember {
            raw_name,
            payload: &bytes[start..end],
        });
        cursor = end + (size % 2);
    }
    members
}

pub fn build_stagex_archive(long_names: Option<&[u8]>, members: &[(&str, &[u8])]) -> Vec<u8> {
    let mut out = Vec::from(ARCHIVE_MAGIC);
    if let Some(table) = long_names {
        append_raw_ar_member(&mut out, "//", table);
    }
    for &(name, payload) in members {
        append_raw_ar_member(&mut out, name, payload);
    }
    out
}

pub fn append_raw_ar_member(out: &mut Vec<u8>, name: &str, payload: &[u8]) {
    let hdr = format!("{name:<16}0           0     0     100644  {:<10}`\n", payload.len());
    assert_eq!(hdr.len(), ARCHIVE_HEADER_BYTES);
    out.extend_from_slice(hdr.as_bytes());
    out.extend_from_slice(payload);
    if payload.len() % 2 == 1 {
        out.push(b'\n');
    }
}
