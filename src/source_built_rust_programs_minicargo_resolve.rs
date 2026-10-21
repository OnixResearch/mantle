//! mrustc 0.12 package policy. Original Cargo.lock is deliberately not an oracle:
//! main.cpp selects ResolveDependencies_Cargo and leaves its lock reader disabled.
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::VecDeque;
use std::io::Read;

use sha2::Digest;
use sha2::Sha256;
pub(crate) use source::Edge;
pub(crate) use source::Key;
pub(crate) use source::Package;
pub(crate) use source::Sources;
pub(crate) use source::Topology;
pub(crate) use source::directory;
pub(crate) use source::normalize;
pub(crate) use source::strings;
pub(crate) use source::table;

use crate::errors::RunError;
use crate::source_built_fixed_point::SourceAuthorityRole;
use crate::source_built_program_authoring::AuthoringRequest;
use crate::source_built_rust_programs::resolve as source;
fn error(s: impl Into<String>) -> RunError {
    RunError::Build(format!("original minicargo resolution: {}", s.into()))
}

#[derive(Clone, Debug)]
struct Version {
    numbers: [u64; 3],
    patch_set: bool,
}
impl Version {
    fn parse(text: &str) -> Result<Self, RunError> {
        let number = text.split(['+', '-']).next().unwrap_or(text);
        let mut numbers = [0; 3];
        let parts = number.split('.').collect::<Vec<_>>();
        if parts.len() > 3 {
            return Err(error("invalid original version"));
        }
        for (i, n) in parts.iter().enumerate() {
            numbers[i] = n.parse().map_err(|_| error(format!("invalid original version {text}")))?;
        }
        Ok(Self {
            numbers,
            patch_set: parts.len() == 3,
        })
    }
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.numbers[..2].cmp(&other.numbers[..2]).then_with(|| {
            if self.patch_set && other.patch_set {
                self.numbers[2].cmp(&other.numbers[2])
            } else {
                std::cmp::Ordering::Equal
            }
        })
    }
    fn previous(&self) -> Self {
        if self.numbers[0] == 0 {
            self.clone()
        } else {
            Self {
                numbers: [self.numbers[0], self.numbers[1], 0],
                patch_set: true,
            }
        }
    }
    fn next_minor(&self) -> Self {
        Self {
            numbers: [self.numbers[0], self.numbers[1] + 1, 0],
            patch_set: false,
        }
    }
    fn next_breaking(&self) -> Self {
        if self.numbers[0] == 0 {
            self.next_minor()
        } else {
            Self {
                numbers: [self.numbers[0] + 1, 0, 0],
                patch_set: false,
            }
        }
    }
    fn family(&self) -> [u64; 2] {
        if self.numbers[0] == 0 {
            [0, self.numbers[1]]
        } else {
            [self.numbers[0], 0]
        }
    }
}
fn accepts(requirement: &str, version: &Version) -> Result<bool, RunError> {
    for clause in requirement.split(',') {
        let clause = clause.trim();
        if clause.is_empty() || clause == "*" {
            continue;
        }
        let (op, number) = [">=", "<=", ">", "<", "=", "~", "^"]
            .into_iter()
            .find_map(|op| clause.strip_prefix(op).map(|v| (op, v.trim())))
            .unwrap_or(("^", clause));
        let components = number.split('.').collect::<Vec<_>>();
        if components.iter().any(|c| matches!(*c, "*" | "x" | "X")) {
            for (i, n) in components.iter().take(3).enumerate() {
                if matches!(*n, "*" | "x" | "X") {
                    break;
                }
                if version.numbers[i] != n.parse::<u64>().map_err(|_| error("invalid wildcard requirement"))? {
                    return Ok(false);
                }
            }
            continue;
        }
        let lower = Version::parse(number)?;
        // manifest.h deliberately ignores prerelease/free text and compares a
        // patch only when BOTH operands have patch_set. Its ^/~ lower bound is
        // prev_compat, not Cargo's conventional lower bound; preserve that.
        let cmp = version.cmp(&lower);
        let good = match op {
            ">=" => cmp.is_ge(),
            "<=" => cmp.is_le(),
            ">" => cmp.is_gt(),
            "<" => cmp.is_lt(),
            "=" => cmp.is_eq(),
            "~" => {
                version.cmp(&lower.previous()).is_ge()
                    && version
                        .cmp(&if components.len() == 1 {
                            lower.next_breaking()
                        } else {
                            lower.next_minor()
                        })
                        .is_lt()
            }
            "^" => version.cmp(&lower.previous()).is_ge() && version.cmp(&lower.next_breaking()).is_lt(),
            _ => return Err(error("unknown original requirement operator")),
        };
        if !good {
            return Ok(false);
        }
    }
    Ok(true)
}

