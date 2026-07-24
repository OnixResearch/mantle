use std::ffi::OsString;
use std::path::PathBuf;

use crunch_glue::CrunchDerivation;
use crunch_glue::Input;
use serde::Deserialize;

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

fn input_derivation_names(drv: &CrunchDerivation) -> Vec<String> {
    let mut names = Vec::new();
    for input in &drv.inputs {
        if let Input::Derivation(dep) = input {
            names.push(dep.name.clone());
        }
    }
    names
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

    assert!(text.contains("import \"tcc-musl-native.ncl\""));
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
    assert!(text.contains("for src in array builtin dfa eval field io"));
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
    assert!(text.contains("CFLAGS=\"-I. -I$MUSL/include -DHAVE_CONFIG_H\""));
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
fn source_built_sbase_handoff_replaces_busybox_path_fallbacks() {
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
    assert!(gcc.contains("crunch.derivationFile \"sbase-tools.ncl\""));
    assert!(gcc.contains("find_input sbase-tools-c546c3a"));
    assert!(!gcc.contains("find_input coreutils-5.0-musl"));
    assert!(!gcc.contains("TOOLBIN=/tmp/gcc40-native-tools"));
}

#[test]
fn gcc_native_diagnostic_uses_runtime_tcc_without_autotools_claims() {
    let text = std::fs::read_to_string(bootstrap_path("gcc-4.0-native.ncl")).unwrap();
    let selected_seed = std::fs::read_to_string(bootstrap_path("seed.ncl")).unwrap();

    assert!(text.contains("crunch.derivationFile \"tcc-musl-native.ncl\""));
    assert!(text.contains("crunch.derivationFile \"musl-1.1.24-native.ncl\""));
    assert!(text.contains("Use the release tarball's generated configure"));
    assert!(text.contains("test -f gcc/c-parse.c"));
    assert!(text.contains("ucnid-table.o"));
    assert!(text.contains("admission still forbids this bridge"));
    assert!(text.contains("diagnostic two-pass build complete; not provider admission"));
    assert!(text.contains("export CONFIG_SHELL=/bin/sh"));
    assert!(text.contains("build_pass \"$WORK/pass1-src\""));
    assert!(text.contains("build_pass \"$WORK/pass2-src\""));
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
fn all_bootstrap_entrypoints_evaluate() {
    for entrypoint in BOOTSTRAP_ENTRYPOINTS {
        let drv = eval_bootstrap(entrypoint);
        assert!(!drv.name.is_empty(), "entrypoint must evaluate: {entrypoint}");
        assert!(!drv.builder.is_empty(), "builder must stay present: {entrypoint}");
    }
}
