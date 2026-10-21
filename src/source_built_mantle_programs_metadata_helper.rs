//! Source text compiled by the Mantle author; neither program launches children.

pub(super) const SCRIPT_INITIALIZER: &str = r#"
mod mantle_cairn_script_environment {
    pub struct PhaseGuard;
    impl Drop for PhaseGuard {
        fn drop(&mut self) {
            // Missing-next suspension uses process::exit, so it never drops
            // this guard. A panicking script likewise keeps diagnostic state.
            // A Result::Err main is rejected by its exact-zero action policy.
            if std::thread::panicking() || std::env::var_os("MANTLE_CC_REPLIES").is_none() { return; }
            let directory = std::path::PathBuf::from(std::env::var_os("OUT_DIR").expect("missing script OUT_DIR"))
                .join(".mantle-cc-random");
            match std::fs::remove_dir_all(directory) {
                Ok(()) => (),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
                Err(error) => panic!("removing completed source phase instrumentation: {}", error),
            }
        }
    }
    pub fn initialize() -> PhaseGuard {
        std::fs::create_dir_all(std::env::var_os("OUT_DIR").expect("missing script OUT_DIR"))
            .expect("creating original generator OUT_DIR");
        PhaseGuard
    }
}
"#;

pub(super) fn process_source(process_source: &str) -> String {
    format!("mod mantle_cairn_script_process {{\n{process_source}\n{ABSENT_PROCESS}\n}}\n")
}

pub(super) fn source(process_source: &str, native_source: Option<&str>) -> String {
    let process = self::process_source(process_source);
    let dispatch = if native_source.is_some() {
        r#"
    if matches!(destination.as_str(), "--native-request" | "--native-result") {
        let arguments: Vec<_> = std::iter::once(_executable).chain(std::iter::once(destination)).chain(args).collect();
        return mantle_native_request::run(&arguments).map_err(|error| error.to_string());
    }
"#
    } else {
        ""
    };
    SOURCE
        .replace("/*SCRIPT_ENTRY_TEXT*/", &format!("{SCRIPT_INITIALIZER:?}"))
        .replace("/*SCRIPT_PROCESS_TEXT*/", &format!("{process:?}"))
        .replace("/*NATIVE_REQUEST_SOURCE*/", native_source.unwrap_or(""))
        .replace("/*NATIVE_REQUEST_DISPATCH*/", dispatch)
        .replace("/*INVOCATION_SOURCE*/", INVOCATION_SOURCE)
}

const ABSENT_PROCESS: &str = r#"
fn absent_program(command: &std::process::Command) -> std::io::Error {
    use std::os::unix::{ffi::OsStrExt, fs::PermissionsExt};
    let cwd = command.get_current_dir().map(std::path::Path::to_path_buf)
        .unwrap_or_else(|| std::env::current_dir().expect("reading original process cwd"));
    let program = command.get_program();
    let candidates = if program.as_bytes().contains(&b'/') {
        vec![cwd.join(program)]
    } else {
        let mut path = std::env::var_os("PATH");
        for (key, value) in command.get_envs() {
            if key == "PATH" { path = value.map(std::ffi::OsStr::to_owned); }
        }
        std::env::split_paths(&path.unwrap_or_else(|| "/bin:/usr/bin".into()))
            .map(|directory| cwd.join(directory).join(program)).collect()
    };
    let mut denied = false;
    for candidate in candidates {
        match std::fs::metadata(&candidate) {
            Ok(metadata) if metadata.is_file() && metadata.permissions().mode() & 0o111 != 0 => {
                panic!("original process {:?} exists in the admitted namespace and requires a genuine source phase", candidate);
            }
            Ok(_) => denied = true,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
            Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => denied = true,
            Err(error) => return error,
        }
    }
    std::io::Error::new(if denied { std::io::ErrorKind::PermissionDenied } else { std::io::ErrorKind::NotFound },
        "original program lookup in admitted isolated namespace")
}
pub(crate) fn spawn_if_absent(command: &mut std::process::Command) -> std::io::Result<std::process::Child> {
    Err(absent_program(command))
}
pub(crate) fn output_if_absent(command: &mut std::process::Command) -> std::io::Result<std::process::Output> {
    Err(absent_program(command))
}
"#;

const SOURCE: &str = r#"
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::Path;
/*NATIVE_REQUEST_SOURCE*/

