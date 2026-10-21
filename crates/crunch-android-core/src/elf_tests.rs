use super::*;

fn word(data: &mut [u8], offset: usize, value: u64, size: usize) {
    data[offset..offset + size].copy_from_slice(&value.to_le_bytes()[..size]);
}

fn fixture() -> Vec<u8> {
    let mut data = alloc::vec![0; 0x400];
    data[..7].copy_from_slice(b"\x7fELF\x02\x01\x01");
    for (offset, value, size) in [
        (16, 3, 2),
        (18, 183, 2),
        (20, 1, 4),
        (32, 64, 8),
        (40, 0x100, 8),
        (52, 64, 2),
        (54, 56, 2),
        (56, 2, 2),
        (58, 64, 2),
        (60, 4, 2),
        (64, 1, 4),
        (68, 5, 4),
        (96, 0x400, 8),
        (104, 0x400, 8),
        (112, 16384, 8),
        (120, 2, 4),
        (128, 0x200, 8),
        (136, 0x200, 8),
        (152, 96, 8),
        (160, 96, 8),
        (168, 8, 8),
    ] {
        word(&mut data, offset, value, size);
    }
    // Dynamic, dynsym and dynstr all describe bytes mapped by PT_LOAD.
    for (index, kind, address, size, link, stride) in [
        (1, 6, 0x200, 96, 3, 16),
        (2, 11, 0x260, 72, 3, 24),
        (3, 3, 0x300, 60, 0, 0),
    ] {
        let offset = 0x100 + index * 64;
        for (field, value, width) in [
            (4, kind, 4),
            (16, address, 8),
            (24, address, 8),
            (32, size, 8),
            (40, link, 4),
            (56, stride, 8),
        ] {
            word(&mut data, offset + field, value, width);
        }
    }
    for (index, tag, value) in [(0, 1, 1), (1, 5, 0x300), (2, 6, 0x260), (3, 10, 60), (4, 11, 24)] {
        word(&mut data, 0x200 + index * 16, tag, 8);
        word(&mut data, 0x208 + index * 16, value, 8);
    }
    let strings = b"\0libc.so\0Java_org_example_nativeapp_MainActivity_value\0puts\0";
    data[0x300..0x300 + strings.len()].copy_from_slice(strings);
    word(&mut data, 0x278, 9, 4);
    data[0x27c] = 0x12;
    word(&mut data, 0x27e, 1, 2);
    word(&mut data, 0x290, (9 + b"Java_org_example_nativeapp_MainActivity_value".len() + 1) as u64, 4);
    data[0x294] = 0x12;
    data
}

#[test]
fn inspects_jni_exports_and_platform_imports() {
    let parsed = inspect(&fixture(), true).unwrap();
    assert_eq!(parsed.needed, alloc::vec!["libc.so"]);
    assert!(parsed.exports.contains("Java_org_example_nativeapp_MainActivity_value"));
    assert!(parsed.imports.contains("puts"));
    assert!(!parsed.exports.contains("puts"));
}

#[test]
fn rejects_host_architecture_and_four_kib_native_loads() {
    let mut data = fixture();
    word(&mut data, 18, 62, 2);
    assert_eq!(inspect(&data, true), Err("expected AArch64 little-endian ELF64 shared object"));
    word(&mut data, 18, 183, 2);
    word(&mut data, 112, 4096, 8);
    assert_eq!(inspect(&data, true), Err("Android ELF load segment is not 16KiB aligned"));
    assert!(inspect(&data, false).is_ok());
}

#[test]
fn rejects_loader_section_disagreement_and_runtime_search_paths() {
    let mut data = fixture();
    word(&mut data, 0x208 + 2 * 16, 0x280, 8);
    assert_eq!(inspect(&data, true), Err("ELF dynamic address/size differs from symbol sections"));
    let mut data = fixture();
    word(&mut data, 0x200, 29, 8);
    assert_eq!(inspect(&data, true), Err("ELF RPATH/RUNPATH/text relocation is forbidden"));
}

#[test]
fn truncated_or_overflowing_tables_fail_without_panicking() {
    let data = fixture();
    for size in [0, 6, 18, 63, 119, 175, 511, 575, 650, 767, 800] {
        assert!(inspect(&data[..size], true).is_err(), "truncation at {size}");
    }
    let mut data = data;
    word(&mut data, 40, u64::MAX, 8);
    assert!(inspect(&data, true).is_err());
}

#[test]
fn reads_the_mapped_android_api_note_instead_of_assuming_a_version_label() {
    let mut data = fixture();
    let headers = data[0x100..0x200].to_vec();
    data.resize(0x540, 0);
    data[0x400..0x500].copy_from_slice(&headers);
    word(&mut data, 40, 0x400, 8);
    word(&mut data, 60, 5, 2);
    for (offset, value, size) in [
        (0x504, 7, 4),
        (0x510, 0x380, 8),
        (0x518, 0x380, 8),
        (0x520, 24, 8),
        (0x380, 8, 4),
        (0x384, 4, 4),
        (0x388, 1, 4),
        (0x394, 24, 4),
    ] {
        word(&mut data, offset, value, size);
    }
    data[0x38c..0x394].copy_from_slice(b"Android\0");
    assert_eq!(inspect(&data, true).unwrap().android_api, Some(24));
    word(&mut data, 0x394, 35, 4);
    assert_eq!(inspect(&data, true).unwrap().android_api, Some(35));
}
