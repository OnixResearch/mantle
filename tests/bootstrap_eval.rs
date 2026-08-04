use std::collections::BTreeSet;
use std::ffi::OsString;
use std::path::PathBuf;

use crunch_glue::CrunchDerivation;
use crunch_glue::Input;
use serde::Deserialize;

const RUST_SOURCE_ROUTE_COUNT: usize = 5;
const BOOTSTRAP_ENTRYPOINTS: &[&str] = &[
    "make.ncl",
    "dash.ncl",
    "binutils.ncl",
    "musl.ncl",
    "gcc.ncl",
    "integration-test.ncl",
    "busybox.ncl",
    "bwrap.ncl",
    "rust.ncl",
    "crunch.ncl",
    "selftest.ncl",
];

fn bootstrap_import_paths() -> Vec<OsString> {
    vec![
        crunch_eval::stdlib::stdlib_import_path().unwrap().into_os_string(),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bootstrap").into_os_string(),
    ]
}

fn bootstrap_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bootstrap").join(name)
}

fn eval_bootstrap(name: &str) -> CrunchDerivation {
    crunch_eval::evaluate_and_deserialize(&bootstrap_path(name), &bootstrap_import_paths()).unwrap()
}

#[derive(Debug, Deserialize)]
struct SeedProviderMetadata {
    id: String,
    mode: String,
    summary: String,
    retained_tools: Vec<String>,
    dropped_components: Vec<String>,
    notes: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SeedModule {
    name: String,
    target: String,
    dynamic_linker: String,
    toolchain: CrunchDerivation,
    provider: SeedProviderMetadata,
}

fn eval_seed_module() -> SeedModule {
    crunch_eval::evaluate_and_deserialize(&bootstrap_path("seed.ncl"), &bootstrap_import_paths()).unwrap()
}

#[derive(Debug, Deserialize)]
struct RustSourceRouteRegistry {
    schema: String,
    selected_candidate: String,
    routes: Vec<RustSourceRoute>,
}

#[derive(Debug, Deserialize)]
struct RustSourceRoute {
    mechanism_id: String,
    provider_origin: String,
    compiler_host: String,
    source_policy: String,
    ambient_discovery: bool,
    uses_prebuilt_rust: bool,
    full_bootstrap_candidate: bool,
    completion_probe: String,
    disqualifying_non_claim: String,
}

fn eval_rust_source_route_registry() -> RustSourceRouteRegistry {
    crunch_eval::evaluate_and_deserialize(&bootstrap_path("rust-source-route-registry.ncl"), &bootstrap_import_paths())
        .unwrap()
}

fn input_derivation_names(drv: &CrunchDerivation) -> Vec<String> {
    let mut names = Vec::new();
    for input in &drv.inputs {
        if let Input::Derivation(dep) = input {
            names.push(dep.name.clone());
        }
    }
    names
}

fn contract_violations<'a>(source: &str, required: &'a [&'a str], forbidden: &'a [&'a str]) -> Vec<&'a str> {
    assert!(!required.is_empty());
    assert!(!forbidden.is_empty());

    let mut violations = Vec::new();
    for marker in required {
        if !source.contains(marker) {
            violations.push(*marker);
        }
    }
    for marker in forbidden {
        if source.contains(marker) {
            violations.push(*marker);
        }
    }
    violations
}

#[test]
fn full_source_busybox_binds_the_selected_linux_header_identity() {
    let source = std::fs::read_to_string(bootstrap_path("busybox-1.37.0-gcc10.ncl")).unwrap();
    let required = ["HEADERS=$(find_input linux-headers-6.6-full-source-gcc10-v3)"];
    let forbidden = ["HEADERS=$(find_input linux-headers-6.6-full-source-gcc10-v2)"];
    let violations = contract_violations(&source, &required, &forbidden);
    let invalid_source = source.replace(required[0], forbidden[0]);
    let invalid_violations = contract_violations(&invalid_source, &required, &forbidden);
    let headers = eval_bootstrap("linux-headers-6.6-gcc10.ncl");
    let busybox = eval_bootstrap("busybox-1.37.0-gcc10.ncl");

    assert!(violations.is_empty(), "BusyBox Linux-header contract violations: {violations:?}");
    assert_eq!(invalid_violations, vec![required[0], forbidden[0]]);
    assert_eq!(headers.name, "linux-headers-6.6-full-source-gcc10-v3");
    assert_eq!(busybox.name, "busybox-1.37.0-full-source-gcc10-v1");
}

#[test]
fn gcc_pass3_linker_uses_only_relocatable_stagex_authority() {
    let source = std::fs::read_to_string(bootstrap_path("gcc-4.0-musl-pass3.ncl")).unwrap();
    let required = [
        "name = \"gcc-4.0.4-musl-pass3-v23\"",
        "linker=\"\\${MANTLE_GCC_LINKER:-$STAGEX/bin/x86_64-linux-musl-ld}\"",
        "exec \"\\$linker\" \\$link_arguments",
    ];
    let forbidden = ["gcc40-pass3-final-ld: $BINUTILS/bin/ld", "gcc-4.0.4-musl-pass3-v22"];
    let violations = contract_violations(&source, &required, &forbidden);
    let invalid_source = source.replace(required[1], forbidden[0]);
    let invalid_violations = contract_violations(&invalid_source, &required, &forbidden);
    let derivation = eval_bootstrap("gcc-4.0-musl-pass3.ncl");

    assert!(violations.is_empty(), "GCC pass3 linker contract violations: {violations:?}");
    assert_eq!(invalid_violations, vec![required[1], forbidden[0]]);
    assert_eq!(derivation.name, "gcc-4.0.4-musl-pass3-v23");
    assert!(source.contains("let stagex_provider = import \"stagex-provider-proof-input.ncl\""));
}

#[test]
fn gcc_pass4_tool_wrappers_use_only_relocatable_stagex_authority() {
    let source = std::fs::read_to_string(bootstrap_path("gcc-4.0-musl-pass4.ncl")).unwrap();
    let (_, final_assembler_wrapper) = source.rsplit_once("      $BB cat > \"$out/bin/as\" <<EOF").unwrap();
    let (_, final_linker_wrapper) = source.rsplit_once("      $BB cat > \"$out/bin/ld\" <<EOF").unwrap();
    let assembler_required = [
        "assembler=\"\\${MANTLE_GCC_ASSEMBLER:-$STAGEX/bin/x86_64-linux-musl-as}\"",
        "exec \"\\$assembler\" \"\\$@\"",
    ];
    let assembler_forbidden = ["assembler=\"\\${MANTLE_GCC_ASSEMBLER:-$BINUTILS/bin/as}\""];
    let linker_required = [
        "linker=\"\\${MANTLE_GCC_LINKER:-$STAGEX/bin/x86_64-linux-musl-ld}\"",
        "exec \"\\$linker\" \\$link_arguments",
    ];
    let linker_forbidden = ["linker=\"\\${MANTLE_GCC_LINKER:-$BINUTILS/bin/ld}\""];
    let assembler_violations = contract_violations(final_assembler_wrapper, &assembler_required, &assembler_forbidden);
    let invalid_assembler = final_assembler_wrapper.replace(assembler_required[0], assembler_forbidden[0]);
    let linker_violations = contract_violations(final_linker_wrapper, &linker_required, &linker_forbidden);
    let invalid_linker = final_linker_wrapper.replace(linker_required[0], linker_forbidden[0]);
    let derivation = eval_bootstrap("gcc-4.0-musl-pass4.ncl");

    assert!(assembler_violations.is_empty(), "GCC pass4 assembler contract: {assembler_violations:?}");
    assert_eq!(contract_violations(&invalid_assembler, &assembler_required, &assembler_forbidden), vec![
        assembler_required[0],
        assembler_forbidden[0]
    ]);
    assert!(linker_violations.is_empty(), "GCC pass4 linker contract: {linker_violations:?}");
    assert_eq!(contract_violations(&invalid_linker, &linker_required, &linker_forbidden), vec![
        linker_required[0],
        linker_forbidden[0]
    ]);
    assert!(source.contains("name = \"gcc-4.0.4-musl-pass4-v7\""));
    assert!(!source.contains("gcc-4.0.4-musl-pass4-v6"));
    assert_eq!(derivation.name, "gcc-4.0.4-musl-pass4-v7");
    assert!(source.contains("let stagex_provider = import \"stagex-provider-proof-input.ncl\""));
}

#[test]
fn gcc_pass5_archives_bind_deterministic_binutils_metadata() {
    let source = std::fs::read_to_string(bootstrap_path("gcc-4.0-musl-pass5.ncl")).unwrap();
    let required = [
        "name = \"gcc-4.0.4-musl-pass5-v2\"",
        "grep -Fxc 'AR_CREATE_FOR_TARGET = $(AR_FOR_TARGET) $(AR_FLAGS_FOR_TARGET) rc'",
        "'s|^AR_CREATE_FOR_TARGET = $(AR_FOR_TARGET) $(AR_FLAGS_FOR_TARGET) rc$|AR_CREATE_FOR_TARGET = $(AR_FOR_TARGET) $(AR_FLAGS_FOR_TARGET) rcD|'",
        "grep -Fxc 'AR_CREATE_FOR_TARGET = $(AR_FOR_TARGET) $(AR_FLAGS_FOR_TARGET) rcD'",
        "DETERMINISTIC_RANLIB=\"$BINUTILS/bin/ranlib -D\"",
        "RANLIB_FOR_TARGET=\"$DETERMINISTIC_RANLIB\"",
    ];
    let forbidden = [
        "name = \"gcc-4.0.4-musl-pass5-v1\"",
        "'s|^AR_CREATE_FOR_TARGET = $(AR_FOR_TARGET) $(AR_FLAGS_FOR_TARGET) rc$|AR_CREATE_FOR_TARGET = $(AR_FOR_TARGET) $(AR_FLAGS_FOR_TARGET) rc|'",
        "DETERMINISTIC_RANLIB=\"$BINUTILS/bin/ranlib\"",
    ];
    let invalid_ar = source.replace(required[2], forbidden[1]);
    let invalid_ranlib = source.replace(required[4], forbidden[2]);
    let derivation = eval_bootstrap("gcc-4.0-musl-pass5.ncl");

    assert!(contract_violations(&source, &required, &forbidden).is_empty());
    assert_eq!(contract_violations(&invalid_ar, &required, &forbidden), vec![required[2], forbidden[1]]);
    assert_eq!(contract_violations(&invalid_ranlib, &required, &forbidden), vec![required[4], forbidden[2]]);
    assert_eq!(source.matches(required[1]).count(), 1);
    assert_eq!(source.matches(required[3]).count(), 1);
    assert_eq!(source.matches(required[5]).count(), 2);
    assert_eq!(derivation.name, "gcc-4.0.4-musl-pass5-v2");
}

#[test]
fn gcc_built_binutils_defaults_to_deterministic_archives() {
    let source = std::fs::read_to_string(bootstrap_path("binutils-2.30-gcc.ncl")).unwrap();
    let required = [
        "name = \"binutils-2.30-gcc-pass4-v17\"",
        "--enable-deterministic-archives",
        "\"$out/bin/ar\" rc \"$ARCHIVE_PROBE/default-first.a\" \"$ARCHIVE_MEMBER\"",
        "\"$out/bin/ar\" rcU \"$ARCHIVE_PROBE/timestamp-first.a\" \"$ARCHIVE_MEMBER\"",
        "ERROR: explicit timestamp-sensitive archive probe produced equal archives",
        "default_archive_mode=deterministic",
    ];
    let forbidden = [
        "name = \"binutils-2.30-gcc-pass4-v16\"",
        "--disable-deterministic-archives",
    ];
    let invalid_flag = source.replacen(required[1], forbidden[1], 1);
    let derivation = eval_bootstrap("binutils-2.30-gcc.ncl");

    assert!(contract_violations(&source, &required, &forbidden).is_empty());
    assert_eq!(contract_violations(&invalid_flag, &required, &forbidden), vec![required[1], forbidden[1]]);
    assert_eq!(source.matches(required[1]).count(), 1);
    assert!(source.contains("ARCHIVE_DATE_EARLY=200001010000"));
    assert!(source.contains("ARCHIVE_DATE_LATE=201001010000"));
    assert_eq!(derivation.name, "binutils-2.30-gcc-pass4-v17");
}

#[test]
fn gcc40_cxx_defaults_random_seeds_to_main_input_identity() {
    let source = std::fs::read_to_string(bootstrap_path("gcc-4.0-musl-cxx.ncl")).unwrap();
    let required = [
        "name = \"gcc-4.0.4-musl-cxx-v8\"",
        "RANDOM_SEED_NEW='      value = crc32_string (0, main_input_filename ? main_input_filename : \"mantle-gcc40-no-input\");'",
        "RANDOM_SEED_ASSIGN_NEW='      flag_random_seed = random_seed; local_tick = -1;'",
        "\"$CC1PLUS\" -quiet -O2 \"$WORK/random-seed-probe.cc\" -o \"$WORK/random-seed-default-a.s\"",
        "-frandom-seed=mantle-negative-a",
        "ERROR: GCC random-seed negative control produced equal assembly",
        "default_random_seed=main-input-filename-crc32",
    ];
    let forbidden = [
        "name = \"gcc-4.0.4-musl-cxx-v7\"",
        "RANDOM_SEED_NEW='      value = local_tick ^ getpid ();'",
        "RANDOM_SEED_ASSIGN_NEW='      flag_random_seed = random_seed;'",
    ];
    let invalid_default = source.replacen(required[1], forbidden[1], 1);
    let invalid_tick = source.replacen(required[2], forbidden[2], 1);
    let derivation = eval_bootstrap("gcc-4.0-musl-cxx.ncl");

    assert!(contract_violations(&source, &required, &forbidden).is_empty());
    assert_eq!(contract_violations(&invalid_default, &required, &forbidden), vec![required[1], forbidden[1]]);
    assert_eq!(contract_violations(&invalid_tick, &required, &forbidden), vec![required[2], forbidden[2]]);
    assert_eq!(source.matches(required[1]).count(), 1);
    assert_eq!(source.matches(required[2]).count(), 1);
    assert_eq!(derivation.name, "gcc-4.0.4-musl-cxx-v8");
}

#[test]
fn eval_make_bootstrap_imports_shared_seed() {
    let drv = eval_bootstrap("make.ncl");
    let input_names = input_derivation_names(&drv);

    assert_eq!(drv.name, "gnumake");
    assert!(input_names.contains(&"full-source-seed-toolchain".to_string()));
    assert!(input_names.contains(&"make-src".to_string()));
}

#[test]
fn eval_crunch_bootstrap_imports_shared_seed() {
    let drv = eval_bootstrap("crunch.ncl");
    let input_names = input_derivation_names(&drv);

    assert_eq!(drv.name, "crunch");
    assert!(input_names.contains(&"full-source-seed-toolchain".to_string()));
    assert!(input_names.contains(&"gcc".to_string()));
    assert!(input_names.contains(&"rust".to_string()));
}

#[test]
fn eval_seed_module_exposes_selected_full_source_provider_metadata() {
    let seed = eval_seed_module();

    assert_eq!(seed.name, "full-source-seed-toolchain");
    assert_eq!(seed.toolchain.name, "full-source-seed-toolchain");
    assert_eq!(seed.target, "x86_64-linux-musl");
    assert_eq!(seed.dynamic_linker, "ld-musl-x86_64.so.1");
    assert_eq!(seed.provider.id, "full-source-v1");
    assert_eq!(seed.provider.mode, "source-built");
    assert!(seed.provider.summary.contains("Selected runtime-admitted final GCC 10.5.0"));
    assert!(seed.provider.retained_tools.contains(&"x86_64-linux-musl-gcc".to_string()));
    assert!(seed.provider.retained_tools.contains(&"x86_64-linux-musl-g++".to_string()));
    assert!(seed.provider.retained_tools.contains(&"x86_64-linux-musl-ld".to_string()));
    assert!(seed.provider.dropped_components.contains(&"fortran".to_string()));
    assert!(seed.provider.notes.iter().any(|note| note.contains("complete source-closure admission")));
}

#[test]
fn rust_source_route_registry_selects_only_the_full_source_musl_candidate() {
    let registry = eval_rust_source_route_registry();
    let route_ids = registry.routes.iter().map(|route| route.mechanism_id.as_str()).collect::<BTreeSet<_>>();
    let candidates = registry.routes.iter().filter(|route| route.full_bootstrap_candidate).collect::<Vec<_>>();

    assert_eq!(registry.schema, "mantle-rust-source-route-registry-v1");
    assert_eq!(registry.routes.len(), RUST_SOURCE_ROUTE_COUNT);
    assert_eq!(route_ids.len(), RUST_SOURCE_ROUTE_COUNT);
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].mechanism_id, registry.selected_candidate);
    assert_eq!(candidates[0].compiler_host, "admitted-full-source-native-provider");
    assert_eq!(candidates[0].source_policy, "authenticated-offline-only");
    assert!(!candidates[0].ambient_discovery);
    assert!(!candidates[0].uses_prebuilt_rust);
    assert!(candidates[0].completion_probe.contains("binding-v1"));
}

#[test]
fn rust_source_route_registry_disqualifies_compatibility_and_import_routes() {
    let registry = eval_rust_source_route_registry();
    let rejected = registry.routes.iter().filter(|route| !route.full_bootstrap_candidate).collect::<Vec<_>>();

    assert_eq!(rejected.len(), RUST_SOURCE_ROUTE_COUNT - 1);
    assert!(rejected.iter().all(|route| !route.disqualifying_non_claim.is_empty()));
    assert!(rejected.iter().all(|route| !route.provider_origin.is_empty()));
    assert!(rejected.iter().all(|route| !route.completion_probe.is_empty()));
    assert!(rejected.iter().any(|route| route.mechanism_id == "imported-provider"));
    assert!(rejected.iter().any(|route| route.ambient_discovery));
    assert!(rejected.iter().any(|route| route.uses_prebuilt_rust));
    assert!(!rejected.iter().any(|route| route.mechanism_id == registry.selected_candidate));
}

#[test]
fn bootstrap_entrypoints_do_not_inline_raw_seed_provider_details() {
    for entrypoint in BOOTSTRAP_ENTRYPOINTS {
        let text = std::fs::read_to_string(bootstrap_path(entrypoint)).unwrap();
        assert!(!text.contains("https://musl.cc/"), "raw seed URL leaked into {entrypoint}");
        assert!(!text.contains("musl-gcc-raw"), "raw seed name leaked into {entrypoint}");
        assert!(text.contains("import \"seed.ncl\""), "shared seed import missing in {entrypoint}");
    }
}

#[test]
fn legacy_seed_derivation_writes_shared_provider_metadata_schema() {
    let text = std::fs::read_to_string(bootstrap_path("seed-legacy.ncl")).unwrap();

    assert!(text.contains("\"raw_size_bytes\": $RAW_SIZE_BYTES"));
    assert!(text.contains("\"reduced_size_bytes\": $REDUCED_SIZE_BYTES"));
    assert!(text.contains("\"retained_tools\": %{std.serialize 'Json provider_retained_tools}"));
    assert!(text.contains("\"notes\": %{std.serialize 'Json provider_notes_list}"));
    assert!(!text.contains("raw_size_mb"));
    assert!(!text.contains("reduced_size_mb"));
}

#[test]
fn seed_selector_selects_full_source_without_legacy_fallback_or_self_recursion() {
    let selector_text = std::fs::read_to_string(bootstrap_path("seed.ncl")).unwrap();
    let full_text = std::fs::read_to_string(bootstrap_path("seed-full.ncl")).unwrap();
    let legacy_text = std::fs::read_to_string(bootstrap_path("seed-legacy.ncl")).unwrap();

    assert!(selector_text.contains("import \"seed-full.ncl\""));
    assert!(!selector_text.contains("import \"seed-legacy.ncl\""));
    assert!(!selector_text.contains("CRUNCH_LEGACY_SEED"));
    assert!(!full_text.contains("import \"seed-full.ncl\""));
    assert!(!legacy_text.contains("import \"seed-legacy.ncl\""));
    assert!(legacy_text.contains("musl.cc-native-reduced-v1"));
}

#[test]
fn gzip_marks_generated_crc_helper_executable_before_running_it() {
    let text = std::fs::read_to_string(bootstrap_path("gzip-tcc.ncl")).unwrap();
    let compile_offset = text.find("tcc -static -o makecrc /tmp/makecrc_patch.c").unwrap();
    let chmod_offset = text.find("$BB chmod \"$EXECUTABLE_MODE\" makecrc").unwrap();
    let run_offset = text.find("./makecrc").unwrap();

    assert!(text.contains("EXECUTABLE_MODE=755"));
    assert!(compile_offset < chmod_offset);
    assert!(chmod_offset < run_offset);
    assert!(!text.contains("\n      chmod 755 makecrc"));
    assert!(!text.contains("\n      ./makecrc\n      $BB chmod"));
}

#[test]
fn source_built_linux_headers_replace_legacy_seed_header_assumption() {
    let source = std::fs::read_to_string(bootstrap_path("linux-6.6-source.ncl")).unwrap();
    let headers = std::fs::read_to_string(bootstrap_path("linux-headers.ncl")).unwrap();
    let bwrap = std::fs::read_to_string(bootstrap_path("bwrap.ncl")).unwrap();
    let busybox = std::fs::read_to_string(bootstrap_path("busybox.ncl")).unwrap();

    assert!(source.contains("https://cdn.kernel.org/pub/linux/kernel/v6.x/linux-6.6.tar.xz"));
    assert!(source.contains("sha256-+GqM1/8Qw0N6Z2t45jjapVZrG0JysMs9PVi79GIK1Ec="));
    assert!(headers.contains("headers_install"));
    assert!(headers.contains("linux/capability.h"));
    assert!(headers.contains("linux/loop.h"));
    assert!(bwrap.contains("derivationFile \"linux-headers.ncl\""));
    assert!(busybox.contains("derivationFile \"linux-headers.ncl\""));
    assert!(bwrap.contains("$LINUX_HEADERS/include"));
    assert!(busybox.contains("$LINUX_HEADERS/include"));
    assert!(!bwrap.contains("SEED_INC"));
    assert!(!busybox.contains("SEED_INC"));
}

#[test]
fn m4_musl_builds_genuine_prefix_capable_binary_with_fixture_polarities() {
    let text = std::fs::read_to_string(bootstrap_path("m4-1.4.7-musl.ncl")).unwrap();

    assert!(text.contains("https://mirrors.kernel.org/gnu/m4/m4-1.4.7.tar.bz2"));
    assert!(text.contains("import \"tcc-musl-v2.ncl\""));
    assert!(text.contains("import \"musl-1.1.24-native.ncl\""));
    assert!(text.contains("export PATH=\"$TCC/bin:$STAGE0/bin\""));
    assert!(text.contains("LIB_SOURCE_COUNT=18"));
    assert!(text.contains("M4_SOURCE_COUNT=11"));
    assert!(text.contains("tcc -c $CFLAGS \"lib/$source_name.c\""));
    assert!(text.contains("tcc -c $CFLAGS \"src/$source_name.c\""));
    assert!(text.contains("\"$MUSL/lib/crt1.o\" $OBJECTS \"$MUSL/lib/libc.a\""));
    assert!(text.contains("GNU M4 -P prefixed macro expansion failed"));
    assert!(text.contains("GNU M4 -P accepted the unprefixed define builtin"));
    assert!(text.contains("GNU M4 accepted malformed quoted input"));
    assert!(text.contains("GNU M4 malformed-input rejection emitted no diagnostic"));
    assert!(!text.contains("m4-bin"));
    assert!(!text.contains("/bin/busybox awk"));
    assert!(!text.contains("deliberately small bootstrap m4 bridge"));
}

#[test]
fn flex_musl_bounds_varargs_and_declares_m4_with_both_fixture_polarities() {
    let text = std::fs::read_to_string(bootstrap_path("flex-2.6.4-musl.ncl")).unwrap();
    assert!(text.contains("CFLAGS=\"-I. -Isrc -I$MUSL/include -DHAVE_CONFIG_H\""));
    assert!(text.contains("ERROR: unexpected Flex external-filter call shape"));
    assert!(text.contains("ERROR: multiple Flex external-filter calls require varargs"));
    assert!(text.contains("FLEX_EXTERNAL_FILTER_ARGUMENT_COUNT"));
    assert!(text.contains("FLEX_EXTERNAL_FILTER_VECTOR_LENGTH"));
    assert!(text.contains("unexpected external-filter chain shape"));
    assert!(text.contains("f->argv[1] = \"-P\""));
    assert!(text.contains("void lerr (const char *msg, ...)"));
    assert!(text.contains("void lerr_fatal (const char *msg, ...)"));
    assert!(text.contains("ERROR: Flex varargs remained after bounded source normalization"));
    assert!(text.contains("int filter_fix_linedirs (struct filter *chain)"));
    assert!(text.contains("error flushing filtered output"));
    assert!(text.contains("tables tables_shared tblcmp"));
    assert!(text.contains("/tmp/flex-compat.c"));
    assert!(text.contains("FLEX_BYTE_ZERO_MASK"));
    assert!(text.contains("ERROR: bounded Flex compatibility object missing"));
    assert!(!text.contains("flex_trace_marker"));
    assert!(text.contains("import \"tcc-musl-v2.ncl\""));
    assert!(text.contains("import \"musl-1.1.24-native.ncl\""));
    assert!(text.contains("TCC=$(find_input tcc-0.9.27-musl-v2)"));
    assert!(text.contains("MUSL=$(find_input musl-1.1.24-native-candidate)"));
    assert!(text.contains("tcc -c $CFLAGS \"src/$src.c\""));
    assert!(text.contains("\"$MUSL/lib/crt1.o\" $OBJS \"$MUSL/lib/libc.a\""));
    assert!(!text.contains("tcc-stdarg-prefix.h"));
    assert!(text.contains("import \"m4-1.4.7-musl.ncl\""));
    assert!(text.contains("M4=$(find_input m4-1.4.7-musl)"));
    assert!(text.contains("#define M4 \"$M4/bin/m4\""));
    assert!(text.contains("M4=\"$M4/bin/m4\" \"$out/bin/flex\" -o /tmp/flex-smoke.c"));
    assert!(text.contains("/tmp/flex-negative.l"));
    assert!(text.contains("ERROR: flex accepted malformed scanner input"));
    assert!(text.contains("ERROR: flex retained malformed scanner output"));
    assert!(!text.contains("#define M4 \"/bin/m4\""));
}

#[test]
fn gawk_musl_uses_proven_tcc_with_bounded_numeric_runtime_and_generator_checks() {
    let text = std::fs::read_to_string(bootstrap_path("gawk-3.0.4-musl.ncl")).unwrap();

    assert!(text.contains("import \"tcc-musl-v2.ncl\""));
    assert!(text.contains("import \"musl-1.1.24-native.ncl\""));
    assert!(text.contains("TCC=$(find_input tcc-0.9.27-musl-v2)"));
    assert!(text.contains("MUSL=$(find_input musl-1.1.24-native-candidate)"));
    assert!(text.contains("#define MANTLE_DECIMAL_INPUT_BYTES_MAX 4096"));
    assert!(text.contains("mantle_double_to_ulong_bounded(value, valid_out)"));
    assert!(text.contains("-Dstrtod=mantle_strtod -Datof=mantle_atof"));
    assert!(text.contains("\"$MUSL/lib/crt1.o\" $OBJS \"$MUSL/lib/libc.a\""));
    assert!(text.contains("gawk decimal arithmetic smoke failed"));
    assert!(text.contains("gawk generator string-length smoke failed"));
    assert!(text.contains("gawk generator hexadecimal-format smoke failed"));
    assert!(text.contains("invalid gawk program unexpectedly succeeded"));
    assert!(!text.contains("import \"tinycc-mes.ncl\""));
    assert!(!text.contains("TCC_RUNTIME="));
    assert!(!text.contains("$TCC_RUNTIME/lib/mes/tcc/libtcc1.a"));
    assert!(!text.contains("$TCC_RUNTIME/lib/mes/libc.a"));
    assert!(!text.contains("tcc-musl-native.ncl"));
    assert!(!text.contains("tcc-0.9.27-musl-native-runtime"));
}

#[test]
fn diffutils_musl_uses_proven_tcc_and_rejects_compiler_teardown_outputs() {
    let text = std::fs::read_to_string(bootstrap_path("diffutils-2.7-musl.ncl")).unwrap();

    assert!(text.contains("derivationFile \"tcc-musl-v2.ncl\""));
    assert!(text.contains("derivationFile \"musl-1.1.24-native.ncl\""));
    assert!(text.contains("TCC=$(find_input tcc-0.9.27-musl-v2)"));
    assert!(text.contains("MUSL=$(find_input musl-1.1.24-native-candidate)"));
    assert!(text.contains("ERROR: TCC-v2 failed compiling $source_path"));
    assert!(text.contains("diffutils 2.7 source-built cmp/diff runtime and rejection checks passed"));
    assert!(text.contains("ERROR: cmp did not detect different files"));
    assert!(text.contains("ERROR: diff accepted malformed input"));
    assert!(!text.contains("tcc-musl-native.ncl"));
    assert!(!text.contains("tcc-musl-selfhost.ncl"));
    assert!(!text.contains("accepted complete object before known compiler teardown fault"));
    assert!(!text.contains("COMPILER_TEARDOWN_SIGNAL_STATUS"));
    assert!(!text.contains("libtcc1.a"));
}

#[test]
fn bison_23_musl_builds_complete_declared_sources_with_bounded_hash_semantics() {
    let text = std::fs::read_to_string(bootstrap_path("bison-2.3-musl.ncl")).unwrap();

    assert!(text.contains("import \"tcc-musl-v2.ncl\""));
    assert!(text.contains("import \"musl-1.1.24-native.ncl\""));
    assert!(text.contains("import \"m4-1.4.7-musl.ncl\""));
    assert!(text.contains("EXPECTED_LIB_SOURCE_COUNT=27"));
    assert!(text.contains("EXPECTED_PROGRAM_SOURCE_COUNT=29"));
    assert!(text.contains("mantle_hash_ratio_ceil"));
    assert!(text.contains("mantle_hash_growth_reached"));
    assert!(text.contains("bison23-runtime-check=positive-grammar"));
    assert!(text.contains("ERROR: Bison accepted malformed grammar"));
    assert!(text.contains("ERROR: Bison retained malformed parser output"));
    assert!(!text.contains("musl-1.1.24-tcc-musl.ncl"));
    assert!(!text.contains("tcc-musl-native.ncl"));
    assert!(!text.contains("2>/dev/null || true"));
    assert!(!text.contains("tcc -ar"));
}

#[test]
fn binutils_tcc_uses_declared_generators_and_checks_the_full_tool_matrix() {
    let text = std::fs::read_to_string(bootstrap_path("binutils-tcc.ncl")).unwrap();

    assert!(text.contains("derivationFile \"tcc-musl-v2.ncl\""));
    assert!(text.contains("derivationFile \"musl-1.1.24-native.ncl\""));
    assert!(text.contains("derivationFile \"bison-2.3-musl.ncl\""));
    assert!(text.contains("derivationFile \"flex-2.6.4-musl.ncl\""));
    assert!(text.contains("derivationFile \"gawk-3.0.4-musl.ncl\""));
    assert!(text.contains("authenticated-release-configure-plus-declared-bison-flex-native-generators"));
    assert!(text.contains("CONFIG_SUB_TIMEOUT_SECONDS=10"));
    assert!(text.contains("timeout \"$CONFIG_SUB_TIMEOUT_SECONDS\" /bin/sh ./config.sub sun4"));
    assert!(text.contains("timeout \"$CONFIGURE_TIMEOUT_SECONDS\" /bin/sh ./configure"));
    assert!(text.contains("CONFIGURE_PROBE_SOURCE_BYTES_MAX=65536"));
    assert!(text.contains("CONFIGURE_PROBE_INVOCATION_COUNT_MAX=4096"));
    assert!(text.contains("intl|libiberty|zlib|bfd|opcodes|binutils|gas|gprof|ld"));
    assert!(text.contains("reject_probe missing-authority-directory"));
    assert!(text.contains("reject_probe source-too-large"));
    assert!(text.contains("BFD_BOOTSTRAP_CONFIG_INPUT_REFS_EXPECTED=2"));
    assert!(text.contains("bfd-configure.authenticated"));
    assert!(text.contains("ERROR: second-pass BFD header missing"));
    assert!(text.contains("ARCHIVE_MEMBER_COUNT_MAX=1024"));
    assert!(text.contains("ARCHIVE_MEMBER_BYTES_MAX=67108864"));
    assert!(text.contains("ARCHIVE_OUTPUT_BYTES_MAX=1073741824"));
    assert!(text.contains("reject_archive unsafe-member-name"));
    assert!(text.contains("reject_archive member-too-large"));
    assert!(text.contains("reject_archive output-too-large"));
    assert!(text.contains("MANTLE_BINARY_RADIX = 2"));
    assert!(text.contains("RUNTIME_ASM_OBJECT=\"$WORK/binutils-runtime-asm.o\""));
    assert!(text.contains("sigsetjmp:\n  jmp setjmp"));
    assert!(text.contains("ERROR: missing regenerated artifact $generated_path"));
    assert!(text.contains("ERROR: unused zlib CRC table was retained"));
    assert!(text.contains("ERROR: unused zlib inflate table was retained"));
    assert!(text.contains("for required_tool in as ld ar ranlib nm objcopy objdump readelf size strings strip"));
    assert!(text.contains("ERROR: source-built assembler accepted malformed input"));
    assert!(text.contains("ERROR: source-built linker accepted malformed object input"));
    assert!(text.contains("ERROR: source-built archive tool accepted malformed archive input"));
    assert!(!text.contains("musl-1.1.24-tcc-musl.ncl"));
    assert!(!text.contains("perl-5.6.2-musl.ncl"));
    assert!(!text.contains("autoconf-2.64.ncl"));
    assert!(!text.contains("automake-1.11.2.ncl"));
    assert!(!text.contains("coreutils-6.10-musl.ncl"));
    assert!(!text.contains("bash-2.05b-tcc.ncl"));
}

#[test]
fn tcc_musl_v2_installs_and_exercises_its_stdarg_contract() {
    let text = std::fs::read_to_string(bootstrap_path("tcc-musl-v2.ncl")).unwrap();

    assert!(text.contains("CONFIG_TCCDIR=\\\"$out/lib/tcc\\\""));
    assert!(text.contains("CONFIG_TCC_SYSINCLUDEPATHS=\"'\"$MUSL/include\"'\""));
    assert!(text.contains("$BB cp \"$out/lib/tcc/include/stdarg.h\" ./tcc-stdarg-prefix.h"));
    assert!(text.contains("#include \"tcc-stdarg-prefix.h\""));
    assert!(text.contains("-static -o variadic-positive variadic-positive.c"));
    assert!(text.contains("-c -o variadic-negative.o variadic-negative.c"));
    assert!(!text.contains("-nostdinc"));
    assert!(!text.contains("$out/lib/tcc/include:$MUSL/include"));
    assert!(text.contains("$BB cp -r include/. \"$out/lib/tcc/include/\""));
    assert!(text.contains("ERROR: TinyCC stdarg header missing from configured include root"));
    assert!(text.contains("/tmp/tcc-stdarg-runtime.c"));
    assert!(text.contains("TCC_VA_GENERAL_REGISTER_BYTES"));
    assert!(text.contains("TCC_VA_FLOAT_REGISTER_BYTES"));
    assert!(text.contains("-ar rcs \"$out/lib/tcc/libtcc1.a\" /tmp/tcc-stdarg-runtime.o"));
    assert!(text.contains("ERROR: extended TinyCC runtime archive missing"));
    assert!(text.contains("variadic-positive.c"));
    assert!(text.contains("value = va_arg(arguments, int)"));
    assert!(text.contains("./variadic-positive"));
    assert!(text.contains("variadic-negative.c"));
    assert!(text.contains("return va_arg(arguments, )"));
    assert!(text.contains("ERROR: TinyCC accepted malformed variadic source"));
}

#[test]
fn selfhosted_tinycc_candidate_rebuilds_compiler_source() {
    let text = std::fs::read_to_string(bootstrap_path("tcc-musl-selfhost.ncl")).unwrap();

    assert!(text.contains("name = \"tcc-0.9.27-musl-selfhost\""));
    assert!(text.contains("crunch.derivationFile \"tcc-musl-v2.ncl\""));
    assert!(text.contains("$HOST_TCC/share/tcc-source"));
    assert!(text.contains("ERROR: normalized TinyCC source missing"));
    assert!(text.contains("$HOST_TCC/bin/tcc"));
    assert!(text.contains("compile compiler sources separately"));
    assert!(text.contains("-I \"$TCC26/include/mes\""));
    assert!(text.contains("memcpy(\\&dtype, type, sizeof(CType))"));
    assert!(text.contains("sym->st_info == 0x20"));
    assert!(text.contains("sym->st_info == 0x22"));
    assert!(text.contains("if (!attr) return;"));
    assert!(text.contains("bootstrap_undefined_symbol(name)"));
    assert!(text.contains("tcc: undefined symbol: "));
    assert!(text.contains("if (!s1->got || !s1->got->reloc)"));
    assert!(text.contains("rel->r_offset < s1->got->sh_addr"));
    assert!(text.contains("local GOT relocation exceeds GOT"));
    assert!(!text.contains("unsigned offset = attr->got_offset;\\n            if (offset != rel->r_offset"));
    assert!(text.contains("else if (sizeof (long double) == sizeof (double))"));
    assert!(text.contains("vsnprintf(buf + len, buf_size - len, fmt, ap)"));
    assert!(text.contains("-D ONE_SOURCE=0"));
    assert!(text.contains("-ar rc libtcc-selfhost.a"));
    assert!(text.contains("link source-built objects with declared predecessor linker"));
    assert!(text.contains("$TCC26/bin/tcc"));
    assert!(text.contains("ERROR: self-hosted TinyCC stdarg header missing"));
    assert!(text.contains("syntax-smoke.c"));
    assert!(text.contains("va_arg(arguments, int)"));
    assert!(!text.contains("-D ONE_SOURCE=1"));
    assert!(!text.contains("SValue tmp = vtop[0]"));
}

#[test]
fn native_tcc_diagnostics_remain_bounded_and_non_admitted() {
    let text = std::fs::read_to_string(bootstrap_path("tcc-musl-native.ncl")).unwrap();
    let selected_seed = std::fs::read_to_string(bootstrap_path("seed.ncl")).unwrap();

    assert!(text.contains("emitting the format literal without interpolated arguments"));
    assert!(text.contains("include-smoke.h"));
    assert!(text.contains("compile nested-include smoke"));
    assert!(text.contains("Dependency-list growth crashes"));
    assert!(text.contains("malformed-negative.c"));
    assert!(text.contains("tcc: fatal compile error"));
    assert!(!selected_seed.contains("tcc-musl-native.ncl"));
}

#[test]
fn native_musl_candidate_preserves_runtime_sources() {
    let text = std::fs::read_to_string(bootstrap_path("musl-1.1.24-native.ncl")).unwrap();

    assert!(text.contains("name = \"musl-1.1.24-native-candidate\""));
    assert!(text.contains("crunch.derivationFile \"tcc-musl-selfhost.ncl\""));
    assert!(text.contains("crunch.derivationFile \"musl-1.1.24-src.ncl\""));
    assert!(text.contains("--host=x86_64-linux-musl"));
    assert!(text.contains("CFLAGS_AUTO="));
    assert!(text.contains("CFLAGS=\"-DSYSCALL_NO_TLS\""));
    assert!(text.contains("ERROR: musl configure script missing"));
    assert!(text.contains("void __va_start(void *, void *);"));
    assert!(text.contains("__builtin_va_arg_types(type)"));
    assert!(text.contains("va_general_register_bytes = 48"));
    assert!(text.contains("arguments->overflow_arg_area - argument_bytes"));
    assert!(text.contains("for math_source in __fpclassifyl.c __signbitl.c frexp.c frexpl.c ldexp.c"));
    assert!(text.contains(
        "$BB rm -rf src/aio src/complex src/fenv src/ldso src/legacy src/linux src/math src/network src/sched"
    ));
    assert!(text.contains("andq $STACK_ALIGNMENT_MASK,%rsp"));
    assert!(text.contains("call __libc_start_main"));
    assert!(text.contains("$TCC26/bin/tcc\" -c crt/crt1.c"));
    assert!(text.contains("src/string/x86_64/memcpy.s"));
    assert!(text.contains("src/string/x86_64/memmove.s src/string/x86_64/memset.s"));
    assert!(text.contains("for assembly_source in src/*/x86_64/*.s"));
    assert!(text.contains("static void force_text_section(void) {}"));
    assert!(text.contains("scratch=\"/tmp/mantle-tcc-ar-\\$\\$\""));
    assert!(text.contains("short=\"\\$scratch/o\\$index.o\""));
    assert!(!text.contains("exec \"$TCC_CLEAN\" -ar rcs \"\\$@\""));
    assert!(text.contains("src/locale/dcngettext.c"));
    assert!(!text.contains("rm -f src/locale/__lctrans.c"));
    assert!(!text.contains("rm -f src/locale/setlocale.c"));
    assert!(text.contains("$BB rm -f crt/Scrt1.c crt/rcrt1.c"));
    assert!(text.contains("char __buf[15+3*sizeof(int)]"));
    assert!(text.contains("hidden struct pthread *__pthread_self(void);"));
    assert!(text.contains("#define TP_ADJ(p) (p)"));
    assert!(text.contains("#define MC_PC gregs[REG_RIP]"));
    assert!(text.contains("mov %fs:0,%rax"));
    assert!(!text.contains("__asm__ (\"mov %%fs:0,%0\""));
    assert!(text.contains("td->next = td; td->prev = td;"));
    assert!(text.contains("td->dtv = dtv; td->dtv_copy = dtv;"));
    assert!(text.contains("libc.global_locale.cat[i] = tmp_locale.cat[i]"));
    assert!(text.contains("#define MIN_TLS_ALIGN ((size_t)\\&((struct builtin_tls *)0)->pt)"));
    assert!(text.contains("mem = __bootstrap_anonymous_mmap(libc.tls_size);"));
    assert!(text.contains("__syscall_ret(__bootstrap_raw_anonymous_mmap(size))"));
    assert!(text.contains(".equ SYS_MMAP, 9"));
    assert!(text.contains(".equ MAP_ANONYMOUS_PRIVATE, 0x22"));
    assert!(text.contains("UTF8_LOCALE_MB_MAX"));
    assert!(text.contains("current->cat[LC_CTYPE]"));
    assert!(text.contains("INTERNAL_SIGNAL_COUNT = 3"));
    assert!(text.contains("(unsigned)signal_number - INTERNAL_SIGNAL_BASE"));
    assert!(text.contains("kernel_action.restorer = __restore_rt;"));
    assert!(text.contains("static int allocation_size(size_t requested_bytes, size_t *mapped_bytes)"));
    assert!(text.contains("if (!IS_MMAPPED(chunk)) abort();"));
    assert!(text.contains("size_t copy_bytes = requested_bytes < available_bytes"));
    assert!(text.contains("weak_alias(__memalign, memalign);"));
    assert!(text.contains("CONFSTR_MAX_OFFSET = 33"));
    assert!(text.contains("buffer[copy_bytes] = 0;"));
    assert!(text.contains("bit_index <= (int)(sizeof(bits) * CHAR_BIT)"));
    assert!(text.contains(".global __syscall_cp_c"));
    assert!(text.contains("void __lock(volatile int *lock)"));
    assert!(text.contains("(void)lock;"));
    assert!(text.contains("MIN_SORTABLE_OBJECTS = 2"));
    assert!(text.contains("if (object_count > SIZE_MAX / object_bytes) return;"));
    assert!(text.contains("ORPHANED_LOCK_VALUE = 0x40000000"));
    assert!(text.contains("stream->lockcount == LONG_MAX"));
    assert!(text.contains("int acquired = __lockfile(stream);"));
    assert!(text.contains("if (acquired) __unlockfile(stream);"));
    assert!(text.contains("if (libc.threads_minus_1) abort();"));
    assert!(text.contains("stream->lock = thread_id;"));
    assert!(text.contains("status->st_ctim.tv_nsec = kernel_status.st_ctime_nsec;"));
    assert!(text.contains("__bootstrap_syscall_array(SYS_fstatat, arguments)"));
    assert!(text.contains("MMAP2_OFFSET_MASK_BASE 0x2000ULL"));
    assert!(text.contains("result = __bootstrap_syscall_array(SYS_mmap, arguments);"));
    assert!(text.contains("SYSCALL_ARGUMENT_COUNT = 6"));
    assert!(text.contains("__syscall_ret(__bootstrap_syscall_array(number, arguments))"));
    assert!(text.contains(".equ ARGUMENT_5_OFFSET, 40"));
    assert!(text.contains("assembler crashes on symbolic .equ operands"));
    assert!(!text.contains("return 4;"));
    assert!(text.contains("typedef __mantle_va_list_struct va_list[1];"));
    assert!(text.contains("typedef __mantle_va_list_struct __isoc_va_list[1];"));
    assert!(!text.contains("typedef char *__builtin_va_list;"));
    assert!(text.contains("ERROR: musl x86_64 alltypes input missing"));
    assert!(text.contains("chmod u+rw $@"));
    assert!(text.contains("native-musl-smoke-ok"));
    assert!(text.contains("sigaction(0, &action, NULL) != -1"));
    assert!(text.contains("malloc(SIZE_MAX) != NULL"));
    assert!(text.contains("snprintf(output, truncated_capacity"));
    assert!(text.contains("$TCC_CLEAN\" -nostdlib -static"));
    assert!(text.contains("ERROR: native musl runtime smoke output mismatch"));
    assert!(text.contains("test -f \"$out/lib/libc.a\""));
    assert!(text.contains("test -f \"$out/lib/crt1.o\""));
    assert!(!text.contains("minstdio.c"));
    assert!(!text.contains("minstdlib.c"));
    assert!(!text.contains("rm -rf src/complex src/aio"));
}

#[test]
fn gawk_string_macros_are_config_header_owned() {
    let text = std::fs::read_to_string(bootstrap_path("gawk-3.0.4-musl.ncl")).unwrap();

    assert!(text.contains("import \"tcc-musl-v2.ncl\""));
    assert!(text.contains("import \"musl-1.1.24-native.ncl\""));
    assert!(!text.contains("import \"tcc-musl-native.ncl\""));
    assert!(text.contains("$TCC/bin/tcc\" -c $CFLAGS -Dalloca=malloc awktab.c"));
    assert!(text.contains("#define VERSION \"3.0.4\""));
    assert!(text.contains("#define PACKAGE \"gawk\""));
    assert!(text.contains("#define DEFPATH \".:/usr/share/awk\""));
    assert!(text.contains("#define HAVE_STDARG_H 1"));
    assert!(text.contains("#define HAVE_VPRINTF 1"));
    assert!(text.contains("#define GETPGRP_VOID 1"));
    assert!(text.contains("#define GETGROUPS_T int"));
    assert!(text.contains("#define RETSIGTYPE void"));
    assert!(text.contains("#define HAVE_ALLOCA_H 1"));
    assert!(!text.contains("#define C_ALLOCA 1"));
    assert!(
        text.contains(
            "for src in mantle_decimal array builtin dfa eval field io main msg node random re regex version"
        )
    );
    assert!(!text.contains("field getopt getopt1 io"));
    assert!(text.contains("$CFLAGS gawkmisc.c -o gawkmisc.o"));
    assert!(!text.contains("$CFLAGS posix/gawkmisc.c"));
    assert!(text.contains("$MUSL/lib/crt1.o\" $OBJS \"$MUSL/lib/libc.a"));
    assert!(text.contains("EXPECTED_TOTAL=7"));
    assert!(text.contains("EXPECTED_MATCHES=2"));
    assert!(text.contains("printf 'alpha\\nbeta\\n' | LC_ALL=C ./gawk"));
    assert!(text.contains("invalid gawk program unexpectedly succeeded"));
    assert!(text.contains("invalid gawk program emitted no diagnostic"));
    assert!(!text.contains("mantle_compat.c"));
    assert!(text.contains("CFLAGS=\"-I. -I$MUSL/include -DHAVE_CONFIG_H -Dstrtod=mantle_strtod -Datof=mantle_atof\""));
    assert!(!text.contains("-DVERSION=\\\\\\\""));
    assert!(!text.contains("-DPACKAGE=\\\\\\\""));
}

#[test]
fn source_built_coreutils_handoff_rejects_busybox_output_wrappers() {
    let coreutils5 = std::fs::read_to_string(bootstrap_path("coreutils-5.0-musl.ncl")).unwrap();
    let coreutils6 = std::fs::read_to_string(bootstrap_path("coreutils-6.10-musl.ncl")).unwrap();
    let selected_seed = std::fs::read_to_string(bootstrap_path("seed.ncl")).unwrap();

    assert!(coreutils5.contains("link_utility cat src/cat.c"));
    assert!(coreutils5.contains("link_utility rm src/rm.c src/remove.c"));
    assert!(coreutils5.contains("source-built coreutils output is a script"));
    assert!(coreutils5.contains("source-built rm accepted malformed input"));
    assert!(!coreutils5.contains("coreutils-5.0-predecessor-tool="));
    assert!(!coreutils5.contains("exec /bin/busybox %s"));
    assert!(coreutils6.contains("coreutils-6.10-predecessor-tool="));
    assert!(coreutils6.contains("cannot satisfy source-built provider admission"));
    assert!(!selected_seed.contains("coreutils-5.0-musl.ncl"));
    assert!(!selected_seed.contains("coreutils-6.10-musl.ncl"));
}

#[test]
fn source_built_sbase_remains_separate_from_bounded_gcc_tool_path() {
    let sbase = std::fs::read_to_string(bootstrap_path("sbase-tools.ncl")).unwrap();
    let gcc = std::fs::read_to_string(bootstrap_path("gcc-4.0-native.ncl")).unwrap();

    assert!(sbase.contains("git://git.suckless.org/sbase"));
    assert!(sbase.contains("c546c3a5724c81cee9a11d816a38ccdf17472129"));
    assert!(sbase.contains("TOOLS=\"basename cat chmod cmp cp cut dirname echo expr"));
    assert!(sbase.contains("sbase source-built utility runtime and rejection checks passed"));
    assert!(sbase.contains("sbase expr accepted malformed input"));
    assert!(sbase.contains("RANLIB=\"$BINUTILS/bin/ranlib\""));
    assert!(!sbase.contains("RANLIB=true"));
    assert!(!sbase.contains("ln -sf \"$BB\""));
    assert!(!gcc.contains("crunch.derivationFile \"sbase-tools.ncl\""));
    assert!(!gcc.contains("find_input sbase-tools-c546c3a"));
    assert!(!gcc.contains("find_input coreutils-5.0-musl"));
    assert!(gcc.contains("TOOLBIN=/tmp/gcc40-tools"));
    assert!(gcc.contains("export PATH=\"$TOOLBIN:$TCC/bin:$BINUTILS/bin"));
    assert!(!gcc.contains("$STAGE0/bin:$PATH"));
}

#[test]
fn gcc_native_diagnostic_uses_runtime_tcc_without_autotools_claims() {
    let text = std::fs::read_to_string(bootstrap_path("gcc-4.0-native.ncl")).unwrap();
    let selected_seed = std::fs::read_to_string(bootstrap_path("seed.ncl")).unwrap();

    assert!(text.contains("let stagex_provider = import \"stagex-provider-proof-input.ncl\""));
    assert!(text.contains("let stagex_transition = import \"stagex-transition-proof-input.ncl\""));
    assert!(text.contains("STAGEX=$(find_input mantle-stagex-intermediate-provider)"));
    assert!(text.contains("MUSL=\"$STAGEX/x86_64-linux-musl\""));
    assert!(!text.contains("crunch.derivationFile \"tcc-musl-native.ncl\""));
    assert!(!text.contains("crunch.derivationFile \"musl-1.1.24-native.ncl\""));
    assert!(text.contains("Use the release tarball's generated configure"));
    assert!(text.contains("test -f gcc/c-parse.c"));
    assert!(text.contains("ucnid-table.o"));
    assert!(text.contains("admission still forbids this bridge"));
    assert!(text.contains("diagnostic two-pass build complete; not provider admission"));
    assert!(text.contains("export CONFIG_SHELL=/bin/sh"));
    assert!(text.contains("build_pass \"$WORK/pass1-src\""));
    assert!(text.contains("build_pass \"$WORK/pass2-src\""));
    assert!(text.contains("current_dir=\\$(/bin/busybox pwd -P)"));
    assert!(text.contains("source_canonical=\\$(/bin/busybox readlink -f"));
    assert!(text.contains("probe_count=\\$(/bin/busybox cat"));
    assert!(!text.contains("current_dir=$(/bin/busybox pwd -P)"));
    assert!(!text.contains("source_canonical=$(/bin/busybox readlink -f"));
    assert!(!text.contains("find_input perl-5.6.2-musl"));
    assert!(!text.contains("find_input automake-1.9.6"));
    assert!(!selected_seed.contains("gcc-4.0-native.ncl"));
}

#[test]
fn full_source_provider_records_authenticated_closure_and_is_selected() {
    let candidate = std::fs::read_to_string(bootstrap_path("seed-full.ncl")).unwrap();
    let final_gcc = std::fs::read_to_string(bootstrap_path("gcc-10-final.ncl")).unwrap();
    let selected_seed = std::fs::read_to_string(bootstrap_path("seed.ncl")).unwrap();

    assert!(candidate.contains("\"status\": \"derivational-closure-passed\""));
    assert!(candidate.contains("\"state_pinned_inputs\": false"));
    assert!(candidate.contains("\"release_generated_sources\": []"));
    assert!(candidate.contains("--sysroot=\"\\$sysroot\""));
    assert!(!candidate.contains("-isystem \"\\$sysroot/include\""));
    assert!(candidate.contains("NEEDED.*\\[libc.so\\]"));
    assert!(!candidate.contains("blocked-pending-authenticated-closure"));
    assert!(!candidate.contains("/mantle/store/"));
    assert!(final_gcc.contains("dynamic_library_argument()"));
    assert!(final_gcc.contains("-l:\\$dynamic_library_name"));
    assert!(final_gcc.contains("NEEDED.*\\[libc.so\\]"));
    assert!(!final_gcc.contains("libc_library=\"\\${MANTLE_GCC_LINK_LIBC:-$MUSL/lib/libc.so}\""));
    assert!(selected_seed.contains("import \"seed-full.ncl\""));
    assert!(!selected_seed.contains("import \"seed-legacy.ncl\""));
    assert!(!selected_seed.contains("CRUNCH_LEGACY_SEED"));
}

#[test]
fn self_hosting_proof_defaults_to_full_source_provider_identity() {
    let proof_helper =
        std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/prove-self-hosting.sh"))
            .unwrap();

    assert!(proof_helper.contains("readonly PROOF_PROVIDER_KIND_FULL_SOURCE=\"full-source\""));
    assert!(proof_helper.contains("proof_provider_kind=\"$PROOF_PROVIDER_KIND_FULL_SOURCE\""));
    assert!(!proof_helper.contains("proof_provider_kind=\"$PROOF_PROVIDER_KIND_LEGACY_FETCH\""));
}

#[test]
fn linux_header_derivation_evaluates_with_direct_runtime_inputs() {
    const EXPECTED_DERIVATION_INPUT_COUNT: usize = 1;
    const EXPECTED_LAZY_INPUT_COUNT: usize = 2;
    let drv = eval_bootstrap("linux-headers.ncl");
    let derivation_input_count = drv.inputs.iter().filter(|input| matches!(input, Input::Derivation(_))).count();
    let lazy_input_count = drv.inputs.iter().filter(|input| matches!(input, Input::DerivationFile(_))).count();

    assert_eq!(drv.name, "linux-headers-6.6");
    assert_eq!(derivation_input_count, EXPECTED_DERIVATION_INPUT_COUNT);
    assert_eq!(lazy_input_count, EXPECTED_LAZY_INPUT_COUNT);
    assert!(!drv.args.join(" ").contains("SEED_ROOT"));
}

#[test]
fn perl_gcc_generators_keep_normalized_inputs_and_runtime_rails() {
    let perl_5000 = std::fs::read_to_string(bootstrap_path("perl-5.000-gcc.ncl")).unwrap();
    let perl_500503 = std::fs::read_to_string(bootstrap_path("perl-5.005_03-gcc.ncl")).unwrap();

    let perl_5000_required = [
        "BASE=$(find_input gcc-generator-base-v4)",
        "GCC=\"$BASE/gcc\"",
        "MANTLE_GCC_LINK_LIBC=\"$MUSL/lib/libc.a\"",
        "-B\"$GCC/bin/\"",
    ];
    let perl_5000_forbidden = [
        "GCC=$(find_input gcc-4.0.4-musl-pass4-v7)",
        "MUSL=$(find_input musl-1.1.24-gcc-pass4-v2)",
    ];
    let perl_500503_required = [
        "version 5.005_03",
        "perl500503-gcc-ok",
        "ERROR: Perl 5.005_03 accepted malformed source",
        "test ! -s \"$WORK/rejected.out\"",
        "ELF64",
    ];
    let perl_500503_forbidden = ["ERROR: Perl 5.003 accepted malformed source", "-O0", "gcc-10.5.0"];

    let perl_5000_violations = contract_violations(&perl_5000, &perl_5000_required, &perl_5000_forbidden);
    let perl_500503_violations = contract_violations(&perl_500503, &perl_500503_required, &perl_500503_forbidden);

    assert!(perl_5000_violations.is_empty(), "Perl 5.000 contract violations: {perl_5000_violations:?}");
    assert!(perl_500503_violations.is_empty(), "Perl 5.005_03 contract violations: {perl_500503_violations:?}");
}

#[test]
fn perl_gcc_runtime_contract_rejects_stale_malformed_source_diagnostic() {
    let source = std::fs::read_to_string(bootstrap_path("perl-5.005_03-gcc.ncl")).unwrap();
    let required = ["ERROR: Perl 5.005_03 accepted malformed source"];
    let forbidden = ["ERROR: Perl 5.003 accepted malformed source"];
    let invalid_source = source.replace(required[0], forbidden[0]);

    let violations = contract_violations(&invalid_source, &required, &forbidden);

    assert_eq!(violations, vec![required[0], forbidden[0]]);
    assert!(!invalid_source.contains(required[0]));
}

#[test]
fn all_bootstrap_entrypoints_evaluate() {
    for entrypoint in BOOTSTRAP_ENTRYPOINTS {
        let drv = eval_bootstrap(entrypoint);
        assert!(!drv.name.is_empty(), "entrypoint must evaluate: {entrypoint}");
        assert!(!drv.builder.is_empty(), "builder must stay present: {entrypoint}");
    }
}
