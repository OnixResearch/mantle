//! Conservative local C/C++ object cache client. Every uncertain invocation runs the real compiler.
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fs::File;
use std::fs::{self};
use std::io::Read;
use std::io::Write;
use std::os::unix::fs::FileTypeExt;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::thread;
use std::time::Duration;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use crunch_rust_cache_core::cc::CC_PROBE_RESULT_SCHEMA;
use crunch_rust_cache_core::cc::CC_REQUEST_SCHEMA;
use crunch_rust_cache_core::cc::CC_RESPONSE_SCHEMA;
use crunch_rust_cache_core::cc::CcActionInput;
use crunch_rust_cache_core::cc::CcDependency;
use crunch_rust_cache_core::cc::CcOperation;
use crunch_rust_cache_core::cc::CcProbeResult;
use crunch_rust_cache_core::cc::CcWireRequest;
use crunch_rust_cache_core::cc::CcWireResponse;
use crunch_rust_cache_core::cc::MAX_CC_DEPENDENCIES as MAX_DEPENDENCIES;
use crunch_rust_cache_core::cc::MAX_CC_FRAME_BYTES as MAX_FRAME;
use crunch_rust_cache_core::cc::MAX_CC_OBJECT_BYTES as MAX_OBJECT;
use crunch_rust_cache_core::cc::MAX_CC_PROBE_DIAGNOSTIC_BYTES;
use crunch_rust_cache_core::cc::MAX_CC_ROOTS as MAX_ROOTS;
use crunch_rust_cache_core::cc::admit_cc_reuse;
use crunch_rust_cache_core::cc::cc_action_key;
use crunch_rust_cache_core::cc::decode_cc_probe_result;
use crunch_rust_cache_core::cc::validate_cc_response;
use serde::Serialize;

const RECEIPT_SCHEMA: &str = "mantle-cc-driver-receipt-v1";
const MAX_DEPFILE: u64 = 1024 * 1024;
const MAX_FILE: u64 = 33_554_432; // 32 MiB
const MAX_TREE: u64 = 134_217_728; // 128 MiB
const MAX_FILES: usize = 8192;
const MAX_ARGS: usize = 256;
const MAX_ARG_BYTES: usize = 4096;
const MAX_RECEIPT: usize = 4096;

#[derive(Clone, Debug)]
pub struct DriverOptions {
    pub compiler: PathBuf,
    pub socket: PathBuf,
    pub receipt: PathBuf,
    pub platform_digest: String,
    pub arguments: Vec<String>,
    pub probe_script: Option<PathBuf>,
}

#[derive(Serialize)]
struct ProbeRead {
    kind: &'static str,
    operation: &'static str,
    disposition: &'static str,
}
#[derive(Serialize)]
struct Receipt<'a> {
    schema: &'static str,
    disposition: &'a str,
    reason: &'a str,
    action_key: Option<&'a str>,
    compiler_exit_code: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    probe_reads: Option<&'a [ProbeRead]>,
}

fn receipt(path: &Path, document: Receipt<'_>) {
    let result = (|| -> Result<(), String> {
        let bytes = serde_json::to_vec(&document).map_err(|e| e.to_string())?;
        if bytes.len() > MAX_RECEIPT {
            return Err("receipt too large".into());
        }
        assert!(!bytes.is_empty());
        assert!(bytes.len() <= MAX_RECEIPT);
        let parent = path.parent().ok_or("receipt has no parent")?;
        let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
        temporary.write_all(&bytes).map_err(|e| e.to_string())?;
        temporary.as_file().sync_all().map_err(|e| e.to_string())?;
        temporary.persist(path).map_err(|e| e.to_string())?;
        Ok(())
    })();
    if let Err(error) = result {
        eprintln!("mantle-cc-cache-driver: cannot record receipt: {error}");
    }
}

fn compiler_status<I, S>(compiler: &Path, arguments: I) -> i32
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    match Command::new(compiler).args(arguments).status() {
        Ok(status) => status.code().unwrap_or_else(|| 128 + status.signal().unwrap_or(1)),
        Err(error) => {
            eprintln!("mantle-cc-cache-driver: cannot execute compiler: {error}");
            127
        }
    }
}

fn fallback(options: &DriverOptions, reason: &str, action_key: Option<&str>) -> i32 {
    let exit = compiler_status(&options.compiler, &options.arguments);
    receipt(&options.receipt, Receipt {
        schema: RECEIPT_SCHEMA,
        disposition: "fallback",
        reason,
        action_key,
        compiler_exit_code: exit,
        probe_reads: None,
    });
    exit
}

/// Preserve arbitrary Unix compiler arguments byte-for-byte when classification is impossible.
pub fn run_os(mut options: DriverOptions, arguments: Vec<OsString>) -> i32 {
    if !options.compiler.is_absolute() {
        receipt(&options.receipt, Receipt {
            schema: RECEIPT_SCHEMA,
            disposition: "rejected",
            reason: "compiler-must-be-absolute",
            action_key: None,
            compiler_exit_code: 2,
            probe_reads: None,
        });
        return 2;
    }
    let Some(arguments_utf8) = arguments.iter().map(|arg| arg.to_str().map(str::to_owned)).collect::<Option<Vec<_>>>()
    else {
        let exit = compiler_status(&options.compiler, &arguments);
        receipt(&options.receipt, Receipt {
            schema: RECEIPT_SCHEMA,
            disposition: "fallback",
            reason: "non-utf8-compiler-argument",
            action_key: None,
            compiler_exit_code: exit,
            probe_reads: None,
        });
        return exit;
    };
    assert_eq!(arguments_utf8.len(), arguments.len());
    assert!(options.compiler.is_absolute());
    options.arguments = arguments_utf8;
    run(&options)
}

fn digest_file(path: &Path, bound: u64) -> Result<String, String> {
    let metadata = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if !metadata.file_type().is_file() || metadata.len() > bound {
        return Err("file is nonregular or exceeds bound".into());
    }
    assert!(metadata.len() <= bound);
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut hash = blake3::Hasher::new();
    let mut buffer = [0; 16384];
    let mut total = 0_u64;
    // Each nonempty read advances at least one byte; allow a final EOF read at the bound.
    for _ in 0..=bound {
        let count = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if count == 0 {
            assert!(total <= bound);
            return Ok(hash.finalize().to_hex().to_string());
        }
        total = total.checked_add(count as u64).ok_or("file length overflow")?;
        if total > bound {
            return Err("file exceeds bound".into());
        }
        hash.update(&buffer[..count]);
    }
    Err("file exceeds bound".into())
}

struct Classified {
    source: PathBuf,
    output: PathBuf,
    depfile: PathBuf,
    roots: Vec<PathBuf>,
    normalized: Vec<String>,
}

fn absolute_file(path: &str) -> Result<PathBuf, String> {
    let raw = Path::new(path);
    if !raw.is_absolute() {
        return Err("source and include roots must be absolute".into());
    }
    let canonical = raw.canonicalize().map_err(|e| e.to_string())?;
    if canonical != raw || !fs::symlink_metadata(raw).map_err(|e| e.to_string())?.is_file() {
        return Err("source must be canonical regular file".into());
    }
    Ok(canonical)
}

fn absolute_dir(path: &str) -> Result<PathBuf, String> {
    let raw = Path::new(path);
    if !raw.is_absolute() {
        return Err("include root must be absolute".into());
    }
    let canonical = raw.canonicalize().map_err(|e| e.to_string())?;
    if canonical != raw || !fs::symlink_metadata(raw).map_err(|e| e.to_string())?.is_dir() {
        return Err("include root must be canonical directory".into());
    }
    Ok(canonical)
}

fn output_path(path: &str) -> Result<PathBuf, String> {
    let raw = Path::new(path);
    if !raw.is_absolute() {
        return Err("output and depfile must be absolute".into());
    }
    let name = raw.file_name().ok_or("missing output filename")?;
    let parent = raw.parent().ok_or("missing output parent")?;
    let canonical = parent.canonicalize().map_err(|e| e.to_string())?;
    if canonical != parent || !fs::symlink_metadata(parent).map_err(|e| e.to_string())?.is_dir() {
        return Err("noncanonical output parent".into());
    }
    let output = canonical.join(name);
    if fs::symlink_metadata(&output).is_ok_and(|metadata| !metadata.is_file()) {
        return Err("output is not a regular file".into());
    }
    Ok(output)
}

