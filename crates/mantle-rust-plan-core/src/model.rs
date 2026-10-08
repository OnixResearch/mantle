//! Structural facts and nominal admission for Rust-plan inputs.
//!
//! Every value here is supplied by an adapter. Admission validates bounds,
//! uniqueness, and cross-fact consistency, and returns typed blockers; the
//! core never reparses bytes or reads host state.

use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

/// Maximum admitted packages in one plan request.
pub const MAX_PACKAGES: u32 = 4_096;

/// Maximum admitted targets per package.
pub const MAX_TARGETS_PER_PACKAGE: u32 = 64;

/// Bound decoded package and target scalar values before allocating outputs.
pub(crate) const MAX_NATIVE_SCALAR_BYTES: usize = 1024 * 1024;

/// Maximum admitted dependencies per package.
pub const MAX_DEPENDENCIES_PER_PACKAGE: u32 = 512;

/// Maximum admitted features per package.
pub const MAX_FEATURES_PER_PACKAGE: u32 = 512;

/// Maximum admitted effect arguments.
pub const MAX_ARGS_PER_EFFECT: u32 = 512;

/// Maximum admitted environment entries per effect.
pub const MAX_ENVIRONMENT_ENTRIES_PER_EFFECT: u32 = 256;

/// One bounded typed blocker.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PlanBlocker {
    pub code: String,
    pub subject: String,
    pub message: String,
}

impl PlanBlocker {
    pub(crate) fn new(code: &str, subject: &str, message: &str) -> Self {
        debug_assert!(!code.is_empty() && !subject.is_empty() && !message.is_empty());
        Self {
            code: String::from(code),
            subject: String::from(subject),
            message: String::from(message),
        }
    }
}

/// The accepted bounded target triple heuristic. This is not a claim of full
/// rustc target specification support; the shell supplies the triple.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct NativeTargetClassification<'a> {
    pub os: &'static str,
    pub arch: &'a str,
    pub family: &'static str,
    pub vendor: &'a str,
    pub env: &'static str,
    pub abi: &'static str,
    pub endian: &'static str,
    pub pointer_width: &'static str,
    pub features: &'static str,
    pub is_unix: bool,
}

/// Select the native execution triple without looking up the host or target.
/// The accepted path uses the target triple only for target execution units.
pub fn select_native_execution_triple<'a>(execution_kind: &str, host: &'a str, target: &'a str) -> &'a str {
    if execution_kind == "target" { target } else { host }
}

/// Preserve the existing bounded target cfg / build-script environment facts.
pub fn classify_native_target_triple(triple: &str) -> NativeTargetClassification<'_> {
    const OS_MARKERS: &[(&str, &str)] = &[
        ("linux", "linux"),
        ("darwin", "darwin"),
        ("apple", "darwin"),
        ("windows", "windows"),
        ("msvc", "windows"),
        ("freebsd", "freebsd"),
        ("netbsd", "netbsd"),
        ("openbsd", "openbsd"),
        ("android", "android"),
    ];
    let os = OS_MARKERS
        .iter()
        .find_map(|(marker, os)| triple.contains(marker).then_some(*os))
        .unwrap_or("unknown");
    let is_unix = matches!(os, "linux" | "darwin" | "freebsd" | "netbsd" | "openbsd" | "dragonfly" | "android");
    let arch = triple.split('-').next().unwrap_or("unknown");
    let family = if is_unix {
        "unix"
    } else if os == "windows" {
        "windows"
    } else if triple.contains("wasm") {
        "wasm"
    } else {
        "unknown"
    };
    let vendor = triple.split('-').nth(1).unwrap_or("unknown");
    let env = if triple.contains("musl") {
        "musl"
    } else if triple.contains("msvc") {
        "msvc"
    } else if triple.contains("gnu") {
        "gnu"
    } else {
        ""
    };
    let abi = if triple.contains("llvm") { "llvm" } else { "" };
    let pointer_width = match arch {
        "x86_64" | "aarch64" | "riscv64" | "powerpc64" | "s390x" | "wasm64" => "64",
        _ => "32",
    };
    let features = match arch {
        "x86_64" => "fxsr,sse,sse2,x87",
        "wasm32" => "bulk-memory,multivalue,mutable-globals,nontrapping-fptoint,reference-types,sign-ext",
        _ => "",
    };
    NativeTargetClassification {
        os,
        arch,
        family,
        vendor,
        env,
        abi,
        endian: "little",
        pointer_width,
        features,
        is_unix,
    }
}

