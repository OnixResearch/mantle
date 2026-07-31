use std::collections::BTreeMap;
use std::fs;
use std::io::Write as _;
use std::path::Path;
use std::path::PathBuf;

use serde::Serialize;

const RECIPE_ARTIFACT_ID: &str = "tcc-musl-selfhost-recipe-source";
const RECIPE_BLAKE3: &str = "e24d7a19746cbae7387c7c7d739c7a3ee5ae6c8e343351f3deacebad1be805d2";
const RECIPE: &[u8] = include_bytes!("../bootstrap/tcc-musl-selfhost.ncl");
const REPORT_FORMAT: &str = "mantle-stagex-tcc-musl-selfhost-inventory-v1";
const NON_CLAIM: &str = "this inventory binds TinyCC 0.9.27 rebuilt from separate compiler objects by the protected TinyCC musl-v2 predecessor and linked by the declared TinyCC 0.9.26 path only; it does not prove native-runtime replacement, the later GNU/GCC/binutils chain, or provider admission";
const BUILD_COMMAND_COUNT: u32 = 13;
const SMOKE_COMMAND_COUNT: u32 = 1;
const COMPILER_SOURCE_COUNT: u32 = 10;
const OUTPUT_COUNT: usize = 7;
const FILE_BYTES_MAX: u64 = 64 * 1_024 * 1_024;
const CANONICAL_TEMP_EXTENSION: &str = "mantle-canonical.tmp";
pub(crate) const COMPILER_BLAKE3: &str = "8e6580da40c5892b941423ae108d6218b3636ebd3643bc3ba1e78e33d6c4898a";
const LIBTCC_ARCHIVE_BLAKE3: &str = "dbf67a48eed6afe4a0e74ce5d3b27bbe97dd6531b0eb5a5a648a3e290d9723ca";
const MAIN_OBJECT_BLAKE3: &str = "6df478ee2068dca31c540583925db4116d24b6db9cf21c788d5a0106f02123ef";
const PATCHED_SOURCE_BLAKE3: &str = "5e918d19d4d7151f17a5a81bd07d25fd8a55e57037b4735f038344365326b3d4";
const OBJECT_TREE_BLAKE3: &str = "e5c7d2d26adf9230fa6c713ae526d666089c31d9bca4a32c3bdc4a876a01ae39";
const SMOKE_OBJECT_BLAKE3: &str = "cc40480286f053fc69e7d17431b4cf5de82b477b0f84e646717cd6c3c71cecb1";
const SMOKE_SOURCE: &[u8] = b"#include <stdarg.h>\nstruct item { int first; int second; };\nstatic const struct item item = { .second = 2, .first = 1 };\nstatic int first_variadic(int ignored, ...) { va_list arguments; int value; va_start(arguments, ignored); value = va_arg(arguments, int); va_end(arguments); return value; }\nint smoke(void) { return item.first == 1 && item.second == 2 && first_variadic(0, 1) == 1 ? 0 : 1; }\n";
const COMPILER_SOURCES: [&str; COMPILER_SOURCE_COUNT as usize] = [
    "tcc.c",
    "libtcc.c",
    "tccpp.c",
    "tccgen.c",
    "tccelf.c",
    "tccasm.c",
    "tccrun.c",
    "x86_64-gen.c",
    "x86_64-link.c",
    "i386-asm.c",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ExpectedOutput {
    pub artifact_id: &'static str,
    pub digest_blake3: &'static str,
}

pub(crate) const EXPECTED_OUTPUTS: [ExpectedOutput; OUTPUT_COUNT] = [
    ExpectedOutput {
        artifact_id: "tcc-musl-selfhost",
        digest_blake3: COMPILER_BLAKE3,
    },
    ExpectedOutput {
        artifact_id: "tcc-musl-selfhost-alias",
        digest_blake3: COMPILER_BLAKE3,
    },
    ExpectedOutput {
        artifact_id: "tcc-musl-selfhost-libtcc",
        digest_blake3: LIBTCC_ARCHIVE_BLAKE3,
    },
    ExpectedOutput {
        artifact_id: "tcc-musl-selfhost-main-object",
        digest_blake3: MAIN_OBJECT_BLAKE3,
    },
    ExpectedOutput {
        artifact_id: "tcc-musl-selfhost-patched-source",
        digest_blake3: PATCHED_SOURCE_BLAKE3,
    },
    ExpectedOutput {
        artifact_id: "tcc-musl-selfhost-object-tree",
        digest_blake3: OBJECT_TREE_BLAKE3,
    },
    ExpectedOutput {
        artifact_id: "tcc-musl-selfhost-smoke-object",
        digest_blake3: SMOKE_OBJECT_BLAKE3,
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
    pub build_command_count: u32,
    pub smoke_command_count: u32,
    pub outputs: Vec<OutputReport>,
    pub protected_exec_enforced: bool,
    pub fallback_events: Vec<String>,
    pub non_claim: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct InventoryRequest<'a> {
    pub tcc_musl_v2_root: &'a Path,
    pub tinycc26_root: &'a Path,
    pub musl_pass2_root: &'a Path,
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
            Self::Materialization(message) => write!(formatter, "TinyCC self-host materialization failed: {message}"),
            Self::Runtime(error) => write!(formatter, "TinyCC self-host runtime failed: {error}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<crate::stagex_mes_lib::MesLibraryPlanError> for Error {
    fn from(error: crate::stagex_mes_lib::MesLibraryPlanError) -> Self {
        Self::Runtime(error)
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
        .map_err(|error| Error::Materialization(format!("creating TinyCC self-host scratch: {error}")))?;
    let source_root = request.scratch_dir.join("source");
    crate::stagex_mes_lib::copy_tree_bounded(&request.tcc_musl_v2_root.join("share/tcc-source"), &source_root)?;
    crate::stagex_tinycc::make_tree_owner_writable(&source_root)?;
    apply_selfhost_adjustments(&source_root)?;
    fs::write(source_root.join("config.h"), [])
        .map_err(|error| Error::Materialization(format!("writing TinyCC self-host config: {error}")))?;
    let output_root = request.scratch_dir.join("output");
    prepare_output_contract(&source_root, &output_root)?;
    let (compiler, archive, main_object) = build_selfhost(&request, &source_root, &output_root)?;
    let smoke_object = run_smoke(&request, &output_root, &compiler)?;
    let outputs = collect_outputs(&output_root, &compiler, &archive, &main_object, &smoke_object)?;
    validate_expected_outputs(&outputs)?;
    let report = InventoryReport {
        format: REPORT_FORMAT,
        build_command_count: BUILD_COMMAND_COUNT,
        smoke_command_count: SMOKE_COMMAND_COUNT,
        outputs,
        protected_exec_enforced: request.protected_exec_enforced,
        fallback_events: Vec::new(),
        non_claim: NON_CLAIM,
    };
    fs::write(
        request.scratch_dir.join("tcc-musl-selfhost-inventory.json"),
        serde_json::to_vec_pretty(&report)
            .map_err(|error| Error::Materialization(format!("serializing TinyCC self-host inventory: {error}")))?,
    )
    .map_err(|error| Error::Materialization(format!("writing TinyCC self-host inventory: {error}")))?;
    assert_eq!(report.outputs.len(), OUTPUT_COUNT);
    assert!(report.fallback_events.is_empty());
    Ok(report)
}

fn validate_inputs(request: &InventoryRequest<'_>) -> Result<(), Error> {
    if request.scratch_dir.exists() {
        return Err(Error::Materialization("TinyCC self-host scratch already exists".to_string()));
    }
    validate_recipe()?;
    let required = [
        request.tcc_musl_v2_root.join("bin/tcc-0.9.27-musl-v2"),
        request.tcc_musl_v2_root.join("share/tcc-source/tcc.c"),
        request.tinycc26_root.join("bin/tcc-0.9.26"),
        request.tinycc26_root.join("lib/mes/libc.a"),
        request.musl_pass2_root.join("lib/libc.a"),
    ];
    for path in required {
        if !path.is_file() {
            return Err(Error::Materialization(format!(
                "required TinyCC self-host input is missing: {}",
                path.display()
            )));
        }
    }
    assert!(!request.scratch_dir.as_os_str().is_empty());
    assert!(request.tcc_musl_v2_root.is_absolute());
    Ok(())
}

fn validate_recipe() -> Result<(), Error> {
    let observed = blake3::hash(RECIPE).to_hex().to_string();
    if observed != RECIPE_BLAKE3 {
        return Err(Error::Materialization(format!("TinyCC self-host recipe mismatch: {observed}")));
    }
    assert!(!RECIPE.is_empty());
    assert_eq!(RECIPE_ARTIFACT_ID, "tcc-musl-selfhost-recipe-source");
    Ok(())
}

fn apply_selfhost_adjustments(source_root: &Path) -> Result<(), Error> {
    add_weak_archive_symbols(&source_root.join("tcctools.c"))?;
    crate::stagex_tinycc::replace_required_text(
        &source_root.join("tccgen.c"),
        "    dtype = *type;",
        "    memcpy(&dtype, type, sizeof(CType));",
    )?;
    patch_got_handling(&source_root.join("tccelf.c"))?;
    crate::stagex_tinycc::replace_required_text(
        &source_root.join("tccgen.c"),
        "                else if (sizeof (long double) == sizeof (double))\n                    __asm__(\"fldl %1\\nfstpt %0\\n\" : \"=m\" (*ptr) : \"m\" (vtop->c.ld));",
        "",
    )?;
    restore_variadic_diagnostics(&source_root.join("libtcc.c"))?;
    assert!(source_root.join("tcctools.c").is_file());
    assert!(source_root.join("libtcc.c").is_file());
    Ok(())
}

fn add_weak_archive_symbols(path: &Path) -> Result<(), Error> {
    crate::stagex_tinycc::replace_required_text(
        path,
        "                    || sym->st_info == 0x12\n                    )) {",
        "                    || sym->st_info == 0x12\n                    || sym->st_info == 0x20\n                    || sym->st_info == 0x21\n                    || sym->st_info == 0x22\n                    )) {",
    )?;
    assert!(path.is_file());
    assert!(!path.as_os_str().is_empty());
    Ok(())
}

fn patch_got_handling(path: &Path) -> Result<(), Error> {
    crate::stagex_tinycc::replace_required_text(
        path,
        "    unsigned offset = attr->got_offset;",
        "    if (!attr) return;\n    unsigned offset = attr->got_offset;",
    )?;
    crate::stagex_tinycc::replace_required_text(
        path,
        "ST_FUNC void relocate_syms",
        "static void bootstrap_undefined_symbol(const char *name)\n{\n    fputs(\"tcc: undefined symbol: \", stderr);\n    fputs(name, stderr);\n    fputc('\\n', stderr);\n    tcc_error_noabort(\"undefined symbol\");\n}\n\nST_FUNC void relocate_syms",
    )?;
    crate::stagex_tinycc::replace_required_text(
        path,
        "tcc_error_noabort(\"undefined symbol '%s'\", name);",
        "bootstrap_undefined_symbol(name);",
    )?;
    replace_c_function(path, "static void fill_local_got_entries", local_got_replacement())?;
    assert!(path.is_file());
    assert!(!local_got_replacement().is_empty());
    Ok(())
}

fn local_got_replacement() -> &'static str {
    "static void fill_local_got_entries(TCCState *s1)\n{\n    ElfW_Rel *rel;\n    if (!s1->got || !s1->got->reloc)\n        return;\n    for_each_elem(s1->got->reloc, 0, rel, ElfW_Rel) {\n        if (ELFW(R_TYPE)(rel->r_info) == R_RELATIVE) {\n            int sym_index = ELFW(R_SYM)(rel->r_info);\n            ElfW(Sym) *sym = &((ElfW(Sym) *)symtab_section->data)[sym_index];\n            unsigned long offset;\n            if (rel->r_offset < s1->got->sh_addr) {\n                tcc_error_noabort(\"local GOT relocation precedes GOT\");\n                continue;\n            }\n            offset = rel->r_offset - s1->got->sh_addr;\n            if (offset > s1->got->data_offset ||\n                s1->got->data_offset - offset < PTR_SIZE) {\n                tcc_error_noabort(\"local GOT relocation exceeds GOT\");\n                continue;\n            }\n            rel->r_info = ELFW(R_INFO)(0, R_RELATIVE);\n#if SHT_RELX == SHT_RELA\n            rel->r_addend = sym->st_value;\n#else\n            write32le(s1->got->data + offset, sym->st_value);\n#endif\n        }\n    }\n}"
}

fn restore_variadic_diagnostics(path: &Path) -> Result<(), Error> {
    let content = fs::read_to_string(path)
        .map_err(|error| Error::Materialization(format!("reading TinyCC diagnostics source: {error}")))?;
    let start = content
        .find("static void crunch_diag_putc")
        .ok_or_else(|| Error::Materialization("TinyCC diagnostic helper start is missing".to_string()))?;
    let end = content
        .find("static void strcat_printf")
        .ok_or_else(|| Error::Materialization("TinyCC diagnostic helper end is missing".to_string()))?;
    if end <= start {
        return Err(Error::Materialization("TinyCC diagnostic helper bounds are invalid".to_string()));
    }
    let replacement = "static void strcat_vprintf(char *buf, int buf_size, const char *fmt, va_list ap)\n{\n    int len = strlen(buf);\n    vsnprintf(buf + len, buf_size - len, fmt, ap);\n}\n\n";
    let mut patched = String::with_capacity(content.len());
    patched.push_str(&content[..start]);
    patched.push_str(replacement);
    patched.push_str(&content[end..]);
    fs::write(path, patched)
        .map_err(|error| Error::Materialization(format!("writing TinyCC diagnostics source: {error}")))?;
    assert!(end > start);
    assert!(path.is_file());
    Ok(())
}

fn replace_c_function(path: &Path, marker: &str, replacement: &str) -> Result<(), Error> {
    let content = fs::read_to_string(path)
        .map_err(|error| Error::Materialization(format!("reading TinyCC function source: {error}")))?;
    let start = content
        .find(marker)
        .ok_or_else(|| Error::Materialization(format!("TinyCC function marker is missing: {marker}")))?;
    let open_relative = content[start..]
        .find('{')
        .ok_or_else(|| Error::Materialization(format!("TinyCC function body is missing: {marker}")))?;
    let open = start
        .checked_add(open_relative)
        .ok_or_else(|| Error::Materialization("TinyCC function offset overflow".to_string()))?;
    let end = matching_brace_end(&content, open)?;
    let mut patched = String::with_capacity(content.len());
    patched.push_str(&content[..start]);
    patched.push_str(replacement);
    patched.push_str(&content[end..]);
    fs::write(path, patched)
        .map_err(|error| Error::Materialization(format!("writing TinyCC function source: {error}")))?;
    assert!(end > open);
    assert!(!replacement.is_empty());
    Ok(())
}

fn matching_brace_end(content: &str, open: usize) -> Result<usize, Error> {
    let mut depth = 0u32;
    for (relative, byte) in content.as_bytes()[open..].iter().enumerate() {
        if *byte == b'{' {
            depth = depth
                .checked_add(1)
                .ok_or_else(|| Error::Materialization("TinyCC brace depth overflow".to_string()))?;
        } else if *byte == b'}' {
            depth = depth
                .checked_sub(1)
                .ok_or_else(|| Error::Materialization("TinyCC brace depth underflow".to_string()))?;
            if depth == 0 {
                return open
                    .checked_add(relative + 1)
                    .ok_or_else(|| Error::Materialization("TinyCC brace offset overflow".to_string()));
            }
        }
    }
    Err(Error::Materialization("TinyCC function closing brace is missing".to_string()))
}

fn prepare_output_contract(source_root: &Path, output_root: &Path) -> Result<(), Error> {
    fs::create_dir_all(output_root.join("bin"))
        .map_err(|error| Error::Materialization(format!("creating TinyCC self-host bin: {error}")))?;
    fs::create_dir_all(output_root.join("lib/tcc/include"))
        .map_err(|error| Error::Materialization(format!("creating TinyCC self-host include: {error}")))?;
    fs::create_dir_all(output_root.join("share/selfhost-link/objects"))
        .map_err(|error| Error::Materialization(format!("creating TinyCC self-host object export: {error}")))?;
    crate::stagex_mes_lib::copy_tree_bounded(source_root, &output_root.join("share/tcc-source-patched"))?;
    crate::stagex_mes_lib::copy_tree_bounded(&source_root.join("include"), &output_root.join("lib/tcc/include"))?;
    assert!(output_root.join("bin").is_dir());
    assert!(output_root.join("share/tcc-source-patched/tcc.c").is_file());
    Ok(())
}

fn build_selfhost(
    request: &InventoryRequest<'_>,
    source_root: &Path,
    output_root: &Path,
) -> Result<(PathBuf, PathBuf, PathBuf), Error> {
    let host = request.tcc_musl_v2_root.join("bin/tcc-0.9.27-musl-v2");
    let object_root = output_root.join("share/selfhost-link/objects");
    for source in COMPILER_SOURCES {
        let object = object_root.join(source.replace(".c", ".o"));
        run(
            &host,
            &compile_args(request, output_root, source, &object)?,
            source_root,
            &request.scratch_dir.join(format!("compile-{source}.stderr.txt")),
        )?;
        set_owner_read_write(&object)?;
    }
    let main_object = object_root.join("tcc.o");
    let archive = output_root.join("share/selfhost-link/libtcc-selfhost.a");
    run(
        &host,
        &archive_args(&object_root, &archive)?,
        source_root,
        &request.scratch_dir.join("archive.stderr.txt"),
    )?;
    let compiler = output_root.join("bin/tcc-0.9.27-musl-selfhost");
    let linker = request.tinycc26_root.join("bin/tcc-0.9.26");
    run(
        &linker,
        &link_args(request, &main_object, &archive, &compiler)?,
        source_root,
        &request.scratch_dir.join("link.stderr.txt"),
    )?;
    crate::stagex_tinycc::set_owner_executable(&compiler)?;
    canonicalize_retained_objects(&object_root)?;
    fs::remove_file(&archive)
        .map_err(|error| Error::Materialization(format!("removing pre-canonical TinyCC archive: {error}")))?;
    run(
        &host,
        &archive_args(&object_root, &archive)?,
        source_root,
        &request.scratch_dir.join("archive-canonical.stderr.txt"),
    )?;
    crate::stagex_mes_lib::copy_file_exact(&compiler, &output_root.join("bin/tcc"))?;
    crate::stagex_tinycc::set_owner_executable(&output_root.join("bin/tcc"))?;
    crate::stagex_mes_lib::copy_file_exact(
        &request.tcc_musl_v2_root.join("lib/tcc/libtcc1.a"),
        &output_root.join("lib/tcc/libtcc1.a"),
    )?;
    crate::stagex_mes_lib::copy_file_exact(&main_object, &output_root.join("share/selfhost-link/tcc.o"))?;
    assert!(compiler.is_file());
    assert!(archive.is_file());
    Ok((compiler, archive, main_object))
}

fn canonicalize_retained_objects(object_root: &Path) -> Result<(), Error> {
    let mut rewrite_count = 0u32;
    for source in COMPILER_SOURCES {
        let object = object_root.join(source.replace(".c", ".o"));
        rewrite_count = rewrite_count
            .checked_add(canonicalize_elf_file(&object)?)
            .ok_or_else(|| Error::Materialization("TinyCC canonical rewrite count overflow".to_string()))?;
    }
    if rewrite_count == 0 {
        return Err(Error::Materialization(
            "TinyCC retained objects contain no decimal local symbols to canonicalize".to_string(),
        ));
    }
    assert!(rewrite_count > 0);
    debug_assert!(object_root.is_dir());
    Ok(())
}

fn canonicalize_elf_file(path: &Path) -> Result<u32, Error> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| Error::Materialization(format!("reading TinyCC object metadata: {error}")))?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > FILE_BYTES_MAX {
        return Err(Error::Materialization(format!("TinyCC canonicalization input is invalid: {}", path.display())));
    }
    let input_bytes = fs::read(path)
        .map_err(|error| Error::Materialization(format!("reading TinyCC object {}: {error}", path.display())))?;
    let canonical = crate::elf_local_symbol_core::canonicalize_local_elf_symbol_names(&input_bytes)
        .map_err(|error| Error::Materialization(format!("canonicalizing TinyCC object {}: {error}", path.display())))?;
    let rewrite_count = canonical.rewrite_count;
    let bytes = canonical.bytes;
    let staged = path.with_extension(CANONICAL_TEMP_EXTENSION);
    let write_result = (|| -> Result<(), Error> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&staged)
            .map_err(|error| Error::Materialization(format!("creating canonical TinyCC object: {error}")))?;
        file.write_all(&bytes)
            .map_err(|error| Error::Materialization(format!("writing canonical TinyCC object: {error}")))?;
        file.set_permissions(metadata.permissions())
            .map_err(|error| Error::Materialization(format!("setting canonical TinyCC object mode: {error}")))?;
        file.sync_all()
            .map_err(|error| Error::Materialization(format!("syncing canonical TinyCC object: {error}")))?;
        fs::rename(&staged, path)
            .map_err(|error| Error::Materialization(format!("publishing canonical TinyCC object: {error}")))?;
        Ok(())
    })();
    if write_result.is_err() && staged.exists() {
        let _ = fs::remove_file(&staged);
    }
    write_result?;
    assert!(u64::try_from(bytes.len()).is_ok_and(|bytes_len| bytes_len >= metadata.len()));
    debug_assert!(path.is_file());
    Ok(rewrite_count)
}

