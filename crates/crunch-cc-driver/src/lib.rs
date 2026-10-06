//! Conservative local C/C++ object cache client. Every uncertain invocation runs the real compiler.
use base64::{engine::general_purpose::STANDARD, Engine as _};
use crunch_rust_cache_core::cc::{
    admit_cc_reuse, cc_action_key, decode_cc_probe_result, validate_cc_response, CcActionInput,
    CcDependency, CcOperation, CcProbeResult, CcWireRequest, CcWireResponse,
    CC_PROBE_RESULT_SCHEMA, CC_REQUEST_SCHEMA, CC_RESPONSE_SCHEMA,
    MAX_CC_DEPENDENCIES as MAX_DEPENDENCIES, MAX_CC_FRAME_BYTES as MAX_FRAME,
    MAX_CC_OBJECT_BYTES as MAX_OBJECT, MAX_CC_PROBE_DIAGNOSTIC_BYTES, MAX_CC_ROOTS as MAX_ROOTS,
};
use serde::Serialize;
use std::{collections::{BTreeMap, BTreeSet}, ffi::{OsStr, OsString}, fs::{self, File}, io::{Read, Write},
    os::unix::{fs::{FileTypeExt, PermissionsExt}, net::UnixStream, process::ExitStatusExt}, path::{Path, PathBuf},
    process::{Command, Stdio}, thread, time::Duration};

const MAX_DEPFILE: u64 = 1024 * 1024;
const MAX_FILE: u64 = 32 * 1024 * 1024;
const MAX_TREE: u64 = 128 * 1024 * 1024;
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

fn receipt(path: &Path, disposition: &str, reason: &str, action_key: Option<&str>, exit: i32) {
    let document = Receipt { schema: "mantle-cc-driver-receipt-v1", disposition, reason, action_key, compiler_exit_code: exit, probe_reads: None };
    let result = (|| -> Result<(), String> {
        let bytes = serde_json::to_vec(&document).map_err(|e| e.to_string())?;
        if bytes.len() > MAX_RECEIPT { return Err("receipt too large".into()); }
        let parent = path.parent().ok_or("receipt has no parent")?;
        let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
        temporary.write_all(&bytes).map_err(|e| e.to_string())?;
        temporary.as_file().sync_all().map_err(|e| e.to_string())?;
        temporary.persist(path).map_err(|e| e.to_string())?;
        Ok(())
    })();
    if let Err(error) = result { eprintln!("mantle-cc-cache-driver: cannot record receipt: {error}"); }
}
fn probe_receipt(options: &DriverOptions, disposition: &str, reason: &str, key: &str, exit: i32, reads: &[ProbeRead]) {
    let document = Receipt { schema: "mantle-cc-driver-receipt-v1", disposition, reason,
        action_key: Some(key), compiler_exit_code: exit, probe_reads: Some(reads) };
    let result = (|| -> Result<(), String> {
        let bytes = serde_json::to_vec(&document).map_err(|e| e.to_string())?;
        if bytes.len() > MAX_RECEIPT { return Err("receipt too large".into()); }
        let parent = options.receipt.parent().ok_or("receipt has no parent")?;
        let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
        temporary.write_all(&bytes).map_err(|e| e.to_string())?;
        temporary.as_file().sync_all().map_err(|e| e.to_string())?;
        temporary.persist(&options.receipt).map_err(|e| e.to_string())?;
        Ok(())
    })();
    if let Err(error) = result { eprintln!("mantle-cc-cache-driver: cannot record receipt: {error}"); }
}

fn compiler_status<I, S>(compiler: &Path, arguments: I) -> i32
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    match Command::new(compiler).args(arguments).status() {
        Ok(status) => status.code().unwrap_or_else(|| 128 + status.signal().unwrap_or(1)),
        Err(error) => { eprintln!("mantle-cc-cache-driver: cannot execute compiler: {error}"); 127 }
    }
}

fn fallback(options: &DriverOptions, reason: &str, action_key: Option<&str>) -> i32 {
    let exit = compiler_status(&options.compiler, &options.arguments);
    receipt(&options.receipt, "fallback", reason, action_key, exit);
    exit
}

/// Preserve arbitrary Unix compiler arguments byte-for-byte when classification is impossible.
pub fn run_os(compiler: PathBuf, socket: PathBuf, receipt_path: PathBuf, platform_digest: String, probe_script: Option<PathBuf>, arguments: Vec<OsString>) -> i32 {
    if !compiler.is_absolute() {
        receipt(&receipt_path, "rejected", "compiler-must-be-absolute", None, 2);
        return 2;
    }
    let Some(arguments_utf8) = arguments.iter().map(|arg| arg.to_str().map(str::to_owned)).collect::<Option<Vec<_>>>() else {
        let exit = compiler_status(&compiler, &arguments);
        receipt(&receipt_path, "fallback", "non-utf8-compiler-argument", None, exit);
        return exit;
    };
    run(&DriverOptions { compiler, socket, receipt: receipt_path, platform_digest, arguments: arguments_utf8, probe_script })
}

fn digest_file(path: &Path, bound: u64) -> Result<String, String> {
    let metadata = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if !metadata.file_type().is_file() || metadata.len() > bound { return Err("file is nonregular or exceeds bound".into()); }
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut hash = blake3::Hasher::new();
    let mut buffer = [0; 16384];
    let mut total = 0_u64;
    loop {
        let count = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if count == 0 { break; }
        total = total.checked_add(count as u64).ok_or("file length overflow")?;
        if total > bound { return Err("file exceeds bound".into()); }
        hash.update(&buffer[..count]);
    }
    Ok(hash.finalize().to_hex().to_string())
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
    if !raw.is_absolute() { return Err("source and include roots must be absolute".into()); }
    let canonical = raw.canonicalize().map_err(|e| e.to_string())?;
    if canonical != raw || !fs::symlink_metadata(raw).map_err(|e| e.to_string())?.is_file() {
        return Err("source must be canonical regular file".into());
    }
    Ok(canonical)
}

