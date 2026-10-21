//! Cargo-free package selection from authenticated Rust release archives.
//! Lockfiles choose registry identities; manifests choose edges and features.
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::VecDeque;
use std::io::Read;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use sha2::Digest;
use sha2::Sha256;

use crate::errors::RunError;
use crate::source_built_fixed_point::SourceAuthorityRole;
use crate::source_built_program_authoring::AuthoringRequest;

fn error(message: impl Into<String>) -> RunError {
    RunError::Build(format!("Rust successor source topology: {}", message.into()))
}

pub(crate) struct Sources {
    pub top: String,
    pub manifests: BTreeMap<String, toml::Value>,
    pub locks: BTreeMap<String, toml::Value>,
    pub members: BTreeSet<String>,
    pub policy: BTreeMap<String, String>,
}
pub(crate) fn load(request: &AuthoringRequest<'_>, version: &str) -> Result<Sources, RunError> {
    let root = request
        .source_roots
        .get(&SourceAuthorityRole::RustSourceArchiveSet)
        .ok_or_else(|| error("missing original Rust archive authority"))?;
    let route = crate::source_built_rust_programs::source_route(request.source_roots)?;
    let identity = format!("rust-{version}");
    let pin = &route
        .sources
        .iter()
        .find(|source| source.id == identity && source.version == version)
        .ok_or_else(|| error(format!("version {version} is not pinned by the admitted original route")))?
        .sha256_hex;
    let mut pending = vec![root.to_path_buf()];
    let mut authenticated = None;
    let mut count = 0;
    let mut buffer = [0u8; 65536];
    while let Some(path) = pending.pop() {
        count += 1;
        if count > 4096 {
            return Err(error("archive authority entry bound exceeded"));
        }
        let metadata = std::fs::symlink_metadata(&path).map_err(|e| error(e.to_string()))?;
        if metadata.is_dir() {
            for entry in std::fs::read_dir(path).map_err(|e| error(e.to_string()))? {
                pending.push(entry.map_err(|e| error(e.to_string()))?.path());
            }
        } else if metadata.is_file() {
            let mut file = std::fs::File::open(&path).map_err(|e| error(e.to_string()))?;
            let mut digest = Sha256::new();
            loop {
                let n = file.read(&mut buffer).map_err(|e| error(e.to_string()))?;
                if n == 0 {
                    break;
                }
                digest.update(&buffer[..n]);
            }
            if format!("{:x}", digest.finalize()) == *pin && authenticated.replace(path).is_some() {
                return Err(error("duplicate authenticated original Rust archive"));
            }
        }
    }
    let archive =
        authenticated.ok_or_else(|| error(format!("original source authority lacks Rust {version} SHA256 {pin}")))?;
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(
        std::fs::File::open(archive).map_err(|e| error(e.to_string()))?,
    ));
    let mut result = Sources {
        top: format!("rustc-{version}-src"),
        manifests: BTreeMap::new(),
        locks: BTreeMap::new(),
        members: BTreeSet::new(),
        policy: BTreeMap::new(),
    };
    for entry in archive.entries().map_err(|e| error(e.to_string()))? {
        let mut entry = entry.map_err(|e| error(e.to_string()))?;
        let path = entry.path().map_err(|e| error(e.to_string()))?.into_owned();
        let path = path.to_str().ok_or_else(|| error("non-UTF8 original archive member"))?;
        let Some(path) = path.strip_prefix(&format!("{}/", result.top)) else {
            continue;
        };
        crate::source_built_recipe::relative(path, false)?;
        if !entry.header().entry_type().is_file() {
            continue;
        }
        if !result.members.insert(path.into()) {
            return Err(error("duplicate original source member"));
        }
        let manifest = path == "Cargo.toml" || path.ends_with("/Cargo.toml");
        let lock = path == "Cargo.lock" || path.ends_with("/Cargo.lock");
        let policy = path.starts_with("src/bootstrap/") && (path.ends_with(".rs") || path.ends_with(".toml"))
            || path.ends_with("/build.rs")
            || matches!(path, "version" | "git-commit-hash" | "git-commit-info");
        if !(manifest || lock || policy) {
            continue;
        }
        if entry.size() > 4 * 1024 * 1024 {
            return Err(error(format!("source planner member exceeds bound: {path}")));
        }
        let mut text = String::new();
        entry.read_to_string(&mut text).map_err(|e| error(e.to_string()))?;
        if manifest || lock {
            match text.parse::<toml::Value>() {
                Ok(mut value) => {
                    if manifest {
                        if path == "compiler/rustc_driver/Cargo.toml" {
                            let kinds = value
                                .get_mut("lib")
                                .and_then(|lib| lib.get_mut("crate-type"))
                                .ok_or_else(|| error("original driver manifest lacks selected crate type"))?;
                            if kinds.as_array().map(|kinds| kinds.iter().map(toml::Value::as_str).collect::<Vec<_>>())
                                != Some(vec![Some("dylib")])
                            {
                                return Err(error(
                                    "original driver crate type differs from selected source patch anchor",
                                ));
                            }
                            *kinds = toml::Value::Array(vec![toml::Value::String("rlib".into())]);
                        }
                        result.manifests.insert(path.into(), value);
                    } else {
                        result.locks.insert(path.into(), value);
                    }
                }
                Err(reason) => {
                    result.policy.insert(format!("invalid-manifest:{path}"), reason.to_string());
                }
            }
        } else {
            result.policy.insert(path.into(), text);
        }
    }
    Ok(result)
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Key {
    pub manifest: String,
    pub host: bool,
}
#[derive(Clone)]
pub(crate) struct Edge {
    pub name: String,
    pub key: Key,
}
#[derive(Clone)]
pub(crate) struct Package {
    pub key: Key,
    pub name: String,
    pub version: String,
    pub directory: String,
    pub manifest: toml::Value,
    pub features: BTreeSet<String>,
    pub dependencies: Vec<Edge>,
    pub build_dependencies: Vec<Edge>,
    pub script: Option<String>,
    pub edition: String,
    pub links: Option<String>,
}
pub(crate) struct Topology {
    pub packages: BTreeMap<Key, Package>,
    pub roots: Vec<Key>,
    pub order: Vec<Key>,
}
pub(crate) fn table<'a>(value: &'a toml::Value, name: &str) -> Option<&'a toml::map::Map<String, toml::Value>> {
    value.get(name)?.as_table()
}
pub(crate) fn strings(value: Option<&toml::Value>) -> Vec<String> {
    value
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(toml::Value::as_str)
        .map(str::to_string)
        .collect()
}
pub(crate) fn normalize(base: &str, path: &str) -> Result<String, RunError> {
    let mut parts = base.split('/').filter(|s| !s.is_empty()).map(str::to_string).collect::<Vec<_>>();
    for part in Path::new(path).components() {
        match part {
            Component::Normal(part) => parts.push(part.to_str().ok_or_else(|| error("non-UTF8 manifest path"))?.into()),
            Component::CurDir => (),
            Component::ParentDir => {
                parts.pop().ok_or_else(|| error("manifest path escapes original archive"))?;
            }
            _ => return Err(error("absolute manifest dependency path")),
        }
    }
    Ok(parts.join("/"))
}
pub(crate) fn directory(path: &str) -> &str {
    path.rsplit_once('/').map_or("", |(directory, _)| directory)
}
impl Sources {
    pub(crate) fn workspace(&self, manifest: &str) -> Option<(&str, &toml::Value)> {
        let mut path = PathBuf::from(directory(manifest));
        loop {
            let member = if path.as_os_str().is_empty() {
                "Cargo.toml".into()
            } else {
                format!("{}/Cargo.toml", path.display())
            };
            if let Some((manifest, value)) = self.manifests.get_key_value(&member) {
                if value.get("workspace").is_some() {
                    return Some((directory(manifest), value));
                }
                // An explicit workspace root is authoritative; do not inherit
                // from the enclosing compiler workspace when this package
                // belongs to the separate standard-library/Cargo workspace.
                if let Some(workspace) =
                    value.get("package").and_then(|package| package.get("workspace")).and_then(toml::Value::as_str)
                {
                    let root = normalize(directory(manifest), workspace).ok()?;
                    let manifest = if root.is_empty() {
                        "Cargo.toml".into()
                    } else {
                        format!("{root}/Cargo.toml")
                    };
                    let (manifest, value) = self.manifests.get_key_value(&manifest)?;
                    return value.get("workspace").map(|_| (directory(manifest), value));
                }
            }
            if !path.pop() {
                return None;
            }
        }
    }
    fn lock(&self, manifest: &str) -> Option<&toml::Value> {
        let mut path = PathBuf::from(directory(manifest));
        loop {
            let lock = if path.as_os_str().is_empty() {
                "Cargo.lock".into()
            } else {
                format!("{}/Cargo.lock", path.display())
            };
            if let Some(lock) = self.locks.get(&lock) {
                return Some(lock);
            }
            if !path.pop() {
                return None;
            }
        }
    }
    pub(crate) fn package(&self, key: &Key) -> Result<Package, RunError> {
        let manifest = self.manifests.get(&key.manifest).ok_or_else(|| {
            error(match self.policy.get(&format!("invalid-manifest:{}", key.manifest)) {
                Some(reason) => format!("selected original manifest {}: {reason}", key.manifest),
                None => format!("missing original manifest {}", key.manifest),
            })
        })?;
        let package =
            table(manifest, "package").ok_or_else(|| error(format!("manifest {} has no package", key.manifest)))?;
        let inherited = |field: &str, fallback: &str| -> Result<String, RunError> {
            if let Some(value) = package.get(field).and_then(toml::Value::as_str) {
                return Ok(value.into());
            }
            if package.get(field).and_then(|v| v.get("workspace")).and_then(toml::Value::as_bool) == Some(true) {
                return self
                    .workspace(&key.manifest)
                    .and_then(|(_, v)| v.get("workspace")?.get("package")?.get(field)?.as_str())
                    .map(str::to_string)
                    .ok_or_else(|| error(format!("unresolved workspace package {field}")));
            }
            Ok(fallback.into())
        };
        let directory = directory(&key.manifest).to_string();
        let script = match package.get("build") {
            Some(toml::Value::Boolean(false)) => None,
            Some(toml::Value::String(path)) => Some(normalize(&directory, path)?),
            None if self.members.contains(&format!("{directory}/build.rs")) => Some(format!("{directory}/build.rs")),
            None => None,
            _ => return Err(error("invalid original package build field")),
        };
        Ok(Package {
            key: key.clone(),
            name: inherited("name", "")?,
            version: inherited("version", "0.0.0")?,
            edition: inherited("edition", "2015")?,
            links: package.get("links").and_then(toml::Value::as_str).map(str::to_string),
            directory,
            manifest: manifest.clone(),
            features: BTreeSet::new(),
            dependencies: Vec::new(),
            build_dependencies: Vec::new(),
            script,
        })
    }
    fn resolve_dependency(
        &self,
        package: &Package,
        alias: &str,
        value: &toml::Value,
        host: bool,
    ) -> Result<(Key, Vec<String>, bool, bool), RunError> {
        let mut value = value.clone();
        let mut base = package.directory.clone();
        if value.get("workspace").and_then(toml::Value::as_bool) == Some(true) {
            let (workspace_dir, workspace) =
                self.workspace(&package.key.manifest).ok_or_else(|| error("missing dependency workspace"))?;
            let inherited = workspace
                .get("workspace")
                .and_then(|v| v.get("dependencies"))
                .and_then(|v| v.get(alias))
                .ok_or_else(|| error(format!("missing inherited dependency {alias}")))?;
            let mut inherited = match inherited {
                toml::Value::String(version) => {
                    toml::map::Map::from_iter([("version".into(), toml::Value::String(version.clone()))])
                }
                toml::Value::Table(table) => table.clone(),
                _ => return Err(error("invalid inherited dependency")),
            };
            let mut features = strings(inherited.get("features"));
            features.extend(strings(value.get("features")));
            inherited
                .insert("features".into(), toml::Value::Array(features.into_iter().map(toml::Value::String).collect()));
            for field in ["optional", "default-features", "public"] {
                if let Some(v) = value.get(field) {
                    inherited.insert(field.into(), v.clone());
                }
            }
            value = toml::Value::Table(inherited);
            base = workspace_dir.into();
        }
        let name = value.get("package").and_then(toml::Value::as_str).unwrap_or(alias);
        let path = if let Some(path) = value.get("path").and_then(toml::Value::as_str) {
            format!("{}/Cargo.toml", normalize(&base, path)?)
        } else if let Some((workspace_dir, patch)) = self
            .workspace(&package.key.manifest)
            .and_then(|(dir, v)| v.get("patch")?.get("crates-io")?.get(name).map(|p| (dir, p)))
        {
            let path = patch
                .get("path")
                .and_then(toml::Value::as_str)
                .ok_or_else(|| error("nonlocal original dependency patch"))?;
            format!("{}/Cargo.toml", normalize(workspace_dir, path)?)
        } else {
            let lock = self
                .lock(&package.key.manifest)
                .ok_or_else(|| error(format!("no original lockfile for {}", package.name)))?;
            let locked = lock
                .get("package")
                .and_then(toml::Value::as_array)
                .ok_or_else(|| error("missing original lock package table"))?;
            let owner = locked
                .iter()
                .find(|v| {
                    v.get("name").and_then(toml::Value::as_str) == Some(&package.name)
                        && v.get("version").and_then(toml::Value::as_str) == Some(&package.version)
                })
                .ok_or_else(|| error(format!("original lock lacks {} {}", package.name, package.version)))?;
            let matches = strings(owner.get("dependencies"))
                .into_iter()
                .filter(|v| v.split_whitespace().next() == Some(name))
                .collect::<Vec<_>>();
            if matches.len() != 1 {
                return Err(error(format!("ambiguous locked edge {} -> {name}: {matches:?}", package.name)));
            }
            let explicit_version = matches[0].split_whitespace().nth(1);
            let candidates = locked
                .iter()
                .filter(|v| {
                    v.get("name").and_then(toml::Value::as_str) == Some(name)
                        && explicit_version
                            .is_none_or(|version| v.get("version").and_then(toml::Value::as_str) == Some(version))
                })
                .collect::<Vec<_>>();
            if candidates.len() != 1 {
                return Err(error(format!("ambiguous original locked identity {name}")));
            }
            let version = candidates[0]
                .get("version")
                .and_then(toml::Value::as_str)
                .ok_or_else(|| error("locked package lacks version"))?;
            let candidates = self
                .manifests
                .iter()
                .filter(|(path, v)| {
                    path.starts_with("vendor/")
                        && v.get("package").and_then(|v| v.get("name")).and_then(toml::Value::as_str) == Some(name)
                        && v.get("package").and_then(|v| v.get("version")).and_then(toml::Value::as_str)
                            == Some(version)
                })
                .map(|(path, _)| path.clone())
                .collect::<Vec<_>>();
            if candidates.len() != 1 {
                return Err(error(format!("original vendor identity {name} {version} is not unique")));
            }
            candidates[0].clone()
        };
        Ok((
            Key { manifest: path, host },
            strings(value.get("features")),
            value.get("default-features").and_then(toml::Value::as_bool).unwrap_or(true),
            value.get("optional").and_then(toml::Value::as_bool).unwrap_or(false),
        ))
    }
}