fn cfg(text: &str, features: &BTreeSet<String>) -> Result<bool, RunError> {
    let text = text.trim();
    for name in ["cfg", "all", "any", "not"] {
        if let Some(inner) = text.strip_prefix(&format!("{name}(")).and_then(|s| s.strip_suffix(')')) {
            let mut values = Vec::new();
            let mut depth = 0;
            let mut quoted = false;
            let mut start = 0;
            for (i, c) in inner.char_indices() {
                match c {
                    '"' => quoted = !quoted,
                    '(' if !quoted => depth += 1,
                    ')' if !quoted => depth -= 1,
                    ',' if !quoted && depth == 0 => {
                        values.push(cfg(&inner[start..i], features)?);
                        start = i + 1;
                    }
                    _ => (),
                }
            }
            if !inner[start..].trim().is_empty() {
                values.push(cfg(&inner[start..], features)?);
            }
            return match name {
                "not" if values.len() == 1 => Ok(!values[0]),
                "not" => Err(error("invalid original not cfg")),
                "any" => Ok(values.into_iter().any(|b| b)),
                _ => Ok(values.into_iter().all(|b| b)),
            };
        }
    }
    if let Some((name, value)) = text.split_once('=') {
        let value = value.trim().trim_matches('"');
        return Ok(match name.trim() {
            "feature" => features.contains(value),
            "rust_compiler" => value == "mrustc",
            "target_arch" => value == "x86_64",
            "target_os" => value == "linux",
            "target_env" => value == "musl",
            "target_family" => value == "unix",
            "target_vendor" => value == "gnu" || value.is_empty(),
            "target_pointer_width" => value == "64",
            "target_endian" => value == "little",
            "target_abi" => value == "llvm",
            "target_has_atomic" => ["8", "16", "32", "64", "ptr", "cas"].contains(&value),
            "target_has_atomic_load_store" | "target_has_atomic_equal_alignment" => {
                ["8", "16", "32", "64", "ptr"].contains(&value)
            }
            _ => false,
        });
    }
    // target.cpp's original Linux cfgs include linux+unix; callback-only values
    // (feature and target_feature) aren't present in Cfg_Dump's queried table.
    Ok(matches!(text, "unix" | "linux" | "x86_64-unknown-linux-musl"))
}
fn dep_tables(manifest: &toml::Value) -> Vec<(String, String, toml::Value)> {
    let mut result = Vec::new();
    for kind in ["dependencies", "build-dependencies"] {
        if let Some(t) = table(manifest, kind) {
            for (name, v) in t {
                result.push((kind.into(), name.clone(), v.clone()));
            }
        }
    }
    if let Some(t) = table(manifest, "target") {
        for (target, groups) in t {
            for kind in ["dependencies", "build-dependencies"] {
                if let Some(t) = table(groups, kind) {
                    for (name, v) in t {
                        result.push((format!("target.{target}.{kind}"), name.clone(), v.clone()));
                    }
                }
            }
        }
    }
    result
}
fn inherited(s: &Sources, manifest: &str, alias: &str, value: &toml::Value) -> Result<(String, toml::Value), RunError> {
    let base = directory(manifest).to_string();
    if value.get("workspace").and_then(toml::Value::as_bool) != Some(true) {
        return Ok((base, value.clone()));
    }
    let (base, ws) = s.workspace(manifest).ok_or_else(|| error("missing original dependency workspace"))?;
    let mut result = ws
        .get("workspace")
        .and_then(|v| v.get("dependencies"))
        .and_then(|v| v.get(alias))
        .ok_or_else(|| error(format!("missing inherited edge {alias}")))?
        .clone();
    if let toml::Value::String(version) = result {
        result = toml::Value::Table(toml::map::Map::from_iter([("version".into(), toml::Value::String(version))]));
    }
    let target = result.as_table_mut().ok_or_else(|| error("invalid inherited dependency"))?;
    let mut features = strings(target.get("features"));
    features.extend(strings(value.get("features")));
    target.insert("features".into(), toml::Value::Array(features.into_iter().map(toml::Value::String).collect()));
    for field in ["optional", "default-features"] {
        if let Some(v) = value.get(field) {
            target.insert(field.into(), v.clone());
        }
    }
    Ok((base.into(), result))
}
fn patch_path(s: &Sources, owner: &str, name: &str) -> Result<Option<String>, RunError> {
    if let Some((base, p)) = s
        .workspace(owner)
        .and_then(|(base, w)| w.get("patch")?.get("crates-io")?.get(name).map(|p| (base, p)))
    {
        let path = p.get("path").and_then(toml::Value::as_str).ok_or_else(|| error("non-source dependency patch"))?;
        return Ok(Some(format!("{}/Cargo.toml", normalize(base, path)?)));
    }
    Ok(None)
}
#[derive(Clone)]
struct Pending {
    owner: String,
    alias: String,
    value: toml::Value,
}
#[derive(Clone, Default)]
struct Selection {
    queue: VecDeque<Pending>,
    visited: BTreeSet<String>,
    selected: BTreeSet<String>,
    edges: BTreeMap<(String, String), String>,
}
fn push(s: &Sources, state: &mut Selection, path: &str) -> Result<(), RunError> {
    if !state.visited.insert(path.into()) {
        return Ok(());
    }
    let manifest = s.manifests.get(path).ok_or_else(|| {
        error(
            s.policy
                .get(&format!("invalid-manifest:{path}"))
                .map(|e| format!("selected original manifest {path}: {e}"))
                .unwrap_or_else(|| format!("missing original path {path}")),
        )
    })?;
    for (kind, alias, value) in dep_tables(manifest) {
        if let Some(rest) = kind.strip_prefix("target.") {
            let (predicate, _) = rest.rsplit_once('.').ok_or_else(|| error("invalid original target group"))?;
            if !cfg(predicate, &BTreeSet::new())? {
                continue;
            }
        }
        state.queue.push_back(Pending {
            owner: path.into(),
            alias,
            value,
        });
    }
    Ok(())
}
fn choose(s: &Sources, mut state: Selection, depth: usize) -> Result<Option<Selection>, RunError> {
    if depth > 16384 {
        return Err(error("original dependency resolver exceeded finite graph bound"));
    }
    while let Some(dep) = state.queue.pop_front() {
        let (base, value) = inherited(s, &dep.owner, &dep.alias, &dep.value)?;
        let name = value.get("package").and_then(toml::Value::as_str).unwrap_or(&dep.alias);
        let path = if let Some(path) = value.get("path").and_then(toml::Value::as_str) {
            Some(format!("{}/Cargo.toml", normalize(&base, path)?))
        } else {
            patch_path(s, &dep.owner, name)?
        };
        if let Some(path) = path {
            state.edges.insert((dep.owner.clone(), dep.alias), path.clone());
            push(s, &mut state, &path)?;
            continue;
        }
        let requirement = value.as_str().or_else(|| value.get("version").and_then(toml::Value::as_str)).unwrap_or("*");
        let mut candidates = Vec::new();
        for (path, m) in &s.manifests {
            if !path.starts_with("vendor/") {
                continue;
            }
            if m.get("package").and_then(|p| p.get("name")).and_then(toml::Value::as_str) != Some(name) {
                continue;
            }
            let v = Version::parse(
                m.get("package")
                    .and_then(|p| p.get("version"))
                    .and_then(toml::Value::as_str)
                    .ok_or_else(|| error("original vendor lacks version"))?,
            )?;
            if accepts(requirement, &v)? {
                candidates.push((v, path.clone()));
            }
        }
        // Original resolve_cargo.cpp soft-fails absent optional vendor packages;
        // load_dependencies later rejects one if its feature actually enables it.
        if candidates.is_empty() {
            continue;
        }
        candidates.sort_by(|a, b| a.0.cmp(&b.0));
        if let Some((_, path)) = candidates.iter().find(|(_, path)| state.selected.contains(path)) {
            state.edges.insert((dep.owner.clone(), dep.alias), path.clone());
            continue;
        }
        for (version, path) in candidates.into_iter().rev() {
            let conflict = state.selected.iter().any(|p| {
                s.manifests[p].get("package").and_then(|v| v.get("name")).and_then(toml::Value::as_str) == Some(name)
                    && Version::parse(s.manifests[p]["package"]["version"].as_str().unwrap_or(""))
                        .map(|v| v.family() == version.family())
                        .unwrap_or(false)
            });
            if conflict {
                continue;
            }
            let mut branch = state.clone();
            branch.edges.insert((dep.owner.clone(), dep.alias.clone()), path.clone());
            branch.selected.insert(path.clone());
            push(s, &mut branch, &path)?;
            if let Some(done) = choose(s, branch, depth + 1)? {
                return Ok(Some(done));
            }
        }
        return Ok(None);
    }
    Ok(Some(state))
}
pub(crate) fn archive(request: &AuthoringRequest<'_>, source_id: &str) -> Result<std::path::PathBuf, RunError> {
    const ROUTE: &str = include_str!("../bootstrap/rust-source-musl-host-plan.ncl");
    let marker = format!("id = \"{source_id}\"");
    let pin = ROUTE
        .split(&marker)
        .nth(1)
        .and_then(|s| s.split("},").next())
        .and_then(|s| s.split("sha256_hex = \"").nth(1))
        .and_then(|s| s.split('"').next())
        .ok_or_else(|| error("source is not pinned by original route"))?;
    let root = request
        .source_roots
        .get(&SourceAuthorityRole::RustSourceArchiveSet)
        .ok_or_else(|| error("missing original source archive authority"))?;
    let mut pending = vec![root.clone()];
    let mut found = None;
    let mut count = 0;
    let mut buffer = [0u8; 65536];
    while let Some(path) = pending.pop() {
        count += 1;
        if count > 4096 {
            return Err(error("source authority entry bound exceeded"));
        }
        let metadata = std::fs::symlink_metadata(&path).map_err(|e| error(e.to_string()))?;
        if metadata.is_dir() {
            for entry in std::fs::read_dir(path).map_err(|e| error(e.to_string()))? {
                pending.push(entry.map_err(|e| error(e.to_string()))?.path());
            }
        } else if metadata.is_file() {
            let mut file = std::fs::File::open(&path).map_err(|e| error(e.to_string()))?;
            let mut hash = Sha256::new();
            loop {
                let n = file.read(&mut buffer).map_err(|e| error(e.to_string()))?;
                if n == 0 {
                    break;
                }
                hash.update(&buffer[..n]);
            }
            if format!("{:x}", hash.finalize()) == pin && found.replace(path).is_some() {
                return Err(error("duplicate authenticated original archive"));
            }
        }
    }
    found.ok_or_else(|| error(format!("original archive authority lacks {source_id} SHA256 {pin}")))
}

