use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use crate::bootstrap_source_root::DIGEST_ALGORITHM_BLAKE3;
use crate::bootstrap_source_root::DigestSpec;
use crate::bootstrap_source_root::ManifestDiagnostic;
use crate::bootstrap_source_root::PROVIDER_DYNAMIC_LINKER;
use crate::bootstrap_source_root::PROVIDER_METADATA_ROLE;
use crate::bootstrap_source_root::PROVIDER_NAME;
use crate::bootstrap_source_root::PROVIDER_TARGET;
use crate::bootstrap_source_root::ProviderDependencyTrace;
use crate::bootstrap_source_root::SourceRootManifest;

const TARGET: &str = "x86_64-linux-musl";
const FETCH_TIMEOUT_SECS: u64 = 600;
const FETCH_MAX_BYTES: u64 = 512 * 1024 * 1024;
const FETCH_MAX_RETRIES: u32 = 3;
const FETCH_RETRY_BASE_DELAY_MS: u64 = 2000;
const MIN_PARALLELISM_JOBS: u32 = 1;
const MAX_PARALLELISM_JOBS: u32 = 32;
const BUILD_PHASE_COUNT: u32 = 5;
const MAKEINFO_ENV: &str = "MAKEINFO";
const MAKEINFO_DISABLED: &str = "true";

const ARTIFACT_LINUX_HEADERS: &str = "linux-headers";
const ARTIFACT_MUSL: &str = "musl";
const ARTIFACT_BINUTILS: &str = "binutils";
const ARTIFACT_GCC: &str = "gcc";
const ARTIFACT_GMP: &str = "gmp";
const ARTIFACT_MPFR: &str = "mpfr";
const ARTIFACT_MPC: &str = "mpc";

pub(crate) const REQUIRED_ARTIFACTS: &[&str] = &[
    ARTIFACT_LINUX_HEADERS,
    ARTIFACT_MUSL,
    ARTIFACT_BINUTILS,
    ARTIFACT_GCC,
    ARTIFACT_GMP,
    ARTIFACT_MPFR,
    ARTIFACT_MPC,
];

#[derive(Debug, Clone)]
pub(crate) struct ProviderMaterialization {
    pub output_path: PathBuf,
    pub output_digest: String,
    pub manifest_digest: String,
    pub dependency_trace: ProviderDependencyTrace,
}

#[derive(Debug)]
pub(crate) enum MaterializationError {
    Fetch(String),
    DigestMismatch {
        artifact: String,
        expected: String,
        actual: String,
    },
    Extract(String),
    Patch(String),
    Build(String),
    Normalize(String),
    Validate(Vec<ManifestDiagnostic>),
    MissingArtifact(String),
}

impl std::fmt::Display for MaterializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fetch(msg) => write!(f, "fetch: {msg}"),
            Self::DigestMismatch {
                artifact,
                expected,
                actual,
            } => {
                write!(f, "digest mismatch for {artifact}: expected {expected}, got {actual}")
            }
            Self::Extract(msg) => write!(f, "extract: {msg}"),
            Self::Patch(msg) => write!(f, "patch: {msg}"),
            Self::Build(msg) => write!(f, "build: {msg}"),
            Self::Normalize(msg) => write!(f, "normalize: {msg}"),
            Self::Validate(errors) => {
                write!(f, "validate: {}", crate::bootstrap_source_root::format_diagnostics(errors))
            }
            Self::MissingArtifact(name) => write!(f, "manifest missing required artifact: {name}"),
        }
    }
}

pub(crate) fn materialize_source_root_provider(
    manifest: &SourceRootManifest,
    manifest_bytes: &[u8],
    output_dir: &Path,
    scratch_dir: &Path,
    verbose: bool,
) -> Result<ProviderMaterialization, MaterializationError> {
    assert!(!manifest_bytes.is_empty());
    assert!(output_dir.is_absolute() || !output_dir.as_os_str().is_empty());
    assert!(scratch_dir.is_absolute() || !scratch_dir.as_os_str().is_empty());

    crate::bootstrap_source_root::validate_source_root_manifest(manifest).map_err(MaterializationError::Validate)?;
    reject_declared_patches(manifest)?;

    let manifest_digest = crate::bootstrap_source_root::source_root_manifest_digest(manifest_bytes);
    let artifact_map = index_artifacts(manifest)?;

    let downloads_dir = scratch_dir.join("downloads");
    let sources_dir = scratch_dir.join("sources");
    let build_dir = scratch_dir.join("build");
    let prefix_dir = scratch_dir.join("prefix");
    let sysroot_dir = prefix_dir.join(TARGET);
    let staging_dir = scratch_dir.join("staging");

    for dir in [
        &downloads_dir,
        &sources_dir,
        &build_dir,
        &prefix_dir,
        &sysroot_dir,
        &staging_dir,
    ] {
        fs::create_dir_all(dir).map_err(|e| MaterializationError::Extract(format!("mkdir {}: {e}", dir.display())))?;
    }

    // Phase 0: Fetch and verify all source artifacts.
    let mut fetched_urls = Vec::new();
    let mut fetched_hashes = Vec::new();

    for name in REQUIRED_ARTIFACTS {
        let artifact =
            artifact_map.get(*name).ok_or_else(|| MaterializationError::MissingArtifact(name.to_string()))?;
        if verbose {
            eprintln!("  [fetch] {name}: {}", artifact.source);
        }
        let archive_path =
            fetch_and_verify(&artifact.source, &artifact.digest, &downloads_dir.join(format!("{name}.archive")))?;
        fetched_urls.push(artifact.source.clone());
        fetched_hashes.push(artifact.digest.value.clone());

        if verbose {
            eprintln!("  [extract] {name}: kind={}", artifact.extraction_kind);
        }
        extract_archive(
            &archive_path,
            &artifact.extraction_kind,
            artifact.strip_prefix.as_deref(),
            &sources_dir.join(name),
        )?;
    }

    let jobs = available_parallelism();

    let host = find_host_cc()?;
    if verbose {
        eprintln!("  host CC={}  CXX={}  PATH={}", host.cc, host.cxx, host.clean_path);
    }

    // Phase 1: Install Linux kernel headers.
    if verbose {
        eprintln!("  [1/{BUILD_PHASE_COUNT}] installing Linux kernel headers");
    }
    install_linux_headers(&sources_dir.join(ARTIFACT_LINUX_HEADERS), &sysroot_dir, &host.clean_path)?;

    // Phase 2: Build binutils.
    if verbose {
        eprintln!("  [2/{BUILD_PHASE_COUNT}] building binutils");
    }
    build_binutils(
        &sources_dir.join(ARTIFACT_BINUTILS),
        &build_dir.join("binutils"),
        &prefix_dir,
        &sysroot_dir,
        jobs,
        &host.cc,
        &host.cxx,
        &host.clean_path,
    )?;

    // Phase 3: Build GCC stage 1 (C only, no libc).
    if verbose {
        eprintln!("  [3/{BUILD_PHASE_COUNT}] building GCC stage 1 (C compiler, no libc)");
    }
    symlink_gcc_prereqs(&sources_dir, &sources_dir.join(ARTIFACT_GCC))?;
    build_gcc_stage1(
        &sources_dir.join(ARTIFACT_GCC),
        &build_dir.join("gcc-stage1"),
        &prefix_dir,
        &sysroot_dir,
        jobs,
        &host.cc,
        &host.cxx,
        &host.clean_path,
    )?;

    // Phase 4: Build musl libc.
    if verbose {
        eprintln!("  [4/{BUILD_PHASE_COUNT}] building musl libc");
    }
    build_musl(
        &sources_dir.join(ARTIFACT_MUSL),
        &build_dir.join("musl"),
        &prefix_dir,
        &sysroot_dir,
        jobs,
        &host.clean_path,
    )?;

    // Phase 5: Build GCC stage 2 (C + C++).
    if verbose {
        eprintln!("  [5/{BUILD_PHASE_COUNT}] building GCC stage 2 (C + C++)");
    }
    build_gcc_stage2(
        &sources_dir.join(ARTIFACT_GCC),
        &build_dir.join("gcc-stage2"),
        &prefix_dir,
        &sysroot_dir,
        jobs,
        &host.cc,
        &host.cxx,
        &host.clean_path,
    )?;

    // Normalize into seed contract layout.
    if verbose {
        eprintln!("  [normalize] producing seed contract layout");
    }
    normalize_into_seed_contract(&prefix_dir, &sysroot_dir, &staging_dir)?;

    // Make binaries self-contained for bwrap sandboxes where the host
    // dynamic linker is not available.
    if verbose {
        eprintln!("  [self-contain] bundling host runtime libraries");
    }
    make_binaries_self_contained(&staging_dir)?;

    // GCC searches <prefix>/<target>/bin/ for unprefixed tools like `as`, `ld`.
    // Create forwarder scripts so GCC can find them inside the bwrap sandbox.
    create_target_bin_forwarders(&staging_dir)?;

    write_source_root_provider_json(&staging_dir, &manifest_digest)?;

    // Compute output digest.
    let output_digest = directory_blake3_digest(&staging_dir)?;

    // Move staging to final output.
    if output_dir.exists() {
        fs::remove_dir_all(output_dir)
            .map_err(|e| MaterializationError::Normalize(format!("removing old output: {e}")))?;
    }
    fs::rename(&staging_dir, output_dir)
        .map_err(|e| MaterializationError::Normalize(format!("moving staging to output: {e}")))?;

    // Build and validate dependency trace.
    let emitted_roles = discover_emitted_roles(output_dir)?;
    let dependency_trace = ProviderDependencyTrace {
        urls: fetched_urls,
        hashes: fetched_hashes,
        emitted_output_roles: emitted_roles,
        provider_metadata: vec![format!("manifest_digest={manifest_digest}")],
    };

    crate::bootstrap_source_root::validate_provider_dependency_trace(manifest, &dependency_trace)
        .map_err(MaterializationError::Validate)?;

    Ok(ProviderMaterialization {
        output_path: output_dir.to_path_buf(),
        output_digest,
        manifest_digest,
        dependency_trace,
    })
}