fn next_argument<'a>(args: &'a [String], index: &mut usize) -> Result<&'a str, String> {
    *index += 1;
    args.get(*index).map(String::as_str).ok_or("missing option argument".into())
}

struct ParsedArguments {
    source: Option<PathBuf>,
    output: Option<PathBuf>,
    depfile: Option<PathBuf>,
    includes: Vec<(String, PathBuf)>,
    maps: Vec<(PathBuf, String)>,
    normalized: Vec<String>,
}

fn admitted_definition(arg: &str) -> bool {
    if !(arg.starts_with("-D") || arg.starts_with("-U")) {
        return false;
    }
    if arg.len() <= 2 {
        return false;
    }
    if ["__DATE__", "__TIME__", "__TIMESTAMP__"].iter().any(|name| arg.contains(name)) {
        return false;
    }
    arg[2..].bytes().all(|b| b.is_ascii_alphanumeric() || b"_=.()+-*".contains(&b))
}

fn include_root(includes: &mut Vec<(String, PathBuf)>, arg_count: usize, kind: String, root: PathBuf) {
    if includes.is_empty() {
        includes.reserve(arg_count);
    }
    includes.push((kind, root));
}

fn prefix_map(arg: &str) -> Result<(PathBuf, String), String> {
    let value = &arg[18..];
    let (from, to) = value.split_once('=').ok_or("invalid prefix map")?;
    if to.is_empty() || to.contains('\\') {
        return Err("unsafe prefix map destination".into());
    }
    if to.contains("/nix/store") || to.contains("/mantle/store") {
        return Err("unsafe prefix map destination".into());
    }
    Ok((absolute_dir(from)?, to.into()))
}

fn parse_arguments(args: &[String]) -> Result<ParsedArguments, String> {
    let (mut source, mut output, mut depfile) = (None, None, None);
    let (mut compile, mut deps, mut nostdinc, mut suppress_warnings) = (false, false, false, false);
    let mut includes: Vec<(String, PathBuf)> = Vec::new();
    let mut maps: Vec<(PathBuf, String)> = Vec::with_capacity(args.len());
    let mut normalized = Vec::with_capacity(args.len());
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        // Values are consumed without reordering or modifying the compiler's argv.
        match arg.as_str() {
            "-c" if !compile => compile = true,
            "-MMD" | "-MD" if !deps => {
                deps = true;
                normalized.push(arg.clone());
            }
            "-nostdinc" if !nostdinc => {
                nostdinc = true;
                normalized.push(arg.clone());
            }
            "-nostdinc++" => normalized.push(arg.clone()),
            "-w" if !suppress_warnings => {
                suppress_warnings = true;
                normalized.push(arg.clone());
            }
            "-fdiagnostics-color=never" | "-fmessage-length=0" => normalized.push(arg.clone()),
            "-o" if output.is_none() => output = Some(output_path(next_argument(args, &mut index)?)?),
            "-MF" if depfile.is_none() => depfile = Some(output_path(next_argument(args, &mut index)?)?),
            "-I" | "-isystem" => {
                let root = absolute_dir(next_argument(args, &mut index)?)?;
                include_root(&mut includes, args.len(), arg.clone(), root);
            }
            _ if arg.starts_with("-I") && arg.len() > 2 => {
                include_root(&mut includes, args.len(), "-I".into(), absolute_dir(&arg[2..])?);
            }
            _ if arg.starts_with("-ffile-prefix-map=") => maps.push(prefix_map(arg)?),
            "-O0" | "-O1" | "-O2" | "-O3" | "-Os" | "-Og" | "-Oz" | "-fPIC" | "-fpic" => normalized.push(arg.clone()),
            _ if arg.starts_with("-std=") && arg[5..].bytes().all(|b| b.is_ascii_alphanumeric() || b == b'+') => {
                normalized.push(arg.clone())
            }
            _ if admitted_definition(arg) => normalized.push(arg.clone()),
            _ if !arg.starts_with('-') && source.is_none() => source = Some(absolute_file(arg)?),
            _ => return Err("unknown compiler argument or side effect".into()),
        }
        index += 1;
    }
    if !compile || !deps {
        return Err("required -c, -MD/-MMD, -nostdinc, -w".into());
    }
    if !nostdinc || !suppress_warnings {
        return Err("required -c, -MD/-MMD, -nostdinc, -w".into());
    }
    assert!(includes.len() <= args.len());
    assert!(maps.len() <= args.len());
    Ok(ParsedArguments {
        source,
        output,
        depfile,
        includes,
        maps,
        normalized,
    })
}

fn validate_root_maps(roots: &[PathBuf], maps: &[(PathBuf, String)]) -> Result<(), String> {
    if roots.len() > MAX_ROOTS {
        return Err("too many roots".into());
    }
    for (index, root) in roots.iter().enumerate() {
        if roots
            .iter()
            .skip(index.saturating_add(1))
            .any(|other| root.starts_with(other) || other.starts_with(root))
        {
            return Err("overlapping roots make prefix remapping ambiguous".into());
        }
    }
    if maps.len() != roots.len() {
        return Err("one explicit -ffile-prefix-map per root required".into());
    }
    for (index, root) in roots.iter().enumerate() {
        let expected = format!("/cc-root-{index}");
        if maps.iter().filter(|(from, to)| from == root && to == &expected).count() != 1 {
            return Err("missing stable source/include path remapping".into());
        }
    }
    assert!(roots.len() <= MAX_ROOTS);
    assert!(maps.len() <= MAX_ROOTS);
    Ok(())
}

/// Classify only an explicit dependency-producing object compilation, without implicit includes.
fn classify(args: &[String]) -> Result<Classified, String> {
    if args.len() > MAX_ARGS || args.iter().any(|arg| arg.is_empty() || arg.len() > MAX_ARG_BYTES || arg.contains('\0'))
    {
        return Err("argument bound or encoding".into());
    }
    let ParsedArguments {
        source,
        output,
        depfile,
        includes,
        maps,
        mut normalized,
    } = parse_arguments(args)?;
    let source = source.ok_or("missing source")?;
    if !matches!(source.extension().and_then(|e| e.to_str()), Some("c" | "cc" | "cpp" | "cxx")) {
        return Err("unsupported source extension".into());
    }
    let output = output.ok_or("missing -o")?;
    let depfile = depfile.ok_or("missing -MF")?;
    if output == depfile || output == source || depfile == source {
        return Err("overlapping source and outputs".into());
    }
    if !safe_make_path(&output) || !safe_make_path(&depfile) || !safe_make_path(&source) {
        return Err("exotic make path".into());
    }
    if output.extension().and_then(|e| e.to_str()) != Some("o") {
        return Err("expected .o output".into());
    }
    let mut roots = Vec::with_capacity(includes.len().saturating_add(1));
    roots.push(source.parent().ok_or("missing source parent")?.to_path_buf());
    for (_, root) in &includes {
        if roots.contains(root) {
            return Err("duplicate include root".into());
        }
        roots.push(root.clone());
    }
    validate_root_maps(&roots, &maps)?;
    assert!(roots.len() <= MAX_ROOTS);
    for (kind, root) in &includes {
        let index = roots.iter().position(|item| item == root).ok_or("unmapped include root")?;
        normalized.push(format!("{kind}@root:{index}"));
    }
    for index in 0..roots.len() {
        normalized.push(format!("map:@root:{index}:cc-root-{index}"));
    }
    normalized.push(format!("source:@root:0/{}", source.file_name().ok_or("missing filename")?.to_string_lossy()));
    assert!(normalized.len() <= args.len());
    // Normalize semantic switches in their original order instead of normalizing away -I precedence.
    // Input root tree identities still bind every physical path's complete content.
    Ok(Classified {
        source,
        output,
        depfile,
        roots,
        normalized,
    })
}

