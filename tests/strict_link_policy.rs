//! Strict duplicate-symbol link policy tests (#62).
//!
//! Verifies that host `.cargo/config.toml`, `bootstrap/crunch.ncl`, and both
//! `mantle`/`crunch` binaries enforce linker duplicate-definition diagnostics
//! without global suppression, while keeping valid COMDAT coalescing and
//! permuted admitted dependency diamonds linkable.

use std::path::Path;
use std::process::Command;
use std::process::Output;

use serde::Deserialize;

const FORBIDDEN_LINK_SUPPRESSIONS: &[&str] = &[
    "--allow-multiple-definition",
    "multiply_defined",
    "muldefs",
    "--allow-shlib-undefined",
    "--unresolved-symbols",
    "-undefined",
];

#[derive(Debug, Deserialize)]
struct TargetLinkConfig {
    linker: Option<String>,
    rustflags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct CargoLinkConfig {
    target: Option<std::collections::BTreeMap<String, TargetLinkConfig>>,
}

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn run_checked(program: &str, args: &[&str], workdir: &Path) -> Output {
    let output = Command::new(program)
        .args(args)
        .current_dir(workdir)
        .output()
        .unwrap_or_else(|err| panic!("failed to spawn {program} {args:?}: {err}"));
    assert!(
        output.status.success(),
        "{program} {args:?} failed (status {:?}):\nstdout: {}\nstderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn run_raw(program: &str, args: &[&str], workdir: &Path) -> Output {
    Command::new(program)
        .args(args)
        .current_dir(workdir)
        .output()
        .unwrap_or_else(|err| panic!("failed to spawn {program} {args:?}: {err}"))
}

fn assert_no_forbidden_suppressions(label: &str, text: &str) {
    for flag in FORBIDDEN_LINK_SUPPRESSIONS {
        assert!(!text.contains(flag), "{label} must not contain blanket linker suppression {flag:?}");
    }
}

fn assert_duplicate_diagnostic(stderr: &str, symbol: &str, label: &str) {
    let lower = stderr.to_ascii_lowercase();
    assert!(
        lower.contains("duplicate") || lower.contains("multiple definition"),
        "{label} expected duplicate-definition diagnostic for {symbol}, got:\n{stderr}"
    );
    assert!(stderr.contains(symbol), "{label} diagnostic did not name symbol {symbol}:\n{stderr}");
}

#[test]
fn host_and_bootstrap_link_configs_forbid_blanket_duplicate_suppression() {
    let cargo_config_text =
        std::fs::read_to_string(repo_root().join(".cargo/config.toml")).expect("read .cargo/config.toml");
    let bootstrap_ncl_text =
        std::fs::read_to_string(repo_root().join("bootstrap/crunch.ncl")).expect("read bootstrap/crunch.ncl");

    assert_no_forbidden_suppressions(".cargo/config.toml", &cargo_config_text);
    assert_no_forbidden_suppressions("bootstrap/crunch.ncl", &bootstrap_ncl_text);

    let parsed: CargoLinkConfig = toml::from_str(&cargo_config_text).expect("parse .cargo/config.toml");
    let targets = parsed.target.expect(".cargo/config.toml [target] table");

    for linux_triple in ["x86_64-unknown-linux-gnu", "aarch64-unknown-linux-gnu"] {
        let cfg = targets.get(linux_triple).unwrap_or_else(|| panic!("missing target config for {linux_triple}"));
        assert_eq!(cfg.linker.as_deref(), Some("clang"));
        assert_eq!(
            cfg.rustflags.as_deref(),
            Some(["-C".to_string(), "link-arg=-fuse-ld=mold".to_string()].as_slice()),
            "unexpected rustflags for {linux_triple}"
        );
    }

    for darwin_triple in ["x86_64-apple-darwin", "aarch64-apple-darwin"] {
        if let Some(cfg) = targets.get(darwin_triple)
            && let Some(flags) = &cfg.rustflags
        {
            for flag in flags {
                assert_no_forbidden_suppressions(darwin_triple, flag);
            }
        }
    }

    assert!(
        bootstrap_ncl_text.contains("rustflags = [\"-C\", \"link-arg=-L${GCC_LIB}\"]"),
        "bootstrap/crunch.ncl must pass only -L${{GCC_LIB}} without duplicate suppression"
    );
}

#[test]
fn both_mantle_and_crunch_binaries_run_under_strict_host_link() {
    for (name, bin_path) in [
        ("mantle", env!("CARGO_BIN_EXE_mantle")),
        ("crunch", env!("CARGO_BIN_EXE_crunch")),
    ] {
        let out = Command::new(bin_path)
            .arg("--version")
            .output()
            .unwrap_or_else(|err| panic!("failed to run {name} ({bin_path}): {err}"));
        assert!(out.status.success(), "{name} --version failed: {}", String::from_utf8_lossy(&out.stderr));
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(!stdout.trim().is_empty(), "{name} --version printed empty stdout");
    }
}

#[test]
fn strict_link_rejects_deliberate_duplicate_strong_symbol_in_mold_and_bfd() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();

    std::fs::write(root.join("dup_a.c"), "int mantle_strict_dup_probe(void) { return 11; }\n").unwrap();
    std::fs::write(root.join("dup_b.c"), "int mantle_strict_dup_probe(void) { return 22; }\n").unwrap();
    std::fs::write(
        root.join("main.c"),
        "int mantle_strict_dup_probe(void);\nint main(void) { return mantle_strict_dup_probe(); }\n",
    )
    .unwrap();

    run_checked("clang", &["-c", "dup_a.c", "-o", "dup_a.o"], root);
    run_checked("clang", &["-c", "dup_b.c", "-o", "dup_b.o"], root);
    run_checked("clang", &["-c", "main.c", "-o", "main.o"], root);

    for (linker_flag, label) in [("-fuse-ld=mold", "clang+mold"), ("-fuse-ld=bfd", "clang+ld.bfd")] {
        let out = run_raw("clang", &[linker_flag, "main.o", "dup_a.o", "dup_b.o", "-o", "dup_out"], root);
        assert!(!out.status.success(), "{label} unexpectedly linked duplicate strong symbol");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert_duplicate_diagnostic(&stderr, "mantle_strict_dup_probe", label);
    }
}

#[test]
fn strict_link_coalesces_valid_equivalent_comdat_definitions() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();