fn compile_args(
    request: &InventoryRequest<'_>,
    output_root: &Path,
    source: &str,
    object: &Path,
) -> Result<Vec<String>, Error> {
    let logical_tcc = "output/lib/tcc";
    let logical_musl = "../../musl-pass2-stage/runtime/output";
    let values = [
        "BOOTSTRAP=1".to_string(),
        "HAVE_BITFIELD=1".to_string(),
        "HAVE_FLOAT=1".to_string(),
        "HAVE_LONG_LONG=1".to_string(),
        "HAVE_SETJMP=1".to_string(),
        "TCC_TARGET_X86_64=1".to_string(),
        format!("CONFIG_TCCDIR=\"{logical_tcc}\""),
        format!("CONFIG_TCC_CRTPREFIX=\"{logical_musl}/lib\""),
        "CONFIG_TCC_ELFINTERP=\"/lib/ld-musl-x86_64.so.1\"".to_string(),
        format!("CONFIG_TCC_LIBPATHS=\"{logical_musl}/lib:{logical_tcc}\""),
        format!("CONFIG_TCC_SYSINCLUDEPATHS=\"{logical_tcc}/include:{logical_musl}/include\""),
        "CONFIG_SYSROOT=\"/\"".to_string(),
        format!("TCC_LIBGCC=\"{logical_musl}/lib/libc.a\""),
        "TCC_LIBTCC1=\"libtcc1.a\"".to_string(),
        "CONFIG_TCCBOOT=1".to_string(),
        "CONFIG_TCC_STATIC=1".to_string(),
        "CONFIG_USE_LIBGCC=1".to_string(),
        "TCC_VERSION=\"0.9.27\"".to_string(),
        "ONE_SOURCE=0".to_string(),
    ];
    let mut args = vec![
        "-c".to_string(),
        "-I".to_string(),
        ".".to_string(),
        "-I".to_string(),
        absolute_utf8(&request.tinycc26_root.join("include/mes"), "TinyCC self-host Mes include")?,
    ];
    for value in values {
        args.push("-D".to_string());
        args.push(value);
    }
    args.extend([
        "-o".to_string(),
        absolute_utf8(object, "TinyCC self-host object")?,
        source.to_string(),
    ]);
    assert!(output_root.is_absolute());
    assert!(args.len() > 1);
    Ok(args)
}