fn file_bytes(path: &Path, bound: u64) -> Result<Vec<u8>, String> {
    let metadata = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if !metadata.is_file() || metadata.len() > bound {
        return Err("file exceeds bound or is nonregular".into());
    }
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mut data = Vec::new();
    file.take(bound.saturating_add(1)).read_to_end(&mut data).map_err(|e| e.to_string())?;
    if data.len() as u64 > bound {
        return Err("file exceeds bound".into());
    }
    Ok(data)
}

fn scan_root_file(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0_u8; 16384];
    let mut inspection = [0_u8; 16398];
    let mut trailing = 0_usize;
    let mut total = 0_u64;
    // Short reads still advance by at least one byte; allow an EOF read at exactly 32 MiB.
    for _ in 0..=MAX_FILE {
        let count = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if count == 0 {
            assert!(total <= MAX_FILE);
            return Ok(hasher.finalize().to_hex().to_string());
        }
        total = total.checked_add(count as u64).ok_or("root file size overflow")?;
        if total > MAX_FILE {
            return Err("root file exceeds bound".into());
        }
        hasher.update(&buffer[..count]);
        assert!(trailing <= 14);
        let length_bytes = trailing.checked_add(count).ok_or("inspection length overflow")?;
        assert!(length_bytes <= inspection.len());
        inspection[trailing..length_bytes].copy_from_slice(&buffer[..count]);
        let examined = &inspection[..length_bytes];
        if examined.windows(2).any(|window| window == b"##")
            || [b"__DATE__".as_slice(), b"__TIME__", b"__TIMESTAMP__"]
                .iter()
                .any(|needle| examined.windows(needle.len()).any(|window| window == *needle))
        {
            return Err("time-dependent or dynamically synthesized compiler macro".into());
        }
        trailing = length_bytes.min(14);
        inspection.copy_within(length_bytes.saturating_sub(trailing)..length_bytes, 0);
    }
    Err("root file exceeds bound".into())
}

fn tree_digests(classified: &Classified, options: &DriverOptions) -> Result<Vec<String>, String> {
    let mut total = 0_u64;
    let mut entries_seen = 0_usize;
    let mut result = Vec::with_capacity(classified.roots.len());
    for root in &classified.roots {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"mantle.cc.complete-root.v1\0");
        let mut pending = vec![root.clone()];
        let mut entries = BTreeMap::new();
        while let Some(directory) = pending.pop() {
            for item in fs::read_dir(directory).map_err(|e| e.to_string())? {
                let entry = item.map_err(|e| e.to_string())?;
                let path = entry.path();
                entries_seen = entries_seen.checked_add(1).ok_or("root entries overflow")?;
                let max_depth_components = root.components().count().saturating_add(128);
                if entries_seen > MAX_FILES || path.components().count() > max_depth_components {
                    return Err("root entries or depth exceeded".into());
                }
                assert!(entries_seen <= MAX_FILES);
                if path == options.receipt || path == options.socket {
                    continue;
                }
                let metadata = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
                if metadata.file_type().is_symlink()
                    || metadata.file_type().is_socket()
                    || metadata.file_type().is_fifo()
                {
                    return Err("unsafe root entry".into());
                }
                if metadata.is_dir() {
                    pending.push(path);
                    continue;
                }
                if !metadata.is_file() {
                    return Err("nonregular root entry".into());
                }
                if path == classified.output || path == classified.depfile {
                    continue;
                }
                total = total.checked_add(metadata.len()).ok_or("root size overflow")?;
                if total > MAX_TREE || metadata.len() > MAX_FILE {
                    return Err("root bound exceeded".into());
                }
                assert!(total <= MAX_TREE);
                let relative =
                    path.strip_prefix(root).map_err(|e| e.to_string())?.to_str().ok_or("non-UTF8 root path")?;
                if relative.contains('\\') || relative.len() > 4096 {
                    return Err("unsupported root filename".into());
                }
                // Read once while checking macro safety and hashing the complete file.
                if entries.len() >= MAX_FILES {
                    return Err("root entries exceeded".into());
                }
                entries.insert(relative.to_owned(), scan_root_file(&path)?);
            }
        }
        for (path, digest) in entries {
            hasher.update(&(path.len() as u64).to_be_bytes());
            hasher.update(path.as_bytes());
            hasher.update(digest.as_bytes());
        }
        result.push(hasher.finalize().to_hex().to_string());
    }
    Ok(result)
}

fn depfile_target_separator(bytes: &[u8]) -> Result<usize, String> {
    let mut colon = None;
    let mut is_escaped = false;
    for (index, byte) in bytes.iter().enumerate() {
        if is_escaped {
            is_escaped = false;
            continue;
        }
        if *byte == b'\\' {
            is_escaped = true;
            continue;
        }
        if *byte == b':' {
            colon = Some(index);
            break;
        }
        if *byte == b'\n' {
            return Err("depfile has no target".into());
        }
    }
    let colon = colon.ok_or("depfile missing target separator")?;
    if colon == 0 {
        return Err("depfile empty target".into());
    }
    assert!(colon < bytes.len());
    assert_eq!(bytes[colon], b':');
    Ok(colon)
}

fn push_depfile_token(paths: &mut Vec<String>, token: &mut Vec<u8>) -> Result<(), String> {
    if token.is_empty() {
        return Ok(());
    }
    if paths.len() >= MAX_DEPENDENCIES {
        return Err("depfile bound exceeded".into());
    }
    paths.push(String::from_utf8(std::mem::take(token)).map_err(|e| e.to_string())?);
    Ok(())
}

/// Parse a single make-style depfile with backslash-escaped whitespace and continuations.
/// Unknown constructs fail closed rather than publishing an incomplete manifest.
pub fn parse_depfile(bytes: &[u8]) -> Result<Vec<String>, String> {
    if bytes.is_empty() || bytes.len() as u64 > MAX_DEPFILE || bytes.contains(&0) {
        return Err("invalid depfile length".into());
    }
    let colon = depfile_target_separator(bytes)?;
    let mut paths = Vec::with_capacity(bytes.len().min(MAX_DEPENDENCIES));
    let mut token = Vec::with_capacity(bytes.len().min(4096));
    let mut index = colon.checked_add(1).ok_or("depfile index overflow")?;
    assert!(index <= bytes.len());
    assert!(bytes.len() as u64 <= MAX_DEPFILE);
    while index < bytes.len() {
        let byte = bytes[index];
        if byte == b'\\' {
            index += 1;
            if index == bytes.len() {
                return Err("unterminated depfile escape".into());
            }
            if bytes[index] == b'\r' && bytes.get(index.saturating_add(1)) == Some(&b'\n') {
                index += 2;
                continue;
            }
            if bytes[index] == b'\n' {
                index += 1;
                continue;
            }
            token.push(bytes[index]);
        } else if byte.is_ascii_whitespace() {
            push_depfile_token(&mut paths, &mut token)?;
        } else if byte == b':' || byte == b'#' || byte == b'$' {
            return Err("unsupported make dependency syntax".into());
        } else {
            token.push(byte);
        }
        if token.len() > 4096 || paths.len() > MAX_DEPENDENCIES {
            return Err("depfile bound exceeded".into());
        }
        index += 1;
    }
    push_depfile_token(&mut paths, &mut token)?;
    if paths.is_empty() {
        return Err("depfile lacks prerequisites".into());
    }
    assert!(paths.len() <= MAX_DEPENDENCIES);
    Ok(paths)
}

fn dependency(path: &Path, classified: &Classified) -> Result<CcDependency, String> {
    let canonical = path.canonicalize().map_err(|e| e.to_string())?;
    if fs::symlink_metadata(path).map_err(|e| e.to_string())?.file_type().is_symlink() {
        return Err("symlink dependency".into());
    }
    for (index, root) in classified.roots.iter().enumerate() {
        if let Ok(relative) = canonical.strip_prefix(root) {
            let value = relative.to_str().ok_or("non-UTF8 dependency")?;
            if value.is_empty() || value.len() > 4096 {
                return Err("unsafe dependency label".into());
            }
            if value.contains('\\') || value.split('/').any(|c| c == "." || c == ".." || c.is_empty()) {
                return Err("unsafe dependency label".into());
            }
            if canonical == classified.output || canonical == classified.depfile {
                return Err("dependency is output".into());
            }
            assert!(index < MAX_ROOTS);
            assert!(value.len() <= 4096);
            return Ok(CcDependency {
                root_index: u32::try_from(index).map_err(|_| "root index overflow")?,
                relative_path: value.into(),
                digest_blake3: digest_file(&canonical, MAX_FILE)?,
            });
        }
    }
    Err("dependency outside declared roots".into())
}