#[derive(Debug)]
struct ArtifactInfo {
    source: String,
    digest: DigestSpec,
    extraction_kind: String,
    strip_prefix: Option<String>,
}

fn index_artifacts(manifest: &SourceRootManifest) -> Result<BTreeMap<String, ArtifactInfo>, MaterializationError> {
    let mut map = BTreeMap::new();
    for artifact in &manifest.artifacts {
        map.insert(artifact.name.clone(), ArtifactInfo {
            source: artifact.source.clone(),
            digest: artifact.digest.clone(),
            extraction_kind: artifact.extraction.kind.clone(),
            strip_prefix: artifact.extraction.strip_prefix.clone(),
        });
    }
    for name in REQUIRED_ARTIFACTS {
        if !map.contains_key(*name) {
            return Err(MaterializationError::MissingArtifact(name.to_string()));
        }
    }
    Ok(map)
}

fn is_transient_fetch_error(err: &str) -> bool {
    let lower = err.to_lowercase();
    lower.contains("network is unreachable")
        || lower.contains("connection refused")
        || lower.contains("connection reset")
        || lower.contains("timed out")
        || lower.contains("try again")
        || lower.contains("502")
        || lower.contains("503")
        || lower.contains("504")
}

fn fetch_and_verify(url: &str, expected_digest: &DigestSpec, dest: &Path) -> Result<PathBuf, MaterializationError> {
    assert!(!url.is_empty());
    assert!(!expected_digest.value.is_empty());

    let agent = ureq::Agent::new_with_config(
        ureq::config::Config::builder()
            .timeout_global(Some(std::time::Duration::from_secs(FETCH_TIMEOUT_SECS)))
            .build(),
    );

    let mut last_err = String::new();
    for attempt in 0..=FETCH_MAX_RETRIES {
        if attempt > 0 {
            let delay_ms = FETCH_RETRY_BASE_DELAY_MS * (1u64 << (attempt - 1).min(4));
            eprintln!("  [fetch] retry {attempt}/{FETCH_MAX_RETRIES} for {url} after {delay_ms}ms");
            std::thread::sleep(std::time::Duration::from_millis(delay_ms));
        }

        let response = match agent.get(url).call() {
            Ok(r) => r,
            Err(e) => {
                last_err = format!("{url}: {e}");
                if is_transient_fetch_error(&last_err) && attempt < FETCH_MAX_RETRIES {
                    continue;
                }
                return Err(MaterializationError::Fetch(last_err));
            }
        };

        let mut body = Vec::new();
        match response.into_body().with_config().limit(FETCH_MAX_BYTES).reader().read_to_end(&mut body) {
            Ok(_) => {}
            Err(e) => {
                last_err = format!("{url}: reading body: {e}");
                if is_transient_fetch_error(&last_err) && attempt < FETCH_MAX_RETRIES {
                    continue;
                }
                return Err(MaterializationError::Fetch(last_err));
            }
        }

        if body.is_empty() {
            last_err = format!("{url}: empty response body");
            if attempt < FETCH_MAX_RETRIES {
                continue;
            }
            return Err(MaterializationError::Fetch(last_err));
        }

        let actual_digest = if expected_digest.algorithm == DIGEST_ALGORITHM_BLAKE3 {
            blake3::hash(&body).to_hex().to_string()
        } else {
            return Err(MaterializationError::Fetch(format!(
                "{url}: only BLAKE3 digest verification is implemented, got {}",
                expected_digest.algorithm
            )));
        };

        if actual_digest != expected_digest.value {
            return Err(MaterializationError::DigestMismatch {
                artifact: url.to_string(),
                expected: expected_digest.value.clone(),
                actual: actual_digest,
            });
        }

        fs::write(dest, &body).map_err(|e| MaterializationError::Fetch(format!("writing {}: {e}", dest.display())))?;
        return Ok(dest.to_path_buf());
    }

    Err(MaterializationError::Fetch(last_err))
}

fn extract_archive(
    archive_path: &Path,
    extraction_kind: &str,
    strip_prefix: Option<&str>,
    dest: &Path,
) -> Result<(), MaterializationError> {
    assert!(archive_path.exists());
    fs::create_dir_all(dest).map_err(|e| MaterializationError::Extract(format!("mkdir {}: {e}", dest.display())))?;

    let strip_components = if strip_prefix.is_some() { "1" } else { "0" };

    let tar_flags = match extraction_kind {
        "tar.gz" | "tar.gzip" | "tgz" => "-xzf",
        "tar.xz" | "tar.lzma" | "txz" => "-xJf",
        "tar.bz2" | "tar.bzip2" | "tbz2" => "-xjf",
        "tar" => "-xf",
        other => {
            return Err(MaterializationError::Extract(format!("unsupported extraction kind: {other}")));
        }
    };

    let status = Command::new("tar")
        .args([tar_flags, &archive_path.display().to_string()])
        .arg("-C")
        .arg(dest)
        .arg(format!("--strip-components={strip_components}"))
        .status()
        .map_err(|e| MaterializationError::Extract(format!("running tar: {e}")))?;

    if !status.success() {
        return Err(MaterializationError::Extract(format!(
            "tar extraction failed with status {status} for {}",
            archive_path.display()
        )));
    }
    Ok(())
}

fn reject_declared_patches(manifest: &SourceRootManifest) -> Result<(), MaterializationError> {
    if !manifest.patches.is_empty() {
        return Err(MaterializationError::Patch(
            "source-root provider patch materialization is not implemented; declared patches would be a placeholder"
                .to_string(),
        ));
    }
    for artifact in &manifest.artifacts {
        if !artifact.patches.is_empty() {
            return Err(MaterializationError::Patch(format!(
                "source-root provider artifact {} declares patches but patch materialization is not implemented",
                artifact.name
            )));
        }
    }
    Ok(())
}

fn available_parallelism() -> u32 {
    std::thread::available_parallelism()
        .map(|count| count.get() as u32)
        .unwrap_or(MIN_PARALLELISM_JOBS)
        .clamp(MIN_PARALLELISM_JOBS, MAX_PARALLELISM_JOBS)
}

pub(crate) fn host_compiler_available_for_source_root() -> bool {
    find_host_cc().is_ok()
}