    let comdat_header = "inline int mantle_comdat_probe(int x) { return x + 41; }\n";
    std::fs::write(
        root.join("comdat_a.cc"),
        format!("{comdat_header}extern \"C\" int call_comdat_a(int x) {{ int (*fn)(int) = &mantle_comdat_probe; return fn(x); }}\n"),
    )
    .unwrap();
    std::fs::write(
        root.join("comdat_b.cc"),
        format!("{comdat_header}extern \"C\" int call_comdat_b(int x) {{ int (*fn)(int) = &mantle_comdat_probe; return fn(x); }}\n"),
    )
    .unwrap();
    std::fs::write(
        root.join("main.c"),
        "#include <stdio.h>\nint call_comdat_a(int x);\nint call_comdat_b(int x);\nint main(void) { printf(\"%d:%d\\n\", call_comdat_a(7), call_comdat_b(7)); return 0; }\n",
    )
    .unwrap();

    run_checked("clang++", &["-O0", "-c", "comdat_a.cc", "-o", "comdat_a.o"], root);
    run_checked("clang++", &["-O0", "-c", "comdat_b.cc", "-o", "comdat_b.o"], root);
    run_checked("clang", &["-c", "main.c", "-o", "main.o"], root);

    for obj in ["comdat_a.o", "comdat_b.o"] {
        let groups = run_checked("readelf", &["-gW", obj], root);
        let text = String::from_utf8_lossy(&groups.stdout);
        assert!(text.contains("COMDAT group section"), "expected COMDAT group section in {obj}, got:\n{text}");
    }

    for (linker_flag, out_name) in [("-fuse-ld=mold", "comdat_mold"), ("-fuse-ld=bfd", "comdat_bfd")] {
        run_checked("clang", &[linker_flag, "main.o", "comdat_a.o", "comdat_b.o", "-o", out_name], root);
        let run = run_checked(&format!("./{out_name}"), &[], root);
        assert_eq!(String::from_utf8_lossy(&run.stdout).trim(), "48:48");
    }
}