fn dependencies_from_depfile(classified: &Classified) -> Result<Vec<CcDependency>, String> {
    let data = file_bytes(&classified.depfile, MAX_DEPFILE)?;
    let paths = parse_depfile(&data)?;
    let mut dependencies = Vec::with_capacity(paths.len());
    let mut seen = BTreeSet::new();
    for name in paths {
        let path = Path::new(&name);
        if !path.is_absolute() || !safe_make_path(path) {
            return Err("nonliteral dependency path".into());
        }
        let entry = dependency(path, classified)?;
        let root_index = usize::try_from(entry.root_index).map_err(|_| "root index overflow")?;
        let root = classified.roots.get(root_index).ok_or("unknown root index")?;
        if path != root.join(&entry.relative_path) {
            return Err("noncanonical or duplicate dependency".into());
        }
        if !seen.insert((entry.root_index, entry.relative_path.clone())) {
            return Err("noncanonical or duplicate dependency".into());
        }
        dependencies.push(entry);
    }
    let source = dependency(&classified.source, classified)?;
    if !seen.contains(&(source.root_index, source.relative_path)) {
        return Err("depfile omits source".into());
    }
    if data != serialized_depfile(classified, &dependencies)?.as_bytes() {
        return Err("unknown depfile formatting".into());
    }
    assert_eq!(dependencies.len(), seen.len());
    assert!(dependencies.len() <= MAX_DEPENDENCIES);
    Ok(dependencies)
}

fn dependencies_from_record(classified: &Classified, record: &[CcDependency]) -> Result<Vec<CcDependency>, String> {
    let mut result = Vec::with_capacity(record.len());
    for entry in record {
        let root_index = usize::try_from(entry.root_index).map_err(|_| "root index overflow")?;
        let root = classified.roots.get(root_index).ok_or("unknown root index")?;
        let path = root.join(&entry.relative_path);
        let current = dependency(&path, classified)?;
        if current.root_index != entry.root_index || current.relative_path != entry.relative_path {
            return Err("dependency label changed".into());
        }
        result.push(current);
    }
    Ok(result)
}

fn connect(socket: &Path) -> Result<UnixStream, String> {
    let stream = UnixStream::connect(socket).map_err(|e| e.to_string())?;
    let read_write_duration = Duration::from_secs(3);
    stream.set_read_timeout(Some(read_write_duration)).map_err(|e| e.to_string())?;
    stream.set_write_timeout(Some(read_write_duration)).map_err(|e| e.to_string())?;
    Ok(stream)
}

fn request(socket: &Path, wire: CcWireRequest) -> Result<CcWireResponse, String> {
    let mut stream = connect(socket)?;
    let bytes = serde_json::to_vec(&wire).map_err(|e| e.to_string())?;
    if bytes.is_empty() || bytes.len() as u64 > MAX_FRAME {
        return Err("request frame bound exceeded".into());
    }
    assert!(!bytes.is_empty());
    assert!(bytes.len() as u64 <= MAX_FRAME);
    stream.write_all(&(bytes.len() as u64).to_be_bytes()).map_err(|e| e.to_string())?;
    stream.write_all(&bytes).map_err(|e| e.to_string())?;
    let mut prefix = [0; 8];
    stream.read_exact(&mut prefix).map_err(|e| e.to_string())?;
    let count = u64::from_be_bytes(prefix);
    if count == 0 || count > MAX_FRAME {
        return Err("response frame bound exceeded".into());
    }
    let length_bytes = usize::try_from(count).map_err(|_| "response frame length exceeds platform")?;
    assert_eq!(length_bytes as u64, count);
    let mut body = vec![0; length_bytes];
    stream.read_exact(&mut body).map_err(|e| e.to_string())?;
    let response: CcWireResponse = serde_json::from_slice(&body).map_err(|e| e.to_string())?;
    if response.schema != CC_RESPONSE_SCHEMA || response.action_key != wire.action_key {
        return Err("response identity mismatch".into());
    }
    Ok(response)
}

fn wire(
    operation: CcOperation,
    key: &str,
    dependencies: Vec<CcDependency>,
    object_base64: Option<String>,
) -> CcWireRequest {
    CcWireRequest {
        schema: CC_REQUEST_SCHEMA.into(),
        operation,
        action_key: key.into(),
        dependencies,
        object_base64,
    }
}

fn safe_make_path(path: &Path) -> bool {
    path.to_str().is_some_and(|text| {
        !text.is_empty() && text.bytes().all(|b| b.is_ascii_alphanumeric() || b"/._-+".contains(&b))
    })
}

fn serialized_depfile(classified: &Classified, dependencies: &[CcDependency]) -> Result<String, String> {
    if !safe_make_path(&classified.output) {
        return Err("unsafe make target".into());
    }
    if dependencies.len() > MAX_DEPENDENCIES {
        return Err("restored depfile exceeds bound".into());
    }
    assert!(dependencies.len() <= MAX_DEPENDENCIES);
    let mut dep = format!("{}:", classified.output.display());
    let mut column = dep.len();
    for entry in dependencies {
        let root_index = usize::try_from(entry.root_index).map_err(|_| "root index overflow")?;
        let root = classified.roots.get(root_index).ok_or("unknown dep root")?;
        let path = root.join(&entry.relative_path);
        if !safe_make_path(&path) {
            return Err("unsafe make prerequisite".into());
        }
        let text = path.to_str().ok_or("non-UTF8 prerequisite")?;
        if column.saturating_add(text.len()) > 72 {
            dep.push_str(" \\\n");
            column = 0;
        }
        dep.push(' ');
        dep.push_str(text);
        column = column.saturating_add(1).saturating_add(text.len());
    }
    dep.push('\n');
    if dep.len() as u64 > MAX_DEPFILE {
        return Err("restored depfile exceeds bound".into());
    }
    assert!(dep.len() as u64 <= MAX_DEPFILE);
    Ok(dep)
}

fn artifact_permissions(path: &Path) -> Result<fs::Permissions, String> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() => Ok(fs::Permissions::from_mode(metadata.permissions().mode() & 0o777)),
        Ok(_) => Err("artifact destination is not regular".into()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(fs::Permissions::from_mode(0o644)),
        Err(error) => Err(error.to_string()),
    }
}

fn restore(classified: &Classified, dependencies: &[CcDependency], object: &[u8]) -> Result<(), String> {
    if object.len() as u64 > MAX_OBJECT {
        return Err("oversized restored object".into());
    }
    let dep = serialized_depfile(classified, dependencies)?;
    assert!(object.len() as u64 <= MAX_OBJECT);
    assert!(dep.len() as u64 <= MAX_DEPFILE);
    // Stage both files before publishing either. A failed restoration never
    // commits a cached object before the fallback compiler has run.
    let dep_parent = classified.depfile.parent().ok_or("depfile parent missing")?;
    let object_parent = classified.output.parent().ok_or("object parent missing")?;
    let mut staged_dep = tempfile::NamedTempFile::new_in(dep_parent).map_err(|e| e.to_string())?;
    let mut staged_object = tempfile::NamedTempFile::new_in(object_parent).map_err(|e| e.to_string())?;
    staged_dep.write_all(dep.as_bytes()).map_err(|e| e.to_string())?;
    staged_object.write_all(object).map_err(|e| e.to_string())?;
    staged_dep
        .as_file()
        .set_permissions(artifact_permissions(&classified.depfile)?)
        .map_err(|e| e.to_string())?;
    staged_object
        .as_file()
        .set_permissions(artifact_permissions(&classified.output)?)
        .map_err(|e| e.to_string())?;
    staged_dep.as_file().sync_all().map_err(|e| e.to_string())?;
    staged_object.as_file().sync_all().map_err(|e| e.to_string())?;
    staged_dep.persist(&classified.depfile).map_err(|e| e.to_string())?;
    staged_object.persist(&classified.output).map_err(|e| e.to_string())?;
    Ok(())
}