/// Borrowed scalar from an adapter-decoded manifest or workspace package.
/// The adapter owns TOML and path I/O; the core owns fallback precedence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeInheritedValue<'a> {
    pub literal: Option<&'a str>,
    pub workspace: bool,
    pub workspace_value: Option<&'a str>,
}

fn validate_native_package_name(name: &str) -> Result<(), PlanBlocker> {
    if name.is_empty() || name.len() > MAX_NATIVE_SCALAR_BYTES {
        return Err(PlanBlocker::new(
            "package-identity",
            "packages",
            "native package name must be non-empty and within the admitted scalar bound",
        ));
    }
    Ok(())
}

fn admit_native_package_scalar(package_name: &str, value: &str) -> Result<String, PlanBlocker> {
    if value.is_empty() || value.len() > MAX_NATIVE_SCALAR_BYTES {
        return Err(PlanBlocker::new(
            "package-identity",
            package_name,
            "native package version or edition must be non-empty and within the admitted scalar bound",
        ));
    }
    Ok(String::from(value))
}

pub fn resolve_native_package_version(
    package_name: &str,
    value: NativeInheritedValue<'_>,
) -> Result<String, PlanBlocker> {
    validate_native_package_name(package_name)?;
    if let Some(literal) = value.literal {
        return admit_native_package_scalar(package_name, literal);
    }
    if !value.workspace {
        return Err(PlanBlocker::new(
            "missing-package-version",
            package_name,
            &alloc::format!("package `{package_name}` lacks a literal version or workspace version inheritance"),
        ));
    }
    let workspace = value.workspace_value.ok_or_else(|| PlanBlocker::new(
        "missing-workspace-package-version",
        package_name,
        &alloc::format!(
            "package `{package_name}` inherits version from [workspace.package], but no workspace package version is declared"
        ),
    ))?;
    admit_native_package_scalar(package_name, workspace)
}

pub fn resolve_native_package_edition(
    package_name: &str,
    value: NativeInheritedValue<'_>,
) -> Result<String, PlanBlocker> {
    validate_native_package_name(package_name)?;
    if let Some(literal) = value.literal {
        return admit_native_package_scalar(package_name, literal);
    }
    if !value.workspace {
        return Ok(String::from("2015"));
    }
    let workspace = value.workspace_value.ok_or_else(|| {
        PlanBlocker::new(
            "missing-workspace-package-edition",
            package_name,
            &alloc::format!(
                "package `{package_name}` inherits workspace edition but workspace.package.edition is missing"
            ),
        )
    })?;
    admit_native_package_scalar(package_name, workspace)
}

/// A target the shell discovered in declared order, with path evidence.
/// A failed readability probe supplies its display path only on that branch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeTargetCandidate {
    pub name: String,
    pub kind: String,
    pub source_path: String,
    pub edition: String,
    pub source_failure_display: Option<String>,
}

/// Native target admitted from the shell's source readability observation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NativeAdmittedTarget {
    pub name: String,
    pub kind: String,
    pub crate_name: String,
    pub source_path: String,
    pub edition: String,
}

