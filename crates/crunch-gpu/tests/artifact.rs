use crunch_gpu::inspect;

const TEXT: usize = 512;
const DESCRIPTOR: usize = 576;
const SYMTAB: usize = 640;
const STRTAB: usize = 736;
const RELA: usize = 776;
const NOTE: usize = 800;
const CODE_SYMBOL: usize = SYMTAB + 48;
const DESCRIPTOR_SYMBOL: usize = SYMTAB + 72;
const ENTRY: &str = "mantle_add_one";

fn put16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}
fn put32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}
fn put64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}
fn mp_string(value: &str) -> Vec<u8> {
    let mut bytes = vec![0xd9, u8::try_from(value.len()).unwrap()];
    bytes.extend_from_slice(value.as_bytes());
    bytes
}
fn mp_uint(value: u32) -> Vec<u8> {
    let mut bytes = vec![0xce];
    bytes.extend_from_slice(&value.to_be_bytes());
    bytes
}
fn mp_array(values: &[Vec<u8>]) -> Vec<u8> {
    let mut bytes = vec![0xdc];
    bytes.extend_from_slice(&u16::try_from(values.len()).unwrap().to_be_bytes());
    for value in values {
        bytes.extend_from_slice(value);
    }
    bytes
}
fn mp_map(values: &[(&str, Vec<u8>)]) -> Vec<u8> {
    let mut bytes = vec![0xde];
    bytes.extend_from_slice(&u16::try_from(values.len()).unwrap().to_be_bytes());
    for (key, value) in values {
        bytes.extend(mp_string(key));
        bytes.extend_from_slice(value);
    }
    bytes
}
fn kernel_metadata() -> Vec<(&'static str, Vec<u8>)> {
    vec![
        (".name", mp_string(ENTRY)),
        (".symbol", mp_string("mantle_add_one.kd")),
        (".kernarg_segment_size", mp_uint(8)),
        (".group_segment_fixed_size", mp_uint(0)),
        (".private_segment_fixed_size", mp_uint(0)),
        (".kernarg_segment_align", mp_uint(8)),
        (".wavefront_size", mp_uint(64)),
        (".sgpr_count", mp_uint(8)),
        (".vgpr_count", mp_uint(1)),
        (".max_flat_workgroup_size", mp_uint(256)),
    ]
}
fn metadata(kernels: &[Vec<u8>]) -> Vec<u8> {
    mp_map(&[
        ("amdhsa.version", mp_array(&[mp_uint(1), mp_uint(1)])),
        ("amdhsa.target", mp_string("amdgcn-amd-amdhsa--gfx900")),
        ("amdhsa.kernels", mp_array(kernels)),
    ])
}
fn section(bytes: &mut [u8], table: usize, index: usize, kind: u32, flags: u64, extent: (usize, usize), align: u64) {
    let at = table + index * 64;
    put32(bytes, at + 4, kind);
    put64(bytes, at + 8, flags);
    put64(bytes, at + 24, extent.0 as u64);
    put64(bytes, at + 32, extent.1 as u64);
    put64(bytes, at + 48, align);
}
// Deterministic encoding of the actual ABI, not a claim of compiler execution.
// Authority: ROCm/llvm-project rocm-7.2.3 AMDGPUUsage.rst (Code Object V4
// Metadata, Kernel Descriptor) and AMDGPUTargetStreamer.cpp:981-1046.
// Default-visible descriptors retain STV_DEFAULT, code becomes STV_PROTECTED;
// descriptor+16 has REL64(code, addend=16), yielding code - descriptor.
fn artifact(metadata: &[u8]) -> Vec<u8> {
    let note_size = 20 + ((metadata.len() + 3) & !3);
    let table = (NOTE + note_size + 7) & !7;
    let mut bytes = vec![0; table + 7 * 64];
    bytes[..9].copy_from_slice(b"\x7fELF\x02\x01\x01\x40\x02");
    put16(&mut bytes, 16, 1);
    put16(&mut bytes, 18, 224);
    put32(&mut bytes, 20, 1);
    put64(&mut bytes, 40, table as u64);
    put32(&mut bytes, 48, 0x12c); // gfx900, xnack any
    put16(&mut bytes, 52, 64);
    put16(&mut bytes, 58, 64);
    put16(&mut bytes, 60, 7);
    section(&mut bytes, table, 1, 1, 6, (TEXT, 4), 256);
    section(&mut bytes, table, 2, 1, 2, (DESCRIPTOR, 64), 64);
    section(&mut bytes, table, 3, 2, 0, (SYMTAB, 96), 8);
    put32(&mut bytes, table + 3 * 64 + 40, 4);
    put32(&mut bytes, table + 3 * 64 + 44, 2);
    put64(&mut bytes, table + 3 * 64 + 56, 24);
    section(&mut bytes, table, 4, 3, 0, (STRTAB, 34), 1);
    section(&mut bytes, table, 5, 4, 0, (RELA, 24), 8);
    put32(&mut bytes, table + 5 * 64 + 40, 3);
    put32(&mut bytes, table + 5 * 64 + 44, 2);
    put64(&mut bytes, table + 5 * 64 + 56, 24);
    section(&mut bytes, table, 6, 7, 0, (NOTE, note_size), 4);
    put32(&mut bytes, TEXT, 0xbf81_0000); // s_endpgm
    put32(&mut bytes, DESCRIPTOR + 8, 8);
    put32(&mut bytes, DESCRIPTOR + 48, 0x00ac_0000);
    put32(&mut bytes, DESCRIPTOR + 52, 0x84); // two user SGPRs, workgroup X
    put16(&mut bytes, DESCRIPTOR + 56, 8); // kernarg pointer
    bytes[SYMTAB + 24 + 4] = 3; // local STT_SECTION .text
    put16(&mut bytes, SYMTAB + 24 + 6, 1);
    put32(&mut bytes, CODE_SYMBOL, 1);
    bytes[CODE_SYMBOL + 4] = 0x12;
    bytes[CODE_SYMBOL + 5] = 3;
    put16(&mut bytes, CODE_SYMBOL + 6, 1);
    put64(&mut bytes, CODE_SYMBOL + 16, 4);
    put32(&mut bytes, DESCRIPTOR_SYMBOL, 16);
    bytes[DESCRIPTOR_SYMBOL + 4] = 0x11;
    put16(&mut bytes, DESCRIPTOR_SYMBOL + 6, 2);
    put64(&mut bytes, DESCRIPTOR_SYMBOL + 16, 64);
    bytes[STRTAB..STRTAB + 34].copy_from_slice(b"\0mantle_add_one\0mantle_add_one.kd\0");
    put64(&mut bytes, RELA, 16);
    put64(&mut bytes, RELA + 8, (2 << 32) | 5);
    put64(&mut bytes, RELA + 16, 16);
    put32(&mut bytes, NOTE, 7);
    put32(&mut bytes, NOTE + 4, metadata.len() as u32);
    put32(&mut bytes, NOTE + 8, 32);
    bytes[NOTE + 12..NOTE + 19].copy_from_slice(b"AMDGPU\0");
    bytes[NOTE + 20..NOTE + 20 + metadata.len()].copy_from_slice(metadata);
    bytes
}
fn valid() -> Vec<u8> {
    artifact(&metadata(&[mp_map(&kernel_metadata())]))
}
fn table(bytes: &[u8]) -> usize {
    u64::from_le_bytes(bytes[40..48].try_into().unwrap()) as usize
}
fn accepted(bytes: &[u8]) -> bool {
    inspect(bytes, &[ENTRY.into()]).is_ok()
}