fn gnu_compiler(compiler: &Path) -> bool {
    let Ok(canonical) = compiler.canonicalize() else {
        return false;
    };
    let Some(name) = canonical.file_name().and_then(|s| s.to_str()) else {
        return false;
    };
    assert!(!name.is_empty());
    assert!(canonical.is_absolute());
    let stem = name
        .split_once("-gcc")
        .map(|(prefix, suffix)| (!prefix.is_empty(), suffix))
        .or_else(|| name.split_once("-g++").map(|(prefix, suffix)| (!prefix.is_empty(), suffix)));
    matches!(name, "gcc" | "g++")
        || stem.is_some_and(|(prefix, suffix)| {
            prefix
                && (suffix.is_empty()
                    || suffix.strip_prefix('-').is_some_and(|version| {
                        !version.is_empty() && version.bytes().all(|b| b.is_ascii_digit() || b == b'.')
                    }))
        })
        || ["gcc-", "g++-"].iter().any(|prefix| {
            name.strip_prefix(prefix)
                .is_some_and(|version| !version.is_empty() && version.bytes().all(|b| b.is_ascii_digit() || b == b'.'))
        })
}

#[derive(Clone, Copy)]
struct ProbeIdentity<'a> {
    key: &'a str,
    roots: &'a [String],
    kind: &'static str,
    script_digest: &'a str,
    tool_digest: &'a str,
    roots_count: usize,
}

fn probe_attempt(
    options: &DriverOptions,
    identity: ProbeIdentity<'_>,
    operation: CcOperation,
    dependencies: Vec<CcDependency>,
    reads: &mut Vec<ProbeRead>,
) -> Option<CcWireResponse> {
    assert!(identity.roots_count <= MAX_ROOTS);
    let label = if operation == CcOperation::Manifest {
        "manifest"
    } else {
        "read"
    };
    let response = request(&options.socket, wire(operation, identity.key, dependencies, None));
    let (result, disposition) = match response {
        Ok(reply) if validate_cc_response(&reply, identity.roots_count).is_ok() => {
            if reply.disposition == "hit" || reply.disposition == "miss" {
                let label = if reply.disposition == "hit" { "hit" } else { "miss" };
                (Some(reply), label)
            } else {
                (None, "invalid")
            }
        }
        Ok(_) => (None, "invalid"),
        Err(_) => (None, "unavailable"),
    };
    assert!(reads.len() < 4);
    reads.push(ProbeRead {
        kind: identity.kind,
        operation: label,
        disposition,
    });
    result
}

fn capture_diagnostic<R: Read, W: Write>(mut input: R, mut output: W) -> (Vec<u8>, bool) {
    let mut captured = Vec::new();
    let mut is_complete = true;
    let mut buffer = [0; 8192];
    let mut next = input.read(&mut buffer);
    // Compiler streams are forwarded until EOF; only the cacheable copy is byte-bounded.
    while let Ok(count) = next {
        if count == 0 {
            assert!(captured.len() <= MAX_CC_PROBE_DIAGNOSTIC_BYTES);
            return (captured, is_complete);
        }
        assert!(count <= buffer.len());
        if output.write_all(&buffer[..count]).is_err() {
            is_complete = false;
        }
        if captured.len().checked_add(count).is_some_and(|size| size <= MAX_CC_PROBE_DIAGNOSTIC_BYTES) {
            captured.extend_from_slice(&buffer[..count]);
        } else {
            is_complete = false;
        }
        next = input.read(&mut buffer);
    }
    (captured, false)
}

type CompilerDiagnostics = (Vec<u8>, Vec<u8>);

fn abort_compiler_after_missing_pipe(child: &mut std::process::Child) {
    if let Err(error) = child.kill() {
        eprintln!("mantle-cc-cache-driver: cannot stop compiler after missing pipe: {error}");
    }
    if let Err(error) = child.wait() {
        eprintln!("mantle-cc-cache-driver: cannot reap compiler after missing pipe: {error}");
    }
}

fn compiler_with_diagnostics(options: &DriverOptions) -> (i32, Option<CompilerDiagnostics>) {
    assert!(options.compiler.is_absolute());
    assert!(options.arguments.len() <= MAX_ARGS);
    let mut child = match Command::new(&options.compiler)
        .args(&options.arguments)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            eprintln!("mantle-cc-cache-driver: cannot execute compiler: {error}");
            return (127, None);
        }
    };
    let Some(stdout) = child.stdout.take() else {
        abort_compiler_after_missing_pipe(&mut child);
        return (127, None);
    };
    let Some(stderr) = child.stderr.take() else {
        abort_compiler_after_missing_pipe(&mut child);
        return (127, None);
    };
    let out = thread::spawn(move || capture_diagnostic(stdout, std::io::stdout().lock()));
    let err = thread::spawn(move || capture_diagnostic(stderr, std::io::stderr().lock()));
    let status = child.wait();
    let out = out.join();
    let err = err.join();
    let exit = status
        .map(|status| status.code().unwrap_or_else(|| 128 + status.signal().unwrap_or(1)))
        .unwrap_or(127);
    match (out, err) {
        (Ok((stdout, true)), Ok((stderr, true))) => (exit, Some((stdout, stderr))),
        _ => (exit, None),
    }
}

fn restore_failure_depfile(classified: &Classified, dependencies: &[CcDependency]) -> Result<(), String> {
    if classified.output.exists() || classified.depfile.exists() {
        return Err("stale artifact".into());
    }
    let bytes = serialized_depfile(classified, dependencies)?;
    assert!(bytes.len() as u64 <= MAX_DEPFILE);
    let parent = classified.depfile.parent().ok_or("depfile parent missing")?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    temporary.write_all(bytes.as_bytes()).map_err(|e| e.to_string())?;
    temporary
        .as_file()
        .set_permissions(artifact_permissions(&classified.depfile)?)
        .map_err(|e| e.to_string())?;
    temporary.as_file().sync_all().map_err(|e| e.to_string())?;
    temporary.persist(&classified.depfile).map_err(|e| e.to_string())?;
    Ok(())
}

struct VerifiedProbeArtifact<'a> {
    dependencies: &'a [CcDependency],
    bytes: &'a [u8],
    reads: &'a [ProbeRead],
}

fn restore_probe_hit(
    options: &DriverOptions,
    classified: &Classified,
    identity: ProbeIdentity<'_>,
    artifact: VerifiedProbeArtifact<'_>,
) -> Option<i32> {
    if identity.kind == "object" {
        assert!(artifact.bytes.len() as u64 <= MAX_OBJECT);
        assert!(!artifact.dependencies.is_empty());
        restore(classified, artifact.dependencies, artifact.bytes).ok()?;
        receipt(&options.receipt, Receipt {
            schema: RECEIPT_SCHEMA,
            disposition: "hit",
            reason: "verified-object-and-dependencies",
            action_key: Some(identity.key),
            compiler_exit_code: 0,
            probe_reads: Some(artifact.reads),
        });
        return Some(0);
    }
    let result: CcProbeResult = serde_json::from_slice(artifact.bytes).ok()?;
    let (stdout, stderr) = decode_cc_probe_result(&result).ok()?;
    if result.script_digest_blake3 != identity.script_digest || result.compiler_digest_blake3 != identity.tool_digest {
        return None;
    }
    if result.source_path != classified.source.to_str()?
        || result.output_path != classified.output.to_str()?
        || result.depfile_path != classified.depfile.to_str()?
    {
        return None;
    }
    let script = options.probe_script.as_ref()?;
    if digest_file(script, MAX_FILE).ok().as_deref() != Some(identity.script_digest) {
        return None;
    }
    if digest_file(&options.compiler.canonicalize().ok()?, MAX_TREE).ok().as_deref() != Some(identity.tool_digest) {
        return None;
    }
    restore_failure_depfile(classified, artifact.dependencies).ok()?;
    std::io::stdout().write_all(&stdout).ok()?;
    std::io::stderr().write_all(&stderr).ok()?;
    receipt(&options.receipt, Receipt {
        schema: RECEIPT_SCHEMA,
        disposition: "probe-failure-hit",
        reason: "verified-failure-and-dependencies",
        action_key: Some(identity.key),
        compiler_exit_code: result.compiler_exit_code,
        probe_reads: Some(artifact.reads),
    });
    Some(result.compiler_exit_code)
}