type Result<T, E = String> = std::result::Result<T, E>;
/*INVOCATION_SOURCE*/
#[derive(Default)]
struct Metadata {
    cfg: BTreeSet<String>,
    env: BTreeMap<String, String>,
    libs: BTreeSet<String>,
    search: BTreeSet<String>,
    custom: BTreeMap<String, String>,
}
fn safe_name(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|c| c.is_ascii_alphanumeric() || b"_-.+".contains(&c))
}
fn validate_lib(value: &str) -> Result<()> {
    if value.chars().any(char::is_whitespace) { return Err("rustc-link-lib contains whitespace".into()); }
    let name = if let Some((prefix, name)) = value.split_once('=') {
        let (kind, modifiers) = prefix.split_once(':').map_or((prefix, None), |(k, m)| (k, Some(m)));
        if !matches!(kind, "static" | "dylib" | "framework") { return Err("unsupported rustc-link-lib kind".into()); }
        if let Some(modifiers) = modifiers {
            for modifier in modifiers.split(',') {
                let token = modifier.strip_prefix('+').or_else(|| modifier.strip_prefix('-'))
                    .ok_or("invalid rustc-link-lib modifier")?;
                if token.is_empty() || !token.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-') {
                    return Err("invalid rustc-link-lib modifier".into());
                }
            }
        }
        name
    } else { value };
    if !safe_name(name) { return Err("unsafe rustc-link-lib name".into()); }
    Ok(())
}
fn validate_search(value: &str) -> Result<()> {
    if value.chars().any(char::is_whitespace) { return Err("rustc-link-search contains whitespace".into()); }
    let path = if let Some((kind, path)) = value.split_once('=') {
        if !matches!(kind, "dependency" | "crate" | "native" | "framework" | "all") {
            return Err("unsupported rustc-link-search kind".into());
        }
        path
    } else { value };
    if path.is_empty() || path.contains("..") { return Err("unsafe rustc-link-search path".into()); }
    Ok(())
}
fn rebase(value: &str, directory: &str, package: &str) -> String {
    // Each stdout has its own writable output and original source package.
    // A consumer may be a different package and must bind both separately.
    let bindings = [("/outputs/out-dir", directory), ("/inputs/package", package)];
    let mut result = String::with_capacity(value.len());
    let mut rest = value;
    loop {
        let found = bindings.iter().filter_map(|(old, new)| rest.find(*old).map(|index| (index, *old, *new)))
            .min_by_key(|(index, _, _)| *index);
        let Some((index, old, new)) = found else { result.push_str(rest); break; };
        result.push_str(&rest[..index]);
        let tail = &rest[index + old.len()..];
        if tail.is_empty() || tail.starts_with('/') || tail.starts_with(char::is_whitespace) {
            result.push_str(new);
        } else { result.push_str(old); }
        rest = tail;
    }
    result
}
fn parse(stdout: &str, directory: &str, package: &str) -> Result<Metadata> {
    let mut metadata = Metadata::default();
    for (index, line) in stdout.lines().enumerate() {
        let Some(payload) = line.strip_prefix("cargo::").or_else(|| line.strip_prefix("cargo:")) else { continue; };
        let Some((key, raw)) = payload.split_once('=') else { continue; };
        if raw.contains('\0') { return Err(format!("metadata line {} contains NUL", index + 1)); }
        let value = rebase(raw, directory, package);
        match key {
            "rustc-cfg" => { if value.is_empty() { return Err("empty rustc-cfg".into()); } metadata.cfg.insert(value); }
            "rustc-env" => {
                let (key, value) = value.split_once('=').ok_or("rustc-env must be KEY=VALUE")?;
                if key.is_empty() { return Err("empty rustc-env key".into()); }
                metadata.env.insert(key.into(), value.into());
            }
            "rustc-link-lib" => { validate_lib(&value)?; metadata.libs.insert(value); }
            "rustc-link-search" => { validate_search(&value)?; metadata.search.insert(value); }
            "rerun-if-changed" => { if value.is_empty() { return Err("empty rerun-if-changed".into()); } }
            // Match the source-only topology parser, including its intentional
            // omission of link-arg and check-cfg from consumer compiler flags.
            "warning" | "rerun-if-env-changed" | "rustc-check-cfg" | "rustc-link-arg" => (),
            _ => {
                if key.is_empty() || key.starts_with("rustc-") || key.starts_with("rerun-") || key.starts_with("warning")
                    || !key.bytes().all(|c| c.is_ascii_alphanumeric() || b"_-".contains(&c)) {
                    return Err(format!("invalid custom metadata key {key:?}"));
                }
                metadata.custom.insert(key.into(), value);
            }
        }
    }
    Ok(metadata)
}
fn component(s: &str) -> String {
    s.chars().map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_uppercase() } else { '_' }).collect()
}
fn environment(path: &Path, entries: BTreeMap<String, String>) -> Result<()> {
    let mut file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    for (key, value) in entries {
        frame_entry(&mut file, &key, &value)?;
    }
    Ok(())
}
fn frame_entry(file: &mut std::fs::File, key: &str, value: &str) -> Result<()> {
    if key.is_empty() || key.contains(['=', '\0']) || value.contains('\0') { return Err("invalid environment entry".into()); }
    let key_len = u32::try_from(key.len()).map_err(|_| "environment key too long")?;
    let value_len = u32::try_from(value.len()).map_err(|_| "environment value too long")?;
    for bytes in [&key_len.to_le_bytes()[..], &value_len.to_le_bytes()[..], key.as_bytes(), value.as_bytes()] {
        file.write_all(bytes).map_err(|e| e.to_string())?;
    }
    Ok(())
}