#[test]
fn admits_abi4_descriptor_with_symbol_or_section_relative_entry() {
    let mut bytes = valid();
    let result = inspect(&bytes, &[ENTRY.into()]).unwrap();
    assert_eq!(result.entry_points, [ENTRY]);
    assert_eq!(result.architecture, "gfx900");
    // LLVM may retain the protected function or fold it to the .text symbol.
    put64(&mut bytes, RELA + 8, (1 << 32) | 5);
    assert!(accepted(&bytes));
    // Linkage arithmetic must not depend on the descriptor being at offset 0.
    let sh = table(&bytes) + 2 * 64;
    put64(&mut bytes, sh + 32, 128);
    // Move the descriptor and every subsequent file range, keeping alignments.
    bytes.splice(DESCRIPTOR..DESCRIPTOR, [0; 64]);
    let old_table = table(&bytes);
    put64(&mut bytes, 40, (old_table + 64) as u64);
    let new_table = table(&bytes);
    for index in 3..7 {
        let at = new_table + index * 64 + 24;
        let offset = u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap());
        put64(&mut bytes, at, offset + 64);
    }
    put64(&mut bytes, DESCRIPTOR_SYMBOL + 64 + 8, 64);
    put64(&mut bytes, RELA + 64, 80);
    assert!(accepted(&bytes));
}