fn probe_hit(
    options: &DriverOptions,
    classified: &Classified,
    identity: ProbeIdentity<'_>,
    reads: &mut Vec<ProbeRead>,
) -> Option<i32> {
    let manifest = probe_attempt(options, identity, CcOperation::Manifest, Vec::new(), reads)?;
    if manifest.disposition != "hit" {
        return None;
    }
    let record = manifest.record.as_ref()?;
    let current = dependencies_from_record(classified, &record.dependencies).ok()?;
    if record.action_key != identity.key || record.dependencies.is_empty() {
        return None;
    }
    if admit_cc_reuse(record, &current, identity.roots_count) != Ok(true)
        || tree_digests(classified, options).as_deref() != Ok(identity.roots)
    {
        return None;
    }
    let response = probe_attempt(options, identity, CcOperation::Read, current.clone(), reads)?;
    let read_record = response.record.as_ref()?;
    let encoded = response.object_base64.as_ref()?;
    if response.disposition != "hit" || read_record != record {
        return None;
    }
    if admit_cc_reuse(read_record, &current, identity.roots_count) != Ok(true) {
        return None;
    }
    let bytes = STANDARD.decode(encoded).ok()?;
    if bytes.len() as u64 != record.object_bytes || bytes.len() as u64 > MAX_OBJECT {
        return None;
    }
    if blake3::hash(&bytes).to_hex().as_str() != record.object_digest_blake3
        || tree_digests(classified, options).as_deref() != Ok(identity.roots)
    {
        return None;
    }
    assert!(bytes.len() as u64 <= MAX_OBJECT);
    assert!(identity.roots_count <= MAX_ROOTS);
    restore_probe_hit(options, classified, identity, VerifiedProbeArtifact {
        dependencies: &current,
        bytes: &bytes,
        reads,
    })
}

fn probe_fallback(options: &DriverOptions, identity: ProbeIdentity<'_>, reads: &[ProbeRead], reason: &str) -> i32 {
    let exit = compiler_status(&options.compiler, &options.arguments);
    receipt(&options.receipt, Receipt {
        schema: RECEIPT_SCHEMA,
        disposition: "fallback",
        reason,
        action_key: Some(identity.key),
        compiler_exit_code: exit,
        probe_reads: Some(reads),
    });
    exit
}

fn probe_read_uncertain(reads: &mut [ProbeRead]) -> bool {
    if let Some(attempt) = reads.last_mut()
        && attempt.disposition == "hit"
    {
        attempt.disposition = "invalid";
    }
    reads.last().is_some_and(|attempt| matches!(attempt.disposition, "invalid" | "unavailable"))
}

fn probe_cache_hits(
    options: &DriverOptions,
    classified: &Classified,
    identity: ProbeIdentity<'_>,
    failure_key: &str,
    reads: &mut Vec<ProbeRead>,
) -> Result<Option<i32>, i32> {
    assert!(reads.is_empty());
    assert!(identity.roots_count <= MAX_ROOTS);
    let failure = ProbeIdentity {
        key: failure_key,
        kind: "failure",
        ..identity
    };
    if let Some(exit) = probe_hit(options, classified, failure, reads) {
        return Ok(Some(exit));
    }
    if probe_read_uncertain(reads) {
        return Err(probe_fallback(options, identity, reads, "unknown-failure-manifest-or-read"));
    }
    if let Some(exit) = probe_hit(options, classified, identity, reads) {
        return Ok(Some(exit));
    }
    if probe_read_uncertain(reads) {
        return Err(probe_fallback(options, identity, reads, "unknown-object-manifest-or-read"));
    }
    Ok(None)
}

fn publish_ack(
    options: &DriverOptions,
    classified: &Classified,
    key: &str,
    dependencies: Vec<CcDependency>,
    bytes: &[u8],
) -> Result<(), String> {
    assert!(bytes.len() as u64 <= MAX_OBJECT);
    assert!(dependencies.len() <= MAX_DEPENDENCIES);
    let digest = blake3::hash(bytes).to_hex().to_string();
    let reply =
        request(&options.socket, wire(CcOperation::Publish, key, dependencies.clone(), Some(STANDARD.encode(bytes))))?;
    validate_cc_response(&reply, classified.roots.len()).map_err(|e| e.to_string())?;
    if reply.disposition != "published" {
        return Err("publication unacknowledged".into());
    }
    let record = reply.record.as_ref().ok_or("publication unacknowledged")?;
    if record.action_key != key || record.dependencies != dependencies {
        return Err("publication unacknowledged".into());
    }
    if record.object_digest_blake3 != digest || record.object_bytes != bytes.len() as u64 {
        return Err("publication unacknowledged".into());
    }
    assert!(record.object_bytes <= MAX_OBJECT);
    Ok(())
}

struct ProbeCompilation<'a> {
    options: &'a DriverOptions,
    classified: &'a Classified,
    input: &'a CcActionInput,
    identity: ProbeIdentity<'a>,
    failure_key: &'a str,
    exit: i32,
    diagnostics: Option<CompilerDiagnostics>,
    is_clean_depfile: bool,
    is_clean_object: bool,
}

fn probe_inputs_current(compilation: &ProbeCompilation<'_>) -> Result<(), String> {
    let script = compilation.options.probe_script.as_ref().ok_or("classified probe script missing")?;
    if digest_file(script, MAX_FILE)? != compilation.identity.script_digest {
        return Err("probe inputs changed".into());
    }
    let compiler = compilation.options.compiler.canonicalize().map_err(|e| e.to_string())?;
    if digest_file(&compiler, MAX_TREE)? != compilation.identity.tool_digest {
        return Err("probe inputs changed".into());
    }
    if tree_digests(compilation.classified, compilation.options)?.as_slice() != compilation.identity.roots {
        return Err("probe inputs changed".into());
    }
    Ok(())
}

fn publish_probe_failure(compilation: &ProbeCompilation<'_>) -> Result<(), String> {
    if !(1..=125).contains(&compilation.exit) || !compilation.is_clean_depfile {
        return Err("ineligible failure artifacts".into());
    }
    if !compilation.is_clean_object || compilation.classified.output.exists() {
        return Err("ineligible failure artifacts".into());
    }
    let (stdout, stderr) = compilation.diagnostics.as_ref().ok_or("unbounded diagnostics")?;
    assert!((1..=125).contains(&compilation.exit));
    assert!(stdout.len() <= MAX_CC_PROBE_DIAGNOSTIC_BYTES);
    assert!(stderr.len() <= MAX_CC_PROBE_DIAGNOSTIC_BYTES);
    probe_inputs_current(compilation)?;
    let dependencies = dependencies_from_depfile(compilation.classified)?;
    let payload = CcProbeResult {
        schema: CC_PROBE_RESULT_SCHEMA.into(),
        script_digest_blake3: compilation.identity.script_digest.into(),
        compiler_digest_blake3: compilation.input.tool_digest_blake3.clone(),
        source_path: compilation.classified.source.to_str().ok_or("source encoding")?.into(),
        output_path: compilation.classified.output.to_str().ok_or("output encoding")?.into(),
        depfile_path: compilation.classified.depfile.to_str().ok_or("depfile encoding")?.into(),
        compiler_exit_code: compilation.exit,
        stdout_base64: STANDARD.encode(stdout),
        stderr_base64: STANDARD.encode(stderr),
    };
    decode_cc_probe_result(&payload).map_err(|e| e.to_string())?;
    let bytes = serde_json::to_vec(&payload).map_err(|e| e.to_string())?;
    publish_ack(compilation.options, compilation.classified, compilation.failure_key, dependencies, &bytes)
}