fn archive_args(object_root: &Path, archive: &Path) -> Result<Vec<String>, Error> {
    let mut args = vec![
        "-ar".to_string(),
        "rc".to_string(),
        absolute_utf8(archive, "TinyCC self-host archive")?,
    ];
    for source in COMPILER_SOURCES.iter().skip(1) {
        args.push(absolute_utf8(&object_root.join(source.replace(".c", ".o")), "TinyCC self-host library object")?);
    }
    assert_eq!(args.len(), COMPILER_SOURCE_COUNT as usize + 2);
    assert!(args.iter().all(|argument| !argument.is_empty()));
    Ok(args)
}

fn link_args(
    request: &InventoryRequest<'_>,
    main_object: &Path,
    archive: &Path,
    output: &Path,
) -> Result<Vec<String>, Error> {
    let lib = request.tinycc26_root.join("lib/mes");
    let args = vec![
        "-nostdlib".to_string(),
        "-static".to_string(),
        "-o".to_string(),
        absolute_utf8(output, "TinyCC self-host compiler")?,
        absolute_utf8(&lib.join("crt1.o"), "TinyCC self-host crt1")?,
        absolute_utf8(&lib.join("crti.o"), "TinyCC self-host crti")?,
        absolute_utf8(main_object, "TinyCC self-host main object")?,
        absolute_utf8(archive, "TinyCC self-host library")?,
        absolute_utf8(&lib.join("libc.a"), "TinyCC self-host Mes libc")?,
        absolute_utf8(&lib.join("tcc/libtcc1.a"), "TinyCC self-host Mes libtcc1")?,
        absolute_utf8(&lib.join("crtn.o"), "TinyCC self-host crtn")?,
    ];
    assert!(args.len() > 1);
    assert!(args.iter().all(|argument| !argument.is_empty()));
    Ok(args)
}