fn absolute_dir(path: &str) -> Result<PathBuf, String> {
    let raw = Path::new(path);
    if !raw.is_absolute() { return Err("include root must be absolute".into()); }
    let canonical = raw.canonicalize().map_err(|e| e.to_string())?;
    if canonical != raw || !fs::symlink_metadata(raw).map_err(|e| e.to_string())?.is_dir() {
        return Err("include root must be canonical directory".into());
    }
    Ok(canonical)
}

fn output_path(path: &str) -> Result<PathBuf, String> {
    let raw = Path::new(path);
    if !raw.is_absolute() { return Err("output and depfile must be absolute".into()); }
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

/// Classify only an explicit dependency-producing object compilation, without implicit includes.
fn classify(args: &[String]) -> Result<Classified, String> {
    if args.len() > MAX_ARGS || args.iter().any(|arg| arg.is_empty() || arg.len() > MAX_ARG_BYTES || arg.contains('\0')) {
        return Err("argument bound or encoding".into());
    }
    let (mut source, mut output, mut depfile) = (None, None, None);
    let (mut compile, mut deps, mut nostdinc, mut suppress_warnings) = (false, false, false, false);
    let mut includes: Vec<(String, PathBuf)> = Vec::new();
    let mut maps: Vec<(PathBuf, String)> = Vec::new();
    let mut normalized = Vec::new();
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        // Values are consumed without reordering or modifying the compiler's argv.
        match arg.as_str() {
            "-c" if !compile => compile = true,
            "-MMD" | "-MD" if !deps => { deps = true; normalized.push(arg.clone()); },
            "-nostdinc" if !nostdinc => { nostdinc = true; normalized.push(arg.clone()); },
            "-nostdinc++" => normalized.push(arg.clone()),
            "-w" if !suppress_warnings => { suppress_warnings = true; normalized.push(arg.clone()); },
            "-fdiagnostics-color=never" | "-fmessage-length=0" => normalized.push(arg.clone()),
            "-o" if output.is_none() => output = Some(output_path(next_argument(args, &mut index)?)?),
            "-MF" if depfile.is_none() => depfile = Some(output_path(next_argument(args, &mut index)?)?),
            "-I" | "-isystem" => {
                let root = absolute_dir(next_argument(args, &mut index)?)?;
                includes.push((arg.clone(), root));
            }
            _ if arg.starts_with("-I") && arg.len() > 2 => includes.push(("-I".into(), absolute_dir(&arg[2..])?)),
            _ if arg.starts_with("-ffile-prefix-map=") => {
                let value = &arg[18..];
                let (from, to) = value.split_once('=').ok_or("invalid prefix map")?;
                if to.is_empty() || to.contains('\\') || to.contains("/nix/store") || to.contains("/mantle/store") {
                    return Err("unsafe prefix map destination".into());
                }
                maps.push((absolute_dir(from)?, to.into()));
            }
            "-O0" | "-O1" | "-O2" | "-O3" | "-Os" | "-Og" | "-Oz" | "-fPIC" | "-fpic" => normalized.push(arg.clone()),
            _ if arg.starts_with("-std=") && arg[5..].bytes().all(|b| b.is_ascii_alphanumeric() || b == b'+') => normalized.push(arg.clone()),
            _ if (arg.starts_with("-D") || arg.starts_with("-U")) && arg.len() > 2
                && !["__DATE__", "__TIME__", "__TIMESTAMP__"].iter().any(|name| arg.contains(name))
                && arg[2..].bytes().all(|b| b.is_ascii_alphanumeric() || b"_=.()+-*".contains(&b)) => normalized.push(arg.clone()),
            _ if !arg.starts_with('-') && source.is_none() => source = Some(absolute_file(arg)?),
            _ => return Err("unknown compiler argument or side effect".into()),
        }
        index += 1;
    }
    if !compile || !deps || !nostdinc || !suppress_warnings { return Err("required -c, -MD/-MMD, -nostdinc, -w".into()); }
    let source = source.ok_or("missing source")?;
    if !matches!(source.extension().and_then(|e| e.to_str()), Some("c" | "cc" | "cpp" | "cxx")) {
        return Err("unsupported source extension".into());
    }
    let output = output.ok_or("missing -o")?;
    let depfile = depfile.ok_or("missing -MF")?;
    if output == depfile || output == source || depfile == source { return Err("overlapping source and outputs".into()); }
    if !safe_make_path(&output) || !safe_make_path(&depfile) || !safe_make_path(&source) {
        return Err("exotic make path".into());
    }
    if output.extension().and_then(|e| e.to_str()) != Some("o") { return Err("expected .o output".into()); }
    let mut roots = vec![source.parent().ok_or("missing source parent")?.to_path_buf()];
    for (_, root) in &includes {
        if roots.contains(root) { return Err("duplicate include root".into()); }
        roots.push(root.clone());
    }
    if roots.len() > MAX_ROOTS { return Err("too many roots".into()); }
    for (index, root) in roots.iter().enumerate() {
        if roots.iter().skip(index + 1).any(|other| root.starts_with(other) || other.starts_with(root)) {
            return Err("overlapping roots make prefix remapping ambiguous".into());
        }
    }
    if maps.len() != roots.len() { return Err("one explicit -ffile-prefix-map per root required".into()); }
    for (index, root) in roots.iter().enumerate() {
        let expected = format!("/cc-root-{index}");
        if maps.iter().filter(|(from, to)| from == root && to == &expected).count() != 1 {
            return Err("missing stable source/include path remapping".into());
        }
    }
    for (kind, root) in &includes {
        let index = roots.iter().position(|item| item == root).ok_or("unmapped include root")?;
        normalized.push(format!("{kind}@root:{index}"));
    }
    for index in 0..roots.len() { normalized.push(format!("map:@root:{index}:cc-root-{index}")); }
    normalized.push(format!("source:@root:0/{}", source.file_name().ok_or("missing filename")?.to_string_lossy()));
    // Normalize semantic switches in their original order instead of normalizing away -I precedence.
    // Input root tree identities still bind every physical path's complete content.
    Ok(Classified { source, output, depfile, roots, normalized })
}