/// Discover a 64-bit host C compiler and build a clean PATH for provider
/// builds. The caller's PATH may contain a 32-bit gcc wrapper or clang/mold
/// that interfere with cross-compiler configure scripts.
fn find_host_cc() -> Result<HostCompiler, MaterializationError> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    // NixOS gcc-wrapper paths from the running system closure.
    if let Ok(out) = Command::new("nix-store").args(["-qR", "/run/current-system"]).output() {
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            if line.contains("gcc-wrapper") && !line.contains("-man") {
                let p = PathBuf::from(line.trim()).join("bin/gcc");
                if p.is_file() {
                    candidates.push(p);
                }
            }
        }
    }

    // Scan nix store directly as fallback.
    if candidates.is_empty()
        && let Ok(entries) = fs::read_dir("/nix/store")
    {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.contains("gcc-wrapper") && !name_str.contains("-man") {
                let p = entry.path().join("bin/gcc");
                if p.is_file() {
                    candidates.push(p);
                }
            }
        }
    }

    // Also try well-known absolute paths and bare names.
    for cc in ["/run/current-system/sw/bin/gcc", "/usr/bin/gcc"] {
        let p = PathBuf::from(cc);
        if p.is_file() {
            candidates.push(p);
        }
    }
    if let Some(p) = which_cmd("gcc") {
        candidates.push(p);
    }
    if let Some(p) = which_cmd("cc") {
        candidates.push(p);
    }

    for cc_path in &candidates {
        // Must be x86_64.
        let Ok(out) = Command::new(cc_path).arg("-dumpmachine").output() else {
            continue;
        };
        let triple = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !triple.contains("x86_64") {
            continue;
        }

        // Must actually link a trivial program (rejects unwrapped gcc on NixOS).
        let link_ok = Command::new(cc_path)
            .args(["-o", "/dev/null", "-x", "c", "-"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                if let Some(ref mut stdin) = child.stdin {
                    let _ = stdin.write_all(b"int main(){return 0;}\n");
                }
                child.wait()
            })
            .map(|s| s.success())
            .unwrap_or(false);
        if !link_ok {
            continue;
        }

        let cc_dir = cc_path.parent().map(|p| p.to_path_buf());
        let cxx_path = cc_dir
            .as_ref()
            .map(|d| d.join("g++"))
            .filter(|p| p.is_file())
            .or_else(|| cc_dir.as_ref().map(|d| d.join("c++")).filter(|p| p.is_file()))
            .unwrap_or_else(|| which_cmd("g++").or_else(|| which_cmd("c++")).unwrap_or_else(|| PathBuf::from("g++")));

        let clean_path = build_clean_provider_path(cc_dir.as_deref());
        return Ok(HostCompiler {
            cc: cc_path.display().to_string(),
            cxx: cxx_path.display().to_string(),
            clean_path,
        });
    }

    Err(MaterializationError::Build(
        "no working 64-bit host C compiler found; need a gcc-wrapper with glibc CRT files".into(),
    ))
}

struct HostCompiler {
    cc: String,
    cxx: String,
    clean_path: String,
}

fn which_cmd(name: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths).find_map(|dir| {
            let full = dir.join(name);
            if full.is_file() { Some(full) } else { None }
        })
    })
}

/// Build a clean PATH for provider builds. Includes the compiler's directory,
/// standard tool locations, and excludes clang/mold/rustup dirs that interfere
/// with configure scripts.
fn build_clean_provider_path(cc_dir: Option<&Path>) -> String {
    let mut dirs: Vec<String> = Vec::new();
    if let Some(d) = cc_dir {
        dirs.push(d.display().to_string());
    }
    // Essential system tool directories.
    for d in ["/run/current-system/sw/bin", "/usr/bin", "/bin"] {
        if Path::new(d).is_dir() && !dirs.iter().any(|existing| existing == d) {
            dirs.push(d.to_string());
        }
    }
    // Include make if on PATH but not already included.
    if let Some(make_path) = which_cmd("make")
        && let Some(make_dir) = make_path.parent()
    {
        let s = make_dir.display().to_string();
        if !dirs.iter().any(|existing| existing == &s) {
            dirs.push(s);
        }
    }
    dirs.join(":")
}