fn run_smoke(request: &InventoryRequest<'_>, output_root: &Path, compiler: &Path) -> Result<PathBuf, Error> {
    let smoke = request.scratch_dir.join("smoke");
    fs::create_dir(&smoke)
        .map_err(|error| Error::Materialization(format!("creating TinyCC self-host smoke: {error}")))?;
    crate::stagex_mes_lib::write_create_new(&smoke.join("syntax-smoke.c"), SMOKE_SOURCE)?;
    let object = smoke.join("syntax-smoke.o");
    run(
        compiler,
        &[
            "-c".to_string(),
            "-I".to_string(),
            absolute_utf8(&output_root.join("lib/tcc/include"), "TinyCC self-host include")?,
            "-o".to_string(),
            absolute_utf8(&object, "TinyCC self-host smoke object")?,
            "syntax-smoke.c".to_string(),
        ],
        &smoke,
        &smoke.join("smoke.stderr.txt"),
    )?;
    set_owner_read_write(&object)?;
    assert!(object.is_file());
    assert!(compiler.is_file());
    Ok(object)
}

fn run(program: &Path, args: &[String], cwd: &Path, stderr: &Path) -> Result<(), Error> {
    crate::stagex_mes_lib::run_bounded_process(program, args, cwd, &BTreeMap::<String, String>::new(), stderr)?;
    assert!(program.is_file());
    assert!(!args.is_empty());
    Ok(())
}

