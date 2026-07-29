use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use serde::Serialize;

const RECIPE_ARTIFACT_ID: &str = "musl-native-recipe-source";
const RECIPE_BLAKE3: &str = "0f86c28c5a4f7290002200871edede784e79566b4f02c9ebb1f22b654c1e6c1c";
const RECIPE: &[u8] = include_bytes!("../bootstrap/musl-1.1.24-native.ncl");
const REPORT_FORMAT: &str = "mantle-stagex-musl-1.1.24-native-inventory-v1";
const NON_CLAIM: &str = "this inventory binds a static upstream-shaped musl candidate built by the protected self-hosted TinyCC with declared TinyCC musl-v2 assembly, math, and malformed-source diagnostic assistance; it does not prove complete musl behavior, dynamic runtime, compiler correctness, or provider admission";
const CONFIGURED_SOURCE_DOMAIN: &[u8] = b"mantle-stagex-musl-native-configured-source-v1\0";
pub(crate) const CONFIGURED_SOURCE_BLAKE3: &str = "25136ba596733447cbb6259862f4e0b37dd557fc4dd8cc4392912e2f34880f18";
const FILE_BYTES_MAX: u64 = 64 * 1_024 * 1_024;
const TREE_ENTRY_COUNT_MAX: usize = 32_768;
const EMPTY_ARCHIVE_BYTES: &[u8] = b"!<arch>\n";
const EMPTY_ARCHIVE_BLAKE3: &str = "3ba363b1c314e158a3ed3769a2d8c73272b3a01da712c43ecc25af2e4295a3bb";
const AR_HEADER_BYTES: usize = 60;
const AR_TIMESTAMP_START: usize = 16;
const AR_TIMESTAMP_END: usize = 28;
const AR_OWNER_END: usize = 34;
const AR_GROUP_END: usize = 40;
const AR_MODE_END: usize = 48;
const AR_SIZE_END: usize = 58;
const AR_TRAILER_START: usize = 58;
const AR_TRAILER_BYTES: &[u8] = b"`\n";
const AR_TIMESTAMP_NORMALIZED: &[u8] = b"0           ";
const AR_OWNER_NORMALIZED: &[u8] = b"0     ";
const AR_GROUP_NORMALIZED: &[u8] = b"0     ";
const AR_MODE_NORMALIZED: &[u8] = b"100644  ";
const AR_ALIGNMENT_BYTES: usize = 2;
const SMOKE_COMMAND_COUNT: u32 = 2;
const EXECUTION_COMMAND_COUNT: u32 = 2;
const ARCHIVE_COMMAND_COUNT: u32 = 1;
pub(crate) const SELFHOST_COMPILE_COUNT: u32 = 746;
pub(crate) const PREDECESSOR_COMPILE_COUNT: u32 = 19;
const OUTPUT_COUNT: usize = 15;
const VALIDATION_RECEIPT_BYTES: &[u8] = b"native-musl-validation-ok\n";
const VALIDATION_RECEIPT_BLAKE3: &str = "f578c82fb2c9eb48d87b171daedaef41f63c7f82b64d26b14e6f686f9c3f230e";
const POSITIVE_SMOKE_TARGET: &str = "\"$WORK/native-musl-smoke.c\"";
const MALFORMED_SMOKE_SOURCE: &[u8] = b"int broken( {\n";

const COMMON_COMPILE_FLAGS: [&str; 12] = [
    "-std=c99",
    "-nostdinc",
    "-I./arch/x86_64",
    "-I./arch/generic",
    "-Iobj/src/internal",
    "-I./src/include",
    "-I./src/internal",
    "-Iobj/include",
    "-I./include",
    "-D_XOPEN_SOURCE=700",
    "-DSYSCALL_NO_TLS",
    "-fno-stack-protector",
];

const HELPER_TARGETS: [&str; 31] = [
    "include/stdarg.h",
    "src/internal/va_list.c",
    "src/malloc/bootstrap_alloca.c",
    "src/math/bootstrap_gawk_math.c",
    "crt/crt1.c",
    "arch/x86_64/pthread_arch.h",
    "src/thread/x86_64/__pthread_self.s",
    "src/signal/sigaction.c",
    "src/mman/__bootstrap_anonymous_mmap.c",
    "src/mman/x86_64/__bootstrap_raw_anonymous_mmap.s",
    "src/malloc/malloc.c",
    "src/malloc/memalign.c",
    "src/conf/confstr.c",
    "src/misc/ffs.c",
    "src/misc/ffsl.c",
    "src/misc/ffsll.c",
    "src/thread/x86_64/__syscall_cp.s",
    "src/time/clock_gettime.c",
    "src/thread/vmlock.c",
    "src/thread/pthread_once.c",
    "src/thread/__wait.c",
    "src/thread/__lock.c",
    "src/stdlib/qsort.c",
    "src/stdio/getc.h",
    "src/stdio/putc.h",
    "src/stdio/ftrylockfile.c",
    "src/stdio/__lockfile.c",
    "src/stat/fstatat.c",
    "src/mman/mmap.c",
    "src/misc/syscall.c",
    "src/misc/x86_64/__bootstrap_syscall_array.s",
];

const REMOVED_DIRECTORIES: [&str; 9] = [
    "src/aio",
    "src/complex",
    "src/fenv",
    "src/ldso",
    "src/legacy",
    "src/linux",
    "src/math",
    "src/network",
    "src/sched",
];

const REMOVED_FILES: [&str; 23] = [
    "src/ctype/iswalpha.c",
    "src/ctype/iswalnum.c",
    "src/ctype/iswctype.c",
    "src/ctype/towctrans.c",
    "include/iconv.h",
    "src/locale/iconv.c",
    "src/locale/iconv_close.c",
    "src/locale/bind_textdomain_codeset.c",
    "src/locale/catclose.c",
    "src/locale/catgets.c",
    "src/locale/catopen.c",
    "src/locale/dcngettext.c",
    "src/locale/strfmon.c",
    "src/locale/textdomain.c",
    "crt/Scrt1.c",
    "crt/rcrt1.c",
    "src/misc/syslog.c",
    "src/mman/mremap.c",
    "src/process/fexecve.c",
    "src/signal/x86_64/sigsetjmp.s",
    "src/unistd/fchownat.c",
    "src/unistd/linkat.c",
    "src/string/x86_64/memcpy.s",
];

const REMOVED_GLOB_PREFIXES: [&str; 12] = [
    "src/thread/mtx_",
    "src/thread/cnd_",
    "src/thread/thrd_",
    "src/thread/tss_",
    "src/thread/pthread_barrier_",
    "src/thread/pthread_cond_",
    "src/thread/pthread_mutex_",
    "src/thread/pthread_rwlock_",
    "src/thread/pthread_spin_",
    "src/thread/pthread_key_",
    "src/thread/sem_",
    "src/time/timer_",
];