fn file_bytes(path: &Path, bound: u64) -> Result<Vec<u8>, String> {
    let metadata = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if !metadata.is_file() || metadata.len() > bound { return Err("file exceeds bound or is nonregular".into()); }
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mut data = Vec::new();
    file.take(bound + 1).read_to_end(&mut data).map_err(|e| e.to_string())?;
    if data.len() as u64 > bound { return Err("file exceeds bound".into()); }
    Ok(data)
}

fn scan_root_file(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0_u8; 16384];
    let mut inspection = [0_u8; 16384 + 14];
    let mut trailing = 0_usize;
    let mut total = 0_u64;
    loop {
        let count = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if count == 0 { break; }
        total = total.checked_add(count as u64).ok_or("root file size overflow")?;
        if total > MAX_FILE { return Err("root file exceeds bound".into()); }
        hasher.update(&buffer[..count]);
        inspection[trailing..trailing + count].copy_from_slice(&buffer[..count]);
        let examined = &inspection[..trailing + count];
        if examined.windows(2).any(|window| window == b"##")
            || [b"__DATE__".as_slice(), b"__TIME__", b"__TIMESTAMP__"].iter()
                .any(|needle| examined.windows(needle.len()).any(|window| window == *needle)) {
            return Err("time-dependent or dynamically synthesized compiler macro".into());
        }
        let length = trailing + count;
        trailing = length.min(14);
        inspection.copy_within(length - trailing..length, 0);
    }
    Ok(hasher.finalize().to_hex().to_string())
}

fn tree_digests(classified: &Classified, options: &DriverOptions) -> Result<Vec<String>, String> {
    let mut total = 0_u64;
    let mut entries_seen = 0_usize;
    let mut result = Vec::new();
    for root in &classified.roots {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"mantle.cc.complete-root.v1\0");
        let mut pending = vec![root.clone()];
        let mut entries = BTreeMap::new();
        while let Some(directory) = pending.pop() {
            for item in fs::read_dir(directory).map_err(|e| e.to_string())? {
                let entry = item.map_err(|e| e.to_string())?;
                let path = entry.path();
                entries_seen += 1;
                if entries_seen > MAX_FILES || path.components().count() > root.components().count() + 128 {
                    return Err("root entries or depth exceeded".into());
                }
                if path == options.receipt || path == options.socket { continue; }
                let metadata = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
                if metadata.file_type().is_symlink() || metadata.file_type().is_socket() || metadata.file_type().is_fifo() {
                    return Err("unsafe root entry".into());
                }
                if metadata.is_dir() { pending.push(path); continue; }
                if !metadata.is_file() { return Err("nonregular root entry".into()); }
                if path == classified.output || path == classified.depfile { continue; }
                total = total.checked_add(metadata.len()).ok_or("root size overflow")?;
                if total > MAX_TREE || metadata.len() > MAX_FILE { return Err("root bound exceeded".into()); }
                let relative = path.strip_prefix(root).map_err(|e| e.to_string())?.to_str().ok_or("non-UTF8 root path")?;
                if relative.contains('\\') || relative.len() > 4096 { return Err("unsupported root filename".into()); }
                // Read once while checking macro safety and hashing the complete file.
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

/// Parse a single make-style depfile with backslash-escaped whitespace and continuations.
/// Unknown constructs fail closed rather than publishing an incomplete manifest.
pub fn parse_depfile(bytes: &[u8]) -> Result<Vec<String>, String> {
    if bytes.is_empty() || bytes.len() as u64 > MAX_DEPFILE || bytes.contains(&0) { return Err("invalid depfile length".into()); }
    let mut colon = None;
    let mut escaped = false;
    for (index, byte) in bytes.iter().enumerate() {
        if escaped { escaped = false; continue; }
        if *byte == b'\\' { escaped = true; continue; }
        if *byte == b':' { colon = Some(index); break; }
        if *byte == b'\n' { return Err("depfile has no target".into()); }
    }
    let colon = colon.ok_or("depfile missing target separator")?;
    if colon == 0 { return Err("depfile empty target".into()); }
    let mut paths = Vec::new();
    let mut token = Vec::new();
    let mut index = colon + 1;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte == b'\\' {
            index += 1;
            if index == bytes.len() { return Err("unterminated depfile escape".into()); }
            if bytes[index] == b'\r' && bytes.get(index + 1) == Some(&b'\n') { index += 2; continue; }
            if bytes[index] == b'\n' { index += 1; continue; }
            token.push(bytes[index]);
        } else if byte.is_ascii_whitespace() {
            if !token.is_empty() {
                paths.push(String::from_utf8(std::mem::take(&mut token)).map_err(|e| e.to_string())?);
            }
        } else if byte == b':' || byte == b'#' || byte == b'$' {
            return Err("unsupported make dependency syntax".into());
        } else { token.push(byte); }
        if token.len() > 4096 || paths.len() > MAX_DEPENDENCIES { return Err("depfile bound exceeded".into()); }
        index += 1;
    }
    if !token.is_empty() { paths.push(String::from_utf8(token).map_err(|e| e.to_string())?); }
    if paths.is_empty() { return Err("depfile lacks prerequisites".into()); }
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
            if value.is_empty() || value.len() > 4096 || value.contains('\\') || value.split('/').any(|c| c == "." || c == ".." || c.is_empty()) {
                return Err("unsafe dependency label".into());
            }
            if canonical == classified.output || canonical == classified.depfile { return Err("dependency is output".into()); }
            return Ok(CcDependency { root_index: index as u32, relative_path: value.into(), digest_blake3: digest_file(&canonical, MAX_FILE)? });
        }
    }
    Err("dependency outside declared roots".into())
}