fn collect_outputs(
    output_root: &Path,
    compiler: &Path,
    archive: &Path,
    main_object: &Path,
    smoke_object: &Path,
) -> Result<Vec<OutputReport>, Error> {
    let specs = [
        ("tcc-musl-selfhost", compiler.to_path_buf(), false),
        ("tcc-musl-selfhost-alias", output_root.join("bin/tcc"), false),
        ("tcc-musl-selfhost-libtcc", archive.to_path_buf(), false),
        ("tcc-musl-selfhost-main-object", main_object.to_path_buf(), false),
        ("tcc-musl-selfhost-patched-source", output_root.join("share/tcc-source-patched"), true),
        ("tcc-musl-selfhost-object-tree", output_root.join("share/selfhost-link/objects"), true),
        ("tcc-musl-selfhost-smoke-object", smoke_object.to_path_buf(), false),
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
        return Ok((
            crate::stagex_musl::tree_bytes_len(path).map_err(|error| Error::Materialization(error.to_string()))?,
            crate::stagex_musl::tree_digest_blake3(path).map_err(|error| Error::Materialization(error.to_string()))?,
        ));
    }
    let bytes =
        fs::read(path).map_err(|error| Error::Materialization(format!("reading TinyCC self-host output: {error}")))?;
    if bytes.is_empty() || bytes.len() as u64 > FILE_BYTES_MAX {
        return Err(Error::Materialization(format!("TinyCC self-host output size is invalid: {}", path.display())));
    }
    assert!(!bytes.is_empty());
    assert!(bytes.len() as u64 <= FILE_BYTES_MAX);
    Ok((bytes.len() as u64, blake3::hash(&bytes).to_hex().to_string()))
}

