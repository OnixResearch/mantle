use std::path::Path;
use std::process::Command;

use mantle_fuzz::MANTLE_FUZZING_ENABLED;
use mantle_fuzz::archive_oracle::build_stagex_archive;
use mantle_fuzz::archive_oracle::check_archive_invariants;
use mantle_fuzz::archive_oracle::parse_raw_ar_members;
use mantle_fuzz::archive_oracle::verify_archive_geometry;
use mantle_fuzz::elf_oracle::SymbolSpec;
use mantle_fuzz::elf_oracle::build_synthetic_elf;
use proptest::prelude::*;

fn replay_dir(dir: &Path, expect_ok: bool) {
    assert!(dir.exists(), "expected seed directory {}", dir.display());
    let mut entries: Vec<_> =
        std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()).filter(|p| p.is_file()).collect();
    entries.sort();
    assert!(!entries.is_empty(), "expected seed files in {}", dir.display());
    for path in entries {
        let bytes = std::fs::read(&path).unwrap();
        let res = check_archive_invariants(&bytes);
        let is_fuzz_only_member_limit =
            path.file_name().and_then(|name| name.to_str()) == Some("reject_17_members_no_strtab.a");
        let seed_expect_ok = if is_fuzz_only_member_limit {
            !MANTLE_FUZZING_ENABLED
        } else {
            expect_ok
        };
        if seed_expect_ok {
            assert!(res.is_ok(), "expected seed {} to succeed: {:?}", path.display(), res);
        } else {
            assert!(res.is_err(), "expected regression seed {} to fail", path.display());
        }
    }
}

#[test]
fn replays_archive_corpus_and_regression_seeds() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    replay_dir(&manifest_dir.join("corpus/archive"), true);
    replay_dir(&manifest_dir.join("regressions/archive"), false);
}

#[test]
fn gnu_ar_interop_and_compiled_archive_semantic_control() {
    let dir = tempfile::tempdir().unwrap();
    let unit_a_s = dir.path().join("unit_a.s");
    let unit_b_s = dir.path().join("unit_b.s");
    let main_c = dir.path().join("main.c");
    let unit_a_o = dir.path().join("unit_a.o");
    let unit_b_o = dir.path().join("unit_b.o");
    let main_o = dir.path().join("main.o");

    std::fs::write(
        &unit_a_s,
        ".text\n.globl fn_a\nfn_a:\n mov L.3(%rip), %eax\n add $10, %eax\n ret\n.section .rodata\nL.3: .long 5\n",
    )
    .unwrap();
    std::fs::write(
        &unit_b_s,
        ".text\n.globl fn_b\nfn_b:\n mov L.7777777(%rip), %eax\n add $20, %eax\n ret\n.section .rodata\nL.7777777: .long 7\n",
    )
    .unwrap();
    std::fs::write(
        &main_c,
        "#include <stdio.h>\nint fn_a(void);\nint fn_b(void);\nint main(void){printf(\"%d\\n\", fn_a() + fn_b());return 0;}\n",
    )
    .unwrap();

    assert!(Command::new("as").args(["-o"]).arg(&unit_a_o).arg(&unit_a_s).status().unwrap().success());
    assert!(Command::new("as").args(["-o"]).arg(&unit_b_o).arg(&unit_b_s).status().unwrap().success());
    assert!(Command::new("gcc").args(["-c", "-o"]).arg(&main_o).arg(&main_c).status().unwrap().success());

    let bytes_a = std::fs::read(&unit_a_o).unwrap();
    let bytes_b = std::fs::read(&unit_b_o).unwrap();
    let long_table = b"very-long-unit-alpha-member.o/\nvery-long-unit-beta-member.o/\n";
    let archive = build_stagex_archive(Some(long_table), &[("/0", &bytes_a), ("/31", &bytes_b)]);

    let canon = check_archive_invariants(&archive).unwrap();
    assert_eq!(canon.member_count, 2);
    assert_eq!(canon.rewritten_member_count, 2);
    assert_eq!(canon.rewritten_symbol_count, 2);

    let canon_a_path = dir.path().join("canon.a");
    std::fs::write(&canon_a_path, &canon.bytes).unwrap();

    // GNU `ar t` must resolve the `//` long names cleanly.
    let ar_t = Command::new("ar").args(["t"]).arg(&canon_a_path).output().unwrap();
    assert!(ar_t.status.success(), "ar t failed: {}", String::from_utf8_lossy(&ar_t.stderr));
    let listed = String::from_utf8(ar_t.stdout).unwrap();
    let lines: Vec<_> = listed.lines().collect();
    assert_eq!(lines, vec!["very-long-unit-alpha-member.o", "very-long-unit-beta-member.o"]);

    // GNU `ar p` extracts exact canonicalized member bytes, and linking executes identically.
    let ar_p = Command::new("ar")
        .args(["p"])
        .arg(&canon_a_path)
        .arg("very-long-unit-alpha-member.o")
        .output()
        .unwrap();
    assert!(ar_p.status.success());
    let ext_a = dir.path().join("ext_a.o");
    let ext_b = dir.path().join("ext_b.o");
    std::fs::write(&ext_a, &ar_p.stdout).unwrap();
    let ar_p_b = Command::new("ar")
        .args(["p"])
        .arg(&canon_a_path)
        .arg("very-long-unit-beta-member.o")
        .output()
        .unwrap();
    assert!(ar_p_b.status.success());
    std::fs::write(&ext_b, &ar_p_b.stdout).unwrap();

    let bin_orig = dir.path().join("bin_orig");
    let bin_canon = dir.path().join("bin_canon");
    assert!(
        Command::new("gcc")
            .args(["-no-pie", "-o"])
            .arg(&bin_orig)
            .args([&main_o, &unit_a_o, &unit_b_o])
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new("gcc")
            .args(["-no-pie", "-o"])
            .arg(&bin_canon)
            .args([&main_o, &ext_a, &ext_b])
            .status()
            .unwrap()
            .success()
    );
    let out_orig = Command::new(&bin_orig).output().unwrap();
    let out_canon = Command::new(&bin_canon).output().unwrap();
    assert_eq!(std::str::from_utf8(&out_orig.stdout).unwrap().trim(), "42");
    assert_eq!(out_orig.stdout, out_canon.stdout);
}