struct Request<'a> { rest: &'a [u8] }
impl<'a> Request<'a> {
    fn number(&mut self) -> Result<usize> {
        if self.rest.len() < 8 { return Err("truncated original process request length".into()); }
        let number = u64::from_le_bytes(self.rest[..8].try_into().unwrap());
        self.rest = &self.rest[8..];
        usize::try_from(number).map_err(|_| "original process request length overflow".into())
    }
    fn word(&mut self) -> Result<&'a str> {
        let length = self.number()?;
        if self.rest.len() < length { return Err("truncated original process request word".into()); }
        let value = std::str::from_utf8(&self.rest[..length]).map_err(|_| "non-UTF8 original process request")?;
        if value.contains('\0') { return Err("NUL in original process request".into()); }
        self.rest = &self.rest[length..];
        Ok(value)
    }
}
fn relocate<'a>(value: &'a str, mappings: &[(String, String)]) -> Result<std::borrow::Cow<'a, str>> {
    let mut replacement = None;
    for (original, relocated) in mappings {
        let (prefix, path) = if value.starts_with(original.as_str()) { ("", value) }
            else if let Some((kind, path)) = value.split_once('=') {
                if !path.starts_with(original.as_str()) { continue; }
                (&value[..kind.len() + 1], path)
            } else { continue; };
        let suffix = &path[original.len()..];
        if !suffix.is_empty() && !suffix.starts_with('/') { continue; }
        if replacement.is_some() { return Err("ambiguous original process path relocation".into()); }
        replacement = Some(format!("{prefix}{relocated}{suffix}"));
    }
    Ok(replacement.map(std::borrow::Cow::Owned).unwrap_or(std::borrow::Cow::Borrowed(value)))
}
fn rustc_request(mut args: impl Iterator<Item = String>) -> Result<()> {
    let request_path = args.next().ok_or("missing original compiler request")?;
    let expected_program = args.next().ok_or("missing admitted compiler path")?;
    let expected_cwd = args.next().ok_or("missing source-fixed compiler cwd")?;
    let destination = args.next().ok_or("missing compiler response directory")?;
    let mut mappings = Vec::new();
    let mut autocfg_name = None;
    loop {
        match args.next().as_deref() {
            Some("--") => break,
            Some("--map") => {
                let original = args.next().ok_or("missing original path prefix")?;
                let relocated = args.next().ok_or("missing relocated path prefix")?;
                if !original.starts_with('/') || !relocated.starts_with('/') {
                    return Err("compiler request relocations must be explicit absolute paths".into());
                }
                mappings.push((original, relocated));
            }
            Some("--autocfg-name") => {
                if autocfg_name.is_some() { return Err("duplicate original autocfg name source".into()); }
                let path = args.next().ok_or("missing original autocfg UUID product")?;
                let index: u64 = args.next().ok_or("missing source-fixed autocfg probe index")?.parse()
                    .map_err(|_| "invalid source-fixed autocfg probe index")?;
                let uuid: u64 = std::fs::read_to_string(path).map_err(|e| e.to_string())?.trim().parse()
                    .map_err(|_| "invalid original autocfg UUID product")?;
                autocfg_name = Some(format!("autocfg_{uuid:016x}_{index}"));
            }
            _ => return Err("compiler request requires source-fixed expected argv after --".into()),
        }
    }
    let expected: Vec<_> = args.collect();
    let bytes = std::fs::read(request_path).map_err(|e| e.to_string())?;
    let mut request = Request { rest: bytes.strip_prefix(b"mantle-cc-process-v1\0").ok_or("invalid original process request schema")? };
    if request.word()? != expected_program { return Err("original compiler executable differs from admitted source-fixed compiler".into()); }
    let count = request.number()?;
    if count != expected.len() { return Err("original compiler argument count differs from source-fixed schedule".into()); }
    let mut arguments = Vec::with_capacity(count);
    for expected in &expected {
        let actual = request.word()?;
        let expected = if expected == "@MANTLE_AUTOCFG_CRATE@" {
            autocfg_name.as_deref().ok_or("autocfg placeholder lacks original UUID product")?
        } else { expected.as_str() };
        if actual != expected { return Err(format!("original compiler argument differs from source-fixed schedule: {actual:?} != {expected:?}")); }
        arguments.push(actual);
    }
    if request.word()? != expected_cwd { return Err("original compiler cwd differs from source-fixed schedule".into()); }
    let count = request.number()?;
    if count > request.rest.len() / 16 { return Err("truncated original compiler environment".into()); }
    let mut env = BTreeMap::new();
    for _ in 0..count {
        let key = request.word()?; let value = request.word()?;
        if env.insert(key, value).is_some() { return Err("duplicate original compiler environment key".into()); }
    }
    if !request.rest.is_empty() { return Err("trailing original process request bytes".into()); }
    let root = Path::new(&destination);
    std::fs::create_dir_all(root).map_err(|e| e.to_string())?;
    let mut response = std::fs::File::create(root.join("compiler.rsp")).map_err(|e| e.to_string())?;
    for argument in arguments {
        let argument = relocate(argument, &mappings)?;
        if argument.contains(['\n', '\r', '\0']) { return Err("unrepresentable original compiler response argument".into()); }
        writeln!(response, "{argument}").map_err(|e| e.to_string())?;
    }
    let mut file = std::fs::File::create(root.join("compiler.env")).map_err(|e| e.to_string())?;
    for (key, value) in env { frame_entry(&mut file, key, &relocate(value, &mappings)?)?; }
    Ok(())
}

const SCRIPT_ENTRY: &str = /*SCRIPT_ENTRY_TEXT*/;
const SCRIPT_PROCESS: &str = /*SCRIPT_PROCESS_TEXT*/;