fn publish_probe_object(compilation: &ProbeCompilation<'_>) -> Result<(), String> {
    if compilation
        .diagnostics
        .as_ref()
        .is_none_or(|(stdout, stderr)| !stdout.is_empty() || !stderr.is_empty())
    {
        return Err("nonempty or unbounded successful diagnostics".into());
    }
    probe_inputs_current(compilation)?;
    let dependencies = dependencies_from_depfile(compilation.classified)?;
    let bytes = file_bytes(&compilation.classified.output, MAX_OBJECT)?;
    publish_ack(compilation.options, compilation.classified, compilation.identity.key, dependencies, &bytes)
}

fn run_probe(
    options: &DriverOptions,
    classified: &Classified,
    input: &CcActionInput,
    identity: ProbeIdentity<'_>,
) -> i32 {
    let mut reads = Vec::with_capacity(4);
    let mut failure = input.clone();
    failure.normalized_arguments.push("probe-failure-result-v1".into());
    let failure_key = match cc_action_key(&failure) {
        Ok(key) => key,
        Err(_) => return fallback(options, "unbounded-or-unsafe-input", Some(identity.key)),
    };
    let Some(script) = options.probe_script.as_ref() else {
        return probe_fallback(options, identity, &reads, "probe-script-changed");
    };
    if digest_file(script, MAX_FILE).as_deref() != Ok(identity.script_digest) {
        return probe_fallback(options, identity, &reads, "probe-script-changed");
    }
    match probe_cache_hits(options, classified, identity, &failure_key, &mut reads) {
        Ok(Some(exit)) | Err(exit) => return exit,
        Ok(None) => {}
    }
    assert!(reads.len() <= 4);
    assert_eq!(identity.roots_count, classified.roots.len());
    let is_clean_depfile = !classified.depfile.exists();
    let is_clean_object = !classified.output.exists();
    let (exit, diagnostics) = compiler_with_diagnostics(options);
    let compilation = ProbeCompilation {
        options,
        classified,
        input,
        identity,
        failure_key: &failure_key,
        exit,
        diagnostics,
        is_clean_depfile,
        is_clean_object,
    };
    if exit != 0 {
        let is_published = publish_probe_failure(&compilation).is_ok();
        receipt(&options.receipt, Receipt {
            schema: RECEIPT_SCHEMA,
            disposition: if is_published {
                "probe-failure-published"
            } else {
                "probe-compiler-failure"
            },
            reason: "compiler-failure",
            action_key: Some(if is_published {
                failure_key.as_str()
            } else {
                identity.key
            }),
            compiler_exit_code: exit,
            probe_reads: Some(&reads),
        });
        return exit;
    }
    let is_published = publish_probe_object(&compilation).is_ok();
    receipt(&options.receipt, Receipt {
        schema: RECEIPT_SCHEMA,
        disposition: if is_published { "published" } else { "compiler-only" },
        reason: "compiled",
        action_key: Some(identity.key),
        compiler_exit_code: exit,
        probe_reads: Some(&reads),
    });
    exit
}

fn probe_environment_admitted(options: &DriverOptions, script: &Path) -> bool {
    if !script.is_absolute() {
        return false;
    }
    let script_utf8 = script.to_str();
    if script_utf8.is_none() {
        return false;
    }
    assert!(script.is_absolute());
    assert!(script_utf8.is_some());
    if script.canonicalize().as_deref().ok() != Some(script) {
        return false;
    }
    if !options.arguments.iter().any(|arg| arg == "-fdiagnostics-color=never")
        || !options.arguments.iter().any(|arg| arg == "-fmessage-length=0")
    {
        return false;
    }
    if std::env::var_os("LC_ALL").as_deref() != Some(OsStr::new("C")) {
        return false;
    }
    !std::env::vars_os().any(|(name, _)| {
        let name = name.to_string_lossy();
        if name.starts_with("LC_") && name != "LC_ALL" {
            return true;
        }
        name.starts_with("GCC_") || matches!(name.as_ref(), "LANGUAGE" | "DEPENDENCIES_OUTPUT" | "SUNPRO_DEPENDENCIES")
    })
}

fn action_facts(
    options: &DriverOptions,
    classified: &mut Classified,
    probe_digest: Option<&str>,
) -> Result<(String, CcActionInput), String> {
    let compiler_binary = options.compiler.canonicalize().map_err(|e| e.to_string())?;
    let tool = digest_file(&compiler_binary, MAX_TREE)?;
    let source = digest_file(&classified.source, MAX_FILE)?;
    let roots = tree_digests(classified, options)?;
    let name = options.compiler.file_name().and_then(|s| s.to_str()).ok_or("invalid compiler name")?;
    classified.normalized.push(format!("compiler:{name}"));
    if let Some(digest) = probe_digest {
        classified.normalized.push(format!("probe-script-digest:{digest}"));
    }
    let input = CcActionInput {
        tool_digest_blake3: tool,
        source_digest_blake3: source,
        platform_digest_blake3: options.platform_digest.clone(),
        normalized_arguments: std::mem::take(&mut classified.normalized),
        roots_digest_blake3: roots,
    };
    let key = cc_action_key(&input).map_err(|e| e.to_string())?;
    assert!(input.roots_digest_blake3.len() <= MAX_ROOTS);
    assert!(input.normalized_arguments.len() <= options.arguments.len());
    Ok((key, input))
}

fn try_normal_hit(
    options: &DriverOptions,
    classified: &Classified,
    key: &str,
    roots: &[String],
    manifest: &CcWireResponse,
) -> bool {
    let Some(record) = manifest.record.as_ref() else {
        return false;
    };
    if record.action_key != key || record.dependencies.is_empty() {
        return false;
    }
    let Ok(current) = dependencies_from_record(classified, &record.dependencies) else {
        return false;
    };
    if admit_cc_reuse(record, &current, classified.roots.len()) != Ok(true) {
        return false;
    }
    if tree_digests(classified, options).as_deref() != Ok(roots) {
        return false;
    }
    let Ok(reply) = request(&options.socket, wire(CcOperation::Read, key, current.clone(), None)) else {
        return false;
    };
    if reply.disposition != "hit" {
        return false;
    }
    let (Some(read_record), Some(encoded)) = (&reply.record, &reply.object_base64) else {
        return false;
    };
    if read_record != record || admit_cc_reuse(read_record, &current, classified.roots.len()) != Ok(true) {
        return false;
    }
    let Ok(bytes) = STANDARD.decode(encoded) else {
        return false;
    };
    if bytes.len() as u64 != record.object_bytes || bytes.len() as u64 > MAX_OBJECT {
        return false;
    }
    assert!(bytes.len() as u64 <= MAX_OBJECT);
    assert!(current.len() <= MAX_DEPENDENCIES);
    if blake3::hash(&bytes).to_hex().as_str() != record.object_digest_blake3 {
        return false;
    }
    if tree_digests(classified, options).as_deref() != Ok(roots) {
        return false;
    }
    if restore(classified, &current, &bytes).is_err() {
        return false;
    }
    receipt(&options.receipt, Receipt {
        schema: RECEIPT_SCHEMA,
        disposition: "hit",
        reason: "verified-object-and-dependencies",
        action_key: Some(key),
        compiler_exit_code: 0,
        probe_reads: None,
    });
    true
}