pub fn admit_native_targets(
    package_id: &str,
    candidates: Vec<NativeTargetCandidate>,
) -> Result<Vec<NativeAdmittedTarget>, PlanBlocker> {
    if package_id.is_empty() {
        return Err(PlanBlocker::new("package-identity", "packages", "native target package ID must be non-empty"));
    }
    if candidates.len() > MAX_TARGETS_PER_PACKAGE as usize {
        return Err(PlanBlocker::new("target-limit", package_id, "package targets exceed the admitted target bound"));
    }
    let mut targets = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        if candidate.source_failure_display.as_ref().is_some_and(|display| {
            candidate.name.len() > MAX_NATIVE_SCALAR_BYTES || display.len() > MAX_NATIVE_SCALAR_BYTES
        }) {
            return Err(PlanBlocker::new(
                "target-identity-limit",
                package_id,
                "native target identity fields exceed the admitted scalar bound",
            ));
        }
        if let Some(source_display) = candidate.source_failure_display {
            return Err(PlanBlocker::new(
                "missing-target-source",
                package_id,
                &alloc::format!("target `{}` source {} is not readable", candidate.name, source_display,),
            ));
        }
        if candidate.name.is_empty()
            || candidate.kind.is_empty()
            || candidate.source_path.is_empty()
            || candidate.edition.is_empty()
        {
            return Err(PlanBlocker::new(
                "target-identity",
                package_id,
                "native target name, kind, source path and edition must be non-empty",
            ));
        }
        if candidate.name.len() > MAX_NATIVE_SCALAR_BYTES
            || candidate.kind.len() > MAX_NATIVE_SCALAR_BYTES
            || candidate.source_path.len() > MAX_NATIVE_SCALAR_BYTES
            || candidate.edition.len() > MAX_NATIVE_SCALAR_BYTES
        {
            return Err(PlanBlocker::new(
                "target-identity-limit",
                package_id,
                "native target identity fields exceed the admitted scalar bound",
            ));
        }
        if !matches!(candidate.kind.as_str(), "lib" | "bin" | "custom-build" | "proc-macro") {
            return Err(PlanBlocker::new(
                "unsupported-target-kind",
                package_id,
                "native target kind is outside the supported lib/bin/custom-build/proc-macro fragment",
            ));
        }
        targets.push(NativeAdmittedTarget {
            crate_name: candidate.name.replace('-', "_"),
            name: candidate.name,
            kind: candidate.kind,
            source_path: candidate.source_path,
            edition: candidate.edition,
        });
    }
    if targets.is_empty() {
        return Err(PlanBlocker::new(
            "missing-supported-target",
            package_id,
            "native package/target fragment found no readable lib/bin target source",
        ));
    }
    targets.sort();
    targets.dedup();
    Ok(targets)
}

/// One admitted native package identity and its already selected, resolved
/// normal/build dependency package IDs. The shell resolves paths, source
/// versions, optional activation and target cfg before supplying these facts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedPackageFacts {
    pub package_id: String,
    pub package_name: String,
    pub version: String,
    pub source_identity: String,
    /// Adapter-normalized ordering key, matching the accepted manifest order.
    /// The core compares this as an opaque identity and never resolves a path.
    pub order_key: String,
    pub selected_features: Vec<String>,
    pub selected_dependencies: Vec<String>,
}