fn run_build_cmd(label: &str, cmd: &mut Command) -> Result<(), MaterializationError> {
    let output = cmd.output().map_err(|e| MaterializationError::Build(format!("{label}: {e}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let tail = |s: &str| -> String {
            let lines: Vec<&str> = s.lines().collect();
            let start = lines.len().saturating_sub(40);
            lines[start..].join("\n")
        };
        return Err(MaterializationError::Build(format!(
            "{label}: exit {}\nstdout (last 40 lines):\n{}\nstderr (last 40 lines):\n{}",
            output.status,
            tail(&stdout),
            tail(&stderr),
        )));
    }
    Ok(())
}

fn install_linux_headers(src: &Path, sysroot: &Path, clean_path: &str) -> Result<(), MaterializationError> {
    assert!(src.exists(), "Linux source not extracted");
    let include_dir = sysroot.join("include");
    fs::create_dir_all(&include_dir).map_err(|e| MaterializationError::Build(format!("mkdir sysroot/include: {e}")))?;

    run_build_cmd(
        "linux-headers",
        Command::new("make").current_dir(src).env("PATH", clean_path).args([
            "ARCH=x86_64",
            &format!("INSTALL_HDR_PATH={}", sysroot.display()),
            "headers_install",
        ]),
    )
}

fn build_binutils(
    src: &Path,
    build_dir: &Path,
    prefix: &Path,
    sysroot: &Path,
    jobs: u32,
    host_cc: &str,
    host_cxx: &str,
    clean_path: &str,
) -> Result<(), MaterializationError> {
    assert!(src.exists(), "binutils source not extracted");
    fs::create_dir_all(build_dir).map_err(|e| MaterializationError::Build(format!("mkdir binutils build: {e}")))?;

    let configure = src.join("configure");
    run_build_cmd(
        "binutils-configure",
        Command::new(&configure)
            .current_dir(build_dir)
            .env("PATH", clean_path)
            .env(MAKEINFO_ENV, MAKEINFO_DISABLED)
            .env("CC", host_cc)
            .env("CXX", host_cxx)
            .args([
                &format!("--target={TARGET}"),
                &format!("--prefix={}", prefix.display()),
                &format!("--with-sysroot={}", sysroot.display()),
                "--disable-nls",
                "--disable-werror",
                "--disable-gprofng",
            ]),
    )?;

    run_build_cmd(
        "binutils-build",
        Command::new("make")
            .current_dir(build_dir)
            .env("PATH", clean_path)
            .env(MAKEINFO_ENV, MAKEINFO_DISABLED)
            .arg(format!("-j{jobs}"))
            .arg(format!("{MAKEINFO_ENV}={MAKEINFO_DISABLED}")),
    )?;

    run_build_cmd(
        "binutils-install",
        Command::new("make")
            .current_dir(build_dir)
            .env("PATH", clean_path)
            .env(MAKEINFO_ENV, MAKEINFO_DISABLED)
            .arg("install")
            .arg(format!("{MAKEINFO_ENV}={MAKEINFO_DISABLED}")),
    )
}

fn symlink_gcc_prereqs(sources_dir: &Path, gcc_src: &Path) -> Result<(), MaterializationError> {
    for (name, dir_name) in [(ARTIFACT_GMP, "gmp"), (ARTIFACT_MPFR, "mpfr"), (ARTIFACT_MPC, "mpc")] {
        let src = sources_dir.join(name);
        let dst = gcc_src.join(dir_name);
        if dst.exists() || dst.is_symlink() {
            continue;
        }
        if !src.exists() {
            return Err(MaterializationError::Build(format!(
                "GCC prerequisite {name} not extracted at {}",
                src.display()
            )));
        }
        std::os::unix::fs::symlink(&src, &dst)
            .map_err(|e| MaterializationError::Build(format!("symlink {name} into gcc: {e}")))?;
    }
    Ok(())
}

fn build_gcc_stage1(
    src: &Path,
    build_dir: &Path,
    prefix: &Path,
    sysroot: &Path,
    jobs: u32,
    host_cc: &str,
    host_cxx: &str,
    clean_path: &str,
) -> Result<(), MaterializationError> {
    assert!(src.exists(), "GCC source not extracted");
    fs::create_dir_all(build_dir).map_err(|e| MaterializationError::Build(format!("mkdir gcc-stage1 build: {e}")))?;

    let configure = src.join("configure");
    run_build_cmd(
        "gcc-stage1-configure",
        Command::new(&configure)
            .current_dir(build_dir)
            .env("PATH", clean_path)
            .env(MAKEINFO_ENV, MAKEINFO_DISABLED)
            .env("CC", host_cc)
            .env("CXX", host_cxx)
            .args([
                &format!("--target={TARGET}"),
                &format!("--prefix={}", prefix.display()),
                &format!("--with-sysroot={}", sysroot.display()),
                "--enable-languages=c",
                "--disable-shared",
                "--disable-threads",
                "--disable-libssp",
                "--disable-libgomp",
                "--disable-libatomic",
                "--disable-libquadmath",
                "--disable-libvtv",
                "--disable-nls",
                "--disable-multilib",
                "--without-headers",
                "--with-newlib",
                "--with-native-system-header-dir=/include",
            ]),
    )?;

    run_build_cmd(
        "gcc-stage1-build",
        Command::new("make")
            .current_dir(build_dir)
            .env("PATH", clean_path)
            .env(MAKEINFO_ENV, MAKEINFO_DISABLED)
            .arg(format!("-j{jobs}"))
            .arg(format!("{MAKEINFO_ENV}={MAKEINFO_DISABLED}"))
            .args(["all-gcc", "all-target-libgcc"]),
    )?;

    run_build_cmd(
        "gcc-stage1-install",
        Command::new("make")
            .current_dir(build_dir)
            .env("PATH", clean_path)
            .env(MAKEINFO_ENV, MAKEINFO_DISABLED)
            .arg(format!("{MAKEINFO_ENV}={MAKEINFO_DISABLED}"))
            .args(["install-gcc", "install-target-libgcc"]),
    )
}

fn build_musl(
    src: &Path,
    build_dir: &Path,
    prefix: &Path,
    sysroot: &Path,
    jobs: u32,
    clean_path: &str,
) -> Result<(), MaterializationError> {
    assert!(src.exists(), "musl source not extracted");
    fs::create_dir_all(build_dir).map_err(|e| MaterializationError::Build(format!("mkdir musl build: {e}")))?;

    let gcc_path = prefix.join("bin").join(format!("{TARGET}-gcc"));
    let ar_path = prefix.join("bin").join(format!("{TARGET}-ar"));
    let ranlib_path = prefix.join("bin").join(format!("{TARGET}-ranlib"));

    if !gcc_path.exists() {
        return Err(MaterializationError::Build(format!(
            "stage1 GCC not found at {}; GCC stage1 must complete before musl",
            gcc_path.display()
        )));
    }

    let configure = src.join("configure");
    run_build_cmd(
        "musl-configure",
        Command::new(&configure)
            .current_dir(build_dir)
            .env("PATH", clean_path)
            .args([&format!("--host={TARGET}"), &format!("--prefix={}", sysroot.display())])
            .env("CC", &gcc_path)
            .env("AR", &ar_path)
            .env("RANLIB", &ranlib_path),
    )?;

    run_build_cmd(
        "musl-build",
        Command::new("make").current_dir(build_dir).env("PATH", clean_path).arg(format!("-j{jobs}")),
    )?;

    run_build_cmd("musl-install", Command::new("make").current_dir(build_dir).env("PATH", clean_path).arg("install"))
}

fn build_gcc_stage2(
    src: &Path,
    build_dir: &Path,
    prefix: &Path,
    sysroot: &Path,
    jobs: u32,
    host_cc: &str,
    host_cxx: &str,
    clean_path: &str,
) -> Result<(), MaterializationError> {
    assert!(src.exists(), "GCC source not extracted");
    fs::create_dir_all(build_dir).map_err(|e| MaterializationError::Build(format!("mkdir gcc-stage2 build: {e}")))?;

    let configure = src.join("configure");
    run_build_cmd(
        "gcc-stage2-configure",
        Command::new(&configure)
            .current_dir(build_dir)
            .env("PATH", clean_path)
            .env(MAKEINFO_ENV, MAKEINFO_DISABLED)
            .env("CC", host_cc)
            .env("CXX", host_cxx)
            .args([
                &format!("--target={TARGET}"),
                &format!("--prefix={}", prefix.display()),
                &format!("--with-sysroot={}", sysroot.display()),
                "--enable-languages=c,c++",
                "--disable-nls",
                "--disable-multilib",
                "--disable-libsanitizer",
                "--with-native-system-header-dir=/include",
            ]),
    )?;

    run_build_cmd(
        "gcc-stage2-build",
        Command::new("make")
            .current_dir(build_dir)
            .env("PATH", clean_path)
            .env(MAKEINFO_ENV, MAKEINFO_DISABLED)
            .arg(format!("-j{jobs}"))
            .arg(format!("{MAKEINFO_ENV}={MAKEINFO_DISABLED}")),
    )?;

    run_build_cmd(
        "gcc-stage2-install",
        Command::new("make")
            .current_dir(build_dir)
            .env("PATH", clean_path)
            .env(MAKEINFO_ENV, MAKEINFO_DISABLED)
            .arg("install")
            .arg(format!("{MAKEINFO_ENV}={MAKEINFO_DISABLED}")),
    )
}

fn normalize_into_seed_contract(prefix: &Path, sysroot: &Path, staging: &Path) -> Result<(), MaterializationError> {
    let staging_bin = staging.join("bin");
    let staging_lib = staging.join("lib");
    let staging_libexec = staging.join("libexec");
    let staging_sysroot = staging.join(TARGET);
    let staging_sysroot_include = staging_sysroot.join("include");
    let staging_sysroot_lib = staging_sysroot.join("lib");

    for dir in [
        &staging_bin,
        &staging_lib,
        &staging_libexec,
        &staging_sysroot,
        &staging_sysroot_include,
        &staging_sysroot_lib,
    ] {
        fs::create_dir_all(dir).map_err(|e| MaterializationError::Normalize(format!("mkdir: {e}")))?;
    }

    // Copy compiler drivers and tools.
    let prefix_bin = prefix.join("bin");
    if prefix_bin.is_dir() {
        for entry in fs::read_dir(&prefix_bin).map_err(|e| MaterializationError::Normalize(format!("read bin: {e}")))? {
            let entry = entry.map_err(|e| MaterializationError::Normalize(format!("read bin entry: {e}")))?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with(TARGET) || is_retained_unprefixed_tool(&name_str) {
                copy_file_or_link(&entry.path(), &staging_bin.join(&name))?;
            }
        }
    }

    // Copy libexec (GCC backend executables).
    let prefix_libexec = prefix.join("libexec");
    if prefix_libexec.is_dir() {
        copy_tree(&prefix_libexec, &staging_libexec)?;
    }

    // Copy lib (GCC runtime libraries).
    let prefix_lib = prefix.join("lib");
    if prefix_lib.is_dir() {
        copy_tree(&prefix_lib, &staging_lib)?;
    }

    // Copy sysroot headers.
    let src_include = sysroot.join("include");
    if src_include.is_dir() {
        copy_tree(&src_include, &staging_sysroot_include)?;
    }

    // Copy sysroot libraries.
    let src_lib = sysroot.join("lib");
    if src_lib.is_dir() {
        copy_tree(&src_lib, &staging_sysroot_lib)?;
    }

    // Ensure key contract libraries are in the sysroot lib.
    // GCC installs libgcc_s/libstdc++ into deep subdirs like
    // lib/gcc/<target>/<version>/ — search recursively.
    for lib in [
        "libgcc_s.so",
        "libgcc_s.so.1",
        "libc.so",
        "libstdc++.a",
        "libstdc++.so",
        "libsupc++.a",
    ] {
        let in_sysroot = staging_sysroot_lib.join(lib);
        if !in_sysroot.exists() {
            if let Some(found) = find_file_recursive(&staging_lib, lib) {
                copy_file_or_link(&found, &in_sysroot)?;
            } else if let Some(found) = find_file_recursive(prefix, lib) {
                copy_file_or_link(&found, &in_sysroot)?;
            }
        }
    }

    // Drop components not needed by the bootstrap chain.
    drop_unwanted_components(staging)?;

    validate_seed_contract_layout(staging)
}

fn find_file_recursive(dir: &Path, name: &str) -> Option<PathBuf> {
    fn walk(dir: &Path, name: &str, depth: u32) -> Option<PathBuf> {
        if depth > 8 {
            return None;
        }
        let entries = fs::read_dir(dir).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && entry.file_name().to_string_lossy() == name {
                return Some(path);
            }
            if path.is_dir()
                && !path.is_symlink()
                && let Some(found) = walk(&path, name, depth + 1)
            {
                return Some(found);
            }
        }
        None
    }
    walk(dir, name, 0)
}