// A lexer, not a source-text search: comments, raw/ordinary byte and string
// literals and character literals cannot masquerade as the root entry point.
fn tokens(source: &str) -> Result<Vec<(&str, usize)>> {
    let bytes = source.as_bytes();
    let mut result = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let start = i;
        if bytes[i].is_ascii_whitespace() { i += 1; continue; }
        if bytes[i..].starts_with(b"//") {
            i += 2;
            while i < bytes.len() && bytes[i] != b'\n' { i += 1; }
            continue;
        }
        if bytes[i..].starts_with(b"/*") {
            i += 2;
            let mut depth = 1usize;
            while i < bytes.len() && depth != 0 {
                if bytes[i..].starts_with(b"/*") { depth += 1; i += 2; }
                else if bytes[i..].starts_with(b"*/") { depth -= 1; i += 2; }
                else { i += 1; }
            }
            if depth != 0 { return Err("unterminated source block comment".into()); }
            continue;
        }
        let prefix = if matches!(bytes[i], b'b' | b'c') { i + 1 } else { i };
        if bytes.get(prefix) == Some(&b'r') {
            let mut quote = prefix + 1;
            while bytes.get(quote) == Some(&b'#') { quote += 1; }
            if bytes.get(quote) == Some(&b'"') {
                let hashes = quote - prefix - 1;
                i = quote + 1;
                loop {
                    if i >= bytes.len() { return Err("unterminated raw source string".into()); }
                    if bytes[i] == b'"' && bytes.get(i + 1..i + 1 + hashes)
                        .is_some_and(|suffix| suffix.iter().all(|c| *c == b'#')) {
                        i += 1 + hashes;
                        break;
                    }
                    i += 1;
                }
                continue;
            }
        }
        if bytes.get(prefix) == Some(&b'"') {
            i = prefix + 1;
            loop {
                match bytes.get(i) {
                    None => return Err("unterminated source string".into()),
                    Some(b'\\') => i += 2,
                    Some(b'"') => { i += 1; break; }
                    _ => i += 1,
                }
            }
            continue;
        }
        if bytes.get(prefix) == Some(&b'\'') {
            let content = prefix + 1;
            let mut close = content;
            if bytes.get(close) == Some(&b'\\') {
                close += 1;
                if bytes.get(close) == Some(&b'u') && bytes.get(close + 1) == Some(&b'{') {
                    close += 2;
                    while close < bytes.len() && bytes[close] != b'}' { close += 1; }
                    close += 1;
                } else if bytes.get(close) == Some(&b'x') { close += 3; }
                else { close += 1; }
            } else if close < bytes.len() {
                close += source[close..].chars().next().unwrap().len_utf8();
            }
            if bytes.get(close) == Some(&b'\'') { i = close + 1; continue; }
            // Otherwise this apostrophe belongs to a lifetime, not a literal.
        }
        let character = source[i..].chars().next().unwrap();
        if character == '_' || character.is_alphabetic() {
            i += character.len_utf8();
            while i < bytes.len() {
                let character = source[i..].chars().next().unwrap();
                if character != '_' && !character.is_alphanumeric() { break; }
                i += character.len_utf8();
            }
        } else { i += character.len_utf8(); }
        result.push((&source[start..i], start));
    }
    Ok(result)
}
fn instrument_script(source: &str) -> Result<String> {
    let tokens = tokens(source)?;
    if tokens.iter().any(|(token, _)| matches!(*token, "mantle_cairn_script_environment" | "_mantle_cairn_phase_guard")) {
        return Err("original script conflicts with source entry module".into());
    }
    let mut nesting = Vec::new();
    let mut opening = None;
    let mut i = 0;
    while i < tokens.len() {
        let (token, _) = tokens[i];
        if nesting.is_empty() && token == "fn" && tokens.get(i + 1).map(|t| t.0) == Some("main") {
            if opening.is_some() { return Err("ambiguous original root main".into()); }
            i += 2;
            if tokens.get(i).map(|t| t.0) != Some("(") { return Err("unsupported generic original main".into()); }
            let mut signature = Vec::new();
            while i < tokens.len() {
                let token = tokens[i].0;
                if signature.is_empty() && matches!(token, "{" | ";") { break; }
                match token {
                    "(" => signature.push(")"), "[" => signature.push("]"), "{" => signature.push("}"),
                    ")" | "]" | "}" => if signature.pop() != Some(token) { return Err("unbalanced original main signature".into()); },
                    _ => (),
                }
                i += 1;
            }
            if tokens.get(i).map(|t| t.0) != Some("{") { return Err("original main lacks a body".into()); }
            opening = Some(tokens[i].1 + 1);
            nesting.push("}");
        } else {
            match token {
                "(" => nesting.push(")"), "[" => nesting.push("]"), "{" => nesting.push("}"),
                ")" | "]" | "}" => if nesting.pop() != Some(token) { return Err("unbalanced original source delimiters".into()); },
                _ => (),
            }
        }
        i += 1;
    }
    if !nesting.is_empty() { return Err("unclosed original source delimiter".into()); }
    let opening = opening.ok_or("original script has no lexical root main")?;
    let mut transformed = String::with_capacity(source.len() + SCRIPT_ENTRY.len() + 100);
    transformed.push_str(&source[..opening]);
    transformed.push_str("\n    let _mantle_cairn_phase_guard = mantle_cairn_script_environment::initialize();\n");
    transformed.push_str(&source[opening..]);
    transformed.push('\n');
    transformed.push_str(SCRIPT_ENTRY);
    Ok(transformed)
}
fn process_boundary(source: String, operation: &str, command: &str) -> Result<String> {
    let method = match operation { "--output" | "--absent-output" => "output", "--status" => "status", "--absent-spawn" => "spawn", _ => return Err("unknown source process boundary".into()) };
    let original = format!("{command}.{method}()");
    let lexical = tokens(&source)?;
    let matches: Vec<_> = source.match_indices(original.as_str()).filter(|(start, _)| {
        lexical.iter().any(|(_, position)| position == start)
            && lexical.iter().any(|(token, position)| *token == ")" && *position + 1 == start + original.len())
    }).map(|(start, _)| start).collect();
    if matches.len() != 1 { return Err(format!("original {method} boundary must have one lexical match, got {}", matches.len())); }
    let replacement = if operation == "--absent-spawn" {
        format!("mantle_cairn_script_process::spawn_if_absent(&mut ({command}))")
    } else if operation == "--absent-output" {
        format!("mantle_cairn_script_process::output_if_absent(&mut ({command}))")
    } else {
        let output = format!("mantle_cairn_script_process::output(&mut ({command}))");
        if operation == "--status" { format!("{output}.map(|output| output.status)") } else { output }
    };
    let start = matches[0];
    let mut result = String::with_capacity(source.len() + replacement.len());
    result.push_str(&source[..start]); result.push_str(&replacement); result.push_str(&source[start + original.len()..]);
    Ok(result)
}
fn emit_library(root: &Path, frame: &mut BTreeMap<String, String>, path: String, source: String) -> Result<()> {
    if !path.starts_with("/inputs/") || Path::new(&path).components()
        .any(|part| !matches!(part, std::path::Component::RootDir | std::path::Component::Normal(_))) {
        return Err("original compiler source path must be normalized under /inputs".into());
    }
    let member = format!("transformed-{}.rs", frame.len());
    if frame.contains_key(&path) { return Err("duplicate original compiler overlay path".into()); }
    std::fs::write(root.join(&member), source).map_err(|e| e.to_string())?;
    frame.insert(path, format!("/inputs/source-overlay/{member}"));
    Ok(())
}
fn library_overlays(mut args: impl Iterator<Item = String>) -> Result<()> {
    let destination = args.next().ok_or("missing source overlay output directory")?;
    let root = Path::new(&destination);
    std::fs::create_dir_all(root).map_err(|e| e.to_string())?;
    let mut frame = BTreeMap::new();
    let mut current: Option<(String, String)> = None;
    while let Some(operation) = args.next() {
        match operation.as_str() {
            "--source" => {
                if let Some((path, source)) = current.take() { emit_library(root, &mut frame, path, source)?; }
                let original = args.next().ok_or("missing original library source")?;
                let path = args.next().ok_or("missing original compiler source path")?;
                current = Some((path, std::fs::read_to_string(original).map_err(|e| e.to_string())?));
            }
            "--replace" => {
                let before = args.next().ok_or("missing original library boundary")?;
                let after = args.next().ok_or("missing source-owned boundary replacement")?;
                let (_, source) = current.as_mut().ok_or("library replacement precedes original source")?;
                if before.is_empty() || source.matches(before.as_str()).count() != 1 {
                    return Err("original library boundary must have one exact match".into());
                }
                *source = source.replacen(before.as_str(), &after, 1);
            }
            "--append" => {
                let addition = args.next().ok_or("missing source-owned library module")?;
                let (_, source) = current.as_mut().ok_or("library addition precedes original source")?;
                source.push('\n'); source.push_str(&addition); source.push('\n');
            }
            "--entry" => {
                let (_, source) = current.as_mut().ok_or("script entry precedes original source")?;
                *source = instrument_script(source)?;
            }
            "--output" | "--status" | "--absent-output" | "--absent-spawn" => {
                let command = args.next().ok_or("missing original process receiver expression")?;
                let (_, source) = current.as_mut().ok_or("process boundary precedes original source")?;
                *source = process_boundary(std::mem::take(source), &operation, &command)?;
            }
            "--process-module" => {
                let (_, source) = current.as_mut().ok_or("process module precedes original source")?;
                if tokens(source)?.windows(2).any(|pair| pair[0].0 == "mod" && pair[1].0 == "mantle_cairn_script_process") {
                    return Err("original module conflicts with source process module".into());
                }
                source.push_str(SCRIPT_PROCESS);
            }
            _ => return Err("unknown original library overlay operation".into()),
        }
    }
    if let Some((path, source)) = current { emit_library(root, &mut frame, path, source)?; }
    if frame.is_empty() { return Err("library overlay has no original source".into()); }
    environment(&root.join("overlay.frame"), frame)
}
fn source_package(mut args: impl Iterator<Item = String>) -> Result<()> {
    let source = args.next().ok_or("missing original package")?;
    let destination = args.next().ok_or("missing package output")?;
    let excluded = args.collect::<std::collections::BTreeSet<_>>();
    if excluded.iter().any(|member| member.is_empty() || Path::new(member).components().any(|part| !matches!(part, std::path::Component::Normal(_)))) {
        return Err("source package requires exact normalized generated members".into());
    }
    fn copy(source: &Path, target: &Path, relative: &str, excluded: &std::collections::BTreeSet<String>) -> Result<()> {
        std::fs::create_dir_all(target).map_err(|e| e.to_string())?;
        let mut entries = std::fs::read_dir(source).map_err(|e| e.to_string())?.collect::<std::io::Result<Vec<_>>>().map_err(|e| e.to_string())?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let name = entry.file_name().into_string().map_err(|_| "non-UTF8 original source member")?;
            let member = if relative.is_empty() { name.clone() } else { format!("{relative}/{name}") };
            if excluded.contains(&member) { continue; }
            let kind = entry.file_type().map_err(|e| e.to_string())?;
            if kind.is_dir() { copy(&entry.path(), &target.join(name), &member, excluded)?; }
            else if kind.is_file() { std::fs::copy(entry.path(), target.join(name)).map_err(|e| e.to_string())?; }
            else { return Err("original package copy requires regular source members".into()); }
        }
        Ok(())
    }
    copy(Path::new(&source), Path::new(&destination), "", &excluded)
}
fn rust_probe_reply(mut args: impl Iterator<Item = String>) -> Result<()> {
    let request = args.next().ok_or("missing original probe request")?;
    let observed = args.next().ok_or("missing authenticated probe observation")?;
    let destination = args.next().ok_or("missing original probe reply output")?;
    if args.next().is_some() { return Err("extra original probe reply operand".into()); }
    let observed = Path::new(&observed);
    let outcome = std::fs::read(observed.join("outcome.json")).map_err(|e| e.to_string())?;
    if !outcome.starts_with(b"{\"kind\":\"executed\",\"output_digests\":") || !outcome.ends_with(b"}") {
        return Err("original compiler probe did not execute".into());
    }
    let outputs = observed.join("outputs");
    let status = std::fs::read(outputs.join("exit-status")).map_err(|e| e.to_string())?;
    if status != b"0\n" && status != b"1\n" { return Err("original compiler probe has unsupported normal status".into()); }
    let root = Path::new(&destination);
    std::fs::create_dir_all(root.join("files")).map_err(|e| e.to_string())?;
    for (source, name) in [(Path::new(&request).to_path_buf(), "request"), (outputs.join("stdout"), "stdout"), (outputs.join("stderr"), "stderr"), (outputs.join("exit-status"), "status")] {
        std::fs::copy(source, root.join(name)).map_err(|e| e.to_string())?;
    }
    if status == b"0\n" {
        source_package([outputs.join("compiler-products").to_str().ok_or("non-UTF8 compiler product path")?.to_owned(), root.join("files").to_str().ok_or("non-UTF8 compiler reply path")?.to_owned()].into_iter())?;
    }
    Ok(())
}
fn run() -> Result<()> {
    let mut args = std::env::args();
    let _executable = args.next().ok_or("missing helper executable name")?;
    let destination = args.next().ok_or("missing output directory")?;
    /*NATIVE_REQUEST_DISPATCH*/
    if destination == "--rustc-request" { return rustc_request(args); }
    if destination == "--library-overlays" { return library_overlays(args); }
    if destination == "--source-package" { return source_package(args); }
    if destination == "--rust-probe-reply" { return rust_probe_reply(args); }
    if destination == "--compiler-invocation" { return compiler_invocation(args); }
    if destination == "--copy-tree" {
        let source = args.next().ok_or("missing tree source")?;
        let target = args.next().ok_or("missing tree output")?;
        if args.next().is_some() { return Err("extra tree copy operand".into()); }
        return source_package([source, target].into_iter());
    }
    let mut flags = Vec::new();
    let mut env = BTreeMap::new();
    while let Some(role) = args.next() {
        let links = if role == "--dependency" { Some(args.next().ok_or("missing dependency links")?) }
            else if role == "--own" { None } else { return Err(format!("unknown metadata role {role}")); };
        let stdout_path = args.next().ok_or("missing original stdout")?;
        let directory = args.next().ok_or("missing consumer OUT_DIR binding")?;
        let package = args.next().ok_or("missing original source package binding")?;
        let stdout = std::fs::read_to_string(stdout_path).map_err(|e| e.to_string())?;
        let metadata = parse(&stdout, &directory, &package)?;
        if let Some(links) = links {
            for (key, value) in metadata.custom {
                env.insert(format!("DEP_{}_{}", component(&links), component(&key)), value);
            }
        } else {
            env.insert("OUT_DIR".into(), directory);
            env.extend(metadata.env);
            for (flag, values) in [("--cfg", metadata.cfg), ("-L", metadata.search), ("-l", metadata.libs)] {
                for value in values { flags.push(flag.into()); flags.push(value); }
            }
        }
    }
    let root = Path::new(&destination);
    std::fs::create_dir_all(root).map_err(|e| e.to_string())?;
    // rustc's ordinary @file parser treats each line as one argument; it does
    // not perform shell quoting. No child or environment wrapper is involved.
    let mut response = std::fs::File::create(root.join("compiler.rsp")).map_err(|e| e.to_string())?;
    for flag in flags {
        if flag.contains(['\n', '\r', '\0']) { return Err("unrepresentable rustc response argument".into()); }
        writeln!(response, "{flag}").map_err(|e| e.to_string())?;
    }
    environment(&root.join("compiler.env"), env)
}
fn main() {
    if let Err(error) = run() { eprintln!("source metadata: {error}"); std::process::exit(1); }
}
"#;