fn compile_normal(options: &DriverOptions, classified: &Classified, key: &str, roots: &[String]) -> i32 {
    let exit = compiler_status(&options.compiler, &options.arguments);
    if exit != 0 {
        receipt(&options.receipt, Receipt {
            schema: RECEIPT_SCHEMA,
            disposition: "compiler-failure",
            reason: "not-cacheable",
            action_key: Some(key),
            compiler_exit_code: exit,
            probe_reads: None,
        });
        return exit;
    }
    assert_eq!(exit, 0);
    assert!(classified.roots.len() <= MAX_ROOTS);
    let publish = (|| -> Result<(), String> {
        if tree_digests(classified, options)?.as_slice() != roots {
            return Err("source roots changed during compilation".into());
        }
        let dependencies = dependencies_from_depfile(classified)?;
        let bytes = file_bytes(&classified.output, MAX_OBJECT)?;
        publish_ack(options, classified, key, dependencies, &bytes)
    })();
    let (disposition, reason) = if publish.is_ok() {
        ("published", "compiled-and-published")
    } else {
        ("compiler-only", "publication-ineligible-or-unavailable")
    };
    receipt(&options.receipt, Receipt {
        schema: RECEIPT_SCHEMA,
        disposition,
        reason,
        action_key: Some(key),
        compiler_exit_code: exit,
        probe_reads: None,
    });
    exit
}

fn has_ambient_compiler_input() -> bool {
    [
        "CPATH",
        "C_INCLUDE_PATH",
        "CPLUS_INCLUDE_PATH",
        "OBJC_INCLUDE_PATH",
        "GCC_EXEC_PREFIX",
        "COMPILER_PATH",
        "SOURCE_DATE_EPOCH",
    ]
    .iter()
    .any(|name| std::env::var_os(name).is_some())
}

/// Run one compiler invocation; the integer returned is the compiler-compatible process exit code.
pub fn run(options: &DriverOptions) -> i32 {
    if !options.compiler.is_absolute() {
        receipt(&options.receipt, Receipt {
            schema: RECEIPT_SCHEMA,
            disposition: "rejected",
            reason: "compiler-must-be-absolute",
            action_key: None,
            compiler_exit_code: 2,
            probe_reads: None,
        });
        return 2;
    }
    if !options.socket.is_absolute() || !options.receipt.is_absolute() {
        return fallback(options, "invalid-driver-options", None);
    }
    let digest = options.platform_digest.as_bytes();
    if digest.len() != 64 || !digest.iter().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()) {
        return fallback(options, "invalid-driver-options", None);
    }
    if std::env::var_os("MANTLE_CC_CACHE_DISABLE").is_some() {
        return fallback(options, "disabled", None);
    }
    let mut classified = match classify(&options.arguments) {
        Ok(value) => value,
        Err(_) => return fallback(options, "unclassified-invocation", None),
    };
    assert!(classified.roots.len() <= MAX_ROOTS);
    assert!(options.arguments.len() <= MAX_ARGS);
    if !gnu_compiler(&options.compiler) {
        return fallback(options, "unclassified-compiler-family", None);
    }
    let probe_digest = if let Some(script) = &options.probe_script {
        if !probe_environment_admitted(options, script) {
            return fallback(options, "unclassified-probe-environment", None);
        }
        match digest_file(script, MAX_FILE) {
            Ok(digest) => Some(digest),
            Err(_) => return fallback(options, "unbounded-or-unsafe-probe-script", None),
        }
    } else {
        None
    };
    if has_ambient_compiler_input() {
        return fallback(options, "ambient-compiler-input", None);
    }
    let (key, input) = match action_facts(options, &mut classified, probe_digest.as_deref()) {
        Ok(facts) => facts,
        Err(_) => return fallback(options, "unbounded-or-unsafe-input", None),
    };
    if let Some(script_digest) = probe_digest.as_deref() {
        let identity = ProbeIdentity {
            key: &key,
            roots: &input.roots_digest_blake3,
            kind: "object",
            script_digest,
            tool_digest: &input.tool_digest_blake3,
            roots_count: classified.roots.len(),
        };
        return run_probe(options, &classified, &input, identity);
    }
    run_normal(options, &classified, &key, &input.roots_digest_blake3)
}

fn run_normal(options: &DriverOptions, classified: &Classified, key: &str, roots: &[String]) -> i32 {
    let manifest = match request(&options.socket, wire(CcOperation::Manifest, key, Vec::new(), None)) {
        Ok(reply) => reply,
        Err(_) => return fallback(options, "cache-unavailable", Some(key)),
    };
    if validate_cc_response(&manifest, classified.roots.len()).is_err() {
        return fallback(options, "invalid-cache-response", Some(key));
    }
    if manifest.disposition == "hit" {
        if try_normal_hit(options, classified, key, roots, &manifest) {
            return 0;
        }
    } else if manifest.disposition != "miss" || manifest.record.is_some() || manifest.object_base64.is_some() {
        return fallback(options, "invalid-manifest-disposition", Some(key));
    }
    compile_normal(options, classified, key, roots)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_escaped_spaces_backslashes_and_continuations() {
        assert_eq!(parse_depfile(b"out\\ file.o: /src/a\\ b.c /src/a\\\\b.h \\\n /src/third.h\n").unwrap(), vec![
            "/src/a b.c",
            "/src/a\\b.h",
            "/src/third.h"
        ]);
        assert!(parse_depfile(b"x.o: /src/a.c \\").is_err());
        assert!(parse_depfile(b"x.o: /src/a.c\nother.o: /src/b.c\n").is_err());
    }
    #[test]
    fn classifies_only_remapped_bounded_compilations() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let source = root.join("source.c");
        fs::write(&source, "int answer(void) { return 42; }").unwrap();
        let flags = vec![
            "-c".into(),
            source.display().to_string(),
            "-o".into(),
            root.join("source.o").display().to_string(),
            "-MMD".into(),
            "-MF".into(),
            root.join("source.d").display().to_string(),
            "-nostdinc".into(),
            "-w".into(),
            format!("-ffile-prefix-map={}=/cc-root-0", root.display()),
        ];
        assert!(classify(&flags).is_ok());
        for bad in ["@args", "-E", "-include", "-g", "-fplugin=evil", "-shared"] {
            let mut args = flags.clone();
            args.push(bad.into());
            assert!(classify(&args).is_err(), "accepted {bad}");
        }
        let mut relative_object = flags.clone();
        relative_object[3] = "relative.o".into();
        assert!(classify(&relative_object).is_err(), "relative object changes GCC dep target");
        let mut relative_depfile = flags.clone();
        relative_depfile[6] = "relative.d".into();
        assert!(classify(&relative_depfile).is_err(), "relative -MF is not a stable artifact path");
        let mut unmapped = flags;
        unmapped.pop();
        assert!(classify(&unmapped).is_err());
    }
    #[test]
    fn restores_real_gcc_dependency_order_and_wraps_for_c_and_cpp() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        for (compiler, extension) in [("gcc", "c"), ("g++", "cpp")] {
            let source = root.join(format!("long_source_filename_for_depfile_order.{extension}"));
            fs::write(
                &source,
                "#include \"zebra.h\"\n#include \"alpha.h\"\nint answer(void) { return ZEBRA + ALPHA; }\n",
            )
            .unwrap();
            fs::write(root.join("zebra.h"), "#define ZEBRA 20\n").unwrap();
            fs::write(root.join("alpha.h"), "#define ALPHA 22\n").unwrap();
            let object = root.join(format!("long_target_object_filename_for_depfile_order_{extension}.o"));
            let depfile = root.join(format!("{extension}.d"));
            let flags = vec![
                "-c".into(),
                source.display().to_string(),
                "-o".into(),
                object.display().to_string(),
                "-MMD".into(),
                "-MF".into(),
                depfile.display().to_string(),
                "-nostdinc".into(),
                "-w".into(),
                format!("-ffile-prefix-map={}=/cc-root-0", root.display()),
            ];
            let status = Command::new(compiler).args(&flags).status().unwrap();
            assert!(status.success(), "{compiler} failed");
            let classified = classify(&flags).unwrap();
            let dependencies = dependencies_from_depfile(&classified).unwrap();
            let names: Vec<_> = dependencies.iter().map(|item| item.relative_path.as_str()).collect();
            assert_eq!(names, [source.file_name().unwrap().to_str().unwrap(), "zebra.h", "alpha.h"]);
            let real = fs::read(&depfile).unwrap();
            assert_eq!(real, serialized_depfile(&classified, &dependencies).unwrap().as_bytes());
            assert!(real.windows(3).any(|bytes| bytes == b" \\\n"), "expected GCC wrap");
        }
    }
}