// Evaluate source cfg syntax, not a host rustc observation. All selected route
// host/target triples are x86_64-unknown-linux-musl.
fn split_arguments(text: &str) -> Vec<&str> {
    let mut depth = 0;
    let mut quoted = false;
    let mut start = 0;
    let mut result = Vec::new();
    for (i, c) in text.char_indices() {
        match c {
            '"' => quoted = !quoted,
            '(' if !quoted => depth += 1,
            ')' if !quoted => depth -= 1,
            ',' if !quoted && depth == 0 => {
                result.push(text[start..i].trim());
                start = i + 1;
            }
            _ => (),
        }
    }
    if !text[start..].trim().is_empty() {
        result.push(text[start..].trim());
    }
    result
}
pub(crate) fn cfg(text: &str) -> Result<bool, RunError> {
    let text = text.trim();
    for op in ["cfg", "all", "any", "not"] {
        if let Some(inner) = text.strip_prefix(&format!("{op}(")).and_then(|v| v.strip_suffix(')')) {
            let values = split_arguments(inner).into_iter().map(cfg).collect::<Result<Vec<_>, _>>()?;
            return match op {
                "any" => Ok(values.into_iter().any(|v| v)),
                "not" if values.len() == 1 => Ok(!values[0]),
                "not" => Err(error("malformed original not cfg")),
                _ => Ok(values.into_iter().all(|v| v)),
            };
        }
    }
    if let Some((key, value)) = text.split_once('=') {
        let key = key.trim();
        let value = value.trim().trim_matches('"');
        return Ok(match key {
            "target_arch" => value == "x86_64",
            "target_os" => value == "linux",
            "target_env" => value == "musl",
            "target_vendor" => value == "unknown",
            "target_family" => value == "unix",
            "target_pointer_width" => value == "64",
            "target_endian" => value == "little",
            "target_abi" => value.is_empty(),
            "target_feature" => ["fxsr", "sse", "sse2"].contains(&value),
            _ => return Err(error(format!("unclassified original target cfg {text}"))),
        });
    }
    Ok(match text {
        "unix" => true,
        "windows" => false,
        "x86_64-unknown-linux-musl" => true,
        v if !v.starts_with("cfg(") && v.contains('-') => false,
        _ => return Err(error(format!("unclassified original target predicate {text}"))),
    })
}