#[test]
fn admits_hip_argument_metadata_and_explicit_xnack_modes() {
    let mut fields = kernel_metadata();
    fields.push((".language", mp_string("HIP")));
    fields.push((".language_version", mp_array(&[mp_uint(6), mp_uint(0)])));
    fields.push((
        ".args",
        mp_array(&[mp_map(&[
            (".name", mp_string("values")),
            (".type_name", mp_string("int*")),
            (".size", mp_uint(8)),
            (".offset", mp_uint(0)),
            (".value_kind", mp_string("global_buffer")),
            (".address_space", mp_string("global")),
            (".is_const", vec![0xc2]),
        ])]),
    ));
    for (mode, flags) in [("-", 0x22c), ("+", 0x32c)] {
        let packed = mp_map(&[
            ("amdhsa.version", mp_array(&[mp_uint(1), mp_uint(1)])),
            ("amdhsa.target", mp_string(&format!("amdgcn-amd-amdhsa--gfx900:xnack{mode}"))),
            ("amdhsa.kernels", mp_array(&[mp_map(&fields)])),
        ]);
        let mut bytes = artifact(&packed);
        put32(&mut bytes, 48, flags);
        assert!(accepted(&bytes));
        put32(&mut bytes, 48, 0x12c);
        assert!(!accepted(&bytes));
    }
}

#[test]
fn admits_pinned_llvm_dynamic_stack_property() {
    let mut bytes = valid();
    // ROCm 7.2.3 emits DynamicCallStack in bit 11 even for code object V4.
    put16(&mut bytes, DESCRIPTOR + 56, 8 | (1 << 11));
    assert!(accepted(&bytes));
}

fn text_relocation(kind: u32, offset: u64) -> Vec<u8> {
    let mut bytes = valid();
    let sh = table(&bytes);
    bytes.resize(bytes.len() + 64, 0);
    let at = bytes.len();
    bytes.resize(at + 24, 0);
    put16(&mut bytes, 60, 8);
    section(&mut bytes, sh, 7, 4, 0, (at, 24), 8);
    put32(&mut bytes, sh + 7 * 64 + 40, 3);
    put32(&mut bytes, sh + 7 * 64 + 44, 1);
    put64(&mut bytes, sh + 7 * 64 + 56, 24);
    put64(&mut bytes, at, offset);
    put64(&mut bytes, at + 8, (2 << 32) | u64::from(kind));
    bytes
}

#[test]
fn relocations_require_the_complete_target_field() {
    for kind in [1, 2, 4, 6, 7, 8, 9, 10, 11] {
        assert!(accepted(&text_relocation(kind, 0)), "four-byte relocation {kind}");
        assert!(!accepted(&text_relocation(kind, 1)), "partial four-byte relocation {kind}");
    }
    assert!(accepted(&text_relocation(14, 2)));
    assert!(!accepted(&text_relocation(14, 3)));
    for kind in [3, 5, 13] {
        assert!(!accepted(&text_relocation(kind, 0)), "eight-byte relocation {kind} in four-byte section");
    }
    assert!(!accepted(&text_relocation(5, u64::MAX)));
    assert!(!accepted(&text_relocation(12, 0)));
}

#[test]
fn rejects_helpers_with_short_or_unrelated_descriptor_data() {
    let mut bytes = valid();
    put64(&mut bytes, DESCRIPTOR_SYMBOL + 16, 4);
    assert!(!accepted(&bytes));
    let mut bytes = valid();
    // A syntactically valid descriptor pointing at the wrong instruction is not
    // an association with the advertised entry, even when its name matches.
    put64(&mut bytes, RELA + 16, 20);
    assert!(!accepted(&bytes));
    let mut bytes = valid();
    let sh = table(&bytes);
    put64(&mut bytes, sh + 5 * 64 + 32, 0);
    assert!(!accepted(&bytes));
}