#[test]
fn dependency_diamond_link_order_permutation_preserves_provider_and_rejects_strong_collision() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();

    // Admitted dependency diamond: main -> {left.a, right.a} -> base.a
    std::fs::write(root.join("base.c"), "int diamond_base(int x) { return x * 2; }\n").unwrap();
    std::fs::write(
        root.join("left.c"),
        "int diamond_base(int x);\nint diamond_left(void) { return diamond_base(3) + 10; }\n",
    )
    .unwrap();
    std::fs::write(
        root.join("right.c"),
        "int diamond_base(int x);\nint diamond_right(void) { return diamond_base(5) + 20; }\n",
    )
    .unwrap();
    std::fs::write(
        root.join("main.c"),
        "#include <stdio.h>\nint diamond_left(void);\nint diamond_right(void);\nint main(void) { printf(\"%d\\n\", diamond_left() + diamond_right()); return 0; }\n",
    )
    .unwrap();

    for unit in ["base", "left", "right", "main"] {
        run_checked("clang", &["-c", &format!("{unit}.c"), "-o", &format!("{unit}.o")], root);
    }
    run_checked("ar", &["rcs", "libbase.a", "base.o"], root);
    run_checked("ar", &["rcs", "libleft.a", "left.o"], root);
    run_checked("ar", &["rcs", "libright.a", "right.o"], root);

    for (linker_flag, tag) in [("-fuse-ld=mold", "mold"), ("-fuse-ld=bfd", "bfd")] {
        let bin_lr = format!("diamond_lr_{tag}");
        let bin_rl = format!("diamond_rl_{tag}");
        run_checked(
            "clang",
            &[
                linker_flag,
                "main.o",
                "libleft.a",
                "libright.a",
                "libbase.a",
                "-o",
                &bin_lr,
            ],
            root,
        );
        run_checked(
            "clang",
            &[
                linker_flag,
                "main.o",
                "libright.a",
                "libleft.a",
                "libbase.a",
                "-o",
                &bin_rl,
            ],
            root,
        );

        let out_lr = run_checked(&format!("./{bin_lr}"), &[], root);
        let out_rl = run_checked(&format!("./{bin_rl}"), &[], root);
        assert_eq!(String::from_utf8_lossy(&out_lr.stdout).trim(), "46");
        assert_eq!(out_lr.stdout, out_rl.stdout);

        for bin in [&bin_lr, &bin_rl] {
            let syms = run_checked("nm", &["--defined-only", bin], root);
            let sym_text = String::from_utf8_lossy(&syms.stdout);
            let base_defs: Vec<_> =
                sym_text.lines().filter(|line| line.split_whitespace().last() == Some("diamond_base")).collect();
            assert_eq!(base_defs.len(), 1, "expected single selected diamond_base provider in {bin}: {base_defs:?}");
        }
    }

    // Accidental order-dependent strong-symbol provider in the diamond MUST fail in BOTH link orders.
    std::fs::write(root.join("provider_alpha.c"), "int diamond_provider(void) { return 101; }\n").unwrap();
    std::fs::write(root.join("provider_beta.c"), "int diamond_provider(void) { return 202; }\n").unwrap();
    std::fs::write(
        root.join("diamond_use.c"),
        "int diamond_provider(void);\nint main(void) { return diamond_provider(); }\n",
    )
    .unwrap();
    run_checked("clang", &["-c", "provider_alpha.c", "-o", "provider_alpha.o"], root);
    run_checked("clang", &["-c", "provider_beta.c", "-o", "provider_beta.o"], root);
    run_checked("clang", &["-c", "diamond_use.c", "-o", "diamond_use.o"], root);

    for linker_flag in ["-fuse-ld=mold", "-fuse-ld=bfd"] {
        for (first, second, order_label) in [
            ("provider_alpha.o", "provider_beta.o", "alpha-then-beta"),
            ("provider_beta.o", "provider_alpha.o", "beta-then-alpha"),
        ] {
            let out = run_raw("clang", &[linker_flag, "diamond_use.o", first, second, "-o", "bad_diamond"], root);
            assert!(
                !out.status.success(),
                "{linker_flag} ({order_label}) silently picked an order-dependent strong provider"
            );
            let stderr = String::from_utf8_lossy(&out.stderr);
            assert_duplicate_diagnostic(&stderr, "diamond_provider", &format!("{linker_flag}:{order_label}"));
        }
    }
}

#[test]
fn invalid_program_odr_violation_policy_control() {
    // Separated invalid-program / policy-control bucket for conflicting C++ definitions
    // (ODR violation). Not a compiler-correctness oracle.
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();

    std::fs::write(root.join("odr_a.cc"), "int mantle_odr_bucket_sym(void) { return 7; }\n").unwrap();
    std::fs::write(root.join("odr_b.cc"), "int mantle_odr_bucket_sym(void) { return 9; }\n").unwrap();
    std::fs::write(
        root.join("odr_main.cc"),
        "int mantle_odr_bucket_sym(void);\nint main(void) { return mantle_odr_bucket_sym(); }\n",
    )
    .unwrap();

    run_checked("clang++", &["-c", "odr_a.cc", "-o", "odr_a.o"], root);
    run_checked("clang++", &["-c", "odr_b.cc", "-o", "odr_b.o"], root);
    run_checked("clang++", &["-c", "odr_main.cc", "-o", "odr_main.o"], root);

    for (linker_flag, label) in [("-fuse-ld=mold", "clang++mold"), ("-fuse-ld=bfd", "clang++ld.bfd")] {
        let out = run_raw("clang", &[linker_flag, "odr_main.o", "odr_a.o", "odr_b.o", "-o", "odr_out"], root);
        assert!(!out.status.success(), "{label} unexpectedly linked conflicting ODR definitions");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert_duplicate_diagnostic(&stderr, "mantle_odr_bucket_sym", label);
    }
}