// The helper is compiled without third-party dependencies. This bounded JSON
// reader handles the source-authored canonical invocation template, while the
// original binary environment frame remains the authority for dynamic values.
const INVOCATION_SOURCE: &str = r##"
struct InvocationJson<'a> { text: &'a str, at: usize }
impl<'a> InvocationJson<'a> {
    fn byte(&self) -> Option<u8> { self.text.as_bytes().get(self.at).copied() }
    fn take(&mut self, byte: u8) -> Result<()> {
        if self.byte() != Some(byte) { return Err("invalid invocation template JSON".into()); }
        self.at += 1; Ok(())
    }
    fn string_end(&mut self) -> Result<()> {
        self.take(b'"')?;
        while let Some(byte) = self.byte() {
            self.at += 1;
            match byte {
                b'"' => return Ok(()),
                b'\\' => { self.byte().ok_or("truncated JSON escape")?; self.at += 1; }
                0..=31 => return Err("control byte in JSON string".into()),
                _ => (),
            }
        }
        Err("unterminated JSON string".into())
    }
    fn value(&mut self, depth: usize) -> Result<&'a str> {
        if depth > 64 { return Err("invocation template JSON nesting exceeds bound".into()); }
        let start = self.at;
        match self.byte().ok_or("missing JSON value")? {
            b'"' => self.string_end()?,
            b'[' | b'{' => {
                let object = self.byte() == Some(b'{'); self.at += 1;
                let close = if object { b'}' } else { b']' };
                if self.byte() != Some(close) {
                    loop {
                        if object { self.string_end()?; self.take(b':')?; }
                        self.value(depth + 1)?;
                        if self.byte() == Some(close) { break; }
                        self.take(b',')?;
                    }
                }
                self.take(close)?;
            }
            _ => {
                while self.byte().is_some_and(|byte| byte.is_ascii_alphanumeric() || b"+-.".contains(&byte)) { self.at += 1; }
                if self.at == start { return Err("invalid JSON scalar".into()); }
            }
        }
        Ok(&self.text[start..self.at])
    }
}
fn invocation_string(raw: &str) -> Result<String> {
    let mut reader = InvocationJson { text: raw, at: 0 }; reader.string_end()?;
    if reader.at != raw.len() { return Err("trailing JSON string bytes".into()); }
    let bytes = raw.as_bytes(); let mut at = 1; let mut out = String::new();
    while at + 1 < bytes.len() {
        let start = at;
        while at + 1 < bytes.len() && bytes[at] != b'\\' { at += 1; }
        out.push_str(&raw[start..at]);
        if at + 1 == bytes.len() { break; }
        at += 1; let escape = bytes[at]; at += 1;
        match escape {
            b'"' => out.push('"'), b'\\' => out.push('\\'), b'/' => out.push('/'),
            b'b' => out.push('\u{8}'), b'f' => out.push('\u{c}'), b'n' => out.push('\n'), b'r' => out.push('\r'), b't' => out.push('\t'),
            b'u' => {
                let end = at.checked_add(4).filter(|end| *end < bytes.len()).ok_or("truncated JSON Unicode escape")?;
                let high = u32::from_str_radix(raw.get(at..end).ok_or("invalid Unicode escape boundary")?, 16).map_err(|_| "invalid JSON Unicode escape")?; at = end;
                let scalar = if (0xd800..=0xdbff).contains(&high) {
                    if bytes.get(at..at + 2) != Some(b"\\u") { return Err("missing low JSON surrogate".into()); }
                    at += 2; let end = at.checked_add(4).filter(|end| *end < bytes.len()).ok_or("truncated low surrogate")?;
                    let low = u32::from_str_radix(raw.get(at..end).ok_or("invalid surrogate boundary")?, 16).map_err(|_| "invalid JSON surrogate")?; at = end;
                    if !(0xdc00..=0xdfff).contains(&low) { return Err("invalid low JSON surrogate".into()); }
                    0x10000 + ((high - 0xd800) << 10) + low - 0xdc00
                } else { high };
                out.push(char::from_u32(scalar).ok_or("invalid JSON Unicode scalar")?);
            }
            _ => return Err("invalid JSON escape".into()),
        }
    }
    Ok(out)
}
fn invocation_object(raw: &str) -> Result<BTreeMap<String, &str>> {
    let mut reader = InvocationJson { text: raw, at: 0 }; reader.take(b'{')?;
    let mut fields = BTreeMap::new();
    if reader.byte() != Some(b'}') {
        loop {
            let start = reader.at; reader.string_end()?;
            let key = invocation_string(&raw[start..reader.at])?; reader.take(b':')?;
            if fields.insert(key, reader.value(0)?).is_some() { return Err("duplicate invocation JSON field".into()); }
            if reader.byte() == Some(b'}') { break; } reader.take(b',')?;
        }
    }
    reader.take(b'}')?;
    if reader.at != raw.len() { return Err("trailing invocation JSON object".into()); }
    Ok(fields)
}
fn invocation_array(raw: &str) -> Result<Vec<&str>> {
    let mut reader = InvocationJson { text: raw, at: 0 }; reader.take(b'[')?;
    let mut values = Vec::new();
    if reader.byte() != Some(b']') {
        loop { values.push(reader.value(0)?); if reader.byte() == Some(b']') { break; } reader.take(b',')?; }
    }
    reader.take(b']')?;
    if reader.at != raw.len() { return Err("trailing invocation JSON array".into()); }
    Ok(values)
}
fn invocation_field<'a>(object: &BTreeMap<String, &'a str>, name: &str) -> Result<&'a str> {
    object.get(name).copied().ok_or_else(|| format!("missing source invocation field {name}"))
}
fn invocation_quote(text: &str) -> String {
    use std::fmt::Write as _;
    let mut out = String::from("\"");
    for character in text.chars() {
        match character {
            '"' => out.push_str("\\\""), '\\' => out.push_str("\\\\"), '\n' => out.push_str("\\n"), '\r' => out.push_str("\\r"), '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"), '\u{c}' => out.push_str("\\f"),
            '\0'..='\u{1f}' => { write!(&mut out, "\\u{:04x}", character as u32).unwrap(); }
            _ => out.push(character),
        }
    }
    out.push('"'); out
}
fn invocation_slice(name: &str, offset: usize, length: usize) -> String {
    format!("{{\"kind\":\"text_slice\",\"name\":{},\"offset_bytes\":{offset},\"length_bytes\":{length}}}", invocation_quote(name))
}
fn invocation_word(raw: &str, data: &mut String, data_name: &str, inert_environment: bool) -> Result<String> {
    let object = invocation_object(raw)?;
    let mut parts = Vec::new();
    for raw in invocation_array(invocation_field(&object, "parts")?)? {
        let part = invocation_object(raw)?;
        if invocation_string(invocation_field(&part, "kind")?)? == "literal" {
            let text = invocation_string(invocation_field(&part, "text")?)?;
            if text.contains('\0') || (!inert_environment && text.contains(['/', '\\'])) { return Err("source invocation filesystem operand must remain typed".into()); }
            if !text.is_empty() {
                parts.push(invocation_slice(data_name, data.len(), text.len())); data.push_str(&text);
            } else { parts.push(raw.into()); }
        } else { parts.push(raw.into()); }
    }
    Ok(format!("{{\"parts\":[{}]}}", parts.join(",")))
}
struct InvocationPath { prefix: String, kind: &'static str, name: String }
fn invocation_delimiter(byte: u8) -> bool { byte.is_ascii_whitespace() || b":;,\"'()[]{}\x1f".contains(&byte) }
fn invocation_dynamic(value: &str, key: &str, data: &mut String, data_name: &str, paths: &[InvocationPath]) -> Result<String> {
    if value.contains('\0') { return Err("NUL in original metadata environment".into()); }
    let base = data.len(); data.push_str(value);
    if value.is_empty() { return Ok("{\"parts\":[{\"kind\":\"literal\",\"text\":\"\"}]}".into()); }
    let mut parts = Vec::new(); let mut at = 0;
    while at < value.len() {
        let next = paths.iter().filter_map(|path| {
            value[at..].match_indices(&path.prefix).find_map(|(offset, _)| {
                let start = at + offset; let end = start + path.prefix.len();
                let before = start == 0 || invocation_delimiter(value.as_bytes()[start - 1]) || value.as_bytes()[start - 1] == b'=';
                let after = end == value.len() || value.as_bytes()[end] == b'/' || invocation_delimiter(value.as_bytes()[end]);
                (before && after).then_some((start, path))
            })
        }).min_by(|(a, left), (b, right)| a.cmp(b).then_with(|| right.prefix.len().cmp(&left.prefix.len())));
        let Some((start, path)) = next else {
            if (key == "OUT_DIR" || key.starts_with("DEP_")) && (value[at..].starts_with('/') || value[at..].contains("/inputs/") || value[at..].contains("/outputs/")) {
                return Err(format!("original metadata path lacks declared resolver: {key}"));
            }
            parts.push(invocation_slice(data_name, base + at, value.len() - at)); break;
        };
        if start > at { parts.push(invocation_slice(data_name, base + at, start - at)); }
        let suffix = start + path.prefix.len();
        let end = value.as_bytes()[suffix..].iter().position(|byte| invocation_delimiter(*byte)).map_or(value.len(), |offset| suffix + offset);
        let original = &value[suffix..end];
        let trailing = original.ends_with('/');
        let member = original.strip_prefix('/').unwrap_or(original);
        let member = member.strip_suffix('/').unwrap_or(member);
        if !member.is_empty() && member.split('/').any(|part| part.is_empty() || part == "." || part == ".." || part.contains('\\')) {
            return Err("non-normalized original metadata path".into());
        }
        let name_key = if path.kind == "runtime" { "mount" } else { "name" };
        parts.push(format!("{{\"kind\":{},\"{name_key}\":{},\"path\":{}}}", invocation_quote(path.kind), invocation_quote(&path.name), invocation_quote(member)));
        at = end;
        if trailing {
            let following = value.as_bytes().get(end).copied();
            if following.is_some_and(|byte| byte != b':') { return Err("typed metadata directory suffix requires word end or colon delimiter".into()); }
            parts.push("{\"kind\":\"directory_separator\"}".into());
            if following == Some(b':') {
                parts.push("{\"kind\":\"literal\",\"text\":\":\"}".into());
                at += 1;
            }
        }
    }
    Ok(format!("{{\"parts\":[{}]}}", parts.join(",")))
}
fn compiler_invocation(mut args: impl Iterator<Item = String>) -> Result<()> {
    let template_path = args.next().ok_or("missing invocation template")?;
    let frame_path = args.next().ok_or("missing actual metadata environment")?;
    let destination = args.next().ok_or("missing invocation output")?;
    let data_destination = args.next().ok_or("missing invocation data output")?;
    if args.next().is_some() { return Err("extra invocation operand".into()); }
    let template = std::fs::read_to_string(template_path).map_err(|e| e.to_string())?;
    let root = invocation_object(&template)?;
    let original = invocation_object(invocation_field(&root, "invocation")?)?;
    let data_name = invocation_string(invocation_field(&root, "data_name")?)?;
    let mut paths = Vec::new();
    for (field, prefix, kind) in [("inputs", "/inputs/", "input"), ("outputs", "/outputs/", "output"), ("runtime", "", "runtime")] {
        for raw in invocation_array(invocation_field(&root, field)?)? {
            let name = invocation_string(raw)?;
            paths.push(InvocationPath { prefix: format!("{prefix}{name}"), kind, name });
        }
    }
    let mut data = String::new(); let mut arguments = Vec::new(); let mut environment = BTreeMap::new();
    for raw in invocation_array(invocation_field(&original, "arguments")?)? { arguments.push(invocation_word(raw, &mut data, &data_name, false)?); }
    let frame = std::fs::read(frame_path).map_err(|e| e.to_string())?;
    let mut at = 0usize; let mut seen = BTreeSet::new();
    while at < frame.len() {
        let header = frame.get(at..at.checked_add(8).ok_or("metadata environment offset overflow")?).ok_or("truncated metadata environment header")?;
        let key_len = u32::from_le_bytes(header[..4].try_into().unwrap()) as usize;
        let value_len = u32::from_le_bytes(header[4..].try_into().unwrap()) as usize;
        at += 8; let key_end = at.checked_add(key_len).ok_or("metadata environment key overflow")?;
        let end = key_end.checked_add(value_len).ok_or("metadata environment value overflow")?;
        let key = std::str::from_utf8(frame.get(at..key_end).ok_or("truncated metadata key")?).map_err(|e| e.to_string())?;
        let value = std::str::from_utf8(frame.get(key_end..end).ok_or("truncated metadata value")?).map_err(|e| e.to_string())?;
        if key.is_empty() || !key.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_') || !seen.insert(key) { return Err("invalid or duplicate original metadata environment key".into()); }
        if !matches!(key, "MANTLE_CAIRN_COMPILE_ENV" | "MANTLE_CAIRN_SCRIPT_ENV") {
            environment.insert(key.to_owned(), invocation_dynamic(value, key, &mut data, &data_name, &paths)?);
        }
        at = end;
    }
    for (key, raw) in invocation_object(invocation_field(&original, "environment")?)? {
        if !environment.contains_key(&key) {
            let word = invocation_word(raw, &mut data, &data_name, key.starts_with("CARGO_PKG_"))?;
            environment.insert(key, word);
        }
    }
    let environment = environment.into_iter().map(|(key, word)| format!("{}:{word}", invocation_quote(&key))).collect::<Vec<_>>().join(",");
    let cwd = original.get("cwd").copied().unwrap_or("null");
    let cwd_suffix = if cwd == "null" { String::new() } else { format!(",\"cwd\":{cwd}") };
    let executable = original.get("executable_path").copied().unwrap_or("null");
    let executable_suffix = if executable == "null" { String::new() } else { format!(",\"executable_path\":{executable}") };
    let workspace = original.get("workspace_mounts").copied().unwrap_or("{}");
    let workspace_suffix = if invocation_object(workspace)?.is_empty() { String::new() } else { format!(",\"workspace_mounts\":{workspace}") };
    let invocation = format!("{{\"arguments\":[{}],\"environment\":{{{environment}}}{cwd_suffix}{executable_suffix}{workspace_suffix}}}", arguments.join(","));
    std::fs::write(data_destination, data).map_err(|e| e.to_string())?;
    std::fs::write(destination, invocation).map_err(|e| e.to_string())
}
"##;