#[test]
fn rejects_wrong_sections_extents_alignment_and_linkage() {
    let original = valid();
    let sh = table(&original);
    let mutations: &[(usize, usize, u64)] = &[
        (7, 1, 0),
        (8, 1, 3),
        (48, 4, 0x12d),
        (16, 2, 3),
        (sh + 64 + 4, 4, 8),
        (sh + 64 + 8, 8, 2),
        (sh + 64 + 48, 8, 4),
        (sh + 2 * 64 + 4, 4, 8),
        (sh + 2 * 64 + 8, 8, 3),
        (sh + 2 * 64 + 48, 8, 8),
        (CODE_SYMBOL + 4, 1, 0x1a),
        (CODE_SYMBOL + 5, 1, 0),
        (CODE_SYMBOL + 6, 2, 0),
        (CODE_SYMBOL + 6, 2, 3),
        (CODE_SYMBOL + 6, 2, 0xfff1),
        (CODE_SYMBOL + 8, 8, 4),
        (CODE_SYMBOL + 16, 8, 0),
        (CODE_SYMBOL + 16, 8, 8),
        (DESCRIPTOR_SYMBOL + 4, 1, 0x12),
        (DESCRIPTOR_SYMBOL + 6, 2, 1),
        (DESCRIPTOR_SYMBOL + 8, 8, 1),
        (DESCRIPTOR_SYMBOL + 16, 8, 63),
        (DESCRIPTOR_SYMBOL + 16, 8, 65),
        (DESCRIPTOR_SYMBOL + 8, 8, u64::MAX),
        (RELA, 8, 15),
        (RELA + 8, 8, (2 << 32) | 3),
        (RELA + 8, 8, (3 << 32) | 5),
        (RELA + 8, 8, (4 << 32) | 5),
        (RELA + 16, 8, u64::MAX),
        (sh + 5 * 64 + 40, 4, 4),
        (sh + 5 * 64 + 56, 8, 16),
        (sh + 5 * 64 + 32, 8, 23),
    ];
    for &(offset, size, value) in mutations {
        let mut bytes = original.clone();
        bytes[offset..offset + size].copy_from_slice(&value.to_le_bytes()[..size]);
        assert!(!accepted(&bytes), "accepted mutation at {offset}, width {size}, value {value}");
    }
}

#[test]
fn rejects_reserved_descriptor_bytes_and_invalid_resource_fields() {
    for offset in (12..16).chain(24..48).chain(58..64) {
        let mut bytes = valid();
        bytes[DESCRIPTOR + offset] = 1;
        assert!(!accepted(&bytes), "accepted reserved byte {offset}");
    }
    for (offset, mask) in [(48, 0xfb50_0c00_u32), (52, 0x80ff_e040)] {
        for bit in 0..32 {
            if mask & (1 << bit) != 0 {
                let mut bytes = valid();
                let value = u32::from_le_bytes(bytes[DESCRIPTOR + offset..DESCRIPTOR + offset + 4].try_into().unwrap());
                put32(&mut bytes, DESCRIPTOR + offset, value | (1 << bit));
                assert!(!accepted(&bytes), "accepted resource {offset} bit {bit}");
            }
        }
    }
    for bit in (7..16).filter(|bit| *bit != 11) {
        let mut bytes = valid();
        put16(&mut bytes, DESCRIPTOR + 56, 8 | (1 << bit));
        assert!(!accepted(&bytes));
    }
    let mut bytes = valid();
    put32(&mut bytes, DESCRIPTOR + 52, 0x80 | (3 << 11) | 4);
    assert!(!accepted(&bytes));
    let mut bytes = valid();
    put32(&mut bytes, DESCRIPTOR + 52, 0x82); // user count smaller than enabled kernarg pointer
    assert!(!accepted(&bytes));
}