const PRESERVED_MATH_FILES: [&str; 5] = ["__fpclassifyl.c", "__signbitl.c", "frexp.c", "frexpl.c", "ldexp.c"];
const HEADER_DECLARATIONS_REMOVED: [&str; 7] = [
    "int iswalnum(wint_t);",
    "int iswalpha(wint_t);",
    "int iswcntrl(wint_t);",
    "int iswctype(wint_t, wctype_t);",
    "wint_t towctrans(wint_t, wctrans_t);",
    "wint_t towlower(wint_t);",
    "wint_t towupper(wint_t);",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompilerRoute {
    SelfHosted,
    Predecessor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ExpectedOutput {
    pub artifact_id: &'static str,
    pub digest_blake3: &'static str,
}

pub(crate) const EXPECTED_OUTPUTS: [ExpectedOutput; OUTPUT_COUNT] = [
    ExpectedOutput {
        artifact_id: "musl-native-libc",
        digest_blake3: "a9aa627c3a70fce68d928e0a7ccd423370472a87ae9fde795a85db4d0899366c",
    },
    ExpectedOutput {
        artifact_id: "musl-native-crt1",
        digest_blake3: "bcfefaf8179325fd26e8ecaa83b22a87fe75d3bbadb411e32113b4cb69f21f66",
    },
    ExpectedOutput {
        artifact_id: "musl-native-crti",
        digest_blake3: "ecf4006e5ea51c3ad49a243165909cd4f9eddfbe181b576ac43513ede8578be4",
    },
    ExpectedOutput {
        artifact_id: "musl-native-crtn",
        digest_blake3: "b3b3153225cc100b8c429f5916ae36b96ef4903169b86f841f86498c0fd7251d",
    },
    ExpectedOutput {
        artifact_id: "musl-native-libm",
        digest_blake3: EMPTY_ARCHIVE_BLAKE3,
    },
    ExpectedOutput {
        artifact_id: "musl-native-librt",
        digest_blake3: EMPTY_ARCHIVE_BLAKE3,
    },
    ExpectedOutput {
        artifact_id: "musl-native-libpthread",
        digest_blake3: EMPTY_ARCHIVE_BLAKE3,
    },
    ExpectedOutput {
        artifact_id: "musl-native-libcrypt",
        digest_blake3: EMPTY_ARCHIVE_BLAKE3,
    },
    ExpectedOutput {
        artifact_id: "musl-native-libutil",
        digest_blake3: EMPTY_ARCHIVE_BLAKE3,
    },
    ExpectedOutput {
        artifact_id: "musl-native-libxnet",
        digest_blake3: EMPTY_ARCHIVE_BLAKE3,
    },
    ExpectedOutput {
        artifact_id: "musl-native-libresolv",
        digest_blake3: EMPTY_ARCHIVE_BLAKE3,
    },
    ExpectedOutput {
        artifact_id: "musl-native-libdl",
        digest_blake3: EMPTY_ARCHIVE_BLAKE3,
    },
    ExpectedOutput {
        artifact_id: "musl-native-headers",
        digest_blake3: "315e38bd3f318804adf63fadc9689a9b692ee69986e26c6fddd4670190498700",
    },
    ExpectedOutput {
        artifact_id: "musl-native-smoke-binary",
        digest_blake3: "37b452f13fc423d7a1a5cc061264e4a897b305d144b0e214648738d106cb3e49",
    },
    ExpectedOutput {
        artifact_id: "musl-native-validation-receipt",
        digest_blake3: VALIDATION_RECEIPT_BLAKE3,
    },
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct OutputReport {
    pub artifact_id: String,
    pub path: PathBuf,
    pub bytes_len: u64,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct InventoryReport {
    pub format: &'static str,
    pub configured_source_digest_blake3: String,
    pub selfhost_compile_count: u32,
    pub predecessor_compile_count: u32,
    pub archive_command_count: u32,
    pub smoke_command_count: u32,
    pub execution_command_count: u32,
    pub outputs: Vec<OutputReport>,
    pub protected_exec_enforced: bool,
    pub fallback_events: Vec<String>,
    pub non_claim: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct InventoryRequest<'a> {
    pub source_root: &'a Path,
    pub selfhost_root: &'a Path,
    pub predecessor_root: &'a Path,
    pub scratch_dir: &'a Path,
    pub protected_exec_enforced: bool,
}

#[derive(Debug)]
pub(crate) enum Error {
    Materialization(String),
    Runtime(crate::stagex_mes_lib::MesLibraryPlanError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Materialization(message) => write!(formatter, "native musl materialization failed: {message}"),
            Self::Runtime(error) => write!(formatter, "native musl runtime failed: {error}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<crate::stagex_mes_lib::MesLibraryPlanError> for Error {
    fn from(error: crate::stagex_mes_lib::MesLibraryPlanError) -> Self {
        Self::Runtime(error)
    }
}

impl From<crate::stagex_musl::StagexMuslError> for Error {
    fn from(error: crate::stagex_musl::StagexMuslError) -> Self {
        Self::Materialization(error.to_string())
    }
}

impl From<crate::stagex_tinycc::TinyccError> for Error {
    fn from(error: crate::stagex_tinycc::TinyccError) -> Self {
        Self::Materialization(error.to_string())
    }
}

pub(crate) fn source_artifact_digests() -> [(&'static str, &'static str); 1] {
    [(RECIPE_ARTIFACT_ID, RECIPE_BLAKE3)]
}

pub(crate) fn derive_inventory(request: InventoryRequest<'_>) -> Result<InventoryReport, Error> {
    validate_inputs(&request)?;
    fs::create_dir(request.scratch_dir)
        .map_err(|error| Error::Materialization(format!("creating native musl scratch: {error}")))?;
    let source_root = request.scratch_dir.join("musl-1.1.24");
    prepare_source_tree(request.source_root, &source_root)?;
    let configured_source_digest_blake3 = configured_source_digest(&source_root)?;
    let output_root = request.scratch_dir.join("output");
    prepare_output(&source_root, &output_root)?;
    let sources = selected_sources(&source_root)?;
    let build = compile_source_closure(&request, &source_root, &sources)?;
    let libc = archive_libc(&request, &source_root, &output_root, &build.objects)?;
    install_crt_objects(&output_root, &build.crt_objects)?;
    materialize_empty_archives(&output_root)?;
    let (smoke_binary, validation_receipt) = run_smokes(&request, &source_root, &output_root)?;
    let outputs = collect_outputs(&output_root, &libc, &smoke_binary, &validation_receipt)?;
    validate_expected_outputs(&outputs)?;
    let report = InventoryReport {
        format: REPORT_FORMAT,
        configured_source_digest_blake3,
        selfhost_compile_count: build.selfhost_count,
        predecessor_compile_count: build.predecessor_count,
        archive_command_count: ARCHIVE_COMMAND_COUNT,
        smoke_command_count: SMOKE_COMMAND_COUNT,
        execution_command_count: EXECUTION_COMMAND_COUNT,
        outputs,
        protected_exec_enforced: request.protected_exec_enforced,
        fallback_events: Vec::new(),
        non_claim: NON_CLAIM,
    };
    crate::stagex_mes_lib::write_create_new(
        &request.scratch_dir.join("musl-native-inventory.json"),
        &serde_json::to_vec_pretty(&report)
            .map_err(|error| Error::Materialization(format!("serializing native musl report: {error}")))?,
    )?;
    assert_eq!(report.outputs.len(), OUTPUT_COUNT);
    assert!(report.fallback_events.is_empty());
    Ok(report)
}

fn validate_inputs(request: &InventoryRequest<'_>) -> Result<(), Error> {
    if request.scratch_dir.exists() {
        return Err(Error::Materialization(format!(
            "create-new native musl scratch exists: {}",
            request.scratch_dir.display()
        )));
    }
    validate_recipe()?;
    for (label, path, digest) in [
        (
            "self-hosted TinyCC",
            request.selfhost_root.join("bin/tcc-0.9.27-musl-selfhost"),
            crate::stagex_tcc_selfhost::COMPILER_BLAKE3,
        ),
        (
            "TinyCC musl-v2 predecessor",
            request.predecessor_root.join("bin/tcc-0.9.27-musl-v2"),
            crate::stagex_tcc_musl_v2::COMPILER_BLAKE3,
        ),
    ] {
        validate_file_digest(&path, digest, label)?;
    }
    if !request.source_root.is_absolute() || !request.source_root.join("Makefile").is_file() {
        return Err(Error::Materialization(format!(
            "native musl source is not an absolute source tree: {}",
            request.source_root.display()
        )));
    }
    assert!(request.selfhost_root.is_absolute());
    assert!(request.predecessor_root.is_absolute());
    Ok(())
}

fn validate_recipe() -> Result<(), Error> {
    let observed = blake3::hash(RECIPE).to_hex().to_string();
    if observed != RECIPE_BLAKE3 {
        return Err(Error::Materialization(format!(
            "native musl recipe BLAKE3 mismatch: expected {RECIPE_BLAKE3}, observed {observed}"
        )));
    }
    assert!(!RECIPE.is_empty());
    assert_eq!(observed.len(), blake3::OUT_LEN * 2);
    Ok(())
}

fn prepare_source_tree(source: &Path, destination: &Path) -> Result<(), Error> {
    crate::stagex_mes_lib::copy_tree_bounded(source, destination)?;
    crate::stagex_tinycc::make_tree_owner_writable(destination)?;
    let preserved_math = preserve_math_sources(destination)?;
    remove_declared_sources(destination)?;
    restore_math_sources(destination, &preserved_math)?;
    materialize_recipe_helpers(destination)?;
    apply_text_rewrites(destination)?;
    remove_glob_sources(destination)?;
    crate::stagex_musl::generate_headers(destination)?;
    rewrite_native_alltypes(&destination.join("obj/include/bits/alltypes.h"))?;
    assert!(destination.join("src/internal/va_list.c").is_file());
    assert!(destination.join("obj/include/bits/alltypes.h").is_file());
    Ok(())
}

fn preserve_math_sources(root: &Path) -> Result<Vec<(&'static str, Vec<u8>)>, Error> {
    let mut preserved = Vec::with_capacity(PRESERVED_MATH_FILES.len());
    for name in PRESERVED_MATH_FILES {
        let path = root.join("src/math").join(name);
        let bytes = fs::read(&path)
            .map_err(|error| Error::Materialization(format!("reading preserved native musl math {name}: {error}")))?;
        preserved.push((name, bytes));
    }
    assert_eq!(preserved.len(), PRESERVED_MATH_FILES.len());
    assert!(preserved.iter().all(|(_, bytes)| !bytes.is_empty()));
    Ok(preserved)
}

fn remove_declared_sources(root: &Path) -> Result<(), Error> {
    remove_header_declarations(&root.join("include/wchar.h"))?;
    remove_header_declarations(&root.join("include/wctype.h"))?;
    for relative in REMOVED_FILES {
        remove_file_if_present(&root.join(relative))?;
    }
    remove_file_if_present(&root.join("src/string/x86_64/memmove.s"))?;
    remove_file_if_present(&root.join("src/string/x86_64/memset.s"))?;
    for relative in REMOVED_DIRECTORIES {
        let path = root.join(relative);
        if path.exists() {
            fs::remove_dir_all(&path).map_err(|error| {
                Error::Materialization(format!("removing native musl directory {}: {error}", path.display()))
            })?;
        }
    }
    assert!(!root.join("src/aio").exists());
    assert!(!root.join("src/math").exists());
    Ok(())
}

fn remove_header_declarations(path: &Path) -> Result<(), Error> {
    let text = fs::read_to_string(path)
        .map_err(|error| Error::Materialization(format!("reading native musl header {}: {error}", path.display())))?;
    let filtered = text
        .lines()
        .filter(|line| !HEADER_DECLARATIONS_REMOVED.iter().any(|declaration| line.trim() == *declaration))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    fs::write(path, &filtered)
        .map_err(|error| Error::Materialization(format!("writing native musl header {}: {error}", path.display())))?;
    assert!(path.is_file());
    assert!(!filtered.contains("int iswalnum(wint_t);"));
    Ok(())
}

fn restore_math_sources(root: &Path, preserved: &[(&str, Vec<u8>)]) -> Result<(), Error> {
    let math = root.join("src/math");
    fs::create_dir(&math)
        .map_err(|error| Error::Materialization(format!("creating native musl math directory: {error}")))?;
    for (name, bytes) in preserved {
        crate::stagex_mes_lib::write_create_new(&math.join(name), bytes)?;
    }
    assert_eq!(preserved.len(), PRESERVED_MATH_FILES.len());
    assert!(math.join("frexp.c").is_file());
    Ok(())
}

fn materialize_recipe_helpers(root: &Path) -> Result<(), Error> {
    for target in HELPER_TARGETS {
        let bytes = extract_heredoc(RECIPE, target)?;
        let path = root.join(target);
        if path.exists() {
            remove_file_if_present(&path)?;
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                Error::Materialization(format!("creating native musl helper parent {}: {error}", parent.display()))
            })?;
        }
        crate::stagex_mes_lib::write_create_new(&path, &bytes)?;
    }
    assert_eq!(HELPER_TARGETS.len(), 31);
    assert!(root.join("src/malloc/malloc.c").is_file());
    Ok(())
}

fn extract_heredoc(recipe: &[u8], target: &str) -> Result<Vec<u8>, Error> {
    let text = std::str::from_utf8(recipe)
        .map_err(|error| Error::Materialization(format!("native musl recipe is not UTF-8: {error}")))?;
    let marker = format!("$BB cat > {target} <<'EOF'\n");
    let matches = text.match_indices(&marker).map(|(index, _)| index).collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err(Error::Materialization(format!("native musl helper {target} occurs {} times", matches.len())));
    }
    let start = matches[0]
        .checked_add(marker.len())
        .ok_or_else(|| Error::Materialization("native musl helper start overflow".to_string()))?;
    let relative_end = text[start..]
        .find("\nEOF\n")
        .ok_or_else(|| Error::Materialization(format!("native musl helper {target} lacks terminator")))?;
    let end = start
        .checked_add(relative_end)
        .and_then(|value| value.checked_add(1))
        .ok_or_else(|| Error::Materialization("native musl helper end overflow".to_string()))?;
    let bytes = recipe[start..end].to_vec();
    if bytes.is_empty() || !bytes.ends_with(b"\n") {
        return Err(Error::Materialization(format!("native musl helper {target} is empty or unterminated")));
    }
    assert!(end <= recipe.len());
    assert!(bytes.ends_with(b"\n"));
    Ok(bytes)
}

fn apply_text_rewrites(root: &Path) -> Result<(), Error> {
    replace_required(root, "src/internal/floatscan.c", "if (!x[0]) return sign * 0.0;", "if (!x[0]) return 0;")?;
    replace_required(
        root,
        "src/internal/syscall.h",
        "char __buf[static 15+3*sizeof(int)]",
        "char __buf[15+3*sizeof(int)]",
    )?;
    replace_required(root, "src/env/__init_tls.c", "td->next = td->prev = td;", "td->next = td; td->prev = td;")?;
    replace_required(
        root,
        "src/env/__init_tls.c",
        "td->dtv = td->dtv_copy = dtv;",
        "td->dtv = dtv; td->dtv_copy = dtv;",
    )?;
    replace_required(
        root,
        "src/locale/setlocale.c",
        "libc.global_locale = tmp_locale;",
        "for (i=0; i<LC_ALL; i++) libc.global_locale.cat[i] = tmp_locale.cat[i];",
    )?;
    replace_required(
        root,
        "src/env/__init_tls.c",
        "#define MIN_TLS_ALIGN offsetof(struct builtin_tls, pt)",
        "#define MIN_TLS_ALIGN ((size_t)&((struct builtin_tls *)0)->pt)",
    )?;
    insert_before(
        &root.join("src/env/__init_tls.c"),
        "volatile int __thread_list_lock;",
        "hidden void *__bootstrap_anonymous_mmap(size_t);\n",
    )?;
    replace_range(
        &root.join("src/env/__init_tls.c"),
        "mem = (void *)__syscall(",
        "0);",
        "mem = __bootstrap_anonymous_mmap(libc.tls_size);",
    )?;
    replace_required(
        root,
        "src/env/__init_tls.c",
        "weak_alias(static_init_tls, __init_tls);",
        "void __init_tls(size_t *aux) { static_init_tls(aux); }",
    )?;
    replace_required(
        root,
        "src/errno/__errno_location.c",
        "weak_alias(__errno_location, ___errno_location);",
        "int *___errno_location(void) { return __errno_location(); }",
    )?;
    replace_required(
        root,
        "src/ctype/__ctype_get_mb_cur_max.c",
        "return MB_CUR_MAX;",
        "enum { C_LOCALE_MB_MAX = 1, UTF8_LOCALE_MB_MAX = 4 };\n    struct pthread *thread = __pthread_self();\n    locale_t current = thread->locale;\n    return current->cat[LC_CTYPE] ? UTF8_LOCALE_MB_MAX : C_LOCALE_MB_MAX;",
    )?;
    lower_assembly_constants(root)?;
    assert!(root.join("src/env/__init_tls.c").is_file());
    assert!(root.join("src/internal/floatscan.c").is_file());
    Ok(())
}

fn replace_required(root: &Path, relative: &str, old: &str, new: &str) -> Result<(), Error> {
    let path = root.join(relative);
    let text = fs::read_to_string(&path)
        .map_err(|error| Error::Materialization(format!("reading native musl rewrite {}: {error}", path.display())))?;
    let count = text.matches(old).count();
    if count != 1 {
        return Err(Error::Materialization(format!(
            "native musl rewrite {relative} expected one match, observed {count}"
        )));
    }
    fs::write(&path, text.replacen(old, new, 1))
        .map_err(|error| Error::Materialization(format!("writing native musl rewrite {}: {error}", path.display())))?;
    assert!(path.is_file());
    assert!(!old.is_empty());
    Ok(())
}

fn insert_before(path: &Path, marker: &str, insertion: &str) -> Result<(), Error> {
    let text = fs::read_to_string(path).map_err(|error| {
        Error::Materialization(format!("reading native musl insertion {}: {error}", path.display()))
    })?;
    if text.matches(marker).count() != 1 {
        return Err(Error::Materialization(format!("native musl insertion marker is not unique: {marker}")));
    }
    fs::write(path, text.replacen(marker, &format!("{insertion}{marker}"), 1)).map_err(|error| {
        Error::Materialization(format!("writing native musl insertion {}: {error}", path.display()))
    })?;
    assert!(!marker.is_empty());
    assert!(!insertion.is_empty());
    Ok(())
}

fn replace_range(path: &Path, start_marker: &str, end_marker: &str, replacement: &str) -> Result<(), Error> {
    let text = fs::read_to_string(path)
        .map_err(|error| Error::Materialization(format!("reading native musl range {}: {error}", path.display())))?;
    let start = text
        .find(start_marker)
        .ok_or_else(|| Error::Materialization(format!("native musl range start is missing: {start_marker}")))?;
    let end_relative = text[start..]
        .find(end_marker)
        .ok_or_else(|| Error::Materialization(format!("native musl range end is missing: {end_marker}")))?;
    let end = start
        .checked_add(end_relative)
        .and_then(|value| value.checked_add(end_marker.len()))
        .ok_or_else(|| Error::Materialization("native musl range offset overflow".to_string()))?;
    let mut output = String::with_capacity(text.len());
    output.push_str(&text[..start]);
    output.push_str(replacement);
    output.push_str(&text[end..]);
    fs::write(path, output)
        .map_err(|error| Error::Materialization(format!("writing native musl range {}: {error}", path.display())))?;
    assert!(end > start);
    assert!(!replacement.is_empty());
    Ok(())
}

fn lower_assembly_constants(root: &Path) -> Result<(), Error> {
    replace_tokens(
        &root.join("crt/crt1.c"),
        &[
            ("STACK_ARGUMENT_BYTES", "8"),
            ("STACK_ALIGNMENT_MASK", "-16"),
            ("SYS_EXIT", "60"),
        ],
        false,
    )?;
    replace_tokens(
        &root.join("src/mman/x86_64/__bootstrap_raw_anonymous_mmap.s"),
        &[
            ("SYS_MMAP", "9"),
            ("PROT_READ_WRITE", "3"),
            ("MAP_ANONYMOUS_PRIVATE", "0x22"),
            ("INVALID_FD", "-1"),
        ],
        true,
    )?;
    replace_tokens(
        &root.join("src/misc/x86_64/__bootstrap_syscall_array.s"),
        &[
            ("ARGUMENT_0_OFFSET", "0"),
            ("ARGUMENT_1_OFFSET", "8"),
            ("ARGUMENT_2_OFFSET", "16"),
            ("ARGUMENT_3_OFFSET", "24"),
            ("ARGUMENT_4_OFFSET", "32"),
            ("ARGUMENT_5_OFFSET", "40"),
        ],
        true,
    )?;
    replace_tokens(&root.join("src/thread/x86_64/__syscall_cp.s"), &[("STACK_ARGUMENT_BYTES", "8")], true)?;
    assert!(root.join("crt/crt1.c").is_file());
    assert!(root.join("src/thread/x86_64/__syscall_cp.s").is_file());
    Ok(())
}

fn replace_tokens(path: &Path, replacements: &[(&str, &str)], drop_equ_lines: bool) -> Result<(), Error> {
    let text = fs::read_to_string(path).map_err(|error| {
        Error::Materialization(format!("reading native musl token file {}: {error}", path.display()))
    })?;
    let mut output = if drop_equ_lines {
        text.lines().filter(|line| !line.starts_with(".equ ")).collect::<Vec<_>>().join("\n") + "\n"
    } else {
        text
    };
    for (old, new) in replacements {
        if !output.contains(old) {
            return Err(Error::Materialization(format!("native musl token {old} is missing from {}", path.display())));
        }
        output = output.replace(old, new);
    }
    fs::write(path, output).map_err(|error| {
        Error::Materialization(format!("writing native musl token file {}: {error}", path.display()))
    })?;
    assert!(!replacements.is_empty());
    assert!(path.is_file());
    Ok(())
}

fn remove_glob_sources(root: &Path) -> Result<(), Error> {
    let paths = bounded_tree_paths(root)?;
    for path in paths {
        if !path.is_file() {
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .map_err(|error| Error::Materialization(format!("finding native musl relative path: {error}")))?
            .to_str()
            .ok_or_else(|| Error::Materialization("native musl path is not UTF-8".to_string()))?;
        if should_remove_glob_source(relative) {
            fs::remove_file(&path).map_err(|error| {
                Error::Materialization(format!("removing native musl glob source {}: {error}", path.display()))
            })?;
        }
    }
    assert!(!root.join("src/thread/pthread_create.c").exists());
    assert!(root.join("src/thread/pthread_once.c").is_file());
    assert!(root.join("src/thread/pthread_testcancel.c").is_file());
    assert!(root.join("src/thread/pthread_setcancelstate.c").is_file());
    Ok(())
}

fn should_remove_glob_source(relative: &str) -> bool {
    let matched = REMOVED_GLOB_PREFIXES.iter().any(|prefix| relative.starts_with(prefix) && relative.ends_with(".c"));
    let direct_thread = matches!(
        relative,
        "src/thread/pthread_create.c"
            | "src/thread/pthread_join.c"
            | "src/thread/pthread_detach.c"
            | "src/thread/pthread_cancel.c"
            | "src/thread/pthread_getspecific.c"
            | "src/thread/pthread_setspecific.c"
    );
    assert!(!relative.is_empty());
    assert!(!direct_thread || relative.ends_with(".c"));
    matched || direct_thread
}

fn rewrite_native_alltypes(path: &Path) -> Result<(), Error> {
    let text = fs::read_to_string(path)
        .map_err(|error| Error::Materialization(format!("reading native musl alltypes: {error}")))?;
    let rewritten = rewrite_alltypes_text(&text)?;
    fs::write(path, &rewritten)
        .map_err(|error| Error::Materialization(format!("writing native musl alltypes: {error}")))?;
    assert!(path.is_file());
    assert!(rewritten.contains("__mantle_va_list_struct"));
    Ok(())
}

fn rewrite_alltypes_text(input: &str) -> Result<String, Error> {
    const PREFIX: &str = "#if defined(__TINYC__) && !defined(__DEFINED_mantle_va_list_struct)\n#define __DEFINED_mantle_va_list_struct\ntypedef struct {\n    unsigned int gp_offset;\n    unsigned int fp_offset;\n    union { unsigned int overflow_offset; char *overflow_arg_area; };\n    char *reg_save_area;\n} __mantle_va_list_struct;\n#endif\n";
    let va_list = "#if defined(__TINYC__)\ntypedef __mantle_va_list_struct va_list[1];\n#else\ntypedef __builtin_va_list va_list;\n#endif";
    let isoc = "#if defined(__TINYC__)\ntypedef __mantle_va_list_struct __isoc_va_list[1];\n#else\ntypedef __builtin_va_list __isoc_va_list;\n#endif";
    if input.matches("typedef __builtin_va_list va_list;").count() != 1
        || input.matches("typedef __builtin_va_list __isoc_va_list;").count() != 1
    {
        return Err(Error::Materialization("native musl alltypes lacks unique va_list definitions".to_string()));
    }
    let output = format!(
        "{PREFIX}{}",
        input.replacen("typedef __builtin_va_list va_list;", va_list, 1).replacen(
            "typedef __builtin_va_list __isoc_va_list;",
            isoc,
            1
        )
    );
    assert!(output.contains("__mantle_va_list_struct"));
    assert!(output.len() > input.len());
    Ok(output)
}

fn configured_source_digest(root: &Path) -> Result<String, Error> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(CONFIGURED_SOURCE_DOMAIN);
    hasher.update(RECIPE_BLAKE3.as_bytes());
    for path in bounded_tree_paths(root)? {
        if !path.is_file() {
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .map_err(|error| Error::Materialization(format!("finding configured source path: {error}")))?;
        hasher.update(relative.as_os_str().as_encoded_bytes());
        hasher.update(b"\0");
        hasher.update(&fs::read(&path).map_err(|error| {
            Error::Materialization(format!("reading configured source {}: {error}", path.display()))
        })?);
        hasher.update(b"\0");
    }
    let digest = hasher.finalize().to_hex().to_string();
    if digest != CONFIGURED_SOURCE_BLAKE3 {
        return Err(Error::Materialization(format!(
            "native musl configured source BLAKE3 mismatch: expected {CONFIGURED_SOURCE_BLAKE3}, observed {digest}"
        )));
    }
    assert!(root.is_dir());
    assert_eq!(digest.len(), blake3::OUT_LEN * 2);
    Ok(digest)
}

fn prepare_output(source_root: &Path, output_root: &Path) -> Result<(), Error> {
    fs::create_dir_all(output_root.join("lib"))
        .map_err(|error| Error::Materialization(format!("creating native musl output lib: {error}")))?;
    crate::stagex_musl::install_headers(source_root, &output_root.join("include"))?;
    assert!(output_root.join("lib").is_dir());
    assert!(output_root.join("include/stdio.h").is_file());
    Ok(())
}

#[derive(Debug)]
struct BuildOutputs {
    objects: Vec<PathBuf>,
    crt_objects: BTreeMap<String, PathBuf>,
    selfhost_count: u32,
    predecessor_count: u32,
}

fn selected_sources(root: &Path) -> Result<Vec<String>, Error> {
    let mut sources = crate::stagex_musl::selected_sources(&root.join("src"))?;
    sources.extend(crate::stagex_musl::selected_sources(&root.join("crt"))?);
    sources = remove_arch_replaced_sources(sources);
    sources.sort();
    sources.dedup();
    if sources.is_empty() || sources.len() > TREE_ENTRY_COUNT_MAX {
        return Err(Error::Materialization(format!(
            "native musl selected source count is outside bounds: {}",
            sources.len()
        )));
    }
    assert!(!sources.is_empty());
    assert!(sources.windows(2).all(|pair| pair[0] < pair[1]));
    Ok(sources)
}

fn remove_arch_replaced_sources(sources: Vec<String>) -> Vec<String> {
    const ARCH_COMPONENT: &str = "/x86_64/";
    let arch_stems = sources
        .iter()
        .filter_map(|source| {
            source
                .contains(ARCH_COMPONENT)
                .then(|| source.replace(ARCH_COMPONENT, "/"))
                .and_then(|source| source.rsplit_once('.').map(|(stem, _)| stem.to_string()))
        })
        .collect::<BTreeSet<_>>();
    let selected = sources
        .into_iter()
        .filter(|source| {
            if source.contains(ARCH_COMPONENT) {
                return true;
            }
            let stem = source.rsplit_once('.').map_or(source.as_str(), |(stem, _)| stem);
            !arch_stems.contains(stem)
        })
        .collect::<Vec<_>>();
    assert!(selected.len() >= arch_stems.len());
    assert!(selected.iter().all(|source| !source.is_empty()));
    selected
}

fn compile_source_closure(
    request: &InventoryRequest<'_>,
    root: &Path,
    sources: &[String],
) -> Result<BuildOutputs, Error> {
    let selfhost = request.selfhost_root.join("bin/tcc-0.9.27-musl-selfhost");
    let predecessor = request.predecessor_root.join("bin/tcc-0.9.27-musl-v2");
    let mut objects = Vec::with_capacity(sources.len());
    let mut crt_objects = BTreeMap::new();
    let mut selfhost_count = 0_u32;
    let mut predecessor_count = 0_u32;
    for (index, relative) in sources.iter().enumerate() {
        let route = compiler_route(relative);
        let object = root.join(format!("obj/native/{index:04}.o"));
        if let Some(parent) = object.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                Error::Materialization(format!("creating native musl object parent {}: {error}", parent.display()))
            })?;
        }
        compile_one(request, root, relative, &object, route, &selfhost, &predecessor, index)?;
        if relative.starts_with("crt/") {
            let name = Path::new(relative)
                .file_stem()
                .and_then(|value| value.to_str())
                .ok_or_else(|| Error::Materialization(format!("invalid native musl CRT source: {relative}")))?;
            if crt_objects.insert(name.to_string(), object.clone()).is_some() {
                return Err(Error::Materialization(format!("duplicate native musl CRT object: {name}")));
            }
        } else {
            objects.push(object);
        }
        match route {
            CompilerRoute::SelfHosted => selfhost_count = selfhost_count.saturating_add(1),
            CompilerRoute::Predecessor => predecessor_count = predecessor_count.saturating_add(1),
        }
    }
    if objects.is_empty() || crt_objects.is_empty() {
        return Err(Error::Materialization("native musl source closure is incomplete".to_string()));
    }
    if selfhost_count != SELFHOST_COMPILE_COUNT || predecessor_count != PREDECESSOR_COMPILE_COUNT {
        return Err(Error::Materialization(format!(
            "native musl compiler route count mismatch: expected {SELFHOST_COMPILE_COUNT}/{PREDECESSOR_COMPILE_COUNT}, observed {selfhost_count}/{predecessor_count}"
        )));
    }
    let compile_count = selfhost_count
        .checked_add(predecessor_count)
        .ok_or_else(|| Error::Materialization("native musl compile count overflow".to_string()))?;
    let compile_count = usize::try_from(compile_count)
        .map_err(|_| Error::Materialization("native musl compile count does not fit usize".to_string()))?;
    assert_eq!(compile_count, sources.len());
    assert!(crt_objects.contains_key("crt1"));
    Ok(BuildOutputs {
        objects,
        crt_objects,
        selfhost_count,
        predecessor_count,
    })
}

fn compiler_route(relative: &str) -> CompilerRoute {
    let predecessor_assembly =
        relative.starts_with("src/") && relative.contains("/x86_64/") && relative.ends_with(".s");
    let predecessor_math = relative.starts_with("src/math/") && relative.ends_with(".c");
    let predecessor_crt1 = relative == "crt/crt1.c";
    if predecessor_assembly || predecessor_math || predecessor_crt1 {
        CompilerRoute::Predecessor
    } else {
        CompilerRoute::SelfHosted
    }
}

#[allow(clippy::too_many_arguments)]
fn compile_one(
    request: &InventoryRequest<'_>,
    root: &Path,
    relative: &str,
    object: &Path,
    route: CompilerRoute,
    selfhost: &Path,
    predecessor: &Path,
    index: usize,
) -> Result<(), Error> {
    let (compiler, source_argument) = match route {
        CompilerRoute::SelfHosted => (selfhost, relative.to_string()),
        CompilerRoute::Predecessor if relative.ends_with(".s") => {
            let assembly = fs::read_to_string(root.join(relative))
                .map_err(|error| Error::Materialization(format!("reading native musl assembly {relative}: {error}")))?;
            let inline = inline_assembly_source(&assembly);
            let generated_relative = format!("obj/native/inline-{index:04}.c");
            let generated = root.join(&generated_relative);
            crate::stagex_mes_lib::write_create_new(&generated, inline.as_bytes())?;
            (predecessor, generated_relative)
        }
        CompilerRoute::Predecessor => (predecessor, relative.to_string()),
    };
    let mut args = COMMON_COMPILE_FLAGS.iter().map(|value| (*value).to_string()).collect::<Vec<_>>();
    args.extend([
        "-c".to_string(),
        source_argument,
        "-o".to_string(),
        absolute_utf8(object, "native musl object")?,
    ]);
    crate::stagex_mes_lib::run_bounded_process(
        compiler,
        &args,
        root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join(format!("native-musl-{index:04}.stderr.txt")),
    )
    .map_err(|error| {
        Error::Materialization(format!("compiling native musl source {relative} at index {index}: {error}"))
    })?;
    set_owner_read_write(object)?;
    validate_nonempty_file(object, "native musl object")?;
    assert!(compiler.is_file());
    assert!(object.is_file());
    Ok(())
}

fn inline_assembly_source(input: &str) -> String {
    let mut output = String::from("static void force_text_section(void) {}\n__asm__(\n");
    for line in input.lines() {
        if line == ".text" {
            continue;
        }
        let escaped = line.replace('\\', "\\\\").replace('"', "\\\"");
        output.push('"');
        output.push_str(&escaped);
        output.push_str("\\n\"\n");
    }
    output.push_str(");\n");
    assert!(output.starts_with("static void force_text_section"));
    assert!(output.ends_with(");\n"));
    output
}

fn archive_libc(
    request: &InventoryRequest<'_>,
    root: &Path,
    output_root: &Path,
    objects: &[PathBuf],
) -> Result<PathBuf, Error> {
    let archive_root = root.join("obj/native/archive");
    fs::create_dir(&archive_root)
        .map_err(|error| Error::Materialization(format!("creating native musl archive root: {error}")))?;
    let mut args = vec![
        "-ar".to_string(),
        "rcs".to_string(),
        absolute_utf8(&output_root.join("lib/libc.a"), "native musl libc")?,
    ];
    for (index, object) in objects.iter().enumerate() {
        let short = archive_root.join(format!("o{index:04}.o"));
        crate::stagex_mes_lib::copy_file_exact(object, &short)?;
        args.push(absolute_utf8(&short, "native musl archive object")?);
    }
    let compiler = request.selfhost_root.join("bin/tcc-0.9.27-musl-selfhost");
    crate::stagex_mes_lib::run_bounded_process(
        &compiler,
        &args,
        root,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join("native-musl-archive.stderr.txt"),
    )?;
    let libc = output_root.join("lib/libc.a");
    normalize_archive_file(&libc)?;
    validate_nonempty_file(&libc, "native musl libc")?;
    assert!(!objects.is_empty());
    assert!(libc.is_file());
    Ok(libc)
}

fn normalize_archive_file(path: &Path) -> Result<(), Error> {
    let bytes = crate::stagex_mes_lib::read_bounded_file(path, FILE_BYTES_MAX, "native musl archive")?;
    let normalized = normalize_archive_metadata(&bytes)?;
    fs::write(path, &normalized)
        .map_err(|error| Error::Materialization(format!("writing normalized native musl archive: {error}")))?;
    assert_eq!(normalized.len(), bytes.len());
    assert!(path.is_file());
    Ok(())
}

fn normalize_archive_metadata(bytes: &[u8]) -> Result<Vec<u8>, Error> {
    if !bytes.starts_with(EMPTY_ARCHIVE_BYTES) {
        return Err(Error::Materialization("native musl archive lacks the global header".to_string()));
    }
    let mut normalized = bytes.to_vec();
    let mut cursor = EMPTY_ARCHIVE_BYTES.len();
    let mut member_count = 0usize;
    while cursor < normalized.len() {
        let header_end = cursor
            .checked_add(AR_HEADER_BYTES)
            .ok_or_else(|| Error::Materialization("native musl archive header offset overflow".to_string()))?;
        if header_end > normalized.len() {
            return Err(Error::Materialization("native musl archive has a truncated member header".to_string()));
        }
        let trailer_start = cursor + AR_TRAILER_START;
        if normalized[trailer_start..header_end] != *AR_TRAILER_BYTES {
            return Err(Error::Materialization("native musl archive has an invalid member trailer".to_string()));
        }
        let size_text = std::str::from_utf8(&normalized[cursor + AR_MODE_END..cursor + AR_SIZE_END])
            .map_err(|_| Error::Materialization("native musl archive member size is not UTF-8".to_string()))?;
        let payload_bytes = size_text
            .trim()
            .parse::<usize>()
            .map_err(|_| Error::Materialization("native musl archive member size is invalid".to_string()))?;
        normalized[cursor + AR_TIMESTAMP_START..cursor + AR_TIMESTAMP_END].copy_from_slice(AR_TIMESTAMP_NORMALIZED);
        normalized[cursor + AR_TIMESTAMP_END..cursor + AR_OWNER_END].copy_from_slice(AR_OWNER_NORMALIZED);
        normalized[cursor + AR_OWNER_END..cursor + AR_GROUP_END].copy_from_slice(AR_GROUP_NORMALIZED);
        normalized[cursor + AR_GROUP_END..cursor + AR_MODE_END].copy_from_slice(AR_MODE_NORMALIZED);
        let payload_end = header_end
            .checked_add(payload_bytes)
            .ok_or_else(|| Error::Materialization("native musl archive payload offset overflow".to_string()))?;
        if payload_end > normalized.len() {
            return Err(Error::Materialization("native musl archive has a truncated member payload".to_string()));
        }
        let padding_bytes = payload_bytes % AR_ALIGNMENT_BYTES;
        cursor = payload_end
            .checked_add(padding_bytes)
            .ok_or_else(|| Error::Materialization("native musl archive padding offset overflow".to_string()))?;
        if cursor > normalized.len() {
            return Err(Error::Materialization("native musl archive has truncated alignment padding".to_string()));
        }
        member_count = member_count
            .checked_add(1)
            .ok_or_else(|| Error::Materialization("native musl archive member count overflow".to_string()))?;
        if member_count > TREE_ENTRY_COUNT_MAX {
            return Err(Error::Materialization("native musl archive exceeds the member limit".to_string()));
        }
    }
    if member_count == 0 {
        return Err(Error::Materialization("native musl archive has no members".to_string()));
    }
    assert_eq!(cursor, normalized.len());
    assert_eq!(normalized.len(), bytes.len());
    Ok(normalized)
}

fn install_crt_objects(output_root: &Path, objects: &BTreeMap<String, PathBuf>) -> Result<(), Error> {
    for required in ["crt1", "crti", "crtn"] {
        let source = objects
            .get(required)
            .ok_or_else(|| Error::Materialization(format!("native musl CRT set lacks {required}.o")))?;
        crate::stagex_mes_lib::copy_file_exact(source, &output_root.join(format!("lib/{required}.o")))?;
    }
    assert!(output_root.join("lib/crt1.o").is_file());
    assert!(output_root.join("lib/crtn.o").is_file());
    Ok(())
}

fn materialize_empty_archives(output_root: &Path) -> Result<(), Error> {
    for name in ["m", "rt", "pthread", "crypt", "util", "xnet", "resolv", "dl"] {
        crate::stagex_mes_lib::write_create_new(&output_root.join(format!("lib/lib{name}.a")), EMPTY_ARCHIVE_BYTES)?;
    }
    assert_eq!(blake3::hash(EMPTY_ARCHIVE_BYTES).to_hex().to_string(), EMPTY_ARCHIVE_BLAKE3);
    assert!(output_root.join("lib/libm.a").is_file());
    Ok(())
}

fn run_smokes(request: &InventoryRequest<'_>, root: &Path, output_root: &Path) -> Result<(PathBuf, PathBuf), Error> {
    let smoke = request.scratch_dir.join("smoke");
    fs::create_dir(&smoke)
        .map_err(|error| Error::Materialization(format!("creating native musl smoke root: {error}")))?;
    let positive = extract_heredoc(RECIPE, POSITIVE_SMOKE_TARGET)?;
    crate::stagex_mes_lib::write_create_new(&smoke.join("native-musl-smoke.c"), &positive)?;
    crate::stagex_mes_lib::write_create_new(&smoke.join("malformed.c"), MALFORMED_SMOKE_SOURCE)?;
    let compiler = request.selfhost_root.join("bin/tcc-0.9.27-musl-selfhost");
    let object = smoke.join("native-musl-smoke.o");
    run(
        request,
        &compiler,
        &[
            "-c".to_string(),
            format!("-I{}", absolute_utf8(&output_root.join("include"), "native musl include")?),
            "native-musl-smoke.c".to_string(),
            "-o".to_string(),
            absolute_utf8(&object, "native musl smoke object")?,
        ],
        &smoke,
        "native-musl-smoke-compile.stderr.txt",
    )?;
    set_owner_read_write(&object)?;
    let binary = smoke.join("native-musl-smoke");
    run(
        request,
        &compiler,
        &[
            "-nostdlib".to_string(),
            "-static".to_string(),
            "-o".to_string(),
            absolute_utf8(&binary, "native musl smoke binary")?,
            absolute_utf8(&output_root.join("lib/crt1.o"), "native musl crt1")?,
            absolute_utf8(&object, "native musl smoke object")?,
            absolute_utf8(&output_root.join("lib/libc.a"), "native musl libc")?,
        ],
        root,
        "native-musl-smoke-link.stderr.txt",
    )?;
    crate::stagex_tinycc::set_owner_executable(&binary)?;
    run(request, &binary, &[], &smoke, "native-musl-smoke.stderr.txt")?;
    let diagnostic_compiler = request.predecessor_root.join("bin/tcc-0.9.27-musl-v2");
    require_expected_compile_failure(request, &diagnostic_compiler, &smoke)?;
    let validation_receipt = smoke.join("native-musl-validation-receipt.txt");
    crate::stagex_mes_lib::write_create_new(&validation_receipt, VALIDATION_RECEIPT_BYTES)?;
    validate_nonempty_file(&binary, "native musl smoke binary")?;
    validate_nonempty_file(&validation_receipt, "native musl validation receipt")?;
    assert!(object.is_file());
    assert!(!smoke.join("malformed.o").exists());
    Ok((binary, validation_receipt))
}

fn run(
    request: &InventoryRequest<'_>,
    executable: &Path,
    args: &[String],
    current_dir: &Path,
    stderr_name: &str,
) -> Result<(), Error> {
    crate::stagex_mes_lib::run_bounded_process(
        executable,
        args,
        current_dir,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join(stderr_name),
    )
    .map_err(|error| Error::Materialization(format!("running native musl step {stderr_name}: {error}")))?;
    assert!(executable.is_file());
    assert!(current_dir.is_dir());
    Ok(())
}

fn require_expected_compile_failure(
    request: &InventoryRequest<'_>,
    compiler: &Path,
    smoke: &Path,
) -> Result<(), Error> {
    let args = ["-c", "malformed.c", "-o", "malformed.o"];
    match crate::stagex_mes_lib::run_bounded_process(
        compiler,
        &args,
        smoke,
        &BTreeMap::<String, String>::new(),
        &request.scratch_dir.join("native-musl-malformed.stderr.txt"),
    ) {
        Err(crate::stagex_mes_lib::MesLibraryPlanError::ProcessFailure {
            exit_code: Some(exit_code),
            stderr,
            ..
        }) if exit_code != 0 && !stderr.trim().is_empty() => {
            assert_ne!(exit_code, 0);
            assert!(!stderr.trim().is_empty());
            Ok(())
        }
        Ok(()) => Err(Error::Materialization("native musl compiler accepted malformed source".to_string())),
        Err(error) => Err(Error::Runtime(error)),
    }
}

fn collect_outputs(
    output_root: &Path,
    libc: &Path,
    smoke_binary: &Path,
    validation_receipt: &Path,
) -> Result<Vec<OutputReport>, Error> {
    let specs = [
        ("musl-native-libc", libc.to_path_buf(), false),
        ("musl-native-crt1", output_root.join("lib/crt1.o"), false),
        ("musl-native-crti", output_root.join("lib/crti.o"), false),
        ("musl-native-crtn", output_root.join("lib/crtn.o"), false),
        ("musl-native-libm", output_root.join("lib/libm.a"), false),
        ("musl-native-librt", output_root.join("lib/librt.a"), false),
        ("musl-native-libpthread", output_root.join("lib/libpthread.a"), false),
        ("musl-native-libcrypt", output_root.join("lib/libcrypt.a"), false),
        ("musl-native-libutil", output_root.join("lib/libutil.a"), false),
        ("musl-native-libxnet", output_root.join("lib/libxnet.a"), false),
        ("musl-native-libresolv", output_root.join("lib/libresolv.a"), false),
        ("musl-native-libdl", output_root.join("lib/libdl.a"), false),
        ("musl-native-headers", output_root.join("include"), true),
        ("musl-native-smoke-binary", smoke_binary.to_path_buf(), false),
        ("musl-native-validation-receipt", validation_receipt.to_path_buf(), false),
    ];
    let mut outputs = Vec::with_capacity(OUTPUT_COUNT);
    for (artifact_id, path, tree) in specs {
        let (bytes_len, digest_blake3) = output_identity(&path, tree)?;
        outputs.push(OutputReport {
            artifact_id: artifact_id.to_string(),
            path,
            bytes_len,
            digest_blake3,
        });
    }
    assert_eq!(outputs.len(), OUTPUT_COUNT);
    assert!(outputs.iter().all(|output| output.bytes_len > 0));
    Ok(outputs)
}

fn output_identity(path: &Path, tree: bool) -> Result<(u64, String), Error> {
    if tree {
        return Ok((crate::stagex_musl::tree_bytes_len(path)?, crate::stagex_musl::tree_digest_blake3(path)?));
    }
    let bytes = crate::stagex_mes_lib::read_bounded_file(path, FILE_BYTES_MAX, "native musl output")?;
    if bytes.is_empty() {
        return Err(Error::Materialization(format!("native musl output is empty: {}", path.display())));
    }
    assert!(path.is_file());
    assert!(!bytes.is_empty());
    let bytes_len = u64::try_from(bytes.len())
        .map_err(|_| Error::Materialization("native musl output size does not fit u64".to_string()))?;
    Ok((bytes_len, blake3::hash(&bytes).to_hex().to_string()))
}

fn validate_expected_outputs(outputs: &[OutputReport]) -> Result<(), Error> {
    let mut mismatches = Vec::new();
    for expected in EXPECTED_OUTPUTS {
        let observed = outputs.iter().find(|output| output.artifact_id == expected.artifact_id).ok_or_else(|| {
            Error::Materialization(format!("native musl output is missing: {}", expected.artifact_id))
        })?;
        if observed.digest_blake3 != expected.digest_blake3 {
            mismatches.push(format!("{}={}", expected.artifact_id, observed.digest_blake3));
        }
    }
    if !mismatches.is_empty() {
        return Err(Error::Materialization(format!("native musl output BLAKE3 mismatches: {}", mismatches.join(","))));
    }
    assert_eq!(outputs.len(), EXPECTED_OUTPUTS.len());
    assert!(outputs.iter().all(|output| output.digest_blake3.len() == blake3::OUT_LEN * 2));
    Ok(())
}

fn validate_file_digest(path: &Path, expected: &str, label: &str) -> Result<(), Error> {
    let bytes = crate::stagex_mes_lib::read_bounded_file(path, FILE_BYTES_MAX, label)?;
    let observed = blake3::hash(&bytes).to_hex().to_string();
    if observed != expected {
        return Err(Error::Materialization(format!(
            "{label} BLAKE3 mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert!(path.is_file());
    assert!(!bytes.is_empty());
    Ok(())
}

fn validate_nonempty_file(path: &Path, label: &str) -> Result<(), Error> {
    let bytes = crate::stagex_mes_lib::read_bounded_file(path, FILE_BYTES_MAX, label)?;
    if bytes.is_empty() {
        return Err(Error::Materialization(format!("{label} is empty")));
    }
    assert!(path.is_file());
    assert!(!bytes.is_empty());
    Ok(())
}

fn bounded_tree_paths(root: &Path) -> Result<Vec<PathBuf>, Error> {
    let mut pending = vec![root.to_path_buf()];
    let mut paths = Vec::new();
    while let Some(directory) = pending.pop() {
        let mut children = fs::read_dir(&directory)
            .map_err(|error| {
                Error::Materialization(format!("reading native musl tree {}: {error}", directory.display()))
            })?
            .map(|entry| {
                entry
                    .map(|value| value.path())
                    .map_err(|error| Error::Materialization(format!("reading native musl tree entry: {error}")))
            })
            .collect::<Result<Vec<_>, _>>()?;
        children.sort();
        for child in children.into_iter().rev() {
            if child.is_dir() {
                pending.push(child.clone());
            }
            paths.push(child);
            if paths.len() > TREE_ENTRY_COUNT_MAX {
                return Err(Error::Materialization(format!("native musl tree exceeds {TREE_ENTRY_COUNT_MAX} entries")));
            }
        }
    }
    paths.sort();
    assert!(paths.len() <= TREE_ENTRY_COUNT_MAX);
    assert!(paths.iter().all(|path| path.starts_with(root)));
    Ok(paths)
}

fn remove_file_if_present(path: &Path) -> Result<(), Error> {
    if path.exists() {
        if !path.is_file() && !path.is_symlink() {
            return Err(Error::Materialization(format!(
                "native musl removal target is not a file: {}",
                path.display()
            )));
        }
        fs::remove_file(path).map_err(|error| {
            Error::Materialization(format!("removing native musl file {}: {error}", path.display()))
        })?;
    }
    assert!(!path.exists());
    assert!(!path.as_os_str().is_empty());
    Ok(())
}

fn absolute_utf8(path: &Path, label: &str) -> Result<String, Error> {
    crate::stagex_mes_lib::utf8_absolute(path, label).map(str::to_string).map_err(Error::Runtime)
}

fn set_owner_read_write(path: &Path) -> Result<(), Error> {
    let mut permissions = fs::metadata(path)
        .map_err(|error| Error::Materialization(format!("reading native musl output mode: {error}")))?
        .permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        const OWNER_READ_WRITE_MODE: u32 = 0o600;
        permissions.set_mode(OWNER_READ_WRITE_MODE);
    }
    fs::set_permissions(path, permissions)
        .map_err(|error| Error::Materialization(format!("setting native musl output mode: {error}")))?;
    assert!(path.is_file());
    assert!(!path.as_os_str().is_empty());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recipe_and_helpers_are_bound() {
        validate_recipe().unwrap();
        for target in HELPER_TARGETS {
            let helper = extract_heredoc(RECIPE, target).unwrap();
            assert!(!helper.is_empty(), "target={target}");
        }
        assert_eq!(source_artifact_digests()[0], (RECIPE_ARTIFACT_ID, RECIPE_BLAKE3));
    }

    #[test]
    fn missing_or_duplicate_heredoc_is_rejected() {
        let missing = extract_heredoc(RECIPE, "missing.c").unwrap_err();
        assert!(missing.to_string().contains("occurs 0 times"));
        let duplicate = b"$BB cat > x.c <<'EOF'\na\nEOF\n$BB cat > x.c <<'EOF'\nb\nEOF\n";
        let error = extract_heredoc(duplicate, "x.c").unwrap_err();
        assert!(error.to_string().contains("occurs 2 times"));
    }

    #[test]
    fn pthread_source_selection_matches_the_checked_recipe() {
        assert!(!should_remove_glob_source("src/thread/pthread_testcancel.c"));
        assert!(!should_remove_glob_source("src/thread/pthread_setcancelstate.c"));
        assert!(should_remove_glob_source("src/thread/pthread_create.c"));
        assert!(should_remove_glob_source("src/thread/pthread_cond_wait.c"));
    }

    #[test]
    fn alltypes_rewrite_accepts_expected_shape_and_rejects_missing_types() {
        let input = "typedef __builtin_va_list va_list;\ntypedef __builtin_va_list __isoc_va_list;\n";
        let output = rewrite_alltypes_text(input).unwrap();
        assert!(output.contains("__mantle_va_list_struct"));
        assert!(output.contains("va_list[1]"));
        let error = rewrite_alltypes_text("typedef int value;\n").unwrap_err();
        assert!(error.to_string().contains("lacks unique"));
    }

    #[test]
    fn archive_metadata_normalization_is_deterministic_and_rejects_bad_trailers() {
        const PAYLOAD_BYTES: &[u8] = b"abc";
        const NONDETERMINISTIC_TIMESTAMP: &str = "1234567890";
        const NONDETERMINISTIC_OWNER: &str = "42";
        const NONDETERMINISTIC_GROUP: &str = "7";
        const NONDETERMINISTIC_MODE: &str = "100755";
        let header = format!(
            "{:<16}{:<12}{:<6}{:<6}{:<8}{:<10}`\n",
            "object.o/",
            NONDETERMINISTIC_TIMESTAMP,
            NONDETERMINISTIC_OWNER,
            NONDETERMINISTIC_GROUP,
            NONDETERMINISTIC_MODE,
            PAYLOAD_BYTES.len()
        );
        assert_eq!(header.len(), AR_HEADER_BYTES);
        let mut archive = EMPTY_ARCHIVE_BYTES.to_vec();
        archive.extend_from_slice(header.as_bytes());
        archive.extend_from_slice(PAYLOAD_BYTES);
        archive.push(b'\n');
        let normalized = normalize_archive_metadata(&archive).unwrap();
        let header_start = EMPTY_ARCHIVE_BYTES.len();
        assert_eq!(
            &normalized[header_start + AR_TIMESTAMP_START..header_start + AR_TIMESTAMP_END],
            AR_TIMESTAMP_NORMALIZED
        );
        assert_eq!(
            &normalized[header_start + AR_HEADER_BYTES..header_start + AR_HEADER_BYTES + PAYLOAD_BYTES.len()],
            PAYLOAD_BYTES
        );

        let mut malformed = archive;
        malformed[header_start + AR_TRAILER_START] = b'!';
        assert!(normalize_archive_metadata(&malformed).unwrap_err().to_string().contains("trailer"));
    }

    #[test]
    fn compiler_routes_preserve_the_declared_two_compiler_boundary() {
        assert_eq!(compiler_route("src/stdio/printf.c"), CompilerRoute::SelfHosted);
        assert_eq!(compiler_route("src/math/frexp.c"), CompilerRoute::Predecessor);
        assert_eq!(compiler_route("src/thread/x86_64/__syscall_cp.s"), CompilerRoute::Predecessor);
        assert_eq!(compiler_route("crt/crt1.c"), CompilerRoute::Predecessor);
        assert_eq!(compiler_route("crt/crti.s"), CompilerRoute::SelfHosted);
    }

    #[test]
    fn arch_sources_replace_generic_sources_by_object_stem() {
        let selected = remove_arch_replaced_sources(vec![
            "src/thread/__syscall_cp.c".to_string(),
            "src/thread/x86_64/__syscall_cp.s".to_string(),
            "src/stdio/printf.c".to_string(),
        ]);
        assert_eq!(selected, vec![
            "src/thread/x86_64/__syscall_cp.s".to_string(),
            "src/stdio/printf.c".to_string()
        ]);
        assert!(!selected.iter().any(|source| source == "src/thread/__syscall_cp.c"));
    }

    #[test]
    fn inline_assembly_conversion_escapes_text_and_drops_text_directive() {
        let output = inline_assembly_source(".text\n.global x\nx:\n  mov \"q\",%rax\n");
        assert!(!output.contains("\".text\\n\""));
        assert!(output.contains(".global x\\n"));
        assert!(output.contains("\\\"q\\\""));
    }

    #[test]
    fn existing_scratch_fails_closed() {
        let scratch = tempfile::tempdir().unwrap();
        let request = InventoryRequest {
            source_root: Path::new("/missing-source"),
            selfhost_root: Path::new("/missing-selfhost"),
            predecessor_root: Path::new("/missing-predecessor"),
            scratch_dir: scratch.path(),
            protected_exec_enforced: false,
        };
        let error = derive_inventory(request).unwrap_err();
        assert!(error.to_string().contains("scratch exists"));
        assert!(scratch.path().exists());
    }

    #[test]
    #[ignore = "requires retained authenticated musl source and protected TinyCC roots"]
    fn derives_retained_native_musl_inventory() {
        let source = PathBuf::from(std::env::var("MANTLE_STAGE_X_MUSL_NATIVE_SOURCE_ROOT").unwrap());
        let selfhost = PathBuf::from(std::env::var("MANTLE_STAGE_X_TCC_SELFHOST_ROOT").unwrap());
        let predecessor = PathBuf::from(std::env::var("MANTLE_STAGE_X_TCC_MUSL_V2_ROOT").unwrap());
        let scratch = PathBuf::from(std::env::var("MANTLE_STAGE_X_MUSL_NATIVE_SCRATCH").unwrap());
        let report = derive_inventory(InventoryRequest {
            source_root: &source,
            selfhost_root: &selfhost,
            predecessor_root: &predecessor,
            scratch_dir: &scratch,
            protected_exec_enforced: false,
        })
        .unwrap();
        assert!(report.selfhost_compile_count > 0);
        assert!(report.predecessor_compile_count > 0);
    }
}