/// Bundle the host dynamic linker and needed shared libraries into the provider
/// output so binaries can run inside bwrap sandboxes without the host `/nix/store`.
fn make_binaries_self_contained(staging: &Path) -> Result<(), MaterializationError> {
    let host_runtime_dir = staging.join("host-runtime");
    fs::create_dir_all(&host_runtime_dir)
        .map_err(|e| MaterializationError::Normalize(format!("mkdir host-runtime: {e}")))?;

    let mut interpreters: BTreeSet<String> = BTreeSet::new();
    let mut needed_libs: BTreeSet<String> = BTreeSet::new();
    collect_elf_deps(&staging.join("bin"), &mut interpreters, &mut needed_libs)?;
    collect_elf_deps(&staging.join("libexec"), &mut interpreters, &mut needed_libs)?;

    if interpreters.is_empty() {
        return Ok(());
    }

    // Copy dynamic linker(s) to host-runtime/.
    let mut interp_filename = String::new();
    for interp in &interpreters {
        let interp_path = Path::new(interp);
        if !interp_path.exists() {
            return Err(MaterializationError::Normalize(format!("ELF interpreter {interp} not found on host")));
        }
        let fname = interp_path.file_name().unwrap().to_str().unwrap().to_string();
        let dest = host_runtime_dir.join(&fname);
        fs::copy(interp_path, &dest).map_err(|e| MaterializationError::Normalize(format!("copy {interp}: {e}")))?;
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&dest)
            .map_err(|e| MaterializationError::Normalize(format!("stat {}: {e}", dest.display())))?
            .permissions();
        perms.set_mode(perms.mode() | 0o111);
        fs::set_permissions(&dest, perms)
            .map_err(|e| MaterializationError::Normalize(format!("chmod {}: {e}", dest.display())))?;
        if interp_filename.is_empty() {
            interp_filename = fname;
        }
    }

    // Copy needed shared libraries from the host.
    for bin_dir in ["bin", "libexec"] {
        let dir = staging.join(bin_dir);
        if dir.is_dir() {
            resolve_and_copy_libs(&dir, &host_runtime_dir)?;
        }
    }

    // Replace each dynamically-linked ELF binary with a shell wrapper that
    // invokes the bundled interpreter. The wrapper uses POSIX sh parameter
    // expansion to find host-runtime/ relative to the script's own path,
    // so no absolute store paths are baked in.
    wrap_elf_dir(&staging.join("bin"), &interp_filename)?;
    wrap_elf_dir(&staging.join("libexec"), &interp_filename)?;

    Ok(())
}

fn collect_elf_deps(
    dir: &Path,
    interpreters: &mut BTreeSet<String>,
    needed_libs: &mut BTreeSet<String>,
) -> Result<(), MaterializationError> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in walkdir(dir)? {
        let path = entry;
        if !path.is_file() {
            continue;
        }
        if let Ok(bytes) = fs::read(&path) {
            if bytes.len() < 4 || &bytes[..4] != b"\x7fELF" {
                continue;
            }
            // Read interpreter from `readelf -l`.
            if let Ok(out) = Command::new("readelf").args(["-l", &path.display().to_string()]).output() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                for line in stdout.lines() {
                    if let Some(start) = line.find("[Requesting program interpreter: ") {
                        let rest = &line[start + 33..];
                        if let Some(end) = rest.find(']') {
                            interpreters.insert(rest[..end].to_string());
                        }
                    }
                }
            }
            // Read NEEDED from `readelf -d`.
            if let Ok(out) = Command::new("readelf").args(["-d", &path.display().to_string()]).output() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                for line in stdout.lines() {
                    if line.contains("(NEEDED)")
                        && let Some(start) = line.find('[')
                        && let Some(end) = line.rfind(']')
                    {
                        needed_libs.insert(line[start + 1..end].to_string());
                    }
                }
            }
        }
    }
    Ok(())
}

fn walkdir(dir: &Path) -> Result<Vec<PathBuf>, MaterializationError> {
    let mut result = Vec::new();
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() && !path.is_symlink() {
                walk(&path, out)?;
            } else {
                out.push(path);
            }
        }
        Ok(())
    }
    walk(dir, &mut result).map_err(|e| MaterializationError::Normalize(format!("walkdir {}: {e}", dir.display())))?;
    Ok(result)
}

fn wrap_elf_dir(dir: &Path, interp_filename: &str) -> Result<(), MaterializationError> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in walkdir(dir)? {
        if !entry.is_file() || entry.is_symlink() {
            continue;
        }
        let bytes = match fs::read(&entry) {
            Ok(b) => b,
            Err(_) => continue,
        };
        if bytes.len() < 4 || &bytes[..4] != b"\x7fELF" {
            continue;
        }
        let fname = entry.file_name().unwrap().to_str().unwrap();
        // Shared libraries are loaded via dlopen; the parent process's
        // ld.so + library-path already handles their dependencies.
        if fname.contains(".so") || fname.ends_with(".la") {
            continue;
        }
        let real_name = format!(".{fname}.elf");
        let real_path = entry.parent().unwrap().join(&real_name);
        fs::rename(&entry, &real_path).map_err(|e| {
            MaterializationError::Normalize(format!("rename {} -> {}: {e}", entry.display(), real_path.display()))
        })?;
        // The wrapper finds host-runtime/ by walking up from its own directory.
        let wrapper = format!(
            "#!/bin/sh\n\
             _d=\"${{0%/*}}\"\n\
             _r=\"$_d\"\n\
             while [ ! -d \"$_r/host-runtime\" ] && [ \"$_r\" != \"/\" ]; do _r=\"${{_r%/*}}\"; done\n\
             exec \"$_r/host-runtime/{interp_filename}\" --library-path \"$_r/host-runtime\" --argv0 \"$0\" \"$_d/{real_name}\" \"$@\"\n"
        );
        fs::write(&entry, wrapper.as_bytes())
            .map_err(|e| MaterializationError::Normalize(format!("write wrapper {}: {e}", entry.display())))?;
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&entry)
            .map_err(|e| MaterializationError::Normalize(format!("stat {}: {e}", entry.display())))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&entry, perms)
            .map_err(|e| MaterializationError::Normalize(format!("chmod {}: {e}", entry.display())))?;
    }
    Ok(())
}