#[test]
fn rejects_missing_conflicting_duplicate_or_malformed_metadata() {
    let mut bytes = valid();
    put32(&mut bytes, NOTE + 8, 33);
    assert!(!accepted(&bytes));
    let kernel = mp_map(&kernel_metadata());
    assert!(!accepted(&artifact(&metadata(&[kernel.clone(), kernel]))));
    for key in [
        ".symbol",
        ".kernarg_segment_size",
        ".wavefront_size",
        ".kernarg_segment_align",
    ] {
        let mut fields = kernel_metadata();
        fields.retain(|(name, _)| *name != key);
        assert!(!accepted(&artifact(&metadata(&[mp_map(&fields)]))));
    }
    for (key, value) in [
        (".symbol", mp_string("other.kd")),
        (".kernarg_segment_size", mp_uint(9)),
        (".group_segment_fixed_size", mp_uint(1)),
        (".private_segment_fixed_size", mp_uint(1)),
        (".wavefront_size", mp_uint(32)),
        (".kernarg_segment_align", mp_uint(3)),
        (".max_flat_workgroup_size", mp_uint(0)),
    ] {
        let mut fields = kernel_metadata();
        fields.iter_mut().find(|(name, _)| *name == key).unwrap().1 = value;
        assert!(!accepted(&artifact(&metadata(&[mp_map(&fields)]))), "accepted {key}");
    }
    let mut fields = kernel_metadata();
    fields.push((".symbol", mp_string("mantle_add_one.kd")));
    assert!(!accepted(&artifact(&metadata(&[mp_map(&fields)]))));
    let mut packed = metadata(&[mp_map(&kernel_metadata())]);
    packed.push(0xc0);
    assert!(!accepted(&artifact(&packed)));
    assert!(!accepted(&artifact(&[0xdd, 0xff, 0xff, 0xff, 0xff])));
    assert!(!accepted(&artifact(&[vec![0x91; 33], vec![0xc0]].concat())));
    for (key, value) in [
        (".args", mp_uint(0)),
        (".language_version", mp_array(&[mp_uint(2)])),
        (
            ".args",
            mp_array(&[mp_map(&[
                (".size", mp_uint(8)),
                (".offset", mp_uint(4)),
                (".value_kind", mp_string("global_buffer")),
            ])]),
        ),
    ] {
        let mut fields = kernel_metadata();
        fields.push((key, value));
        assert!(!accepted(&artifact(&metadata(&[mp_map(&fields)]))), "accepted malformed {key}");
    }
}

#[test]
fn rejects_ambiguous_symbols_overlapping_relocations_and_truncation() {
    let mut bytes = valid();
    put32(&mut bytes, SYMTAB + 24, 1); // second symbol with identical entry name
    assert!(!accepted(&bytes));
    let mut bytes = valid();
    // A relocation in the first descriptor byte as well as the entry relocation.
    let sh = table(&bytes);
    put64(&mut bytes, sh + 5 * 64 + 32, 48);
    let relocation = bytes[RELA..RELA + 24].to_vec();
    bytes.splice(RELA + 24..RELA + 24, relocation);
    put64(&mut bytes, 40, (sh + 24) as u64);
    put64(&mut bytes, sh + 24 + 6 * 64 + 24, (NOTE + 24) as u64);
    put64(&mut bytes, RELA + 24, 0);
    assert!(!accepted(&bytes));
    let bytes = valid();
    for length in 0..bytes.len() {
        assert!(!accepted(&bytes[..length]), "accepted truncated size {length}");
    }
    assert!(inspect(&bytes, &["missing".into()]).is_err());
    assert!(inspect(&bytes, &[]).is_err());
}

#[test]
fn rejects_original_name_only_fake_object() {
    // Exact pre-fix accepted object: both zero-sized symbols refer to SYMTAB,
    // and there is no code, descriptor, relocation, or AMDHSA metadata at all.
    let mut bytes = vec![0; 338];
    bytes[..6].copy_from_slice(b"\x7fELF\x02\x01");
    put16(&mut bytes, 16, 1);
    put16(&mut bytes, 18, 224);
    put64(&mut bytes, 40, 64);
    put32(&mut bytes, 48, 0x2c);
    put16(&mut bytes, 58, 64);
    put16(&mut bytes, 60, 3);
    put32(&mut bytes, 132, 2);
    put64(&mut bytes, 152, 256);
    put64(&mut bytes, 160, 48);
    put32(&mut bytes, 168, 2);
    put64(&mut bytes, 184, 24);
    put32(&mut bytes, 196, 3);
    put64(&mut bytes, 216, 304);
    put64(&mut bytes, 224, 34);
    put32(&mut bytes, 256, 1);
    bytes[260] = 0x12;
    put16(&mut bytes, 262, 1);
    put32(&mut bytes, 280, 16);
    bytes[284] = 0x11;
    put16(&mut bytes, 286, 1);
    bytes[304..338].copy_from_slice(b"\0mantle_add_one\0mantle_add_one.kd\0");
    assert!(!accepted(&bytes));
}
