use std::path::Path;
use std::process::Command;

use mantle_fuzz::elf_oracle::SymbolSpec;
use mantle_fuzz::elf_oracle::build_synthetic_elf;
use mantle_fuzz::elf_oracle::c_canonicalize_elf;
use mantle_fuzz::elf_oracle::c_canonicalize_file;
use mantle_fuzz::elf_oracle::check_elf_differential;
use mantle_fuzz::elf_oracle::check_elf_rust_invariants;
use mantle_fuzz::elf_oracle::verify_elf_structural_preservation;
use proptest::prelude::*;

fn replay_dir(dir: &Path, expect_valid: bool) {
    assert!(dir.exists(), "expected seed directory {}", dir.display());
    let mut entries: Vec<_> =
        std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()).filter(|p| p.is_file()).collect();
    entries.sort();
    assert!(!entries.is_empty(), "expected seed files in {}", dir.display());
    for path in entries {
        let bytes = std::fs::read(&path).unwrap();
        let rust_res = check_elf_rust_invariants(&bytes);
        if expect_valid {
            assert!(
                rust_res.is_ok(),
                "expected valid corpus seed {} to succeed, got {:?}",
                path.display(),
                rust_res.err()
            );
        } else {
            assert!(
                rust_res.is_err(),
                "expected regression seed {} to fail Rust canonicalization, got Ok",
                path.display()
            );
        }
        check_elf_differential(&bytes);
    }
}

#[test]
fn replays_elf_corpus_and_regression_seeds() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    replay_dir(&manifest_dir.join("corpus/elf"), true);
    replay_dir(&manifest_dir.join("regressions/elf"), false);
}

#[test]
fn c_file_wrapper_matches_in_memory_canonicalizer() {
    let elf = build_synthetic_elf(
        &[
            SymbolSpec {
                name: b"L.0000000009".to_vec(),
                local: true,
            },
            SymbolSpec {
                name: b"helper".to_vec(),
                local: false,
            },
        ],
        true,
        3,
    );
    let (mem_bytes, rewrites) = c_canonicalize_elf(&elf).unwrap();
    assert_eq!(rewrites, 1);

    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("sample.o");
    std::fs::write(&file_path, &elf).unwrap();
    let rc = c_canonicalize_file(&file_path);
    assert_eq!(rc, 0);
    let disk_bytes = std::fs::read(&file_path).unwrap();
    assert_eq!(disk_bytes, mem_bytes);
}

#[test]
fn elf_width_variants_converge_to_identical_bytes() {
    let narrow = build_synthetic_elf(
        &[
            SymbolSpec {
                name: b"L.1".to_vec(),
                local: true,
            },
            SymbolSpec {
                name: b"L.99".to_vec(),
                local: true,
            },
            SymbolSpec {
                name: b"public_fn".to_vec(),
                local: false,
            },
        ],
        true,
        0,
    );
    let wide = build_synthetic_elf(
        &[
            SymbolSpec {
                name: b"L.521018500".to_vec(),
                local: true,
            },
            SymbolSpec {
                name: b"L.1234567890".to_vec(),
                local: true,
            },
            SymbolSpec {
                name: b"public_fn".to_vec(),
                local: false,
            },
        ],
        true,
        0,
    );

    let c1 = check_elf_rust_invariants(&narrow).unwrap();
    let c2 = check_elf_rust_invariants(&wide).unwrap();
    assert_eq!(c1.bytes, c2.bytes);
    assert_eq!(c1.rewrite_count, 2);
    assert_eq!(check_elf_rust_invariants(&c1.bytes).unwrap().bytes, c1.bytes);
}