/// Source inventory sizes only: these are resource reservation hints, never
/// claims about an exact command count or a complete macro expansion bound.
#[derive(Default)]
pub(crate) struct ReservationHints {
    pub macro_sites: usize,
    pub native_files: usize,
}

pub(crate) fn source_site_hint(source: &[u8]) -> usize {
    let mut i = 0;
    let mut sites = 0;
    let mut identifier = false;
    let mut attribute = false;
    while i < source.len() {
        let c = source[i];
        if c.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if source[i..].starts_with(b"//") {
            while i < source.len() && source[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if source[i..].starts_with(b"/*") {
            i += 2;
            let mut depth = 1;
            while i < source.len() && depth > 0 {
                if source[i..].starts_with(b"/*") {
                    depth += 1;
                    i += 2;
                } else if source[i..].starts_with(b"*/") {
                    depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            continue;
        }
        // Raw string contents (including byte/C string prefixes) are opaque.
        let raw = if c == b'r' {
            Some(i + 1)
        } else if matches!(c, b'b' | b'c') && source.get(i + 1) == Some(&b'r') {
            Some(i + 2)
        } else {
            None
        };
        if let Some(mut quote) = raw {
            while source.get(quote) == Some(&b'#') {
                quote += 1;
            }
            if source.get(quote) == Some(&b'"') {
                let hashes = quote - raw.unwrap();
                i = quote + 1;
                while i < source.len() {
                    if source[i] == b'"'
                        && source.get(i + 1..i + 1 + hashes).map(|s| s.iter().all(|c| *c == b'#')).unwrap_or(false)
                    {
                        i += 1 + hashes;
                        break;
                    }
                    i += 1;
                }
                identifier = false;
                attribute = false;
                continue;
            }
        }
        if c == b'"' {
            i += 1;
            while i < source.len() {
                if source[i] == b'\\' {
                    i = (i + 2).min(source.len());
                } else if source[i] == b'"' {
                    i += 1;
                    break;
                } else {
                    i += 1;
                }
            }
            identifier = false;
            attribute = false;
            continue;
        }
        // A lifetime is not a character literal. Skip only a closed scalar or
        // escaped scalar; this also handles byte character literals.
        if c == b'\'' {
            let mut end = i + 1;
            if source.get(end) == Some(&b'\\') {
                end += 1;
                if source.get(end) == Some(&b'u') && source.get(end + 1) == Some(&b'{') {
                    end += 2;
                    while end < source.len() && source[end] != b'}' {
                        end += 1;
                    }
                    end += 1;
                } else if source.get(end) == Some(&b'x') {
                    end += 3;
                } else {
                    end += 1;
                }
            } else if let Some(first) = source.get(end) {
                end += if *first < 0x80 {
                    1
                } else if *first < 0xe0 {
                    2
                } else if *first < 0xf0 {
                    3
                } else {
                    4
                };
            }
            if source.get(end) == Some(&b'\'') {
                i = end + 1;
                identifier = false;
                attribute = false;
                continue;
            }
        }
        if c.is_ascii_alphabetic() || c == b'_' || c >= 0x80 {
            i += 1;
            while i < source.len() && (source[i].is_ascii_alphanumeric() || source[i] == b'_' || source[i] >= 0x80) {
                i += 1;
            }
            identifier = true;
            attribute = false;
            continue;
        }
        if c == b'#' {
            attribute = true;
            identifier = false;
        } else if c == b'!' && attribute {
            identifier = false;
        } else {
            if c == b'!' && identifier && source.get(i + 1) != Some(&b'=') {
                sites += 1;
            }
            if c == b'[' && attribute {
                sites += 1;
            }
            identifier = false;
            attribute = false;
        }
        i += 1;
    }
    sites
}

pub(crate) fn reservation_hints(
    request: &AuthoringRequest<'_>,
    sources: &Sources,
    selected: &BTreeSet<String>,
) -> Result<BTreeMap<String, ReservationHints>, RunError> {
    let path = archive(request, "rust-1.90.0")?;
    let mut archive =
        tar::Archive::new(flate2::read::GzDecoder::new(std::fs::File::open(path).map_err(|e| error(e.to_string()))?));
    let mut hints = BTreeMap::<String, ReservationHints>::new();
    let mut bytes = Vec::new();
    for entry in archive.entries().map_err(|e| error(e.to_string()))? {
        let mut entry = entry.map_err(|e| error(e.to_string()))?;
        if !entry.header().entry_type().is_file() {
            continue;
        }
        let path = entry.path().map_err(|e| error(e.to_string()))?.to_string_lossy().into_owned();
        let Some(path) = path.strip_prefix("rustc-1.90.0-src/") else {
            continue;
        };
        let mut dir = directory(path);
        let owner = loop {
            let manifest = if dir.is_empty() {
                "Cargo.toml".to_string()
            } else {
                format!("{dir}/Cargo.toml")
            };
            if sources.members.contains(&manifest) {
                break Some(manifest);
            }
            if dir.is_empty() {
                break None;
            }
            dir = directory(dir);
        };
        let Some(owner) = owner.filter(|owner| selected.contains(owner)) else {
            continue;
        };
        let extension = path.rsplit('.').next().unwrap_or("");
        if extension == "rs" {
            bytes.clear();
            entry.read_to_end(&mut bytes).map_err(|e| error(e.to_string()))?;
            let hint = hints.entry(owner).or_default();
            hint.macro_sites = hint
                .macro_sites
                .checked_add(source_site_hint(&bytes))
                .ok_or_else(|| error("macro source-site reservation hint overflow"))?;
        } else if matches!(extension, "c" | "cc" | "cpp" | "cxx" | "c++" | "C" | "S" | "s" | "asm") {
            let hint = hints.entry(owner).or_default();
            hint.native_files = hint
                .native_files
                .checked_add(1)
                .ok_or_else(|| error("native source-file reservation hint overflow"))?;
        }
    }
    Ok(hints)
}

pub(crate) fn load(request: &AuthoringRequest<'_>) -> Result<Sources, RunError> {
    let path = archive(request, "rust-1.90.0")?;
    let mut archive =
        tar::Archive::new(flate2::read::GzDecoder::new(std::fs::File::open(path).map_err(|e| error(e.to_string()))?));
    let mut s = Sources {
        top: "rustc-1.90.0-src".into(),
        manifests: BTreeMap::new(),
        locks: BTreeMap::new(),
        members: BTreeSet::new(),
        policy: BTreeMap::new(),
    };
    for entry in archive.entries().map_err(|e| error(e.to_string()))? {
        let mut entry = entry.map_err(|e| error(e.to_string()))?;
        let path = entry.path().map_err(|e| error(e.to_string()))?.to_string_lossy().into_owned();
        let Some(path) = path.strip_prefix("rustc-1.90.0-src/") else {
            continue;
        };
        crate::source_built_recipe::relative(path, false)?;
        let kind = entry.header().entry_type();
        if !(kind.is_file() || kind.is_symlink() || kind.is_hard_link()) {
            continue;
        }
        if !s.members.insert(path.into()) {
            return Err(error("duplicate original archive member"));
        }
        if path != "Cargo.toml" && !path.ends_with("/Cargo.toml") {
            continue;
        }
        if !kind.is_file() {
            return Err(error("nonregular original package manifest"));
        }
        if entry.size() > 4 * 1024 * 1024 {
            return Err(error("original manifest exceeds resource bound"));
        }
        let mut text = String::new();
        entry.read_to_string(&mut text).map_err(|e| error(e.to_string()))?;
        match text.parse::<toml::Value>() {
            Ok(value) => {
                s.manifests.insert(path.into(), value);
            }
            Err(e) => {
                s.policy.insert(format!("invalid-manifest:{path}"), e.to_string());
            }
        }
    }
    // Original minicargo parses only selected packages. Cargo's release source
    // intentionally contains invalid Cargo.toml test fixtures; retain their
    // diagnostics and reject one if selected, rather than parsing test data as
    // a build package or treating a lockfile as executable selection authority.
    // Match the cold selected driver_rlib source patch, preserving its exact
    // original dylib anchor rather than inventing a second manifest profile.
    let kinds = s
        .manifests
        .get_mut("compiler/rustc_driver/Cargo.toml")
        .and_then(|m| m.get_mut("lib"))
        .and_then(|lib| lib.get_mut("crate-type"))
        .ok_or_else(|| error("original driver manifest lacks selected crate type"))?;
    if kinds.as_array().map(|kinds| kinds.iter().map(toml::Value::as_str).collect::<Vec<_>>())
        != Some(vec![Some("dylib")])
    {
        return Err(error("original driver crate type differs from selected source patch anchor"));
    }
    *kinds = toml::Value::Array(vec![toml::Value::String("rlib".into())]);
    // Original --manifest-overrides rustc-1.90.0-overrides.toml, not a Cargo
    // compatibility invention: these are its complete two transformations.
    let core = s
        .manifests
        .get_mut("library/rustc-std-workspace-core/Cargo.toml")
        .ok_or_else(|| error("missing original workspace core"))?;
    let dependency = core
        .get_mut("dependencies")
        .and_then(|v| v.get_mut("compiler_builtins"))
        .and_then(toml::Value::as_table_mut)
        .ok_or_else(|| error("missing original compiler_builtins edge"))?;
    let mut features = strings(dependency.get("features"));
    features.push("no-asm".into());
    dependency.insert("features".into(), toml::Value::Array(features.into_iter().map(toml::Value::String).collect()));
    let proc = s
        .manifests
        .get_mut("library/proc_macro/Cargo.toml")
        .and_then(|v| v.get_mut("dependencies"))
        .and_then(toml::Value::as_table_mut)
        .ok_or_else(|| error("missing original proc_macro dependencies"))?;
    proc.remove("std");
    proc.remove("core");
    Ok(s)
}
pub(crate) fn resolve(s: &Sources, goals: &[(&str, &[&str])], scripts: bool) -> Result<Topology, RunError> {
    let mut state = Selection::default();
    for (path, _) in goals {
        push(s, &mut state, path)?;
    }
    let selected = choose(s, state, 0)?.ok_or_else(|| error("original version unification has no solution"))?;
    let mut local = Sources {
        top: s.top.clone(),
        manifests: s.manifests.clone(),
        locks: s.locks.clone(),
        members: s.members.clone(),
        policy: s.policy.clone(),
    };
    // Encode the resolver's own selected edges as source-path references. The
    // shared feature fixed point never consults an imported/frozen Cargo.lock.
    for ((owner, alias), path) in selected.edges {
        let original = local.manifests[&owner].clone();
        let base = directory(&owner);
        for (kind, name, value) in dep_tables(&original) {
            if name != alias {
                continue;
            }
            let (_, mut value) = inherited(s, &owner, &name, &value)?;
            if let toml::Value::String(v) = value {
                value = toml::Value::Table(toml::map::Map::from_iter([("version".into(), toml::Value::String(v))]));
            }
            let table = value.as_table_mut().ok_or_else(|| error("invalid dependency"))?;
            let from = base.split('/').filter(|s| !s.is_empty()).collect::<Vec<_>>();
            let to = directory(&path).split('/').filter(|s| !s.is_empty()).collect::<Vec<_>>();
            let shared = from.iter().zip(&to).take_while(|(a, b)| a == b).count();
            let relative = std::iter::repeat_n("..", from.len() - shared)
                .chain(to[shared..].iter().copied())
                .collect::<Vec<_>>()
                .join("/");
            table.insert("path".into(), toml::Value::String(if relative.is_empty() { ".".into() } else { relative }));
            let mut dest = local.manifests.get_mut(&owner).unwrap();
            if let Some(rest) = kind.strip_prefix("target.") {
                let (predicate, group) = rest.rsplit_once('.').ok_or_else(|| error("invalid target group"))?;
                dest = dest
                    .get_mut("target")
                    .and_then(|v| v.get_mut(predicate))
                    .and_then(|v| v.get_mut(group))
                    .ok_or_else(|| error("missing source target group"))?;
            } else {
                dest = dest.get_mut(&kind).ok_or_else(|| error("missing source group"))?;
            }
            dest.as_table_mut().ok_or_else(|| error("non-table group"))?.insert(name, value);
        }
    }
    if !scripts {
        for manifest in local.manifests.values_mut() {
            if let Some(p) = manifest.get_mut("package").and_then(toml::Value::as_table_mut) {
                p.insert("build".into(), toml::Value::Boolean(false));
            }
        }
    }
    activate(&local, goals)
}

fn activate(s: &Sources, goals: &[(&str, &[&str])]) -> Result<Topology, RunError> {
    // This route's host and target are identical. minicargo shares package
    // feature state across normal/build/proc-macro edges (resolver version 1).
    let roots = goals
        .iter()
        .map(|(path, _)| Key {
            manifest: (*path).into(),
            host: false,
        })
        .collect::<Vec<_>>();
    let mut pending = VecDeque::new();
    for ((_, features), key) in goals.iter().zip(&roots) {
        pending.push_back((key.clone(), features.iter().map(|s| (*s).to_string()).collect::<Vec<_>>(), true));
    }
    let mut requests: BTreeMap<Key, BTreeSet<String>> = BTreeMap::new();
    let mut packages = BTreeMap::new();
    while let Some((key, features, defaults)) = pending.pop_front() {
        let mut p = s.package(&key)?;
        let mut requested = requests.get(&key).cloned().unwrap_or_default();
        let before = requested.clone();
        requested.extend(features);
        let definitions = table(&p.manifest, "features");
        if defaults {
            requested.extend(strings(definitions.and_then(|v| v.get("default"))));
        }
        if requests.contains_key(&key) && before == requested {
            continue;
        }
        requests.insert(key.clone(), requested.clone());
        let mut active = requested;
        let mut forwarded: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let all_deps = dep_tables(&p.manifest);
        let mut definitions = definitions.cloned().unwrap_or_default();
        // manifest.cpp adds implicit dep:name entries for EVERY dependency,
        // including nonoptional dependencies and target/dev groups.
        for (_, alias, _) in &all_deps {
            let mut values = strings(definitions.get(alias));
            values.push(format!("dep:{alias}"));
            definitions
                .insert(alias.clone(), toml::Value::Array(values.into_iter().map(toml::Value::String).collect()));
        }
        loop {
            let old = (active.clone(), forwarded.clone());
            for feature in active.clone() {
                for sub in strings(definitions.get(&feature)) {
                    if let Some((dep, feat)) = sub.split_once('/') {
                        let (name, enable) = dep.strip_suffix('?').map_or((dep, true), |v| (v, false));
                        if enable {
                            active.insert(name.into());
                            active.insert(format!("dep:{name}"));
                        }
                        forwarded.entry(name.into()).or_default().insert(feat.into());
                    } else {
                        active.insert(sub);
                    }
                }
            }
            if old == (active.clone(), forwarded.clone()) {
                break;
            }
        }
        p.features = active.clone();
        for (kind, alias, original) in all_deps {
            let group = kind.rsplit('.').next().unwrap_or(&kind);
            if group == "build-dependencies" && p.script.is_none() {
                continue;
            }
            if let Some(rest) = kind.strip_prefix("target.") {
                let (predicate, _) = rest.rsplit_once('.').ok_or_else(|| error("invalid original target group"))?;
                if !cfg(predicate, &active)? {
                    continue;
                }
            }
            let (base, value) = inherited(s, &key.manifest, &alias, &original)?;
            if value.get("optional").and_then(toml::Value::as_bool) == Some(true)
                && !active.contains(&format!("dep:{alias}"))
            {
                continue;
            }
            let path = value.get("path").and_then(toml::Value::as_str).ok_or_else(|| {
                error(format!("selected dependency {} -> {alias} has no original vendored version", p.name))
            })?;
            let dependency = Key {
                manifest: format!("{}/Cargo.toml", normalize(&base, path)?),
                host: false,
            };
            let mut features = strings(value.get("features"));
            if let Some(f) = forwarded.get(&alias) {
                features.extend(f.iter().cloned());
            }
            let defaults = value.get("default-features").and_then(toml::Value::as_bool).unwrap_or(true);
            let edge = Edge {
                name: alias.replace('-', "_"),
                key: dependency.clone(),
            };
            if group == "build-dependencies" {
                p.build_dependencies.push(edge);
            } else {
                p.dependencies.push(edge);
            }
            pending.push_back((dependency, features, defaults));
        }
        packages.insert(key, p);
        if packages.len() > 4096 {
            return Err(error("original package graph exceeds bound"));
        }
    }
    fn visit(
        key: &Key,
        packages: &BTreeMap<Key, Package>,
        active: &mut BTreeSet<Key>,
        done: &mut BTreeSet<Key>,
        order: &mut Vec<Key>,
    ) -> Result<(), RunError> {
        if done.contains(key) {
            return Ok(());
        }
        if !active.insert(key.clone()) {
            return Err(error(format!("original build dependency cycle at {}", key.manifest)));
        }
        let p = &packages[key];
        for edge in p.dependencies.iter().chain(&p.build_dependencies) {
            visit(&edge.key, packages, active, done, order)?;
        }
        active.remove(key);
        done.insert(key.clone());
        order.push(key.clone());
        Ok(())
    }
    let mut order = Vec::new();
    let mut active = BTreeSet::new();
    let mut done = BTreeSet::new();
    for root in &roots {
        visit(root, &packages, &mut active, &mut done, &mut order)?;
    }
    Ok(Topology { packages, roots, order })
}
pub(crate) fn suffix(package: &Package) -> String {
    let mut suffix = format!("-{}", package.version.split('+').next().unwrap_or(&package.version).replace('.', "_"));
    if !package.features.is_empty() {
        let mut names = table(&package.manifest, "features")
            .into_iter()
            .flat_map(|t| t.keys())
            .filter(|n| n.as_str() != "default")
            .cloned()
            .collect::<BTreeSet<_>>();
        for (_, alias, _) in dep_tables(&package.manifest) {
            names.insert(alias);
        }
        let mut mask = 0u64;
        for (i, name) in names.iter().take(64).enumerate() {
            if package.features.contains(name) {
                mask |= 1u64 << i;
            }
        }
        suffix.push_str(&format!("_H{mask:x}"));
    }
    suffix
}