#[test]
fn rejects_dot_traversal_and_over_limit_without_panic() {
    let elf = build_synthetic_elf(
        &[SymbolSpec {
            name: b"L.1".to_vec(),
            local: true,
        }],
        false,
        0,
    );
    for bad in ["./", "../"] {
        let ar = build_stagex_archive(None, &[(bad, &elf)]);
        assert!(check_archive_invariants(&ar).unwrap_err().contains("member name is invalid"));
    }

    let limit = if MANTLE_FUZZING_ENABLED { 16 } else { 256 };
    let names: Vec<String> = (0..=limit).map(|i| format!("m{i}.o/")).collect();
    let members: Vec<(&str, &[u8])> = names.iter().map(|n| (n.as_str(), elf.as_slice())).collect();
    let over_limit = build_stagex_archive(None, &members);
    assert!(check_archive_invariants(&over_limit).unwrap_err().contains("archive member count"));
}

#[test]
fn handles_archive_member_nobits_in_canonical_strtab_padding_gap() {
    // strtab = "\0L.0000000001\0g123\0" (19 bytes) at offset 217 -> str_end = 236, e_shoff = 240.
    // Gap (236..240) allows placing SHT_NOBITS (.bss = sh[3]) at str_end + 1 = 237.
    let mut elf = build_synthetic_elf(
        &[
            SymbolSpec {
                name: b"L.0000000001".to_vec(),
                local: true,
            },
            SymbolSpec {
                name: b"g123".to_vec(),
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
    let str_end = str_off + str_size;
    assert!(str_end + 1 < e_shoff, "expected non-empty alignment gap after .strtab");

    let bss_hdr = e_shoff + 3 * 64; // sh[3] = .bss (SHT_NOBITS)
    elf[bss_hdr + 24..bss_hdr + 32].copy_from_slice(&((str_end + 1) as u64).to_le_bytes());

    let ar = build_stagex_archive(None, &[("member.o/", &elf)]);
    let canon = check_archive_invariants(&ar).expect("SHT_NOBITS in padding gap is valid");
    verify_archive_geometry(&canon.bytes);
    assert_eq!(canon.member_count, 1);
    assert_eq!(canon.rewritten_member_count, 1);
    assert_eq!(canon.rewritten_symbol_count, 1);

    let out_members = parse_raw_ar_members(&canon.bytes);
    assert_eq!(out_members.len(), 1);
    assert_eq!(out_members[0].raw_name, "member.o/");

    let out_elf = out_members[0].payload;
    assert_eq!(out_elf.len(), elf.len());
    let out_shoff = u64::from_le_bytes(out_elf[40..48].try_into().unwrap()) as usize;
    let out_bss_hdr = out_shoff + 3 * 64;
    let out_bss_off = u64::from_le_bytes(out_elf[out_bss_hdr + 24..out_bss_hdr + 32].try_into().unwrap()) as usize;
    assert_eq!(out_bss_off, str_end);

    let second = check_archive_invariants(&canon.bytes).expect("canonical archive is idempotent");
    assert_eq!(second.bytes, canon.bytes);
    assert_eq!(second.member_count, 1);
    assert_eq!(second.rewritten_member_count, 0);
    assert_eq!(second.rewritten_symbol_count, 1);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    #[test]
    fn prop_archive_roundtrip_and_width_convergence(
        members_spec in prop::collection::vec(0u32..999_999, 1..6),
        use_long_table in any::<bool>(),
        include_nobits in any::<bool>(),
    ) {
        let mut long_table = Vec::new();
        let mut short_elfs = Vec::new();
        let mut wide_elfs = Vec::new();
        let mut names = Vec::new();

        for (idx, num) in members_spec.into_iter().enumerate() {
            let short_elf = build_synthetic_elf(
                &[SymbolSpec {
                    name: format!("L.{}", num % 100).into_bytes(),
                    local: true,
                }],
                include_nobits,
                0,
            );
            let wide_elf = build_synthetic_elf(
                &[SymbolSpec {
                    name: format!("L.{:09}", num).into_bytes(),
                    local: true,
                }],
                include_nobits,
                0,
            );
            if use_long_table {
                let off = long_table.len();
                long_table.extend_from_slice(format!("long_member_entry_{idx}.o/\n").as_bytes());
                names.push(format!("/{off}"));
            } else {
                names.push(format!("m_{idx}.o/"));
            }
            short_elfs.push(short_elf);
            wide_elfs.push(wide_elf);
        }

        let table_opt = if use_long_table { Some(long_table.as_slice()) } else { None };
        let short_refs: Vec<(&str, &[u8])> = names.iter().zip(short_elfs.iter()).map(|(n, b)| (n.as_str(), b.as_slice())).collect();
        let wide_refs: Vec<(&str, &[u8])> = names.iter().zip(wide_elfs.iter()).map(|(n, b)| (n.as_str(), b.as_slice())).collect();

        let ar_short = build_stagex_archive(table_opt, &short_refs);
        let ar_wide = build_stagex_archive(table_opt, &wide_refs);

        let c_short = check_archive_invariants(&ar_short).unwrap();
        let c_wide = check_archive_invariants(&ar_wide).unwrap();
        assert_eq!(c_short.bytes, c_wide.bytes);
    }

    #[test]
    fn prop_archive_mutated_bytes_never_panic(
        mutations in prop::collection::vec((0usize..512, any::<u8>()), 1..6),
        include_nobits in any::<bool>(),
    ) {
        let elf = build_synthetic_elf(
            &[SymbolSpec {
                name: b"L.7".to_vec(),
                local: true,
            }],
            include_nobits,
            0,
        );
        let mut ar = build_stagex_archive(None, &[("a.o/", &elf)]);
        for (off, val) in mutations {
            let idx = off % ar.len();
            ar[idx] ^= val;
        }
        let _ = check_archive_invariants(&ar);
    }
}