pub(crate) fn resolve(sources: &Sources, goals: &[(&str, &[&str])]) -> Result<Topology, RunError> {
    resolve_with_scripts(sources, goals, true)
}
pub(crate) fn resolve_with_scripts(
    sources: &Sources,
    goals: &[(&str, &[&str])],
    build_scripts: bool,
) -> Result<Topology, RunError> {
    let roots = goals
        .iter()
        .map(|(path, _)| Key {
            manifest: (*path).into(),
            host: false,
        })
        .collect::<Vec<_>>();
    let mut packages = BTreeMap::new();
    let mut pending = VecDeque::new();
    for ((_, features), key) in goals.iter().zip(&roots) {
        pending.push_back((key.clone(), features.iter().map(|v| (*v).to_string()).collect::<Vec<_>>(), true));
    }
    let mut feature_requests: BTreeMap<Key, BTreeSet<String>> = BTreeMap::new();
    while let Some((key, features, defaults)) = pending.pop_front() {
        if packages.len() > 4096 {
            return Err(error("source package graph exceeds bound"));
        }
        let mut package = packages.remove(&key).unwrap_or(sources.package(&key)?);
        let before = feature_requests.get(&key).cloned();
        let requested = feature_requests.entry(key.clone()).or_default();
        requested.extend(features);
        if defaults && table(&package.manifest, "features").is_some_and(|features| features.contains_key("default")) {
            requested.insert("default".into());
        }
        if before.as_ref() == Some(requested) {
            packages.insert(key, package);
            continue;
        }
        let feature_defs = table(&package.manifest, "features");
        let mut active = requested.clone();
        let mut optional = BTreeSet::new();
        let mut forwarded: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let mut weak = Vec::new();
        loop {
            let before = (active.clone(), optional.clone(), forwarded.clone());
            for feature in active.clone() {
                if let Some((dependency, f)) = feature.split_once('/') {
                    if let Some(dependency) = dependency.strip_suffix('?') {
                        weak.push((dependency.to_string(), f.to_string()));
                    } else {
                        optional.insert(dependency.to_string());
                        forwarded.entry(dependency.into()).or_default().insert(f.into());
                    }
                } else if let Some(dependency) = feature.strip_prefix("dep:") {
                    optional.insert(dependency.into());
                } else if let Some(definition) = feature_defs.and_then(|v| v.get(&feature)) {
                    active.extend(strings(Some(definition)));
                } else {
                    optional.insert(feature);
                }
            }
            if before == (active.clone(), optional.clone(), forwarded.clone()) {
                break;
            }
        }
        active.retain(|f| !f.starts_with("dep:") && !f.contains('/'));
        package.features = active;
        let mut edges = Vec::new();
        for kind in ["dependencies", "build-dependencies"] {
            if kind == "build-dependencies" && (!build_scripts || package.script.is_none()) {
                continue;
            }
            if let Some(deps) = table(&package.manifest, kind) {
                for (name, value) in deps {
                    edges.push((kind, name.clone(), value.clone()));
                }
            }
            if let Some(targets) = table(&package.manifest, "target") {
                for (predicate, target) in targets {
                    if cfg(predicate)?
                        && let Some(deps) = table(target, kind)
                    {
                        for (name, value) in deps {
                            edges.push((kind, name.clone(), value.clone()));
                        }
                    }
                }
            }
        }
        package.dependencies.clear();
        package.build_dependencies.clear();
        for (kind, alias, value) in edges {
            // Cargo does not resolve disabled optional edges. In particular,
            // an absent lockfile edge for an unselected target/feature is not
            // an ambiguous dependency identity.
            if value.get("optional").and_then(toml::Value::as_bool) == Some(true) && !optional.contains(&alias) {
                continue;
            }
            let (mut dependency, mut features, defaults, is_optional) =
                sources.resolve_dependency(&package, &alias, &value, key.host || kind == "build-dependencies")?;
            if is_optional && !optional.contains(&alias) {
                continue;
            }
            if let Some(lib) = sources.manifests.get(&dependency.manifest).and_then(|m| m.get("lib"))
                && lib.get("proc-macro").and_then(toml::Value::as_bool) == Some(true)
            {
                dependency.host = true;
            }
            if let Some(f) = forwarded.get(&alias) {
                features.extend(f.iter().cloned());
            }
            for (name, feature) in &weak {
                if name == &alias {
                    features.push(feature.clone());
                }
            }
            let edge = Edge {
                name: alias.replace('-', "_"),
                key: dependency.clone(),
            };
            if kind == "dependencies" {
                package.dependencies.push(edge);
            } else {
                package.build_dependencies.push(edge);
            }
            pending.push_back((dependency, features, defaults));
        }
        packages.insert(key, package);
    }
    fn visit(
        key: &Key,
        packages: &BTreeMap<Key, Package>,
        active: &mut BTreeSet<Key>,
        visited: &mut BTreeSet<Key>,
        order: &mut Vec<Key>,
    ) -> Result<(), RunError> {
        if visited.contains(key) {
            return Ok(());
        }
        if !active.insert(key.clone()) {
            return Err(error(format!("original source package cycle at {}", key.manifest)));
        }
        let package = &packages[key];
        for edge in package.dependencies.iter().chain(&package.build_dependencies) {
            visit(&edge.key, packages, active, visited, order)?;
        }
        active.remove(key);
        visited.insert(key.clone());
        order.push(key.clone());
        Ok(())
    }
    let mut order = Vec::new();
    let mut visited = BTreeSet::new();
    let mut active = BTreeSet::new();
    for root in &roots {
        visit(root, &packages, &mut active, &mut visited, &mut order)?;
    }
    Ok(Topology { packages, roots, order })
}