#[test]
fn farid_zakaria_semantic_reloc_comdat_and_odr_controls() {
    let dir = tempfile::tempdir().unwrap();

    // Valid compiler-regression control: assembly object with:
    // - Local decimal symbols L.1, L.42 referenced by .rela.text / .rela.text.comdat_fn
    // - SHT_GROUP (GRP_COMDAT) section group
    // - .rodata payload + non-target symbols
    let asm_src = r#"
        .text
        .globl semantic_compute
        .type semantic_compute, @function
semantic_compute:
        lea L.1(%rip), %rax
        mov (%rax), %eax
        add L.42(%rip), %eax
        add $7, %eax
        ret

        .section .text.comdat_helper,"axG",@progbits,comdat_grp,comdat
        .globl comdat_helper
        .type comdat_helper, @function
comdat_helper:
        mov L.99999999(%rip), %eax
        add $3, %eax
        ret

        .section .rodata
L.1:
        .long 19
L.42:
        .long 23
L.99999999:
        .long 11
"#;
    let main_src = r#"
#include <stdio.h>
int semantic_compute(void);
int comdat_helper(void);
int main(void) {
    printf("%d:%d\n", semantic_compute(), comdat_helper());
    return 0;
}
"#;
    let asm_path = dir.path().join("unit.s");
    let obj_orig_path = dir.path().join("unit_orig.o");
    let obj_rust_path = dir.path().join("unit_rust.o");
    let obj_c_path = dir.path().join("unit_c.o");
    let main_c_path = dir.path().join("main.c");
    let main_o_path = dir.path().join("main.o");

    std::fs::write(&asm_path, asm_src).unwrap();
    std::fs::write(&main_c_path, main_src).unwrap();

    let status_as = Command::new("as").args(["-o"]).arg(&obj_orig_path).arg(&asm_path).status().unwrap();
    assert!(status_as.success(), "as failed");
    let status_cc = Command::new("gcc").args(["-c", "-O2", "-o"]).arg(&main_o_path).arg(&main_c_path).status().unwrap();
    assert!(status_cc.success(), "gcc -c main.c failed");

    let orig_bytes = std::fs::read(&obj_orig_path).unwrap();
    let rust_canon = check_elf_rust_invariants(&orig_bytes).unwrap();
    assert_eq!(rust_canon.rewrite_count, 3);
    assert_eq!(check_elf_rust_invariants(&rust_canon.bytes).unwrap().bytes, rust_canon.bytes);
    verify_elf_structural_preservation(&orig_bytes, &rust_canon.bytes, true);

    let (c_bytes, c_rewrites) = c_canonicalize_elf(&orig_bytes).unwrap();
    assert_eq!(c_rewrites, 3);
    verify_elf_structural_preservation(&orig_bytes, &c_bytes, false);

    // Ensure reloc + COMDAT groups are actually present in object::File view.
    let parsed = object::File::parse(&*rust_canon.bytes).unwrap();
    assert!(object::Object::comdats(&parsed).count() >= 1, "expected SHT_GROUP COMDAT section");
    assert!(
        object::Object::sections(&parsed).any(|s| object::ObjectSection::relocations(&s).count() > 0),
        "expected SHT_RELA relocations referencing local symbols"
    );

    std::fs::write(&obj_rust_path, &rust_canon.bytes).unwrap();
    std::fs::write(&obj_c_path, &c_bytes).unwrap();

    let run_linked = |unit_obj: &Path, bin_name: &str| -> String {
        let bin = dir.path().join(bin_name);
        let out = Command::new("gcc")
            .arg("-no-pie")
            .arg("-o")
            .arg(&bin)
            .arg(&main_o_path)
            .arg(unit_obj)
            .output()
            .unwrap();
        assert!(out.status.success(), "gcc link failed for {}: {}", bin_name, String::from_utf8_lossy(&out.stderr));
        let run = Command::new(&bin).output().unwrap();
        assert!(run.status.success());
        String::from_utf8(run.stdout).unwrap()
    };

    let out_orig = run_linked(&obj_orig_path, "bin_orig");
    let out_rust = run_linked(&obj_rust_path, "bin_rust");
    let out_c = run_linked(&obj_c_path, "bin_c");
    assert_eq!(out_orig.trim(), "49:14");
    assert_eq!(out_rust, out_orig);
    assert_eq!(out_c, out_orig);

    // Separated invalid / ODR-violating control: duplicate strong global symbol MUST fail link
    // both before and after canonicalization (canonicalizer does not hide ODR errors).
    let odr_a_s = dir.path().join("odr_a.s");
    let odr_b_s = dir.path().join("odr_b.s");
    let odr_a_o = dir.path().join("odr_a.o");
    let odr_b_o = dir.path().join("odr_b.o");
    std::fs::write(&odr_a_s, ".text\n.globl dup_sym\ndup_sym:\n mov L.5(%rip), %eax\n ret\nL.5: .long 1\n").unwrap();
    std::fs::write(&odr_b_s, ".text\n.globl dup_sym\ndup_sym:\n mov L.6(%rip), %eax\n ret\nL.6: .long 2\n").unwrap();
    assert!(Command::new("as").args(["-o"]).arg(&odr_a_o).arg(&odr_a_s).status().unwrap().success());
    assert!(Command::new("as").args(["-o"]).arg(&odr_b_o).arg(&odr_b_s).status().unwrap().success());

    let odr_orig_link = Command::new("gcc")
        .args(["-shared", "-o"])
        .arg(dir.path().join("odr_orig.so"))
        .arg(&odr_a_o)
        .arg(&odr_b_o)
        .output()
        .unwrap();
    assert!(!odr_orig_link.status.success(), "ODR control must fail pre-canonicalization");

    let odr_a_canon = check_elf_rust_invariants(&std::fs::read(&odr_a_o).unwrap()).unwrap().bytes;
    let odr_b_canon = check_elf_rust_invariants(&std::fs::read(&odr_b_o).unwrap()).unwrap().bytes;
    let odr_a_canon_o = dir.path().join("odr_a_canon.o");
    let odr_b_canon_o = dir.path().join("odr_b_canon.o");
    std::fs::write(&odr_a_canon_o, odr_a_canon).unwrap();
    std::fs::write(&odr_b_canon_o, odr_b_canon).unwrap();
    let odr_canon_link = Command::new("gcc")
        .args(["-shared", "-o"])
        .arg(dir.path().join("odr_canon.so"))
        .arg(&odr_a_canon_o)
        .arg(&odr_b_canon_o)
        .output()
        .unwrap();
    assert!(!odr_canon_link.status.success(), "ODR control must still fail post-canonicalization");
}