fn validate_expected_outputs(outputs: &[OutputReport]) -> Result<(), Error> {
    let observed = outputs
        .iter()
        .map(|output| (output.artifact_id.as_str(), output.digest_blake3.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mismatches = EXPECTED_OUTPUTS
        .iter()
        .filter_map(|expected| {
            let actual = observed.get(expected.artifact_id).copied().unwrap_or("missing");
            (actual != expected.digest_blake3).then(|| format!("{}={actual}", expected.artifact_id))
        })
        .collect::<Vec<_>>();
    if !mismatches.is_empty() {
        return Err(Error::Materialization(format!(
            "TinyCC self-host output BLAKE3 mismatches: {}",
            mismatches.join(",")
        )));
    }
    assert_eq!(observed.len(), OUTPUT_COUNT);
    assert!(mismatches.is_empty());
    Ok(())
}

fn absolute_utf8(path: &Path, label: &str) -> Result<String, Error> {
    if !path.is_absolute() {
        return Err(Error::Materialization(format!("{label} must be absolute")));
    }
    let value = path.to_str().ok_or_else(|| Error::Materialization(format!("{label} is not UTF-8")))?;
    assert!(!value.is_empty());
    assert!(path.is_absolute());
    Ok(value.to_string())
}

fn set_owner_read_write(path: &Path) -> Result<(), Error> {
    let mut permissions = fs::metadata(path)
        .map_err(|error| Error::Materialization(format!("reading TinyCC self-host output mode: {error}")))?
        .permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        const OWNER_READ_WRITE_MODE: u32 = 0o600;
        permissions.set_mode(OWNER_READ_WRITE_MODE);
    }
    fs::set_permissions(path, permissions)
        .map_err(|error| Error::Materialization(format!("setting TinyCC self-host output mode: {error}")))?;
    assert!(path.is_file());
    assert!(!path.as_os_str().is_empty());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const OBJECT_VARIANT_A_ENV: &str = "MANTLE_TCC_SELFHOST_OBJECT_VARIANT_A";
    const OBJECT_VARIANT_B_ENV: &str = "MANTLE_TCC_SELFHOST_OBJECT_VARIANT_B";

    #[test]
    fn brace_match_accepts_nested_and_rejects_missing_close() {
        assert_eq!(matching_brace_end("x{a{b}c}z", 1).unwrap(), 8);
        assert!(matching_brace_end("x{a{b}z", 1).is_err());
    }

    #[test]
    fn selfhost_recipe_digest_is_bound() {
        validate_recipe().unwrap();
        assert_eq!(source_artifact_digests()[0].0, RECIPE_ARTIFACT_ID);
        assert_ne!(RECIPE_BLAKE3, blake3::hash(b"substituted").to_hex().to_string());
    }

    #[test]
    fn existing_scratch_fails_closed() {
        let scratch = tempfile::tempdir().unwrap();
        let request = InventoryRequest {
            tcc_musl_v2_root: Path::new("/missing-v2"),
            tinycc26_root: Path::new("/missing-tcc26"),
            musl_pass2_root: Path::new("/missing-musl"),
            scratch_dir: scratch.path(),
            protected_exec_enforced: false,
        };
        assert!(derive_inventory(request).is_err());
        assert!(scratch.path().exists());
    }

    #[test]
    #[ignore = "requires two retained TinyCC self-host object trees with different local-symbol widths"]
    fn retained_object_tree_width_variants_converge() {
        let root_a = PathBuf::from(std::env::var_os(OBJECT_VARIANT_A_ENV).unwrap());
        let root_b = PathBuf::from(std::env::var_os(OBJECT_VARIANT_B_ENV).unwrap());
        let mut distinct_input_count = 0u32;

        for source in COMPILER_SOURCES {
            let relative = source.replace(".c", ".o");
            let input_a = fs::read(root_a.join(&relative)).unwrap();
            let input_b = fs::read(root_b.join(&relative)).unwrap();
            distinct_input_count = distinct_input_count
                .checked_add(u32::from(input_a != input_b))
                .expect("bounded compiler source count");
            let canonical_a = crate::elf_local_symbol_core::canonicalize_local_elf_symbol_names(&input_a).unwrap();
            let canonical_b = crate::elf_local_symbol_core::canonicalize_local_elf_symbol_names(&input_b).unwrap();

            if canonical_a.bytes != canonical_b.bytes {
                let first_difference =
                    canonical_a.bytes.iter().zip(&canonical_b.bytes).position(|(left, right)| left != right);
                panic!(
                    "object {relative} canonical forms differ: first={first_difference:?} left_bytes={} right_bytes={} left_rewrites={} right_rewrites={}",
                    canonical_a.bytes.len(),
                    canonical_b.bytes.len(),
                    canonical_a.rewrite_count,
                    canonical_b.rewrite_count
                );
            }
            assert!(canonical_a.rewrite_count > 0, "object {relative}");
            assert!(canonical_b.rewrite_count > 0, "object {relative}");
        }
        assert!(distinct_input_count > 0);
        assert!(distinct_input_count <= COMPILER_SOURCE_COUNT);
    }

    #[test]
    #[ignore = "requires retained protected TinyCC musl-v2, TinyCC 0.9.26, and musl pass2 roots"]
    fn derives_retained_inventory() {
        let v2 = std::env::var_os("MANTLE_STAGE_X_TCC_MUSL_V2_ROOT").unwrap();
        let tcc26 = std::env::var_os("MANTLE_STAGE_X_TINYCC26_ROOT").unwrap();
        let musl = std::env::var_os("MANTLE_STAGE_X_MUSL_PASS2_ROOT").unwrap();
        let scratch = std::env::var_os("MANTLE_STAGE_X_TCC_SELFHOST_SCRATCH").unwrap();
        let report = derive_inventory(InventoryRequest {
            tcc_musl_v2_root: Path::new(&v2),
            tinycc26_root: Path::new(&tcc26),
            musl_pass2_root: Path::new(&musl),
            scratch_dir: Path::new(&scratch),
            protected_exec_enforced: false,
        })
        .unwrap();
        assert_eq!(report.outputs.len(), OUTPUT_COUNT);
        assert!(report.fallback_events.is_empty());
    }
}