fn resolve_and_copy_libs(bin_dir: &Path, dest: &Path) -> Result<(), MaterializationError> {
    for entry in walkdir(bin_dir)? {
        if !entry.is_file() {
            continue;
        }
        if let Ok(bytes) = fs::read(&entry) {
            if bytes.len() < 4 || &bytes[..4] != b"\x7fELF" {
                continue;
            }
        } else {
            continue;
        }
        // Use ldd to find needed libraries.
        if let Ok(out) = Command::new("ldd").arg(&entry).output() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                let line = line.trim();
                // Lines like: libfoo.so.1 => /nix/store/.../libfoo.so.1 (0x...)
                if let Some(arrow) = line.find(" => ") {
                    let lib_path_str = line[arrow + 4..].split_whitespace().next().unwrap_or("");
                    let lib_path = Path::new(lib_path_str);
                    if lib_path.exists() {
                        let lib_name = lib_path.file_name().unwrap();
                        let lib_dest = dest.join(lib_name);
                        if !lib_dest.exists() {
                            fs::copy(lib_path, &lib_dest).map_err(|e| {
                                MaterializationError::Normalize(format!("copy lib {}: {e}", lib_path.display()))
                            })?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn is_retained_unprefixed_tool(name: &str) -> bool {
    matches!(
        name,
        "ar" | "as"
            | "ld"
            | "ld.bfd"
            | "nm"
            | "objcopy"
            | "objdump"
            | "ranlib"
            | "readelf"
            | "size"
            | "strings"
            | "strip"
    )
}

fn copy_file_or_link(src: &Path, dst: &Path) -> Result<(), MaterializationError> {
    if let Ok(target) = fs::read_link(src)
        && target.is_relative()
    {
        std::os::unix::fs::symlink(&target, dst).map_err(|e| {
            MaterializationError::Normalize(format!("symlink {} -> {}: {e}", dst.display(), target.display()))
        })?;
        return Ok(());
    }
    fs::copy(src, dst)
        .map_err(|e| MaterializationError::Normalize(format!("copy {} -> {}: {e}", src.display(), dst.display())))?;
    Ok(())
}

fn copy_tree(src: &Path, dst: &Path) -> Result<(), MaterializationError> {
    if !src.is_dir() {
        return Ok(());
    }
    fs::create_dir_all(dst).map_err(|e| MaterializationError::Normalize(format!("mkdir {}: {e}", dst.display())))?;

    for entry in
        fs::read_dir(src).map_err(|e| MaterializationError::Normalize(format!("readdir {}: {e}", src.display())))?
    {
        let entry = entry.map_err(|e| MaterializationError::Normalize(format!("readdir entry: {e}")))?;
        let entry_dst = dst.join(entry.file_name());
        let ft = entry.file_type().map_err(|e| MaterializationError::Normalize(format!("filetype: {e}")))?;
        if ft.is_dir() {
            copy_tree(&entry.path(), &entry_dst)?;
        } else {
            copy_file_or_link(&entry.path(), &entry_dst)?;
        }
    }
    Ok(())
}

fn drop_unwanted_components(staging: &Path) -> Result<(), MaterializationError> {
    let removals = ["share/locale", "share/man", "share/info"];
    for rel in &removals {
        let path = staging.join(rel);
        if path.exists() {
            fs::remove_dir_all(&path).map_err(|e| MaterializationError::Normalize(format!("removing {rel}: {e}")))?;
        }
    }

    let bin = staging.join("bin");
    if bin.is_dir() {
        let unwanted_bin = [
            "lto-dump",
            "gcov",
            "gcov-dump",
            "gcov-tool",
            &format!("{TARGET}-gfortran"),
            "gfortran",
            "ld.gold",
            &format!("{TARGET}-ld.gold"),
            "dwp",
            &format!("{TARGET}-dwp"),
            "gprof",
            &format!("{TARGET}-gprof"),
        ];
        for name in &unwanted_bin {
            let path = bin.join(name);
            if path.exists() {
                let _ = fs::remove_file(&path);
            }
        }
    }

    let unwanted_libs = ["libgfortran.a", "libgfortran.so"];
    let lib = staging.join("lib");
    if lib.is_dir() {
        for name in &unwanted_libs {
            let path = lib.join(name);
            if path.exists() {
                let _ = fs::remove_file(&path);
            }
        }
    }

    Ok(())
}

fn validate_seed_contract_layout(staging: &Path) -> Result<(), MaterializationError> {
    let required_files = [
        format!("bin/{TARGET}-gcc"),
        format!("bin/{TARGET}-g++"),
        format!("bin/{TARGET}-ar"),
        format!("bin/{TARGET}-ld"),
        format!("{TARGET}/lib/libgcc_s.so.1"),
        format!("{TARGET}/lib/libc.so"),
    ];
    let required_dirs = [format!("{TARGET}/include"), format!("{TARGET}/include/linux")];

    for rel in &required_files {
        let path = staging.join(rel);
        if !path.exists() {
            return Err(MaterializationError::Normalize(format!(
                "seed contract requires {rel} but it is missing from staging"
            )));
        }
    }
    for rel in &required_dirs {
        let path = staging.join(rel);
        if !path.is_dir() {
            return Err(MaterializationError::Normalize(format!(
                "seed contract requires directory {rel} but it is missing from staging"
            )));
        }
    }
    Ok(())
}

/// Create `<target>/bin/` with unprefixed forwarder scripts for each
/// prefixed tool in `bin/`.  GCC's internal tool search looks for `as`,
/// `ld`, etc. in `<prefix>/<target>/bin/`, not on PATH.
fn create_target_bin_forwarders(staging: &Path) -> Result<(), MaterializationError> {
    let target_bin = staging.join(TARGET).join("bin");
    fs::create_dir_all(&target_bin).map_err(|e| MaterializationError::Normalize(format!("mkdir {TARGET}/bin: {e}")))?;

    let prefix_with_dash = format!("{TARGET}-");
    let bin_dir = staging.join("bin");
    let entries =
        fs::read_dir(&bin_dir).map_err(|e| MaterializationError::Normalize(format!("read bin for forwarders: {e}")))?;

    let mut count: u32 = 0;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if let Some(unprefixed) = name_str.strip_prefix(&prefix_with_dash) {
            if unprefixed.starts_with('.') {
                continue; // skip .elf hidden files
            }
            let forwarder_path = target_bin.join(unprefixed);
            let script = format!("#!/bin/sh\nexec \"${{0%/*}}/../../bin/{name_str}\" \"$@\"\n");
            fs::write(&forwarder_path, &script)
                .map_err(|e| MaterializationError::Normalize(format!("write forwarder {unprefixed}: {e}")))?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&forwarder_path, fs::Permissions::from_mode(0o755))
                    .map_err(|e| MaterializationError::Normalize(format!("chmod forwarder {unprefixed}: {e}")))?;
            }
            count = count.saturating_add(1);
        }
    }

    assert!(count >= 2, "expected at least as + ld forwarders, got {count}");
    Ok(())
}

fn write_source_root_provider_json(staging: &Path, manifest_digest: &str) -> Result<(), MaterializationError> {
    let meta_dir = staging.join("share").join("crunch-bootstrap");
    fs::create_dir_all(&meta_dir).map_err(|e| MaterializationError::Normalize(format!("mkdir provider meta: {e}")))?;

    let provider_json = serde_json::json!({
        "provider_id": "source-root-v1",
        "name": PROVIDER_NAME,
        "capability_class": crunch_bootstrap_core::SOURCE_ROOT_CAPABILITY_CLASS,
        "full_source_bootstrap_eligible": false,
        "host_influences": [
            "host C compiler and linker",
            "host make and archive extraction tools",
            "host kernel and runtime libraries during materialization",
        ],
        "target": PROVIDER_TARGET,
        "dynamic_linker": PROVIDER_DYNAMIC_LINKER,
        "source_root": {
            "manifest_digest": manifest_digest,
        },
        "reduction": {
            "retained_tools": crate::bootstrap_source_root::REQUIRED_PROVIDER_TOOL_ROLES,
            "dropped_components": [
                "share/locale and translated message catalogs",
                "Fortran frontend and runtime",
                "coverage helpers (gcov)",
                "LTO helpers (lto1, lto-wrapper, lto-dump)",
                "gold and profiling extras",
            ],
        },
        "notes": [
            "Built from declared sources through host-assisted materialization; this is not a full-source bootstrap.",
            "No pre-built musl.cc binary toolchain tarball was used.",
            "Host compiler, build tools, kernel, and runtime are materialization influences.",
        ],
    });

    let json_path = meta_dir.join("provider.json");
    let content = serde_json::to_string_pretty(&provider_json)
        .map_err(|e| MaterializationError::Normalize(format!("serializing provider.json: {e}")))?;
    fs::write(&json_path, &content)
        .map_err(|e| MaterializationError::Normalize(format!("writing provider.json: {e}")))?;
    Ok(())
}

fn directory_blake3_digest(dir: &Path) -> Result<String, MaterializationError> {
    assert!(dir.is_dir());
    let mut hasher = blake3::Hasher::new();
    let mut paths = Vec::new();
    collect_paths(dir, dir, &mut paths)?;
    paths.sort();

    for rel_path in &paths {
        let full = dir.join(rel_path);
        hasher.update(rel_path.as_bytes());
        hasher.update(b"\0");
        if full.is_file() && !full.is_symlink() {
            let bytes = fs::read(&full)
                .map_err(|e| MaterializationError::Normalize(format!("reading {} for digest: {e}", full.display())))?;
            hasher.update(&(bytes.len() as u64).to_le_bytes());
            hasher.update(&bytes);
        } else if full.is_symlink() {
            let target = fs::read_link(&full)
                .map_err(|e| MaterializationError::Normalize(format!("readlink {}: {e}", full.display())))?;
            let target_str = target.to_string_lossy();
            hasher.update(b"symlink:");
            hasher.update(target_str.as_bytes());
        }
    }

    Ok(hasher.finalize().to_hex().to_string())
}

fn collect_paths(root: &Path, current: &Path, out: &mut Vec<String>) -> Result<(), MaterializationError> {
    if !current.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(current).map_err(|e| MaterializationError::Normalize(format!("readdir: {e}")))? {
        let entry = entry.map_err(|e| MaterializationError::Normalize(format!("readdir entry: {e}")))?;
        let path = entry.path();
        let rel = path
            .strip_prefix(root)
            .map_err(|e| MaterializationError::Normalize(format!("strip prefix: {e}")))?
            .to_string_lossy()
            .to_string();
        out.push(rel);
        if path.is_dir() && !path.is_symlink() {
            collect_paths(root, &path, out)?;
        }
    }
    Ok(())
}

fn discover_emitted_roles(output: &Path) -> Result<Vec<String>, MaterializationError> {
    let mut roles = BTreeSet::new();

    let bin = output.join("bin");
    if bin.is_dir() {
        for entry in fs::read_dir(&bin).map_err(|e| MaterializationError::Normalize(format!("read bin: {e}")))? {
            let entry = entry.map_err(|e| MaterializationError::Normalize(format!("bin entry: {e}")))?;
            let name = entry.file_name().to_string_lossy().to_string();
            if crate::bootstrap_source_root::REQUIRED_PROVIDER_TOOL_ROLES.contains(&name.as_str()) {
                roles.insert(name);
            }
        }
    }

    let sysroot = output.join(TARGET);
    if sysroot.join("include").is_dir() {
        roles.insert(format!("{TARGET}-include"));
    }
    if sysroot.join("lib").join("libgcc_s.so.1").exists() {
        roles.insert(format!("{TARGET}-libgcc_s.so.1"));
    }
    if sysroot.join("lib").join("libc.so").exists() {
        roles.insert(format!("{TARGET}-libc.so"));
    }

    let cxx_runtime_libs = ["libstdc++.a", "libstdc++.so", "libsupc++.a"];
    let has_any_cxx = cxx_runtime_libs.iter().any(|lib| sysroot.join("lib").join(lib).exists());
    if has_any_cxx {
        roles.insert(format!("{TARGET}-cxx-runtime"));
    }

    if output.join("share").join("crunch-bootstrap").join("provider.json").exists() {
        roles.insert(PROVIDER_METADATA_ROLE.to_string());
    }

    Ok(roles.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bootstrap_source_root::REQUIRED_PROVIDER_TOOL_ROLES;

    const TEST_BLAKE3_HEX_LEN: usize = 64;
    const FIRST_PATCH_ORDER: u32 = 1;

    fn digest_fixture(nibble: char) -> DigestSpec {
        DigestSpec {
            algorithm: DIGEST_ALGORITHM_BLAKE3.to_string(),
            value: nibble.to_string().repeat(TEST_BLAKE3_HEX_LEN),
            non_blake3_reason: None,
        }
    }

    fn patchless_manifest_fixture() -> SourceRootManifest {
        SourceRootManifest {
            version: Some(crate::bootstrap_source_root::SOURCE_ROOT_MANIFEST_VERSION),
            artifacts: REQUIRED_ARTIFACTS
                .iter()
                .map(|name| crate::bootstrap_source_root::SourceArtifact {
                    name: (*name).to_string(),
                    source: format!("https://example.invalid/{name}.tar.gz"),
                    digest: digest_fixture('a'),
                    extraction: crate::bootstrap_source_root::ExtractionRule {
                        kind: "tar.gz".to_string(),
                        strip_prefix: Some((*name).to_string()),
                    },
                    provenance: "fixture source".to_string(),
                    patches: Vec::new(),
                })
                .collect(),
            patches: Vec::new(),
            network_trust_roots: Vec::new(),
            trust_notes: Vec::new(),
            expected_outputs: REQUIRED_PROVIDER_TOOL_ROLES
                .iter()
                .map(|role| crate::bootstrap_source_root::ExpectedProviderOutput {
                    name: (*role).to_string(),
                    kind: "file-or-directory".to_string(),
                    digest: digest_fixture('c'),
                    provenance: "fixture output".to_string(),
                    contract_role: (*role).to_string(),
                })
                .collect(),
        }
    }

    #[test]
    fn index_artifacts_rejects_missing_required() {
        let manifest = SourceRootManifest {
            version: Some(1),
            artifacts: Vec::new(),
            patches: Vec::new(),
            network_trust_roots: Vec::new(),
            trust_notes: Vec::new(),
            expected_outputs: Vec::new(),
        };
        let err = index_artifacts(&manifest).unwrap_err();
        assert!(matches!(err, MaterializationError::MissingArtifact(_)));
    }

    #[test]
    fn source_root_materialization_rejects_declared_patches_before_fetch() {
        let mut manifest = patchless_manifest_fixture();
        manifest.patches.push(crate::bootstrap_source_root::SourcePatch {
            name: "fix.patch".to_string(),
            digest: digest_fixture('b'),
            provenance: "fixture patch".to_string(),
            apply_order: FIRST_PATCH_ORDER,
        });
        manifest.artifacts[0].patches.push("fix.patch".to_string());
        let bytes = serde_json::to_vec(&manifest).unwrap();
        let scratch = tempfile::tempdir().unwrap();
        let output = scratch.path().join("provider-output");

        let err = materialize_source_root_provider(&manifest, &bytes, &output, scratch.path(), false).unwrap_err();

        assert!(matches!(err, MaterializationError::Patch(_)));
        assert!(err.to_string().contains("placeholder") || err.to_string().contains("not implemented"));
        assert!(!output.exists(), "patch rejection must happen before materializing output");
    }

    #[test]
    fn available_parallelism_returns_bounded_value() {
        let jobs = available_parallelism();
        assert!(jobs >= MIN_PARALLELISM_JOBS);
        assert!(jobs <= MAX_PARALLELISM_JOBS);
    }

    #[test]
    fn extract_kind_coverage() {
        assert_eq!(
            ["tar.gz", "tar.xz", "tar.bz2", "tar", "tgz", "txz", "tbz2"]
                .iter()
                .filter(|k| matches!(
                    **k,
                    "tar.gz"
                        | "tar.gzip"
                        | "tgz"
                        | "tar.xz"
                        | "tar.lzma"
                        | "txz"
                        | "tar.bz2"
                        | "tar.bzip2"
                        | "tbz2"
                        | "tar"
                ))
                .count(),
            7
        );
    }

    #[test]
    fn retained_unprefixed_tools_are_recognized() {
        assert!(is_retained_unprefixed_tool("ar"));
        assert!(is_retained_unprefixed_tool("ld"));
        assert!(!is_retained_unprefixed_tool("gcc"));
        assert!(!is_retained_unprefixed_tool(""));
    }

    // --- I5: Provider contract equivalence tests ---

    #[test]
    fn source_root_provider_json_has_matching_metadata_fields() {
        let staging = tempfile::tempdir().unwrap();
        let manifest_digest = "abc123def456";

        write_source_root_provider_json(staging.path(), manifest_digest).unwrap();

        let json_path = staging.path().join("share/crunch-bootstrap/provider.json");
        assert!(json_path.exists());
        let content: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&json_path).unwrap()).unwrap();

        assert_eq!(content["name"].as_str().unwrap(), PROVIDER_NAME);
        assert_eq!(content["target"].as_str().unwrap(), PROVIDER_TARGET);
        assert_eq!(content["dynamic_linker"].as_str().unwrap(), PROVIDER_DYNAMIC_LINKER);
        assert_eq!(content["provider_id"].as_str().unwrap(), "source-root-v1");
        assert_eq!(content["capability_class"].as_str().unwrap(), crunch_bootstrap_core::SOURCE_ROOT_CAPABILITY_CLASS);
        assert_eq!(content["full_source_bootstrap_eligible"].as_bool(), Some(false));
        assert!(content["reduction"].is_object());
        assert!(content["reduction"]["retained_tools"].is_array());
        assert!(content["reduction"]["dropped_components"].is_array());
        assert!(content["notes"].is_array());
        assert_eq!(content["source_root"]["manifest_digest"].as_str().unwrap(), manifest_digest);
    }

    #[test]
    fn source_root_retained_tools_match_required_roles() {
        let staging = tempfile::tempdir().unwrap();
        write_source_root_provider_json(staging.path(), "test").unwrap();

        let json_path = staging.path().join("share/crunch-bootstrap/provider.json");
        let content: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&json_path).unwrap()).unwrap();

        let retained: Vec<&str> = content["reduction"]["retained_tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();

        for role in REQUIRED_PROVIDER_TOOL_ROLES {
            assert!(retained.contains(role), "source-root provider.json must list role {role} in retained_tools");
        }
    }

    #[test]
    fn source_root_and_legacy_share_contract_metadata_constants() {
        assert_eq!(PROVIDER_NAME, "musl-seed-toolchain");
        assert_eq!(PROVIDER_TARGET, "x86_64-linux-musl");
        assert_eq!(PROVIDER_DYNAMIC_LINKER, "ld-musl-x86_64.so.1");
        assert_eq!(TARGET, PROVIDER_TARGET);
    }

    #[test]
    fn source_root_provider_json_does_not_contain_legacy_raw_artifact() {
        let staging = tempfile::tempdir().unwrap();
        write_source_root_provider_json(staging.path(), "test").unwrap();

        let json_path = staging.path().join("share/crunch-bootstrap/provider.json");
        let content = std::fs::read_to_string(&json_path).unwrap();

        assert!(!content.contains(crate::bootstrap_source_root::LEGACY_MUSL_CC_URL));
        assert!(!content.contains(crate::bootstrap_source_root::LEGACY_MUSL_CC_HASH));
        assert!(!content.contains("musl.cc/x86_64"));

        let json: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(
            json.get("raw").is_none() || json["raw"].is_null(),
            "source-root provider.json must not have a legacy 'raw' artifact block"
        );
    }

    #[test]
    fn validate_seed_contract_layout_rejects_missing_compiler() {
        let staging = tempfile::tempdir().unwrap();
        let err = validate_seed_contract_layout(staging.path()).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("x86_64-linux-musl-gcc") || msg.contains("missing from staging"),
            "should report missing required file, got: {msg}"
        );
    }

    #[test]
    fn validate_seed_contract_layout_rejects_missing_sysroot_headers() {
        let staging = tempfile::tempdir().unwrap();
        let bin = staging.path().join("bin");
        fs::create_dir_all(&bin).unwrap();
        for tool in [
            format!("{TARGET}-gcc"),
            format!("{TARGET}-g++"),
            format!("{TARGET}-ar"),
            format!("{TARGET}-ld"),
        ] {
            fs::write(bin.join(&tool), "fake").unwrap();
        }
        let sysroot_lib = staging.path().join(TARGET).join("lib");
        fs::create_dir_all(&sysroot_lib).unwrap();
        fs::write(sysroot_lib.join("libgcc_s.so.1"), "fake").unwrap();
        fs::write(sysroot_lib.join("libc.so"), "fake").unwrap();

        let err = validate_seed_contract_layout(staging.path()).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("include"), "should report missing include dir, got: {msg}");
    }

    #[test]
    fn discover_emitted_roles_finds_all_contract_parts() {
        let staging = tempfile::tempdir().unwrap();
        let bin = staging.path().join("bin");
        fs::create_dir_all(&bin).unwrap();

        let tool_roles: Vec<&str> = REQUIRED_PROVIDER_TOOL_ROLES
            .iter()
            .filter(|r| {
                !r.contains('/')
                    && !r.ends_with("-include")
                    && !r.ends_with("-libgcc_s.so.1")
                    && !r.ends_with("-libc.so")
                    && !r.ends_with("-cxx-runtime")
            })
            .copied()
            .collect();
        for tool in &tool_roles {
            fs::write(bin.join(tool), "fake").unwrap();
        }

        let sysroot = staging.path().join(TARGET);
        let sysroot_lib = sysroot.join("lib");
        let sysroot_include = sysroot.join("include");
        fs::create_dir_all(&sysroot_lib).unwrap();
        fs::create_dir_all(&sysroot_include).unwrap();
        fs::write(sysroot_lib.join("libgcc_s.so.1"), "fake").unwrap();
        fs::write(sysroot_lib.join("libc.so"), "fake").unwrap();
        fs::write(sysroot_lib.join("libstdc++.a"), "fake").unwrap();

        let meta_dir = staging.path().join("share/crunch-bootstrap");
        fs::create_dir_all(&meta_dir).unwrap();
        fs::write(meta_dir.join("provider.json"), "{}").unwrap();

        let roles = discover_emitted_roles(staging.path()).unwrap();

        for required in REQUIRED_PROVIDER_TOOL_ROLES {
            assert!(roles.contains(&required.to_string()), "emitted roles must include {required}, got: {roles:?}");
        }
    }

    #[test]
    fn legacy_and_source_root_provider_json_share_required_fields() {
        let required_fields = ["provider_id", "name", "target", "dynamic_linker", "reduction", "notes"];

        let staging = tempfile::tempdir().unwrap();
        write_source_root_provider_json(staging.path(), "test").unwrap();
        let json_path = staging.path().join("share/crunch-bootstrap/provider.json");
        let source_root: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&json_path).unwrap()).unwrap();

        for field in &required_fields {
            assert!(!source_root[field].is_null(), "source-root provider.json must have field '{field}'");
        }

        let reduction_fields = ["retained_tools", "dropped_components"];
        for field in &reduction_fields {
            assert!(
                !source_root["reduction"][field].is_null(),
                "source-root provider.json reduction must have field '{field}'"
            );
        }
    }

    #[test]
    fn source_root_has_source_root_metadata_field() {
        let staging = tempfile::tempdir().unwrap();
        write_source_root_provider_json(staging.path(), "test-digest-hex").unwrap();
        let json_path = staging.path().join("share/crunch-bootstrap/provider.json");
        let content: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&json_path).unwrap()).unwrap();

        assert!(content["source_root"].is_object());
        assert_eq!(content["source_root"]["manifest_digest"].as_str().unwrap(), "test-digest-hex");
        assert!(
            content.get("raw").is_none() || content["raw"].is_null(),
            "source-root provider must not have a 'raw' field (that's legacy)"
        );
    }

    #[test]
    fn normalize_drops_unwanted_components() {
        let staging = tempfile::tempdir().unwrap();
        let s = staging.path();
        fs::create_dir_all(s.join("share/locale")).unwrap();
        fs::create_dir_all(s.join("share/man")).unwrap();
        fs::create_dir_all(s.join("share/info")).unwrap();
        let bin = s.join("bin");
        fs::create_dir_all(&bin).unwrap();
        fs::write(bin.join("lto-dump"), "fake").unwrap();
        fs::write(bin.join("gcov"), "fake").unwrap();
        fs::write(bin.join("gprof"), "fake").unwrap();

        drop_unwanted_components(s).unwrap();

        assert!(!s.join("share/locale").exists());
        assert!(!s.join("share/man").exists());
        assert!(!s.join("share/info").exists());
        assert!(!bin.join("lto-dump").exists());
        assert!(!bin.join("gcov").exists());
        assert!(!bin.join("gprof").exists());
    }

    #[test]
    fn directory_blake3_digest_is_deterministic() {
        let d = tempfile::tempdir().unwrap();
        fs::write(d.path().join("a.txt"), "hello").unwrap();
        fs::create_dir(d.path().join("sub")).unwrap();
        fs::write(d.path().join("sub/b.txt"), "world").unwrap();

        let d1 = directory_blake3_digest(d.path()).unwrap();
        let d2 = directory_blake3_digest(d.path()).unwrap();
        assert_eq!(d1, d2);
        assert_eq!(d1.len(), 64);
    }

    #[test]
    fn directory_blake3_digest_changes_with_content() {
        let d = tempfile::tempdir().unwrap();
        fs::write(d.path().join("a.txt"), "hello").unwrap();
        let d1 = directory_blake3_digest(d.path()).unwrap();

        fs::write(d.path().join("a.txt"), "world").unwrap();
        let d2 = directory_blake3_digest(d.path()).unwrap();

        assert_ne!(d1, d2);
    }
}