#[test]
fn rejects_shared_tail_and_header_overlap_regressions() {
    let elf = build_synthetic_elf(
        &[SymbolSpec {
            name: b"L.1".to_vec(),
            local: true,
        }],
        false,
        0,
    );
    assert!(check_elf_rust_invariants(&elf).is_ok());

    let mut non_rel = elf.clone();
    non_rel[16..18].copy_from_slice(&2u16.to_le_bytes()); // ET_EXEC
    assert!(check_elf_rust_invariants(&non_rel).unwrap_err().contains("relocatable"));
    check_elf_differential(&non_rel);

    // Shared-tail .strtab: sym[1] (local) points to offset 6 ("L.1\0") inside sym[2]'s "fooL.1\0" at
    // offset 3.
    let mut shared_tail = build_synthetic_elf(
        &[
            SymbolSpec {
                name: b"x".to_vec(),
                local: true,
            },
            SymbolSpec {
                name: b"fooL.1".to_vec(),
                local: false,
            },
        ],
        false,
        0,
    );
    let e_shoff_st = u64::from_le_bytes(shared_tail[40..48].try_into().unwrap()) as usize;
    let sym_hdr_st = e_shoff_st + 3 * 64; // sh[3] = SHT_SYMTAB when include_nobits == false
    let sym_off_st = u64::from_le_bytes(shared_tail[sym_hdr_st + 24..sym_hdr_st + 32].try_into().unwrap()) as usize;
    shared_tail[sym_off_st + 24..sym_off_st + 28].copy_from_slice(&6u32.to_le_bytes());
    assert!(check_elf_rust_invariants(&shared_tail).unwrap_err().contains("string-table entry"));
    check_elf_differential(&shared_tail);

    // ELF-header-overlapping .strtab: point .strtab sh_offset=24, sh_size=5 inside bytes[0..64]
    // (e_entry).
    let mut header_overlap = elf.clone();
    header_overlap[24..29].copy_from_slice(b"\0L.1\0");
    let e_shoff_ho = u64::from_le_bytes(header_overlap[40..48].try_into().unwrap()) as usize;
    let str_hdr_ho = e_shoff_ho + 5 * 64; // sh[5] = .strtab when include_nobits == false
    header_overlap[str_hdr_ho + 24..str_hdr_ho + 32].copy_from_slice(&24u64.to_le_bytes());
    header_overlap[str_hdr_ho + 32..str_hdr_ho + 40].copy_from_slice(&5u64.to_le_bytes());
    assert!(check_elf_rust_invariants(&header_overlap).unwrap_err().contains("overlaps ELF header"));
    check_elf_differential(&header_overlap);

    // Non-canonical following padding after .strtab (extra_pad = 8 bumps e_shoff by +8 past
    // align_up(..., 8)).
    let padded = build_synthetic_elf(
        &[SymbolSpec {
            name: b"L.1".to_vec(),
            local: true,
        }],
        true,
        8,
    );
    assert!(check_elf_rust_invariants(&padded).unwrap_err().contains("non-canonical following padding"));
    check_elf_differential(&padded);
}