fn dependencies_from_depfile(classified: &Classified) -> Result<Vec<CcDependency>, String> {
    let data = file_bytes(&classified.depfile, MAX_DEPFILE)?;
    let paths = parse_depfile(&data)?;
    let mut dependencies = Vec::new();
    let mut seen = BTreeSet::new();
    for name in paths {
        let path = Path::new(&name);
        if !path.is_absolute() || !safe_make_path(path) { return Err("nonliteral dependency path".into()); }
        let entry = dependency(path, classified)?;
        if path != classified.roots[entry.root_index as usize].join(&entry.relative_path)
            || !seen.insert((entry.root_index, entry.relative_path.clone())) {
            return Err("noncanonical or duplicate dependency".into());
        }
        dependencies.push(entry);
    }
    let source = dependency(&classified.source, classified)?;
    if !seen.contains(&(source.root_index, source.relative_path)) { return Err("depfile omits source".into()); }
    if data != serialized_depfile(classified, &dependencies)?.as_bytes() { return Err("unknown depfile formatting".into()); }
    Ok(dependencies)
}

fn dependencies_from_record(classified: &Classified, record: &[CcDependency]) -> Result<Vec<CcDependency>, String> {
    let mut result = Vec::new();
    for entry in record {
        let root = classified.roots.get(entry.root_index as usize).ok_or("unknown root index")?;
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
    let timeout = Some(Duration::from_secs(3));
    stream.set_read_timeout(timeout).map_err(|e| e.to_string())?;
    stream.set_write_timeout(timeout).map_err(|e| e.to_string())?;
    Ok(stream)
}

fn request(socket: &Path, wire: CcWireRequest) -> Result<CcWireResponse, String> {
    let mut stream = connect(socket)?;
    let bytes = serde_json::to_vec(&wire).map_err(|e| e.to_string())?;
    if bytes.is_empty() || bytes.len() as u64 > MAX_FRAME { return Err("request frame bound exceeded".into()); }
    stream.write_all(&(bytes.len() as u64).to_be_bytes()).map_err(|e| e.to_string())?;
    stream.write_all(&bytes).map_err(|e| e.to_string())?;
    let mut prefix = [0; 8];
    stream.read_exact(&mut prefix).map_err(|e| e.to_string())?;
    let count = u64::from_be_bytes(prefix);
    if count == 0 || count > MAX_FRAME { return Err("response frame bound exceeded".into()); }
    let mut body = vec![0; count as usize];
    stream.read_exact(&mut body).map_err(|e| e.to_string())?;
    let response: CcWireResponse = serde_json::from_slice(&body).map_err(|e| e.to_string())?;
    if response.schema != CC_RESPONSE_SCHEMA || response.action_key != wire.action_key {
        return Err("response identity mismatch".into());
    }
    Ok(response)
}

fn wire(operation: CcOperation, key: &str, dependencies: Vec<CcDependency>, object_base64: Option<String>) -> CcWireRequest {
    CcWireRequest { schema: CC_REQUEST_SCHEMA.into(), operation, action_key: key.into(), dependencies, object_base64 }
}

fn safe_make_path(path: &Path) -> bool {
    path.to_str().is_some_and(|text| !text.is_empty() && text.bytes().all(|b|
        b.is_ascii_alphanumeric() || b"/._-+".contains(&b)))
}

fn serialized_depfile(classified: &Classified, dependencies: &[CcDependency]) -> Result<String, String> {
    if !safe_make_path(&classified.output) { return Err("unsafe make target".into()); }
    let mut dep = format!("{}:", classified.output.display());
    let mut column = dep.len();
    for entry in dependencies {
        let root = classified.roots.get(entry.root_index as usize).ok_or("unknown dep root")?;
        let path = root.join(&entry.relative_path);
        if !safe_make_path(&path) { return Err("unsafe make prerequisite".into()); }
        let text = path.to_str().ok_or("non-UTF8 prerequisite")?;
        if column + text.len() > 72 {
            dep.push_str(" \\\n");
            column = 0;
        }
        dep.push(' ');
        dep.push_str(text);
        column += 1 + text.len();
    }
    dep.push('\n');
    if dep.len() as u64 > MAX_DEPFILE { return Err("restored depfile exceeds bound".into()); }
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
    if object.len() as u64 > MAX_OBJECT { return Err("oversized restored object".into()); }
    let dep = serialized_depfile(classified, dependencies)?;
    // Stage both files before publishing either. A failed restoration never
    // commits a cached object before the fallback compiler has run.
    let dep_parent = classified.depfile.parent().ok_or("depfile parent missing")?;
    let object_parent = classified.output.parent().ok_or("object parent missing")?;
    let mut staged_dep = tempfile::NamedTempFile::new_in(dep_parent).map_err(|e| e.to_string())?;
    let mut staged_object = tempfile::NamedTempFile::new_in(object_parent).map_err(|e| e.to_string())?;
    staged_dep.write_all(dep.as_bytes()).map_err(|e| e.to_string())?;
    staged_object.write_all(object).map_err(|e| e.to_string())?;
    staged_dep.as_file().set_permissions(artifact_permissions(&classified.depfile)?).map_err(|e| e.to_string())?;
    staged_object.as_file().set_permissions(artifact_permissions(&classified.output)?).map_err(|e| e.to_string())?;
    staged_dep.as_file().sync_all().map_err(|e| e.to_string())?;
    staged_object.as_file().sync_all().map_err(|e| e.to_string())?;
    staged_dep.persist(&classified.depfile).map_err(|e| e.to_string())?;
    staged_object.persist(&classified.output).map_err(|e| e.to_string())?;
    Ok(())
}

fn gnu_compiler(compiler: &Path) -> bool {
    let Ok(canonical) = compiler.canonicalize() else { return false };
    let Some(name) = canonical.file_name().and_then(|s| s.to_str()) else { return false };
    let stem = name.split_once("-gcc").map(|(prefix, suffix)| (!prefix.is_empty(), suffix))
        .or_else(|| name.split_once("-g++").map(|(prefix, suffix)| (!prefix.is_empty(), suffix)));
    matches!(name, "gcc" | "g++")
        || stem.is_some_and(|(prefix, suffix)| prefix && (suffix.is_empty()
            || suffix.strip_prefix('-').is_some_and(|version| !version.is_empty()
                && version.bytes().all(|b| b.is_ascii_digit() || b == b'.'))))
        || ["gcc-", "g++-"].iter().any(|prefix| name.strip_prefix(prefix).is_some_and(|version|
            !version.is_empty() && version.bytes().all(|b| b.is_ascii_digit() || b == b'.')))
}

fn probe_attempt(options: &DriverOptions, key: &str, kind: &'static str, operation: CcOperation,
    dependencies: Vec<CcDependency>, reads: &mut Vec<ProbeRead>, roots_count: usize) -> Option<CcWireResponse> {
    let label = if operation == CcOperation::Manifest { "manifest" } else { "read" };
    let response = request(&options.socket, wire(operation, key, dependencies, None));
    let (result, disposition) = match response {
        Ok(reply) if validate_cc_response(&reply, roots_count).is_ok()
            && (reply.disposition == "hit" || reply.disposition == "miss") => {
                let label = if reply.disposition == "hit" { "hit" } else { "miss" };
                (Some(reply), label)
            }
        Ok(_) => (None, "invalid"),
        Err(_) => (None, "unavailable"),
    };
    reads.push(ProbeRead { kind, operation: label, disposition });
    result
}

fn capture_diagnostic<R: Read, W: Write>(mut input: R, mut output: W) -> (Vec<u8>, bool) {
    let mut captured = Vec::new();
    let mut complete = true;
    let mut buffer = [0; 8192];
    loop {
        match input.read(&mut buffer) {
            Ok(0) => break,
            Ok(count) => {
                if output.write_all(&buffer[..count]).is_err() { complete = false; }
                if captured.len() + count <= MAX_CC_PROBE_DIAGNOSTIC_BYTES {
                    captured.extend_from_slice(&buffer[..count]);
                } else { complete = false; }
            }
            Err(_) => { complete = false; break; }
        }
    }
    (captured, complete)
}

fn compiler_with_diagnostics(options: &DriverOptions) -> (i32, Option<(Vec<u8>, Vec<u8>)>) {
    let mut child = match Command::new(&options.compiler).args(&options.arguments)
        .stdout(Stdio::piped()).stderr(Stdio::piped()).spawn() {
        Ok(child) => child,
        Err(error) => {
            eprintln!("mantle-cc-cache-driver: cannot execute compiler: {error}");
            return (127, None);
        }
    };
    let stdout = child.stdout.take().expect("piped compiler stdout");
    let stderr = child.stderr.take().expect("piped compiler stderr");
    let out = thread::spawn(move || capture_diagnostic(stdout, std::io::stdout().lock()));
    let err = thread::spawn(move || capture_diagnostic(stderr, std::io::stderr().lock()));
    let status = child.wait();
    let out = out.join();
    let err = err.join();
    let exit = status.map(|status| status.code().unwrap_or_else(|| 128 + status.signal().unwrap_or(1))).unwrap_or(127);
    match (out, err) {
        (Ok((stdout, true)), Ok((stderr, true))) => (exit, Some((stdout, stderr))),
        _ => (exit, None),
    }
}

fn restore_failure_depfile(classified: &Classified, dependencies: &[CcDependency]) -> Result<(), String> {
    if classified.output.exists() || classified.depfile.exists() { return Err("stale artifact".into()); }
    let bytes = serialized_depfile(classified, dependencies)?;
    let parent = classified.depfile.parent().ok_or("depfile parent missing")?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    temporary.write_all(bytes.as_bytes()).map_err(|e| e.to_string())?;
    temporary.as_file().set_permissions(artifact_permissions(&classified.depfile)?).map_err(|e| e.to_string())?;
    temporary.as_file().sync_all().map_err(|e| e.to_string())?;
    temporary.persist(&classified.depfile).map_err(|e| e.to_string())?;
    Ok(())
}

fn probe_hit(options: &DriverOptions, classified: &Classified, key: &str, roots: &[String],
    kind: &'static str, script_digest: &str, tool_digest: &str, reads: &mut Vec<ProbeRead>) -> Option<i32> {
    let manifest = probe_attempt(options, key, kind, CcOperation::Manifest, Vec::new(), reads, classified.roots.len())?;
    if manifest.disposition != "hit" { return None; }
    let record = manifest.record.as_ref()?;
    let current = dependencies_from_record(classified, &record.dependencies).ok()?;
    if record.action_key != key || record.dependencies.is_empty()
        || admit_cc_reuse(record, &current, classified.roots.len()) != Ok(true)
        || tree_digests(classified, options).as_deref() != Ok(roots) { return None; }
    let response = probe_attempt(options, key, kind, CcOperation::Read, current.clone(), reads, classified.roots.len())?;
    let read_record = response.record.as_ref()?;
    let encoded = response.object_base64.as_ref()?;
    if response.disposition != "hit" || read_record != record
        || admit_cc_reuse(read_record, &current, classified.roots.len()) != Ok(true) { return None; }
    let bytes = STANDARD.decode(encoded).ok()?;
    if bytes.len() as u64 != record.object_bytes || bytes.len() as u64 > MAX_OBJECT
        || blake3::hash(&bytes).to_hex().as_str() != record.object_digest_blake3
        || tree_digests(classified, options).as_deref() != Ok(roots) { return None; }
    if kind == "object" {
        restore(classified, &current, &bytes).ok()?;
        probe_receipt(options, "hit", "verified-object-and-dependencies", key, 0, reads);
        return Some(0);
    }
    let result: CcProbeResult = serde_json::from_slice(&bytes).ok()?;
    let (stdout, stderr) = decode_cc_probe_result(&result).ok()?;
    if result.script_digest_blake3 != script_digest || result.compiler_digest_blake3 != tool_digest
        || result.source_path != classified.source.to_str()?
        || result.output_path != classified.output.to_str()?
        || result.depfile_path != classified.depfile.to_str()?
        || options.probe_script.as_ref().and_then(|path| digest_file(path, MAX_FILE).ok()).as_deref() != Some(script_digest)
        || digest_file(&options.compiler.canonicalize().ok()?, MAX_TREE).ok().as_deref() != Some(tool_digest) {
        return None;
    }
    restore_failure_depfile(classified, &current).ok()?;
    std::io::stdout().write_all(&stdout).ok()?;
    std::io::stderr().write_all(&stderr).ok()?;
    probe_receipt(options, "probe-failure-hit", "verified-failure-and-dependencies", key, result.compiler_exit_code, reads);
    Some(result.compiler_exit_code)
}

fn run_probe(options: &DriverOptions, classified: &Classified, input: &CcActionInput, key: &str, script_digest: &str) -> i32 {
    let mut reads = Vec::new();
    let mut failure = input.clone();
    failure.normalized_arguments.push("probe-failure-result-v1".into());
    let failure_key = match cc_action_key(&failure) {
        Ok(key) => key,
        Err(_) => return fallback(options, "unbounded-or-unsafe-input", Some(key)),
    };
    let script = options.probe_script.as_ref().expect("classified probe script");
    if digest_file(script, MAX_FILE).as_deref() != Ok(script_digest) {
        let exit = compiler_status(&options.compiler, &options.arguments);
        probe_receipt(options, "fallback", "probe-script-changed", key, exit, &reads);
        return exit;
    }
    if let Some(exit) = probe_hit(options, classified, &failure_key, &input.roots_digest_blake3,
        "failure", script_digest, &input.tool_digest_blake3, &mut reads) { return exit; }
    if let Some(attempt) = reads.last_mut() {
        if attempt.disposition == "hit" { attempt.disposition = "invalid"; }
    }
    if reads.last().is_some_and(|attempt| matches!(attempt.disposition, "invalid" | "unavailable")) {
        let exit = compiler_status(&options.compiler, &options.arguments);
        probe_receipt(options, "fallback", "unknown-failure-manifest-or-read", key, exit, &reads);
        return exit;
    }
    if let Some(exit) = probe_hit(options, classified, key, &input.roots_digest_blake3,
        "object", script_digest, &input.tool_digest_blake3, &mut reads) { return exit; }
    if let Some(attempt) = reads.last_mut() {
        if attempt.disposition == "hit" { attempt.disposition = "invalid"; }
    }
    if reads.last().is_some_and(|attempt| matches!(attempt.disposition, "invalid" | "unavailable")) {
        let exit = compiler_status(&options.compiler, &options.arguments);
        probe_receipt(options, "fallback", "unknown-object-manifest-or-read", key, exit, &reads);
        return exit;
    }
    let clean_depfile = !classified.depfile.exists();
    let clean_object = !classified.output.exists();
    let (exit, diagnostics) = compiler_with_diagnostics(options);
    if exit != 0 {
        let publish = (|| -> Result<(), String> {
            if !(1..=125).contains(&exit) || !clean_depfile || !clean_object
                || classified.output.exists() { return Err("ineligible failure artifacts".into()); }
            let (stdout, stderr) = diagnostics.ok_or("unbounded diagnostics")?;
            if digest_file(script, MAX_FILE)? != script_digest
                || digest_file(&options.compiler.canonicalize().map_err(|e| e.to_string())?, MAX_TREE)? != input.tool_digest_blake3
                || tree_digests(classified, options)? != input.roots_digest_blake3 {
                return Err("probe inputs changed".into());
            }
            let dependencies = dependencies_from_depfile(classified)?;
            let payload = CcProbeResult {
                schema: CC_PROBE_RESULT_SCHEMA.into(), script_digest_blake3: script_digest.into(),
                compiler_digest_blake3: input.tool_digest_blake3.clone(),
                source_path: classified.source.to_str().ok_or("source encoding")?.into(),
                output_path: classified.output.to_str().ok_or("output encoding")?.into(),
                depfile_path: classified.depfile.to_str().ok_or("depfile encoding")?.into(),
                compiler_exit_code: exit, stdout_base64: STANDARD.encode(stdout),
                stderr_base64: STANDARD.encode(stderr),
            };
            decode_cc_probe_result(&payload).map_err(|e| e.to_string())?;
            let bytes = serde_json::to_vec(&payload).map_err(|e| e.to_string())?;
            let digest = blake3::hash(&bytes).to_hex().to_string();
            let reply = request(&options.socket, wire(CcOperation::Publish, &failure_key,
                dependencies.clone(), Some(STANDARD.encode(&bytes))))?;
            validate_cc_response(&reply, classified.roots.len()).map_err(|e| e.to_string())?;
            if reply.disposition != "published" || reply.record.as_ref().is_none_or(|record|
                record.action_key != failure_key || record.dependencies != dependencies
                || record.object_digest_blake3 != digest || record.object_bytes != bytes.len() as u64) {
                return Err("failure publication unacknowledged".into());
            }
            Ok(())
        })();
        let status = if publish.is_ok() { "probe-failure-published" } else { "probe-compiler-failure" };
        probe_receipt(options, status, "compiler-failure", if publish.is_ok() { &failure_key } else { key }, exit, &reads);
        return exit;
    }
    let publish = (|| -> Result<(), String> {
        if diagnostics.as_ref().is_none_or(|(stdout, stderr)| !stdout.is_empty() || !stderr.is_empty()) {
            return Err("nonempty or unbounded successful diagnostics".into());
        }
        if digest_file(script, MAX_FILE)? != script_digest
            || digest_file(&options.compiler.canonicalize().map_err(|e| e.to_string())?, MAX_TREE)? != input.tool_digest_blake3
            || tree_digests(classified, options)? != input.roots_digest_blake3 {
            return Err("probe inputs changed".into());
        }
        let dependencies = dependencies_from_depfile(classified)?;
        let bytes = file_bytes(&classified.output, MAX_OBJECT)?;
        let digest = blake3::hash(&bytes).to_hex().to_string();
        let reply = request(&options.socket, wire(CcOperation::Publish, key, dependencies.clone(),
            Some(STANDARD.encode(&bytes))))?;
        validate_cc_response(&reply, classified.roots.len()).map_err(|e| e.to_string())?;
        if reply.disposition != "published" || reply.record.as_ref().is_none_or(|record|
            record.action_key != key || record.dependencies != dependencies
            || record.object_digest_blake3 != digest || record.object_bytes != bytes.len() as u64) {
            return Err("object publication unacknowledged".into());
        }
        Ok(())
    })();
    let status = if publish.is_ok() { "published" } else { "compiler-only" };
    probe_receipt(options, status, "compiled", key, exit, &reads);
    exit
}

/// Run one compiler invocation; the integer returned is the compiler-compatible process exit code.
pub fn run(options: &DriverOptions) -> i32 {
    if !options.compiler.is_absolute() {
        receipt(&options.receipt, "rejected", "compiler-must-be-absolute", None, 2);
        return 2;
    }
    if !options.socket.is_absolute() || !options.receipt.is_absolute()
        || !matches!(options.platform_digest.as_bytes(), digest if digest.len() == 64 && digest.iter().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())) {
        return fallback(options, "invalid-driver-options", None);
    }
    if std::env::var_os("MANTLE_CC_CACHE_DISABLE").is_some() {
        return fallback(options, "disabled", None);
    }
    let mut classified = match classify(&options.arguments) { Ok(value) => value, Err(_) => return fallback(options, "unclassified-invocation", None) };
    if !gnu_compiler(&options.compiler) {
        return fallback(options, "unclassified-compiler-family", None);
    }
    let probe_digest = if let Some(script) = &options.probe_script {
        if !script.is_absolute() || script.to_str().is_none()
            || script.canonicalize().as_deref().ok() != Some(script.as_path())
            || !options.arguments.iter().any(|arg| arg == "-fdiagnostics-color=never")
            || !options.arguments.iter().any(|arg| arg == "-fmessage-length=0")
            || std::env::var_os("LC_ALL").as_deref() != Some(OsStr::new("C"))
            || std::env::vars_os().any(|(name, _)| {
                let name = name.to_string_lossy();
                (name.starts_with("LC_") && name != "LC_ALL")
                    || name.starts_with("GCC_") || name == "LANGUAGE"
                    || name == "DEPENDENCIES_OUTPUT" || name == "SUNPRO_DEPENDENCIES"
            }) {
            return fallback(options, "unclassified-probe-environment", None);
        }
        match digest_file(script, MAX_FILE) {
            Ok(digest) => Some(digest),
            Err(_) => return fallback(options, "unbounded-or-unsafe-probe-script", None),
        }
    } else { None };
    if ["CPATH", "C_INCLUDE_PATH", "CPLUS_INCLUDE_PATH", "OBJC_INCLUDE_PATH", "GCC_EXEC_PREFIX", "COMPILER_PATH", "SOURCE_DATE_EPOCH"].iter().any(|name| std::env::var_os(name).is_some()) {
        return fallback(options, "ambient-compiler-input", None);
    }
    let facts = (|| -> Result<(String, CcActionInput), String> {
        let compiler_binary = options.compiler.canonicalize().map_err(|e| e.to_string())?;
        let tool = digest_file(&compiler_binary, MAX_TREE)?;
        let source = digest_file(&classified.source, MAX_FILE)?;
        let roots = tree_digests(&classified, options)?;
        let name = options.compiler.file_name().and_then(|s| s.to_str()).ok_or("invalid compiler name")?;
        classified.normalized.push(format!("compiler:{name}"));
        if let Some(digest) = &probe_digest {
            classified.normalized.push(format!("probe-script-digest:{digest}"));
        }
        let input = CcActionInput {
            tool_digest_blake3: tool, source_digest_blake3: source,
            platform_digest_blake3: options.platform_digest.clone(),
            normalized_arguments: std::mem::take(&mut classified.normalized),
            roots_digest_blake3: roots,
        };
        let key = cc_action_key(&input).map_err(|e| e.to_string())?;
        Ok((key, input))
    })();
    let (key, input) = match facts { Ok(facts) => facts, Err(_) => return fallback(options, "unbounded-or-unsafe-input", None) };
    if let Some(digest) = probe_digest.as_deref() {
        return run_probe(options, &classified, &input, &key, digest);
    }
    let roots = input.roots_digest_blake3;
    let manifest = request(&options.socket, wire(CcOperation::Manifest, &key, Vec::new(), None));
    let manifest = match manifest { Ok(reply) => reply, Err(_) => return fallback(options, "cache-unavailable", Some(&key)) };
    if validate_cc_response(&manifest, classified.roots.len()).is_err() {
        return fallback(options, "invalid-cache-response", Some(&key));
    }
    if manifest.disposition == "hit" {
        if let Some(record) = &manifest.record {
            if record.action_key == key && !record.dependencies.is_empty() {
                if let Ok(current) = dependencies_from_record(&classified, &record.dependencies) {
                    if admit_cc_reuse(record, &current, classified.roots.len()) == Ok(true)
                        && tree_digests(&classified, options).as_ref() == Ok(&roots) {
                        let reply = request(&options.socket, wire(CcOperation::Read, &key, current.clone(), None));
                        if let Ok(reply) = reply {
                            if reply.disposition == "hit" {
                                if let (Some(read_record), Some(encoded)) = (&reply.record, &reply.object_base64) {
                                    if read_record == record && admit_cc_reuse(read_record, &current, classified.roots.len()) == Ok(true) {
                                        if let Ok(bytes) = STANDARD.decode(encoded) {
                                            if bytes.len() as u64 == record.object_bytes && bytes.len() as u64 <= MAX_OBJECT
                                                && blake3::hash(&bytes).to_hex().as_str() == record.object_digest_blake3
                                                && tree_digests(&classified, options).as_ref() == Ok(&roots)
                                                && restore(&classified, &current, &bytes).is_ok() {
                                                receipt(&options.receipt, "hit", "verified-object-and-dependencies", Some(&key), 0);
                                                return 0;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    } else if manifest.disposition != "miss" || manifest.record.is_some() || manifest.object_base64.is_some() {
        return fallback(options, "invalid-manifest-disposition", Some(&key));
    }
    let exit = compiler_status(&options.compiler, &options.arguments);
    if exit != 0 { receipt(&options.receipt, "compiler-failure", "not-cacheable", Some(&key), exit); return exit; }
    let publish = (|| -> Result<(), String> {
        if tree_digests(&classified, options)? != roots { return Err("source roots changed during compilation".into()); }
        let dependencies = dependencies_from_depfile(&classified)?;
        let bytes = file_bytes(&classified.output, MAX_OBJECT)?;
        let object_digest = blake3::hash(&bytes).to_hex().to_string();
        let object_size = bytes.len() as u64;
        let reply = request(&options.socket, wire(CcOperation::Publish, &key, dependencies.clone(), Some(STANDARD.encode(bytes))))?;
        validate_cc_response(&reply, classified.roots.len()).map_err(|e| e.to_string())?;
        if reply.disposition != "published" || reply.record.as_ref().is_none_or(|record| {
            record.action_key != key || record.object_digest_blake3 != object_digest
                || record.object_bytes != object_size || record.dependencies != dependencies
        }) {
            return Err("publish not acknowledged".into());
        }
        Ok(())
    })();
    match publish {
        Ok(()) => receipt(&options.receipt, "published", "compiled-and-published", Some(&key), exit),
        Err(_) => receipt(&options.receipt, "compiler-only", "publication-ineligible-or-unavailable", Some(&key), exit),
    }
    exit
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_escaped_spaces_backslashes_and_continuations() {
        assert_eq!(parse_depfile(b"out\\ file.o: /src/a\\ b.c /src/a\\\\b.h \\\n /src/third.h\n").unwrap(), vec!["/src/a b.c", "/src/a\\b.h", "/src/third.h"]);
        assert!(parse_depfile(b"x.o: /src/a.c \\").is_err());
        assert!(parse_depfile(b"x.o: /src/a.c\nother.o: /src/b.c\n").is_err());
    }
    #[test]
    fn classifies_only_remapped_bounded_compilations() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let source = root.join("source.c");
        fs::write(&source, "int answer(void) { return 42; }").unwrap();
        let flags = vec!["-c".into(), source.display().to_string(), "-o".into(), root.join("source.o").display().to_string(),
            "-MMD".into(), "-MF".into(), root.join("source.d").display().to_string(), "-nostdinc".into(), "-w".into(),
            format!("-ffile-prefix-map={}=/cc-root-0", root.display())];
        assert!(classify(&flags).is_ok());
        for bad in ["@args", "-E", "-include", "-g", "-fplugin=evil", "-shared"] {
            let mut args = flags.clone(); args.push(bad.into());
            assert!(classify(&args).is_err(), "accepted {bad}");
        }
        let mut relative_object = flags.clone();
        relative_object[3] = "relative.o".into();
        assert!(classify(&relative_object).is_err(), "relative object changes GCC dep target");
        let mut relative_depfile = flags.clone();
        relative_depfile[6] = "relative.d".into();
        assert!(classify(&relative_depfile).is_err(), "relative -MF is not a stable artifact path");
        let mut unmapped = flags; unmapped.pop();
        assert!(classify(&unmapped).is_err());
    }
    #[test]
    fn restores_real_gcc_dependency_order_and_wraps_for_c_and_cpp() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        for (compiler, extension) in [("gcc", "c"), ("g++", "cpp")] {
            let source = root.join(format!("long_source_filename_for_depfile_order.{extension}"));
            fs::write(&source, "#include \"zebra.h\"\n#include \"alpha.h\"\nint answer(void) { return ZEBRA + ALPHA; }\n").unwrap();
            fs::write(root.join("zebra.h"), "#define ZEBRA 20\n").unwrap();
            fs::write(root.join("alpha.h"), "#define ALPHA 22\n").unwrap();
            let object = root.join(format!("long_target_object_filename_for_depfile_order_{extension}.o"));
            let depfile = root.join(format!("{extension}.d"));
            let flags = vec!["-c".into(), source.display().to_string(), "-o".into(),
                object.display().to_string(), "-MMD".into(), "-MF".into(),
                depfile.display().to_string(), "-nostdinc".into(), "-w".into(),
                format!("-ffile-prefix-map={}=/cc-root-0", root.display())];
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