/// Admit actual package IDs, select their reachable dependency closure, and
/// return package IDs in the accepted deterministic manifest order.
///
/// The shell supplies only matched workspace roots (unmatched paths were
/// silently ignored by the native planner), or every package ID when the
/// workspace root set is empty. Unresolved source edges remain adapter-owned
/// and retain their existing native-unit blockers.
pub fn select_normalized_package_closure(
    roots: &[String],
    packages: &[NormalizedPackageFacts],
) -> Result<Vec<String>, Vec<PlanBlocker>> {
    let mut blockers = Vec::new();
    if count_exceeds(packages.len(), MAX_PACKAGES) || count_exceeds(roots.len(), MAX_PACKAGES) {
        return Err(vec![PlanBlocker::new(
            "package-limit",
            "packages",
            "native package closure exceeds the admitted package bound",
        )]);
    }
    if roots.iter().any(|root| root.len() > MAX_NATIVE_SCALAR_BYTES) {
        return Err(vec![PlanBlocker::new(
            "package-root-limit",
            "roots",
            "native package root identities exceed the admitted scalar bound",
        )]);
    }
    let mut by_id = BTreeMap::new();
    let mut by_order = BTreeMap::new();
    for package in packages {
        if package.package_id.is_empty()
            || package.package_name.is_empty()
            || package.version.is_empty()
            || package.source_identity.is_empty()
            || package.order_key.is_empty()
        {
            blockers.push(PlanBlocker::new(
                "package-identity",
                "packages",
                "native package ID, name, version, source and order identity must be non-empty",
            ));
            continue;
        }
        if package.package_id.len() > MAX_NATIVE_SCALAR_BYTES
            || package.package_name.len() > MAX_NATIVE_SCALAR_BYTES
            || package.version.len() > MAX_NATIVE_SCALAR_BYTES
            || package.source_identity.len() > MAX_NATIVE_SCALAR_BYTES
            || package.order_key.len() > MAX_NATIVE_SCALAR_BYTES
        {
            blockers.push(PlanBlocker::new(
                "package-identity-limit",
                "packages",
                "native package identity fields exceed the admitted scalar bound",
            ));
            continue;
        }
        if package
            .selected_features
            .iter()
            .chain(&package.selected_dependencies)
            .any(|identity| identity.len() > MAX_NATIVE_SCALAR_BYTES)
        {
            blockers.push(PlanBlocker::new(
                "package-selection-identity-limit",
                &package.package_id,
                "selected features or dependency identities exceed the admitted scalar bound",
            ));
            continue;
        }
        if by_id.insert(package.package_id.as_str(), package).is_some() {
            blockers.push(PlanBlocker::new(
                "duplicate-package",
                &package.package_id,
                "native package identity appears more than once",
            ));
        }
        if by_order.insert(package.order_key.as_str(), package.package_id.as_str()).is_some() {
            blockers.push(PlanBlocker::new(
                "ambiguous-package-manifest",
                &package.package_id,
                "different native packages share an ordering identity",
            ));
        }
        if count_exceeds(package.selected_features.len(), MAX_FEATURES_PER_PACKAGE)
            || count_exceeds(package.selected_dependencies.len(), MAX_DEPENDENCIES_PER_PACKAGE)
        {
            blockers.push(PlanBlocker::new(
                "package-bound",
                &package.package_id,
                "native package features or selected dependencies exceed admitted bounds",
            ));
        }
        if package.selected_features.iter().any(String::is_empty)
            || package.selected_dependencies.iter().any(String::is_empty)
        {
            blockers.push(PlanBlocker::new(
                "package-selection-identity",
                &package.package_id,
                "selected features and resolved dependency IDs must be non-empty",
            ));
        }
    }
    if !blockers.is_empty() {
        blockers.sort();
        blockers.dedup();
        return Err(blockers);
    }
    for root in roots {
        if root.is_empty() || !by_id.contains_key(root.as_str()) {
            blockers.push(PlanBlocker::new(
                "missing-package-root",
                if root.is_empty() { "roots" } else { root },
                "matched workspace package root has no supplied package facts",
            ));
        }
    }
    for package in packages {
        for dependency in &package.selected_dependencies {
            if !by_id.contains_key(dependency.as_str()) {
                blockers.push(PlanBlocker::new(
                    "missing-dependency-package",
                    &package.package_id,
                    "selected dependency package has no normalized facts",
                ));
            }
        }
    }
    if !blockers.is_empty() {
        blockers.sort();
        blockers.dedup();
        return Err(blockers);
    }
    let mut queue: VecDeque<&str> = roots.iter().map(String::as_str).collect();
    let mut visited = BTreeSet::new();
    while let Some(package_id) = queue.pop_front() {
        if !visited.insert(package_id) {
            continue;
        }
        if visited.len() > packages.len() {
            return Err(vec![PlanBlocker::new(
                "native-unit-package-closure-limit-exceeded",
                package_id,
                "native unit package traversal exceeded package fact count",
            )]);
        }
        queue.extend(by_id[package_id].selected_dependencies.iter().map(String::as_str));
    }
    let mut selected =
        packages.iter().filter(|package| visited.contains(package.package_id.as_str())).collect::<Vec<_>>();
    selected.sort_by(|left, right| {
        (left.order_key.as_str(), left.package_id.as_str()).cmp(&(right.order_key.as_str(), right.package_id.as_str()))
    });
    Ok(selected.into_iter().map(|package| package.package_id.clone()).collect())
}

pub(crate) fn count_exceeds(count_items: usize, maximum_items: u32) -> bool {
    match u32::try_from(count_items) {
        Ok(count_items) => count_items > maximum_items,
        // A collection wider than the admitted bound is over the bound.
        Err(_) => true,
    }
}