#[test]
fn handles_nobits_section_inside_canonical_strtab_padding_gap() {
    // strtab = "\0L.0000000001\0g\0" (16 bytes) at offset 217 -> old_string_end = 233, e_shoff = 240.
    // Gap (233..240) allows placing SHT_NOBITS (.bss = sh[3]) at old_string_end + 1 = 234.
    let mut elf = build_synthetic_elf(
        &[
            SymbolSpec {
                name: b"L.0000000001".to_vec(),
                local: true,
            },
            SymbolSpec {
                name: b"g".to_vec(),
                local: false,
            },
        ],
        true,
        0,
    );
    let e_shoff = u64::from_le_bytes(elf[40..48].try_into().unwrap()) as usize;
    let str_hdr = e_shoff + 6 * 64; // sh[6] = .strtab when include_nobits == true
    let str_off = u64::from_le_bytes(elf[str_hdr + 24..str_hdr + 32].try_into().unwrap()) as usize;
    let str_size = u64::from_le_bytes(elf[str_hdr + 32..str_hdr + 40].try_into().unwrap()) as usize;
    let old_string_end = str_off + str_size;
    assert!(old_string_end + 1 < e_shoff, "expected non-empty alignment gap after .strtab");

    let bss_hdr = e_shoff + 3 * 64; // sh[3] = .bss (SHT_NOBITS)
    elf[bss_hdr + 24..bss_hdr + 32].copy_from_slice(&((old_string_end + 1) as u64).to_le_bytes());

    let rust = check_elf_rust_invariants(&elf).expect("SHT_NOBITS in trailing alignment gap is valid");
    let rust_shoff = u64::from_le_bytes(rust.bytes[40..48].try_into().unwrap()) as usize;
    let rust_bss_off =
        u64::from_le_bytes(rust.bytes[rust_shoff + 3 * 64 + 24..rust_shoff + 3 * 64 + 32].try_into().unwrap()) as usize;
    assert_eq!(rust_bss_off, old_string_end);
    check_elf_differential(&elf);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn prop_elf_differential_and_structural_oracle(
        specs in prop::collection::vec((0u32..999_999_999, any::<bool>(), any::<bool>()), 0..14),
        include_nobits in any::<bool>(),
    ) {
        let symbols: Vec<SymbolSpec> = specs
            .into_iter()
            .enumerate()
            .map(|(idx, (num, is_l_name, local))| {
                let name = if is_l_name {
                    format!("L.{num}").into_bytes()
                } else {
                    format!("sym_{idx}").into_bytes()
                };
                SymbolSpec { name, local }
            })
            .collect();
        let elf = build_synthetic_elf(&symbols, include_nobits, 0);
        let rust = check_elf_rust_invariants(&elf).expect("canonicalizable synthetic ELF");
        verify_elf_structural_preservation(&elf, &rust.bytes, true);
        check_elf_differential(&elf);
    }

    #[test]
    fn prop_elf_rejects_non_canonical_strtab_padding(
        specs in prop::collection::vec((0u32..999_999_999, any::<bool>(), any::<bool>()), 0..10),
        include_nobits in any::<bool>(),
    ) {
        let symbols: Vec<SymbolSpec> = specs
            .into_iter()
            .enumerate()
            .map(|(idx, (num, is_l_name, local))| {
                let name = if is_l_name {
                    format!("L.{num}").into_bytes()
                } else {
                    format!("sym_{idx}").into_bytes()
                };
                SymbolSpec { name, local }
            })
            .collect();
        let elf = build_synthetic_elf(&symbols, include_nobits, 8);
        assert!(
            check_elf_rust_invariants(&elf)
                .unwrap_err()
                .contains("non-canonical following padding")
        );
        check_elf_differential(&elf);
    }

    #[test]
    fn prop_elf_mutated_bytes_differential(
        num in 0u32..100_000,
        mutations in prop::collection::vec((0usize..1024, any::<u8>()), 1..6),
    ) {
        let mut elf = build_synthetic_elf(
            &[
                SymbolSpec {
                    name: format!("L.{num}").into_bytes(),
                    local: true,
                },
                SymbolSpec {
                    name: b"global_entry".to_vec(),
                    local: false,
                },
            ],
            true,
            2,
        );
        for (offset, val) in mutations {
            let idx = offset % elf.len();
            elf[idx] ^= val;
        }
        check_elf_differential(&elf);
    }
}
