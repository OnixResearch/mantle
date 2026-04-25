//! Self-build: crunch builds itself from source.
//!
//! Copies the source tree (with vendored Cargo deps) directly into
//! the output store, generates a Nickel derivation that references it
//! as a plain source input, and delegates to the normal build pipeline.
//!
//! No tarball hashing, no NAR serialization, no FOD. The source tree
//! is just a directory in the store, like any other Nix source path.
//!
//! The output is a statically-linked crunch binary compiled inside a
//! bwrap sandbox using only the bootstrap toolchain.

use std::collections::BTreeMap;
use std::fmt;
use std::fs::File;
use std::io::Read;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use serde::Deserialize;
use sha2::Sha256;

use crate::build_cmd::build_import_paths;
use crate::build_cmd::load_configured_trusted_public_keys;
use crate::build_cmd::load_or_generate_signing_keypair;
use crate::build_cmd::report_build_result;
use crate::build_cmd::run_build;
use crate::errors::RunError;

/// Maximum source tree size: 2 GiB.
const MAX_SOURCE_BYTES: u64 = 2 * 1024 * 1024 * 1024;

/// Maximum number of source-tree entries copied during stage0 source staging.
const MAX_STAGE_SOURCE_ENTRIES: u32 = 200_000;

/// Maximum recursion depth during stage0 source staging.
const MAX_STAGE_SOURCE_DEPTH: u32 = 64;

/// Maximum locked packages parsed from Cargo.lock during vendor validation.
const MAX_CARGO_LOCK_PACKAGE_COUNT: u32 = 20_000;

/// Maximum top-level package directories accepted under vendor-deps/.
const MAX_VENDOR_PACKAGE_COUNT: u32 = 20_000;

/// Maximum files accepted inside one vendored package checksum manifest.
const MAX_VENDOR_PACKAGE_FILE_COUNT: u32 = 200_000;

/// Expected lowercase SHA-256 hex digest length in Cargo vendor metadata.
///
/// Cargo.lock and `.cargo-checksum.json` define this digest algorithm as part
/// of Cargo's interoperability format. Crunch-owned hashes remain BLAKE3.
const CARGO_SHA256_HEX_LEN: usize = 64;

/// One kibibyte in bytes for fixed-size hashing buffers.
const KIB_BYTES: usize = 1024;

/// Read-buffer capacity for streaming Cargo SHA-256 over vendored package files.
const CARGO_SHA256_READ_BUFFER_BYTES: usize = 64 * KIB_BYTES;

/// Top-level repo entries included in the staged source tree.
pub(crate) const STAGED_SOURCE_TOP_LEVEL_ENTRIES: &[&str] = &[
    ".cargo",
    "Cargo.lock",
    "Cargo.toml",
    "bootstrap",
    "builders",
    "crates",
    "lib",
    "rust-toolchain.toml",
    "src",
    "vendor",
    "vendor-deps",
];

/// Maximum number of `*-crunch` output directories to scan before giving up.
#[cfg_attr(not(test), allow(dead_code))]
const MAX_CRUNCH_OUTPUTS: u32 = 4096;

/// Bootstrap tool NCL files that MUST be built as separate roots before
/// the main crunch derivation. Adding or removing entries here changes
/// the self-build pipeline.
const REQUIRED_BOOTSTRAP_TOOLS: &[&str] = &["bwrap.ncl", "busybox.ncl"];

/// Number of steps in `cmd_self_build`. Tests assert against this to
/// catch step additions/removals.
const SELF_BUILD_STEP_COUNT: u32 = 4;

/// Stable line prefix used by the proof runner to identify structured
/// self-build evidence lines.
const PROOF_PREFIX: &str = "self-build-proof:";

/// Stable progress key carried inside proof lines.
const PROGRESS_KEY: &str = "progress=";

// ── Proof report types ────────────────────────────────────────────────

/// How the bwrap binary was resolved for a self-build stage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BwrapSource {
    /// Crunch-built bwrap found in the output store.
    CrunchBuilt(PathBuf),
    /// Host-provided bwrap found on PATH.
    HostFallback(PathBuf),
}

impl fmt::Display for BwrapSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BwrapSource::CrunchBuilt(p) => write!(f, "crunch-built:{}", p.display()),
            BwrapSource::HostFallback(p) => write!(f, "host-fallback:{}", p.display()),
        }
    }
}

impl BwrapSource {
    /// Parse from the stable string format produced by `Display`.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn parse(s: &str) -> Option<Self> {
        if let Some(rest) = s.strip_prefix("crunch-built:") {
            Some(BwrapSource::CrunchBuilt(PathBuf::from(rest)))
        } else {
            s.strip_prefix("host-fallback:").map(|rest| BwrapSource::HostFallback(PathBuf::from(rest)))
        }
    }

    /// True when this stage used a crunch-built bwrap.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn is_crunch_built(&self) -> bool {
        matches!(self, BwrapSource::CrunchBuilt(_))
    }

    /// Directory that must lead PATH for this bwrap choice.
    fn bin_dir(&self) -> Result<PathBuf, RunError> {
        match self {
            BwrapSource::CrunchBuilt(dir) => {
                assert!(!dir.as_os_str().is_empty(), "crunch-built bwrap dir must not be empty",);
                Ok(dir.clone())
            }
            BwrapSource::HostFallback(path) => {
                assert!(!path.as_os_str().is_empty(), "host fallback bwrap path must not be empty",);
                let parent = path.parent().ok_or_else(|| {
                    RunError::Internal(format!("host fallback bwrap has no parent directory: {}", path.display(),))
                })?;
                assert!(!parent.as_os_str().is_empty(), "host fallback bwrap parent dir must not be empty",);
                Ok(parent.to_path_buf())
            }
        }
    }
}

/// Host-fallback events that a self-build stage observed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelfBuildFallbackEvent {
    /// Self-build had to bootstrap with a host-provided bwrap.
    BwrapHostFallback(PathBuf),
    /// Self-build found its source tree by walking the host checkout.
    SourceHostDiscovery(PathBuf),
}

impl fmt::Display for SelfBuildFallbackEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BwrapHostFallback(path) => write!(f, "bwrap-host-fallback:{}", path.display()),
            Self::SourceHostDiscovery(path) => write!(f, "source-host-discovery:{}", path.display()),
        }
    }
}

impl SelfBuildFallbackEvent {
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn parse(s: &str) -> Option<Self> {
        if let Some(rest) = s.strip_prefix("bwrap-host-fallback:") {
            return Some(Self::BwrapHostFallback(PathBuf::from(rest)));
        }
        if let Some(rest) = s.strip_prefix("source-host-discovery:") {
            return Some(Self::SourceHostDiscovery(PathBuf::from(rest)));
        }
        None
    }
}

/// Structured report from a self-build run.
///
/// Captures the facts a proof runner needs to verify that self-hosting
/// works: which binary drove the build, which sandbox tools were
/// selected, and where the output landed.
#[derive(Debug, Clone)]
pub struct SelfBuildReport {
    /// Hermeticity mode selected for this self-build.
    pub hermeticity_mode: crunch_pipeline::HermeticityMode,
    /// Path to the crunch binary that drove this self-build.
    pub invoking_binary: PathBuf,
    /// Exact staged source tree used for this self-build.
    pub staged_source: PathBuf,
    /// How bwrap was resolved after bootstrap tools were available.
    pub bwrap_source: BwrapSource,
    /// Host-fallback events observed earlier in the stage.
    pub fallback_events: Vec<SelfBuildFallbackEvent>,
    /// Path to the busybox binary that the NCL script will use for
    /// `SNIX_BUILD_SANDBOX_SHELL`. `None` if no crunch-built busybox
    /// was found in the output store (falls back to `/bin/sh`).
    pub busybox_path: Option<PathBuf>,
    /// Path to the produced output binary.
    pub output_binary: PathBuf,
}

fn emit_progress_marker(marker: &str) {
    assert!(!marker.is_empty(), "progress marker must not be empty");
    eprintln!("{PROOF_PREFIX} {PROGRESS_KEY}{marker}");
}

impl SelfBuildReport {
    /// Format the report as stable `self-build-proof:` lines.
    ///
    /// Each line is `self-build-proof: key=value`. A proof runner can
    /// filter stderr for this prefix and parse the key-value pairs.
    pub fn format_proof_lines(&self) -> String {
        let mut out = String::with_capacity(512);
        out.push_str(&format!("{PROOF_PREFIX} hermeticity-mode={}\n", self.hermeticity_mode.as_str(),));
        out.push_str(&format!("{PROOF_PREFIX} invoking-binary={}\n", self.invoking_binary.display(),));
        out.push_str(&format!("{PROOF_PREFIX} staged-source={}\n", self.staged_source.display(),));
        out.push_str(&format!("{PROOF_PREFIX} bwrap-source={}\n", self.bwrap_source,));
        if self.fallback_events.is_empty() {
            out.push_str(&format!("{PROOF_PREFIX} fallback-event=none\n"));
        } else {
            for event in &self.fallback_events {
                out.push_str(&format!("{PROOF_PREFIX} fallback-event={}\n", event));
            }
        }
        match &self.busybox_path {
            Some(p) => out.push_str(&format!("{PROOF_PREFIX} busybox-path={}\n", p.display(),)),
            None => out.push_str(&format!("{PROOF_PREFIX} busybox-path=none\n",)),
        }
        out.push_str(&format!("{PROOF_PREFIX} output-binary={}\n", self.output_binary.display(),));
        out
    }

    /// Parse a report from lines previously produced by
    /// `format_proof_lines`. Returns `None` when any required field
    /// is missing.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn parse_proof_lines(text: &str) -> Option<Self> {
        let mut hermeticity_mode: Option<crunch_pipeline::HermeticityMode> = None;
        let mut invoking_binary: Option<PathBuf> = None;
        let mut staged_source: Option<PathBuf> = None;
        let mut bwrap_source: Option<BwrapSource> = None;
        let mut fallback_events: Vec<SelfBuildFallbackEvent> = Vec::new();
        let mut busybox_path: Option<Option<PathBuf>> = None;
        let mut output_binary: Option<PathBuf> = None;

        for line in text.lines() {
            let trimmed = line.trim();
            let rest = match trimmed.strip_prefix(PROOF_PREFIX) {
                Some(r) => r.trim(),
                None => continue,
            };
            if let Some(val) = rest.strip_prefix("hermeticity-mode=") {
                hermeticity_mode = match val {
                    "practical" => Some(crunch_pipeline::HermeticityMode::Practical),
                    "strict" => Some(crunch_pipeline::HermeticityMode::Strict),
                    _ => None,
                };
            } else if let Some(val) = rest.strip_prefix("invoking-binary=") {
                invoking_binary = Some(PathBuf::from(val));
            } else if let Some(val) = rest.strip_prefix("staged-source=") {
                staged_source = Some(PathBuf::from(val));
            } else if let Some(val) = rest.strip_prefix("bwrap-source=") {
                bwrap_source = BwrapSource::parse(val);
            } else if let Some(val) = rest.strip_prefix("fallback-event=") {
                if val != "none" {
                    let event = SelfBuildFallbackEvent::parse(val)?;
                    fallback_events.push(event);
                }
            } else if let Some(val) = rest.strip_prefix("busybox-path=") {
                if val == "none" {
                    busybox_path = Some(None);
                } else {
                    busybox_path = Some(Some(PathBuf::from(val)));
                }
            } else if let Some(val) = rest.strip_prefix("output-binary=") {
                output_binary = Some(PathBuf::from(val));
            }
        }

        Some(SelfBuildReport {
            hermeticity_mode: hermeticity_mode?,
            invoking_binary: invoking_binary?,
            staged_source: staged_source?,
            bwrap_source: bwrap_source?,
            fallback_events,
            busybox_path: busybox_path?,
            output_binary: output_binary?,
        })
    }
}

/// Stage the crunch source tree into the output store.
///
/// 1. Copy the selected source tree into staging with Rust filesystem calls
/// 2. Check the staged vendored Cargo inputs are present and fresh
/// 3. Compute the staged-tree fingerprint
/// 4. Copy the staged tree into `$store_dir/$hash-crunch-src/`
///
/// Returns the store path name (e.g., "abcdef...-crunch-src").
pub fn stage_source(src_dir: &Path, store_dir: &Path) -> Result<String, RunError> {
    let staging = tempfile::tempdir().map_err(|e| RunError::Internal(format!("tmpdir: {e}")))?;
    let stage_root = staging.path().join("crunch-src");
    std::fs::create_dir_all(&stage_root).map_err(|e| RunError::Internal(format!("mkdir staging: {e}")))?;

    eprintln!("  copying selected source tree...");
    copy_selected_source_tree(src_dir, &stage_root)?;
    assert!(stage_root.join("Cargo.toml").exists(), "staged source must contain Cargo.toml");
    assert!(!stage_root.join(".git").exists(), "staged source must not contain .git");

    eprintln!("  checking staged vendored cargo inputs...");
    require_checked_vendor_inputs(&stage_root)?;

    let size = dir_size(&stage_root);
    assert!(
        size <= MAX_SOURCE_BYTES,
        "source tree {} MiB exceeds {} MiB limit",
        size / (1024 * 1024),
        MAX_SOURCE_BYTES / (1024 * 1024),
    );
    eprintln!("  source tree: {} MiB", size / (1024 * 1024));

    let fingerprint = tree_fingerprint(&stage_root)?;
    let store_name = staged_source_store_name_from_fingerprint(&fingerprint)?;
    let dest = store_dir.join(&store_name);

    if dest.exists() {
        eprintln!("  source already staged: {}", dest.display());
        return Ok(store_name);
    }

    let mut copied_entry_count: u32 = 0;
    copy_tree_entry(&stage_root, &dest, 0, &mut copied_entry_count)?;
    assert!(copied_entry_count > 0, "staged source copy must copy at least one entry");

    eprintln!("  staged: {}", dest.display());
    Ok(store_name)
}

/// Generate the Nickel derivation for building crunch.
///
/// The source tree is referenced as a plain input path (not a FOD).
const SELF_BUILD_NCL_TEMPLATE: &str = r#"# Auto-generated by `crunch self-build`.
let crunch = import "lib/lib.ncl" in

let seed = import "bootstrap/seed.ncl" in
let toolchain = seed.toolchain in
let seed_name = seed.name in
let seed_target = seed.target in
let seed_dynamic_linker = seed.dynamic_linker in
let seed_sysroot_lib = seed_target ++ "/lib" in

let gnumake = (import "bootstrap/make.ncl") in
let dash = (import "bootstrap/dash.ncl") in
let binutils = (import "bootstrap/binutils.ncl") in
let musl = (import "bootstrap/musl.ncl") in
let gcc = (import "bootstrap/gcc.ncl") in
let rust = (import "bootstrap/rust.ncl") in

{
  name = "crunch",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -e
      BB=/bin/busybox
      $BB mkdir -p /tmp/tools
      for cmd in cat mkdir cp chmod ln ls echo rm mv sed grep awk \
                 tr head tail sort wc expr test basename dirname \
                 install find xargs readlink touch true false tee \
                 du cut uname id whoami env printenv; do
        $BB ln -sf $BB /tmp/tools/$cmd
      done

      GCC=""
      for d in $NIX_STORE/*-gcc; do
        if [ -x "$d/bin/gcc" ]; then GCC="$d"; break; fi
      done
      BINUTILS=""
      for d in $NIX_STORE/*-binutils; do
        if [ -d "$d/bin" ]; then BINUTILS="$d"; break; fi
      done
      MUSL=""
      for d in $NIX_STORE/*-musl; do
        if [ -d "$d/include" ]; then MUSL="$d"; break; fi
      done
      DASH=""
      for d in $NIX_STORE/*-dash; do
        if [ -x "$d/bin/dash" ]; then DASH="$d"; break; fi
      done
      MAKE=""
      for d in $NIX_STORE/*-gnumake; do
        if [ -x "$d/bin/make" ]; then MAKE="$d"; break; fi
      done
      RUST=""
      for d in $NIX_STORE/*-rust; do
        if [ -x "$d/bin/rustc" ]; then RUST="$d"; break; fi
      done

      CRUNCH_SRC="$NIX_STORE/__SRC_STORE_PATH__"
      if [ ! -f "$CRUNCH_SRC/Cargo.toml" ]; then
        echo "ERROR: CRUNCH_SRC not found at $CRUNCH_SRC" >&2
        exit 1
      fi

      for tool in GCC BINUTILS MUSL DASH MAKE RUST; do
        eval val=\$$tool
        if [ -z "$val" ]; then
          echo "ERROR: $tool not found" >&2; exit 1
        fi
        echo "$tool=$val"
      done
      echo "CRUNCH_SRC=$CRUNCH_SRC"

      $BB mkdir -p /lib 2>/dev/null || true
      $BB ln -sf $MUSL/lib/libc.so /lib/%{seed_dynamic_linker} 2>/dev/null || true

      MUSL_GCC=""
      for d in $NIX_STORE/*-%{seed_name}; do
        if [ -f "$d/%{seed_sysroot_lib}/libgcc_s.so.1" ]; then MUSL_GCC="$d/%{seed_sysroot_lib}"; break; fi
      done
      if [ -n "$MUSL_GCC" ]; then
        export LD_LIBRARY_PATH="$MUSL_GCC${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
      elif [ -f "$GCC/lib/libgcc_s.so.1" ]; then
        export LD_LIBRARY_PATH="$GCC/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
      fi

      $BB ln -sf "$DASH/bin/dash" /tmp/tools/sh

      for tool in as ld ld.bfd ar nm objcopy objdump ranlib readelf strip; do
        if [ -x "$BINUTILS/bin/%{seed_target}-$tool" ] && [ ! -e "/tmp/tools/$tool" ]; then
          $BB ln -sf "$BINUTILS/bin/%{seed_target}-$tool" /tmp/tools/$tool
        fi
      done

      BWRAP_BIN="__STORE_PREFIX__/__BWRAP_STORE_PATH__/bin"
      BWRAP_PATH=""
      if [ -x "$BWRAP_BIN/bwrap" ]; then
        echo "Using crunch-built bwrap: $BWRAP_BIN"
        BWRAP_PATH="$BWRAP_BIN:"
      else
        echo "WARNING: crunch-built bwrap not found at $BWRAP_BIN/bwrap" >&2
      fi
      export PATH="/tmp/tools:${BWRAP_PATH}$RUST/bin:$GCC/bin:$BINUTILS/bin:$MAKE/bin"

      echo "=== Tool versions ==="
      rustc --version
      cargo --version
      gcc --version | head -1
      make --version | head -1

      $BB mkdir -p /tmp/build
      cp -r "$CRUNCH_SRC" /tmp/build/crunch 2>/dev/null
      chmod -R u+w /tmp/build/crunch
      cd /tmp/build/crunch

      GCC_LIB=""
      if [ -n "$MUSL_GCC" ]; then GCC_LIB="$MUSL_GCC"
      elif [ -f "$GCC/lib/libgcc_s.so" ]; then GCC_LIB="$GCC/lib"
      fi

      $BB mkdir -p .cargo
      cp "$CRUNCH_SRC/.cargo/vendor-config.toml" .cargo/config.toml
      chmod u+w .cargo/config.toml
      cat >> .cargo/config.toml << CARGOEOF

[build]
target = "x86_64-unknown-linux-musl"

[target.x86_64-unknown-linux-musl]
linker = "gcc"
rustflags = ["-C", "link-arg=-Wl,--allow-multiple-definition", "-C", "link-arg=-L${GCC_LIB}"]
CARGOEOF

      export CARGO_HOME=/tmp/cargo-home
      export CARGO_TARGET_DIR=/tmp/cargo-target
      BUSYBOX_BIN="__STORE_PREFIX__/__BUSYBOX_STORE_PATH__/bin/busybox"
      if [ -x "$BUSYBOX_BIN" ]; then
        echo "Using crunch-built busybox: $BUSYBOX_BIN"
        export SNIX_BUILD_SANDBOX_SHELL="$BUSYBOX_BIN"
      else
        echo "WARNING: crunch-built busybox not found at $BUSYBOX_BIN" >&2
        export SNIX_BUILD_SANDBOX_SHELL=/bin/sh
      fi

      export CC=gcc
      export AR=ar
      export TARGET_CC=gcc
      export TARGET_AR=ar
      export HOST_CC=gcc
      export RUSTC_BOOTSTRAP=1

      if [ -n "$GCC_LIB" ]; then
        export LIBRARY_PATH="$GCC_LIB${LIBRARY_PATH:+:$LIBRARY_PATH}"
      fi

      echo "=== Building crunch ==="
      cargo build --release --locked -j 4 2>&1 || exit 1

      echo "=== Installing ==="
      mkdir -p $out/bin
      cp /tmp/cargo-target/x86_64-unknown-linux-musl/release/crunch $out/bin/

      echo "=== Verify ==="
      ls -la $out/bin/crunch
      file $out/bin/crunch 2>/dev/null || true
      $out/bin/crunch --version 2>&1 | head -3 || \
        $out/bin/crunch --help 2>&1 | head -3
    "%,
  ],
  inputs = [
    toolchain, gnumake, dash, binutils, musl, gcc, rust,
    "__STORE_PREFIX__/__BUSYBOX_STORE_PATH__",
    "__STORE_PREFIX__/__BWRAP_STORE_PATH__",
    "__STORE_PREFIX__/__SRC_STORE_PATH__",
  ],
} | crunch.Derivation
"#;

pub fn generate_self_build_ncl(
    src_store_path: &str,
    bwrap_store_path: &str,
    busybox_store_path: &str,
    store_prefix: &str,
) -> String {
    assert!(!src_store_path.is_empty(), "staged source store path must not be empty");
    assert!(!bwrap_store_path.is_empty(), "bwrap store path must not be empty");
    assert!(!busybox_store_path.is_empty(), "busybox store path must not be empty");
    assert!(store_prefix.starts_with('/'), "store prefix must be absolute");
    SELF_BUILD_NCL_TEMPLATE
        .replace("__STORE_PREFIX__", store_prefix)
        .replace("__SRC_STORE_PATH__", src_store_path)
        .replace("__BWRAP_STORE_PATH__", bwrap_store_path)
        .replace("__BUSYBOX_STORE_PATH__", busybox_store_path)
}

/// Verify the self-built binary by running `--help`.
pub fn verify_binary(binary_path: &Path) -> Result<(), RunError> {
    eprintln!("verifying {}...", binary_path.display());

    let out = Command::new(binary_path)
        .arg("--help")
        .output()
        .map_err(|e| RunError::Internal(format!("failed to run self-built binary: {e}")))?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(RunError::Build(format!(
            "self-built binary exited {} on --help:\n{stderr}",
            out.status.code().unwrap_or(-1),
        )));
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("crunch"), "self-built binary --help doesn't mention 'crunch'");

    eprintln!("  binary OK");
    Ok(())
}

// ── helpers ────────────────────────────────────────────────────────────

/// Run a command, returning an error with stderr on failure.
#[cfg_attr(not(test), allow(dead_code))]
fn run_cmd(cmd: &mut Command, label: &str) -> Result<(), RunError> {
    let out = cmd.output().map_err(|e| RunError::Internal(format!("failed to run `{label}`: {e}")))?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(RunError::Internal(format!(
            "{label} failed (exit {}):\n{stderr}",
            out.status.code().unwrap_or(-1),
        )));
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct PackageKey {
    name: String,
    version: String,
}

impl fmt::Display for PackageKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.name, self.version)
    }
}

#[derive(Debug, Clone)]
struct LockedPackage {
    key: PackageKey,
    source: Option<String>,
    checksum: Option<String>,
}

#[derive(Debug, Default)]
struct LockedPackageBuilder {
    name: Option<String>,
    version: Option<String>,
    source: Option<String>,
    checksum: Option<String>,
}

#[derive(Debug)]
struct ExpectedVendorPackage {
    source: String,
    checksum: Option<String>,
}

#[derive(Debug)]
struct VendoredPackage {
    package_checksum: Option<String>,
}

#[derive(Debug, Deserialize)]
struct VendorChecksumManifest {
    files: BTreeMap<String, String>,
    package: Option<String>,
}

/// Require the source-tree vendored Cargo inputs to be present and internally
/// fresh enough for self-build.
///
/// This guard is intentionally host-tool-free: it reads `Cargo.lock`, the
/// source-tree Cargo vendor config, and Cargo's `.cargo-checksum.json` files
/// directly instead of shelling out to host `cargo`, `git`, `nix`, or network
/// tools before the long bootstrap begins.
pub(crate) fn require_checked_vendor_inputs(src_dir: &Path) -> Result<(), RunError> {
    let vendor_dir = src_dir.join("vendor-deps");
    let vendor_config = src_dir.join(".cargo").join("vendor-config.toml");
    if !vendor_dir.is_dir() {
        return Err(RunError::Internal(format!("source-tree vendor-deps/ missing: {}", vendor_dir.display())));
    }
    let vendor_config_text = std::fs::read_to_string(&vendor_config)
        .map_err(|e| RunError::Internal(format!("read {}: {e}", vendor_config.display())))?;
    if !vendor_config_text.contains("directory = \"vendor-deps\"") {
        return Err(RunError::Internal(format!(
            "vendor config must point at source-tree vendor-deps/: {}",
            vendor_config.display(),
        )));
    }
    verify_checked_vendor_freshness(src_dir, &vendor_dir)
}

fn verify_checked_vendor_freshness(src_dir: &Path, vendor_dir: &Path) -> Result<(), RunError> {
    assert!(src_dir.is_dir(), "source dir must exist: {}", src_dir.display());
    assert!(vendor_dir.is_dir(), "vendor dir must exist: {}", vendor_dir.display());
    let lock_path = src_dir.join("Cargo.lock");
    let lock_text = std::fs::read_to_string(&lock_path)
        .map_err(|e| RunError::Internal(format!("read {}: {e}", lock_path.display())))?;
    let locked_packages = parse_cargo_lock_packages(&lock_text)?;
    let expected = expected_vendor_packages(&locked_packages)?;
    let actual = load_vendored_packages(vendor_dir)?;
    verify_vendor_package_set(&expected, &actual)
}

fn parse_cargo_lock_packages(lock_text: &str) -> Result<Vec<LockedPackage>, RunError> {
    let mut packages: Vec<LockedPackage> = Vec::new();
    let mut current: Option<LockedPackageBuilder> = None;
    for raw_line in lock_text.lines() {
        let line = raw_line.trim();
        if line == "[[package]]" {
            push_locked_package(&mut packages, current.take())?;
            current = Some(LockedPackageBuilder::default());
            continue;
        }
        if let Some(builder) = current.as_mut() {
            apply_lock_package_field(builder, line)?;
        }
    }
    push_locked_package(&mut packages, current)?;
    Ok(packages)
}

fn push_locked_package(
    packages: &mut Vec<LockedPackage>,
    builder: Option<LockedPackageBuilder>,
) -> Result<(), RunError> {
    let Some(builder) = builder else {
        return Ok(());
    };
    if packages.len() >= MAX_CARGO_LOCK_PACKAGE_COUNT as usize {
        return Err(RunError::Internal(format!("Cargo.lock package count exceeds {}", MAX_CARGO_LOCK_PACKAGE_COUNT,)));
    }
    let name = builder.name.ok_or_else(|| RunError::Internal("Cargo.lock package missing name".to_string()))?;
    let version = builder
        .version
        .ok_or_else(|| RunError::Internal(format!("Cargo.lock package {name} missing version")))?;
    packages.push(LockedPackage {
        key: PackageKey { name, version },
        source: builder.source,
        checksum: builder.checksum,
    });
    Ok(())
}

fn apply_lock_package_field(builder: &mut LockedPackageBuilder, line: &str) -> Result<(), RunError> {
    if let Some(value) = parse_quoted_field(line, "name")? {
        builder.name = Some(value);
        return Ok(());
    }
    if let Some(value) = parse_quoted_field(line, "version")? {
        builder.version = Some(value);
        return Ok(());
    }
    if let Some(value) = parse_quoted_field(line, "source")? {
        builder.source = Some(value);
        return Ok(());
    }
    if let Some(value) = parse_quoted_field(line, "checksum")? {
        ensure_cargo_sha256_hex("Cargo.lock checksum", &value)?;
        builder.checksum = Some(value);
    }
    Ok(())
}

fn expected_vendor_packages(
    packages: &[LockedPackage],
) -> Result<BTreeMap<PackageKey, ExpectedVendorPackage>, RunError> {
    let mut expected: BTreeMap<PackageKey, ExpectedVendorPackage> = BTreeMap::new();
    for package in packages {
        let Some(source) = package.source.as_ref() else {
            continue;
        };
        if !is_supported_vendor_source(source) {
            return Err(RunError::Internal(format!(
                "unsupported vendored Cargo source for {}: {}",
                package.key, source,
            )));
        }
        if is_registry_vendor_source(source) && package.checksum.is_none() {
            return Err(RunError::Internal(format!("registry package {} is missing Cargo.lock checksum", package.key)));
        }
        let previous = expected.insert(package.key.clone(), ExpectedVendorPackage {
            source: source.clone(),
            checksum: package.checksum.clone(),
        });
        if previous.is_some() {
            return Err(RunError::Internal(format!("duplicate vendored package in Cargo.lock: {}", package.key)));
        }
    }
    Ok(expected)
}

fn load_vendored_packages(vendor_dir: &Path) -> Result<BTreeMap<PackageKey, VendoredPackage>, RunError> {
    let mut children: Vec<PathBuf> = Vec::new();
    let entries = std::fs::read_dir(vendor_dir)
        .map_err(|e| RunError::Internal(format!("read_dir {}: {e}", vendor_dir.display())))?;
    for entry in entries {
        let entry = entry.map_err(|e| RunError::Internal(format!("read_dir entry {}: {e}", vendor_dir.display())))?;
        children.push(entry.path());
    }
    children.sort();
    vendored_package_index(&children)
}

fn vendored_package_index(children: &[PathBuf]) -> Result<BTreeMap<PackageKey, VendoredPackage>, RunError> {
    let mut packages: BTreeMap<PackageKey, VendoredPackage> = BTreeMap::new();
    for child in children {
        if packages.len() >= MAX_VENDOR_PACKAGE_COUNT as usize {
            return Err(RunError::Internal(format!("vendor-deps package count exceeds {}", MAX_VENDOR_PACKAGE_COUNT)));
        }
        let metadata = std::fs::symlink_metadata(child)
            .map_err(|e| RunError::Internal(format!("symlink_metadata {}: {e}", child.display())))?;
        if !metadata.is_dir() {
            return Err(RunError::Internal(format!("vendor-deps entry is not a directory: {}", child.display())));
        }
        let (key, package) = read_vendored_package(child)?;
        let previous = packages.insert(key.clone(), package);
        if previous.is_some() {
            return Err(RunError::Internal(format!("duplicate vendored package directory for {key}")));
        }
    }
    Ok(packages)
}

fn read_vendored_package(package_dir: &Path) -> Result<(PackageKey, VendoredPackage), RunError> {
    let manifest_path = package_dir.join("Cargo.toml");
    let manifest_text = std::fs::read_to_string(&manifest_path)
        .map_err(|e| RunError::Internal(format!("read {}: {e}", manifest_path.display())))?;
    let key = parse_vendor_manifest_package(&manifest_text, &manifest_path)?;
    let checksum_path = package_dir.join(".cargo-checksum.json");
    let checksum = read_vendor_checksum_manifest(&checksum_path)?;
    verify_vendor_file_checksums(package_dir, &checksum)?;
    Ok((key, VendoredPackage {
        package_checksum: checksum.package,
    }))
}

fn parse_vendor_manifest_package(manifest_text: &str, manifest_path: &Path) -> Result<PackageKey, RunError> {
    let mut in_package_section = false;
    let mut name: Option<String> = None;
    let mut version: Option<String> = None;
    for raw_line in manifest_text.lines() {
        let line = raw_line.trim();
        if line.starts_with('[') {
            in_package_section = line == "[package]";
            continue;
        }
        if !in_package_section {
            continue;
        }
        if let Some(value) = parse_quoted_field(line, "name")? {
            name = Some(value);
        }
        if let Some(value) = parse_quoted_field(line, "version")? {
            version = Some(value);
        }
    }
    let name = name.ok_or_else(|| {
        RunError::Internal(format!("vendor manifest missing package name: {}", manifest_path.display()))
    })?;
    let version = version.ok_or_else(|| {
        RunError::Internal(format!("vendor manifest {name} missing package version: {}", manifest_path.display()))
    })?;
    Ok(PackageKey { name, version })
}

fn read_vendor_checksum_manifest(checksum_path: &Path) -> Result<VendorChecksumManifest, RunError> {
    let bytes = std::fs::read(checksum_path)
        .map_err(|e| RunError::Internal(format!("read {}: {e}", checksum_path.display())))?;
    let manifest: VendorChecksumManifest = serde_json::from_slice(&bytes)
        .map_err(|e| RunError::Internal(format!("parse {}: {e}", checksum_path.display())))?;
    if let Some(package_checksum) = manifest.package.as_ref() {
        ensure_cargo_sha256_hex("vendor package checksum", package_checksum)?;
    }
    for digest in manifest.files.values() {
        ensure_cargo_sha256_hex("vendor file checksum", digest)?;
    }
    Ok(manifest)
}

fn verify_vendor_file_checksums(package_dir: &Path, manifest: &VendorChecksumManifest) -> Result<(), RunError> {
    let actual = vendor_file_hashes(package_dir)?;
    for (relative_path, expected_digest) in &manifest.files {
        if !actual.contains_key(relative_path) {
            return Err(RunError::Internal(format!(
                "vendor checksum lists missing file {} in {}",
                relative_path,
                package_dir.display(),
            )));
        }
        let actual_digest = actual.get(relative_path).expect("contains_key checked above");
        if actual_digest != expected_digest {
            return Err(RunError::Internal(format!(
                "vendor file checksum mismatch: {}/{}",
                package_dir.display(),
                relative_path
            )));
        }
    }
    for relative_path in actual.keys() {
        if !manifest.files.contains_key(relative_path) {
            return Err(RunError::Internal(format!(
                "vendor package contains file missing from checksum manifest: {}/{}",
                package_dir.display(),
                relative_path,
            )));
        }
    }
    Ok(())
}

fn vendor_file_hashes(package_dir: &Path) -> Result<BTreeMap<String, String>, RunError> {
    let mut hashes: BTreeMap<String, String> = BTreeMap::new();
    let mut file_count: u32 = 0;
    collect_vendor_file_hashes(package_dir, package_dir, &mut hashes, &mut file_count)?;
    assert!(file_count as usize == hashes.len(), "vendor file count must match hash map length");
    Ok(hashes)
}

fn collect_vendor_file_hashes(
    package_dir: &Path,
    current_dir: &Path,
    hashes: &mut BTreeMap<String, String>,
    file_count: &mut u32,
) -> Result<(), RunError> {
    let mut children: Vec<PathBuf> = Vec::new();
    let entries = std::fs::read_dir(current_dir)
        .map_err(|e| RunError::Internal(format!("read_dir {}: {e}", current_dir.display())))?;
    for entry in entries {
        let entry = entry.map_err(|e| RunError::Internal(format!("read_dir entry {}: {e}", current_dir.display())))?;
        children.push(entry.path());
    }
    children.sort();
    for child in &children {
        collect_vendor_child_hash(package_dir, child, hashes, file_count)?;
    }
    Ok(())
}

fn collect_vendor_child_hash(
    package_dir: &Path,
    child: &Path,
    hashes: &mut BTreeMap<String, String>,
    file_count: &mut u32,
) -> Result<(), RunError> {
    let metadata = std::fs::symlink_metadata(child)
        .map_err(|e| RunError::Internal(format!("symlink_metadata {}: {e}", child.display())))?;
    if metadata.file_type().is_symlink() {
        return Err(RunError::Internal(format!("vendored package contains symlink: {}", child.display())));
    }
    if metadata.is_dir() {
        return collect_vendor_file_hashes(package_dir, child, hashes, file_count);
    }
    if !metadata.is_file() {
        return Err(RunError::Internal(format!("vendored package contains unsupported entry: {}", child.display())));
    }
    collect_vendor_file_hash(package_dir, child, hashes, file_count)
}

fn collect_vendor_file_hash(
    package_dir: &Path,
    child: &Path,
    hashes: &mut BTreeMap<String, String>,
    file_count: &mut u32,
) -> Result<(), RunError> {
    let relative_path = vendor_relative_path(package_dir, child)?;
    if relative_path == ".cargo-checksum.json" {
        return Ok(());
    }
    *file_count = file_count.saturating_add(1);
    if *file_count > MAX_VENDOR_PACKAGE_FILE_COUNT {
        return Err(RunError::Internal(format!(
            "vendor package file count exceeds {} in {}",
            MAX_VENDOR_PACKAGE_FILE_COUNT,
            package_dir.display(),
        )));
    }
    let digest = hash_file_cargo_sha256_hex(child)?;
    let previous = hashes.insert(relative_path, digest);
    assert!(previous.is_none(), "vendor relative file paths must be unique");
    Ok(())
}

fn verify_vendor_package_set(
    expected: &BTreeMap<PackageKey, ExpectedVendorPackage>,
    actual: &BTreeMap<PackageKey, VendoredPackage>,
) -> Result<(), RunError> {
    for (key, expected_package) in expected {
        let actual_package = actual
            .get(key)
            .ok_or_else(|| RunError::Internal(format!("Cargo.lock package missing from vendor-deps/: {key}")))?;
        verify_vendor_package_lock_checksum(key, expected_package, actual_package)?;
    }
    for key in actual.keys() {
        if !expected.contains_key(key) {
            return Err(RunError::Internal(format!("vendor-deps/ contains package not present in Cargo.lock: {key}")));
        }
    }
    Ok(())
}

fn verify_vendor_package_lock_checksum(
    key: &PackageKey,
    expected: &ExpectedVendorPackage,
    actual: &VendoredPackage,
) -> Result<(), RunError> {
    if let Some(expected_checksum) = expected.checksum.as_ref() {
        let actual_checksum = actual
            .package_checksum
            .as_ref()
            .ok_or_else(|| RunError::Internal(format!("vendored package {key} is missing package checksum")))?;
        if actual_checksum != expected_checksum {
            return Err(RunError::Internal(format!("vendored package checksum mismatch for {key}")));
        }
        return Ok(());
    }
    if is_registry_vendor_source(&expected.source) {
        return Err(RunError::Internal(format!("registry package {key} has no Cargo.lock checksum")));
    }
    if actual.package_checksum.is_some() {
        return Err(RunError::Internal(format!(
            "non-registry vendored package {key} unexpectedly has package checksum"
        )));
    }
    Ok(())
}

fn parse_quoted_field(line: &str, expected_key: &str) -> Result<Option<String>, RunError> {
    let Some((key, raw_value)) = line.split_once('=') else {
        return Ok(None);
    };
    if key.trim() != expected_key {
        return Ok(None);
    }
    parse_basic_quoted_value(raw_value).map(Some)
}

fn parse_basic_quoted_value(raw_value: &str) -> Result<String, RunError> {
    let value = raw_value.trim();
    if !value.starts_with('"') {
        return Err(RunError::Internal(format!("expected quoted Cargo value, got: {value}")));
    }
    if !value.ends_with('"') {
        return Err(RunError::Internal(format!("unterminated quoted Cargo value: {value}")));
    }
    if value.len() < 2 {
        return Err(RunError::Internal(format!("empty quoted Cargo delimiter: {value}")));
    }
    let inner = &value[1..value.len() - 1];
    if inner.contains('\\') {
        return Err(RunError::Internal(format!(
            "escaped Cargo value is not supported by self-build validator: {value}"
        )));
    }
    Ok(inner.to_string())
}

fn vendor_relative_path(package_dir: &Path, child: &Path) -> Result<String, RunError> {
    let relative_path = child.strip_prefix(package_dir).map_err(|e| {
        RunError::Internal(format!("strip vendor prefix {} from {}: {e}", package_dir.display(), child.display()))
    })?;
    let relative_str = relative_path.to_str().ok_or_else(|| {
        RunError::Internal(format!("vendored path is not UTF-8 under {}: {}", package_dir.display(), child.display()))
    })?;
    Ok(relative_str.replace(std::path::MAIN_SEPARATOR, "/"))
}

fn hash_file_cargo_sha256_hex(path: &Path) -> Result<String, RunError> {
    let mut file = File::open(path).map_err(|e| RunError::Internal(format!("open {}: {e}", path.display())))?;
    let mut hasher = <Sha256 as sha2::Digest>::new();
    let mut buffer = [0_u8; CARGO_SHA256_READ_BUFFER_BYTES];
    loop {
        let bytes_read =
            file.read(&mut buffer).map_err(|e| RunError::Internal(format!("read {}: {e}", path.display())))?;
        if bytes_read == 0 {
            break;
        }
        <Sha256 as sha2::Digest>::update(&mut hasher, &buffer[..bytes_read]);
    }
    let digest = <Sha256 as sha2::Digest>::finalize(hasher);
    Ok(data_encoding::HEXLOWER.encode(&digest))
}

fn ensure_cargo_sha256_hex(label: &str, value: &str) -> Result<(), RunError> {
    if value.len() != CARGO_SHA256_HEX_LEN {
        return Err(RunError::Internal(format!("{label} must be {CARGO_SHA256_HEX_LEN} lowercase hex chars")));
    }
    if !value.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()) {
        return Err(RunError::Internal(format!("{label} must be lowercase SHA-256 hex: {value}")));
    }
    Ok(())
}

fn is_supported_vendor_source(source: &str) -> bool {
    is_registry_vendor_source(source) || source.starts_with("git+")
}

fn is_registry_vendor_source(source: &str) -> bool {
    source.starts_with("registry+") || source.starts_with("sparse+")
}

fn copy_selected_source_tree(src_dir: &Path, stage_root: &Path) -> Result<(), RunError> {
    assert!(src_dir.is_dir(), "source dir must exist: {}", src_dir.display());
    assert!(stage_root.is_dir(), "stage root must exist: {}", stage_root.display());
    let mut copied_entry_count: u32 = 0;
    for entry_name in STAGED_SOURCE_TOP_LEVEL_ENTRIES {
        let source_path = src_dir.join(entry_name);
        let dest_path = stage_root.join(entry_name);
        if !source_path.exists() {
            return Err(RunError::Internal(
                format!("required staged source entry missing: {}", source_path.display(),),
            ));
        }
        copy_tree_entry(&source_path, &dest_path, 0, &mut copied_entry_count)?;
    }
    assert!(copied_entry_count > 0, "source staging must copy at least one entry");
    assert!(copied_entry_count <= MAX_STAGE_SOURCE_ENTRIES, "source staging copied too many entries");
    Ok(())
}

fn copy_tree_entry(source: &Path, dest: &Path, depth: u32, copied_entry_count: &mut u32) -> Result<(), RunError> {
    assert!(source.exists(), "source entry must exist: {}", source.display());
    assert!(depth <= MAX_STAGE_SOURCE_DEPTH, "stage source recursion depth exceeded {MAX_STAGE_SOURCE_DEPTH}");
    *copied_entry_count = copied_entry_count.saturating_add(1);
    if *copied_entry_count > MAX_STAGE_SOURCE_ENTRIES {
        return Err(RunError::Internal(format!(
            "stage source entry count exceeded {} while copying {}",
            MAX_STAGE_SOURCE_ENTRIES,
            source.display(),
        )));
    }

    let metadata = std::fs::symlink_metadata(source)
        .map_err(|e| RunError::Internal(format!("symlink_metadata {}: {e}", source.display())))?;
    if metadata.file_type().is_symlink() {
        return copy_symlink_entry(source, dest);
    }
    if metadata.is_file() {
        return copy_file_entry(source, dest, metadata.permissions());
    }
    if metadata.is_dir() {
        return copy_dir_entry(source, dest, depth, copied_entry_count, metadata.permissions());
    }

    Err(RunError::Internal(format!("unsupported source entry type while staging {}", source.display(),)))
}

fn copy_file_entry(source: &Path, dest: &Path, permissions: std::fs::Permissions) -> Result<(), RunError> {
    let parent = dest
        .parent()
        .ok_or_else(|| RunError::Internal(format!("staged file destination has no parent: {}", dest.display())))?;
    std::fs::create_dir_all(parent).map_err(|e| RunError::Internal(format!("mkdir {}: {e}", parent.display())))?;
    std::fs::copy(source, dest)
        .map_err(|e| RunError::Internal(format!("copy {} -> {}: {e}", source.display(), dest.display())))?;
    std::fs::set_permissions(dest, permissions)
        .map_err(|e| RunError::Internal(format!("chmod {}: {e}", dest.display())))?;
    Ok(())
}

fn copy_dir_entry(
    source: &Path,
    dest: &Path,
    depth: u32,
    copied_entry_count: &mut u32,
    permissions: std::fs::Permissions,
) -> Result<(), RunError> {
    std::fs::create_dir_all(dest).map_err(|e| RunError::Internal(format!("mkdir {}: {e}", dest.display())))?;
    let mut children: Vec<PathBuf> = Vec::new();
    let entries =
        std::fs::read_dir(source).map_err(|e| RunError::Internal(format!("read_dir {}: {e}", source.display())))?;
    for entry in entries {
        let entry = entry.map_err(|e| RunError::Internal(format!("read_dir entry {}: {e}", source.display())))?;
        children.push(entry.path());
    }
    children.sort();
    for child in &children {
        let child_name = child
            .file_name()
            .ok_or_else(|| RunError::Internal(format!("staged child has no file name: {}", child.display())))?;
        copy_tree_entry(child, &dest.join(child_name), depth.saturating_add(1), copied_entry_count)?;
    }
    std::fs::set_permissions(dest, permissions)
        .map_err(|e| RunError::Internal(format!("chmod {}: {e}", dest.display())))?;
    Ok(())
}

#[cfg(unix)]
fn copy_symlink_entry(source: &Path, dest: &Path) -> Result<(), RunError> {
    let parent = dest
        .parent()
        .ok_or_else(|| RunError::Internal(format!("staged symlink destination has no parent: {}", dest.display())))?;
    std::fs::create_dir_all(parent).map_err(|e| RunError::Internal(format!("mkdir {}: {e}", parent.display())))?;
    let target =
        std::fs::read_link(source).map_err(|e| RunError::Internal(format!("read_link {}: {e}", source.display())))?;
    std::os::unix::fs::symlink(&target, dest)
        .map_err(|e| RunError::Internal(format!("symlink {} -> {}: {e}", dest.display(), target.display())))?;
    Ok(())
}

#[cfg(not(unix))]
fn copy_symlink_entry(source: &Path, _dest: &Path) -> Result<(), RunError> {
    Err(RunError::Internal(format!("symlink staging is only supported on Unix: {}", source.display(),)))
}

/// Quick blake3 fingerprint of a directory tree.
///
/// This is not a NAR hash, but it does include file contents so same-size edits
/// do not silently reuse a stale staged source tree.
fn tree_fingerprint(dir: &Path) -> Result<String, RunError> {
    let mut hasher = blake3::Hasher::new();
    let mut entries: Vec<PathBuf> = Vec::new();

    collect_paths_strict(dir, &mut entries)?;
    entries.sort();

    for entry in &entries {
        hash_tree_entry(dir, entry, &mut hasher)?;
    }

    let hash = hasher.finalize();
    Ok(hash.to_hex().to_string())
}

fn staged_source_store_name_from_fingerprint(fingerprint_hex: &str) -> Result<String, RunError> {
    let digest_bytes = data_encoding::HEXLOWER
        .decode(fingerprint_hex.as_bytes())
        .map_err(|e| RunError::Internal(format!("fingerprint decode failed: {e}")))?;
    if digest_bytes.len() < 20 {
        return Err(RunError::Internal(format!(
            "fingerprint digest too short: expected at least 20 bytes, got {}",
            digest_bytes.len(),
        )));
    }
    let store_hash = nix_compat::nixbase32::encode(&digest_bytes[..20]);
    Ok(format!("{store_hash}-crunch-src"))
}

fn expected_staged_source_store_name(source_dir: &Path) -> Result<String, RunError> {
    let fingerprint = tree_fingerprint(source_dir)?;
    staged_source_store_name_from_fingerprint(&fingerprint)
}

fn hash_tree_entry(dir: &Path, entry: &Path, hasher: &mut blake3::Hasher) -> Result<(), RunError> {
    assert!(dir.is_dir(), "fingerprint root must be a directory: {}", dir.display());

    let rel = entry.strip_prefix(dir).unwrap_or(entry);
    let metadata = std::fs::symlink_metadata(entry)
        .map_err(|e| RunError::Internal(format!("symlink_metadata {}: {e}", entry.display())))?;
    let rel_bytes = rel.as_os_str().as_encoded_bytes();
    hasher.update(&(rel_bytes.len() as u64).to_le_bytes());
    hasher.update(rel_bytes);
    hasher.update(&entry_mode_bits(&metadata).to_le_bytes());

    if metadata.file_type().is_symlink() {
        let target =
            std::fs::read_link(entry).map_err(|e| RunError::Internal(format!("read_link {}: {e}", entry.display())))?;
        hasher.update(b"symlink\0");
        let target_bytes = target.as_os_str().as_encoded_bytes();
        hasher.update(&(target_bytes.len() as u64).to_le_bytes());
        hasher.update(target_bytes);
        return Ok(());
    }

    if metadata.is_dir() {
        hasher.update(b"dir\0");
        return Ok(());
    }

    if metadata.is_file() {
        hasher.update(b"file\0");
        hasher.update(&metadata.len().to_le_bytes());
        let mut file = File::open(entry).map_err(|e| RunError::Internal(format!("open {}: {e}", entry.display())))?;
        let mut buffer = [0_u8; 8192];
        loop {
            let bytes_read =
                file.read(&mut buffer).map_err(|e| RunError::Internal(format!("read {}: {e}", entry.display())))?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }
        return Ok(());
    }

    Err(RunError::Internal(format!(
        "unsupported source entry type while fingerprinting {}",
        entry.display(),
    )))
}

#[cfg(unix)]
fn entry_mode_bits(metadata: &std::fs::Metadata) -> u32 {
    metadata.permissions().mode()
}

#[cfg(not(unix))]
fn entry_mode_bits(_metadata: &std::fs::Metadata) -> u32 {
    0
}

fn collect_paths_strict(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), RunError> {
    let entries = std::fs::read_dir(dir).map_err(|e| RunError::Internal(format!("read_dir {}: {e}", dir.display())))?;
    for entry in entries {
        let entry = entry.map_err(|e| RunError::Internal(format!("read_dir entry {}: {e}", dir.display())))?;
        let path = entry.path();
        out.push(path.clone());
        if path.is_dir() && !path.is_symlink() {
            collect_paths_strict(&path, out)?;
        }
    }
    Ok(())
}

fn collect_paths(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        out.push(path.clone());
        if path.is_dir() && !path.is_symlink() {
            collect_paths(&path, out);
        }
    }
}

/// Total size of a directory tree in bytes.
fn dir_size(dir: &Path) -> u64 {
    let mut total: u64 = 0;
    let mut paths: Vec<PathBuf> = Vec::new();
    collect_paths(dir, &mut paths);
    for p in &paths {
        if let Ok(meta) = std::fs::symlink_metadata(p) {
            total = total.saturating_add(meta.len());
        }
    }
    total
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BootstrapToolPresence {
    has_bwrap_root: bool,
    has_busybox_root: bool,
}

impl BootstrapToolPresence {
    fn has_any(self) -> bool {
        self.has_bwrap_root || self.has_busybox_root
    }
}

fn store_has_entry_with_suffix(output_dir: &Path, suffix: &str) -> bool {
    assert!(!suffix.is_empty(), "suffix must not be empty");
    let Ok(entries) = std::fs::read_dir(output_dir) else {
        return false;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        if name.to_string_lossy().ends_with(suffix) {
            return true;
        }
    }
    false
}

fn inspect_bootstrap_tool_presence(output_dir: &Path) -> BootstrapToolPresence {
    BootstrapToolPresence {
        has_bwrap_root: store_has_entry_with_suffix(output_dir, "-bwrap"),
        has_busybox_root: store_has_entry_with_suffix(output_dir, "-busybox"),
    }
}

fn strict_later_stage_active(
    hermeticity_mode: crunch_pipeline::HermeticityMode,
    presence: BootstrapToolPresence,
) -> bool {
    if !hermeticity_mode.is_strict() {
        return false;
    }
    presence.has_any()
}

fn reject_host_bwrap_fallback(output_dir: &Path, presence: BootstrapToolPresence) -> Result<BwrapSource, RunError> {
    Err(RunError::Internal(format!(
        "strict self-build later stage refuses host bwrap fallback after bootstrap roots already exist in {} \
         (bwrap_root={}, busybox_root={}). Reuse the crunch-built bwrap output instead.",
        output_dir.display(),
        presence.has_bwrap_root,
        presence.has_busybox_root,
    )))
}

/// Locate the best bwrap binary for the self-build pipeline.
///
/// Preference order:
/// 1. Crunch-built bwrap in `output_dir` (from a prior self-build)
/// 2. Any bwrap on the host PATH (first bootstrap)
///
/// Strict later stages reject host fallback once bootstrap-tool roots already
/// exist on disk.
fn resolve_bwrap_source(
    output_dir: &Path,
    hermeticity_mode: crunch_pipeline::HermeticityMode,
) -> Result<BwrapSource, RunError> {
    // 1. Prefer crunch-built bwrap from the output store.
    if let Some(bwrap_dir) = find_crunch_bwrap(output_dir) {
        eprintln!("  bwrap: {} (crunch-built)", bwrap_dir.display());
        return Ok(BwrapSource::CrunchBuilt(bwrap_dir));
    }

    let presence = inspect_bootstrap_tool_presence(output_dir);
    if strict_later_stage_active(hermeticity_mode, presence) {
        return reject_host_bwrap_fallback(output_dir, presence);
    }

    // 2. Fall back to a host bwrap. On NixOS, prefer the wrapper dir
    // so fusermount3 is also resolved from /run/wrappers/bin.
    // These NixOS-specific probes are host-convenience only, not part of any
    // stronger non-Nix-host bootstrap claim.
    match find_host_bwrap() {
        Some(path) => {
            eprintln!(
                "  WARNING: no crunch-built bwrap in {}; using external bwrap at {}",
                output_dir.display(),
                path.display(),
            );
            Ok(BwrapSource::HostFallback(path))
        }
        None => Err(RunError::Build(
            "bwrap (bubblewrap) not found. The first self-build requires bwrap on PATH. \
             Install it from https://github.com/containers/bubblewrap"
                .to_string(),
        )),
    }
}

fn choose_host_bwrap_path(wrapper_bwrap_path: Option<PathBuf>, path_bwrap_path: Option<PathBuf>) -> Option<PathBuf> {
    if let Some(wrapper_path) = wrapper_bwrap_path {
        return Some(wrapper_path);
    }
    path_bwrap_path
}

/// Prefer the NixOS wrapper-installed bwrap when present.
///
/// This is host-convenience discovery only. It does not shell out to any Nix
/// command and should not be read as proof that first bootstrap is already
/// independent of Nix-shaped host layouts.
fn find_nixos_wrapper_bwrap() -> Option<PathBuf> {
    let wrapper_path = Path::new("/run/wrappers/bin/bwrap");
    if wrapper_path.is_file() && is_executable(wrapper_path) {
        return Some(wrapper_path.to_path_buf());
    }
    None
}

fn find_nixos_wrapper_dir() -> Option<PathBuf> {
    let wrapper_dir = Path::new("/run/wrappers/bin");
    let fusermount_path = wrapper_dir.join("fusermount3");
    if fusermount_path.is_file() && is_executable(&fusermount_path) {
        return Some(wrapper_dir.to_path_buf());
    }
    None
}

fn find_host_bwrap() -> Option<PathBuf> {
    let wrapper_bwrap_path = find_nixos_wrapper_bwrap();
    let path_bwrap_path = find_executable_on_path("bwrap");
    choose_host_bwrap_path(wrapper_bwrap_path, path_bwrap_path)
}

fn build_bwrap_path_entries(wrapper_dir: Option<PathBuf>, bwrap_bin_dir: PathBuf) -> Vec<PathBuf> {
    assert!(!bwrap_bin_dir.as_os_str().is_empty(), "bwrap bin dir must not be empty",);
    let mut path_entries = Vec::with_capacity(2);
    path_entries.push(bwrap_bin_dir.clone());
    if let Some(wrapper_dir) = wrapper_dir
        && wrapper_dir != bwrap_bin_dir
    {
        path_entries.push(wrapper_dir);
    }
    assert!(!path_entries.is_empty(), "bwrap PATH entries must not be empty",);
    assert!(path_entries.len() <= 2, "bwrap PATH entries exceeded 2");
    path_entries
}

fn activate_bwrap_source(source: &BwrapSource) -> Result<(), RunError> {
    let bin_dir = source.bin_dir()?;
    let wrapper_dir = find_nixos_wrapper_dir();
    let path_entries = build_bwrap_path_entries(wrapper_dir, bin_dir);
    for dir in path_entries.iter().rev() {
        prepend_to_path(dir)?;
    }
    Ok(())
}

/// Scan the output store for a crunch-built bwrap.
///
/// Looks for `<output_dir>/*-bwrap/bin/bwrap` — the naming convention
/// used by `bootstrap/bwrap.ncl` (derivation name = "bwrap").
fn find_crunch_bwrap(output_dir: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(output_dir).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.ends_with("-bwrap") {
            let bin = entry.path().join("bin").join("bwrap");
            if bin.is_file() && is_executable(&bin) {
                return Some(entry.path().join("bin"));
            }
        }
    }
    None
}

/// Scan the output store for a crunch-built busybox.
///
/// Looks for `<output_dir>/*-busybox/bin/busybox` — the naming convention
/// used by `bootstrap/busybox.ncl`.
#[cfg_attr(not(test), allow(dead_code))]
pub fn find_crunch_busybox(output_dir: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(output_dir).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.ends_with("-busybox") {
            let bin = entry.path().join("bin").join("busybox");
            if bin.is_file() && is_executable(&bin) {
                return Some(bin);
            }
        }
    }
    None
}

/// Find all `*-crunch` output directories in the store.
///
/// Returns a list of `(dir_name, binary_path)` pairs where the binary
/// exists at `<dir>/bin/crunch`.
#[cfg_attr(not(test), allow(dead_code))]
pub fn find_crunch_outputs(output_dir: &Path) -> Vec<(String, PathBuf)> {
    let entries = match std::fs::read_dir(output_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    let mut found: Vec<(String, PathBuf)> = Vec::new();
    let mut scanned: u32 = 0;
    for entry in entries.flatten() {
        scanned = scanned.saturating_add(1);
        if scanned > MAX_CRUNCH_OUTPUTS {
            break;
        }
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.ends_with("-crunch") {
            let binary = entry.path().join("bin").join("crunch");
            if binary.exists() {
                found.push((name_str.into_owned(), binary));
            }
        }
    }
    found
}

/// Remove all `*-crunch` output directories from the store.
///
/// Returns the number of directories removed. Errors from individual
/// removals are collected but do not abort the loop.
#[cfg_attr(not(test), allow(dead_code))]
pub fn invalidate_crunch_outputs(output_dir: &Path) -> Result<u32, RunError> {
    let outputs = find_crunch_outputs(output_dir);
    let mut removed: u32 = 0;
    let mut errors: Vec<String> = Vec::new();
    for (name, _binary) in &outputs {
        let dir = output_dir.join(name);
        match std::fs::remove_dir_all(&dir) {
            Ok(()) => {
                removed = removed.saturating_add(1);
            }
            Err(e) => {
                errors.push(format!("{}: {e}", dir.display()));
            }
        }
    }
    if !errors.is_empty() {
        return Err(RunError::Internal(format!(
            "failed to remove {} of {} crunch outputs:\n{}",
            errors.len(),
            outputs.len(),
            errors.join("\n"),
        )));
    }
    Ok(removed)
}

/// Search PATH for a named executable, returning the first match.
fn find_executable_on_path(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(name);
        if candidate.is_file() && is_executable(&candidate) {
            return Some(candidate);
        }
    }
    None
}

/// Check if a file has executable permission.
#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata().map(|m| m.permissions().mode() & 0o111 != 0).unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(_path: &Path) -> bool {
    true
}

/// Prepend a directory to the process PATH.
///
/// Uses `std::env::join_paths` to avoid malformed PATH entries when
/// the current PATH is empty or unset.
fn prepend_to_path(dir: &Path) -> Result<(), RunError> {
    let current: Vec<PathBuf> = std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).filter(|d| !d.as_os_str().is_empty()).collect())
        .unwrap_or_default();
    let mut dirs = vec![dir.to_path_buf()];
    dirs.extend(current);
    let new_path = std::env::join_paths(&dirs).map_err(|e| RunError::Internal(format!("join_paths: {e}")))?;
    // SAFETY: single-threaded at this point (before tokio runtime).
    unsafe { std::env::set_var("PATH", &new_path) };
    Ok(())
}

/// Verify that all required bootstrap tool NCL files exist.
///
/// Returns `Err` if any file in `REQUIRED_BOOTSTRAP_TOOLS` is missing.
fn validate_bootstrap_tools(bootstrap_dir: &Path) -> Result<(), RunError> {
    for tool_name in REQUIRED_BOOTSTRAP_TOOLS {
        let tool_path = bootstrap_dir.join(tool_name);
        if !tool_path.exists() {
            return Err(RunError::Internal(format!(
                "{tool_name} not found at {}. \
                 The self-build requires all bootstrap tools: {:?}",
                tool_path.display(),
                REQUIRED_BOOTSTRAP_TOOLS,
            )));
        }
    }
    Ok(())
}

/// Verify that crunch-built bwrap and busybox are on disk after building.
///
/// Returns `Err` if either tool is missing from the output store.
#[cfg_attr(not(test), allow(dead_code))]
fn verify_tools_on_disk(output_dir: &Path) -> Result<(BwrapSource, PathBuf), RunError> {
    let bwrap_source = resolve_bwrap_source(output_dir, crunch_pipeline::HermeticityMode::Practical)?;
    if !bwrap_source.is_crunch_built() {
        return Err(RunError::Internal(
            "bwrap was not found on disk after building bwrap.ncl. \
             The bootstrap tool build may have failed silently."
                .to_string(),
        ));
    }
    let busybox_path = find_crunch_busybox(output_dir).ok_or_else(|| {
        RunError::Internal(
            "busybox was not found on disk after building busybox.ncl. \
             The bootstrap tool build may have failed silently."
                .to_string(),
        )
    })?;
    Ok((bwrap_source, busybox_path))
}

/// Resolved bootstrap tool paths from step 2.
struct BootstrapTools {
    bwrap_source: BwrapSource,
    bwrap_store_name: String,
    busybox_path: Option<PathBuf>,
    busybox_store_name: String,
}

/// Build all required bootstrap tools and return their resolved paths.
///
/// Each tool is built as a separate root derivation and exported to disk.
/// After all tools are built, the crunch-built bwrap is activated on PATH.
#[allow(clippy::too_many_arguments)]
fn build_all_bootstrap_tools(
    bootstrap_dir: &Path,
    import_paths: &[std::ffi::OsString],
    output_dir: &Path,
    state_dir: &Path,
    store_dir: &str,
    verbose: bool,
    max_jobs: u32,
    no_substitute: bool,
    keypair: &crunch_build::KeyPair,
    trusted_keys: &[nix_compat::narinfo::VerifyingKey],
    trust_unsigned: bool,
    hermeticity_mode: crunch_pipeline::HermeticityMode,
    bootstrap_bwrap_source: Option<&BwrapSource>,
    bootstrap_busybox_path: Option<&Path>,
) -> Result<BootstrapTools, RunError> {
    validate_bootstrap_tools(bootstrap_dir)?;
    let mut built_bwrap_output_dir: Option<PathBuf> = None;
    let mut built_busybox_output_dir: Option<PathBuf> = None;

    for tool_name in REQUIRED_BOOTSTRAP_TOOLS {
        emit_progress_marker(&format!("bootstrap-tool-start:{tool_name}"));
        let reused_output_dir = match *tool_name {
            "bwrap.ncl" => bootstrap_bwrap_source
                .map(|source| source.bin_dir())
                .transpose()?
                .map(|dir| dir.parent().expect("bwrap bin dir must have store root parent").to_path_buf()),
            "busybox.ncl" => bootstrap_busybox_path.map(|path| {
                path.parent()
                    .and_then(|bin_dir| bin_dir.parent())
                    .expect("busybox path must have store root parent")
                    .to_path_buf()
            }),
            _ => None,
        };
        let tool_output_dir = if let Some(dir) = reused_output_dir {
            eprintln!("  reusing {tool_name} from {}...", dir.display());
            dir
        } else {
            let tool_path = bootstrap_dir.join(tool_name);
            eprintln!("  building {tool_name}...");
            build_bootstrap_tool(
                &tool_path,
                import_paths,
                output_dir,
                state_dir,
                store_dir,
                verbose,
                max_jobs,
                no_substitute,
                keypair,
                trusted_keys,
                trust_unsigned,
                hermeticity_mode,
            )?
        };
        if *tool_name == "bwrap.ncl" {
            let bwrap_path = tool_output_dir.join("bin").join("bwrap");
            ensure_executable_file(&bwrap_path, "crunch-built bwrap")?;
            resolve_store_entry_name(&tool_output_dir, output_dir, "bwrap")?;
            built_bwrap_output_dir = Some(tool_output_dir.clone());
        }
        if *tool_name == "busybox.ncl" {
            let busybox_path = tool_output_dir.join("bin").join("busybox");
            ensure_executable_file(&busybox_path, "crunch-built busybox")?;
            resolve_store_entry_name(&tool_output_dir, output_dir, "busybox")?;
            built_busybox_output_dir = Some(tool_output_dir.clone());
        }
        emit_progress_marker(&format!("bootstrap-tool-done:{tool_name}"));
    }

    let bwrap_output_dir = built_bwrap_output_dir.ok_or_else(|| {
        RunError::Internal(
            "bwrap was not found on disk after building bwrap.ncl. \
             The bootstrap tool build may have failed silently."
                .to_string(),
        )
    })?;
    let busybox_output_dir = built_busybox_output_dir.ok_or_else(|| {
        RunError::Internal(
            "busybox was not found on disk after building busybox.ncl. \
             The bootstrap tool build may have failed silently."
                .to_string(),
        )
    })?;

    let bwrap_path = bwrap_output_dir.join("bin").join("bwrap");
    ensure_executable_file(&bwrap_path, "crunch-built bwrap")?;
    let bwrap_dir = bwrap_path
        .parent()
        .ok_or_else(|| RunError::Internal(format!("bwrap binary has no parent directory: {}", bwrap_path.display())))?;
    let busybox_path = busybox_output_dir.join("bin").join("busybox");
    ensure_executable_file(&busybox_path, "crunch-built busybox")?;

    let bwrap_store_name = resolve_store_entry_name(&bwrap_output_dir, output_dir, "bwrap")?;
    let busybox_store_name = resolve_store_entry_name(&busybox_output_dir, output_dir, "busybox")?;
    let bwrap_source = match bootstrap_bwrap_source {
        Some(source) => source.clone(),
        None => BwrapSource::CrunchBuilt(bwrap_dir.to_path_buf()),
    };
    let busybox_path = match bootstrap_busybox_path {
        Some(path) => path.to_path_buf(),
        None => busybox_path,
    };
    activate_bwrap_source(&bwrap_source)?;

    Ok(BootstrapTools {
        bwrap_source,
        bwrap_store_name,
        busybox_path: Some(busybox_path),
        busybox_store_name,
    })
}

/// Build crunch from source using the bootstrap toolchain.
///
/// Generates the self-build NCL, runs the build pipeline, and returns
/// the output binary path.
#[allow(clippy::too_many_arguments)]
fn build_crunch_binary(
    src_dir: &Path,
    src_store_name: &str,
    bwrap_store_name: &str,
    busybox_store_name: &str,
    output_dir: &Path,
    state_dir: &Path,
    store_dir: &str,
    verbose: bool,
    max_jobs: u32,
    no_substitute: bool,
    keypair: crunch_build::KeyPair,
    trusted_keys: Vec<nix_compat::narinfo::VerifyingKey>,
    trust_unsigned: bool,
    hermeticity_mode: crunch_pipeline::HermeticityMode,
) -> Result<PathBuf, RunError> {
    let ncl_content = generate_self_build_ncl(src_store_name, bwrap_store_name, busybox_store_name, store_dir);

    let tmp_dir = tempfile::tempdir().map_err(|e| RunError::Internal(format!("tmpdir: {e}")))?;
    let ncl_path = tmp_dir.path().join("self-build.ncl");
    std::fs::write(&ncl_path, &ncl_content).map_err(|e| RunError::Internal(format!("writing ncl: {e}")))?;

    let stage3_import_paths = build_import_paths(&[src_dir.to_path_buf(), src_dir.join("lib")])?;
    let config = crunch_pipeline::BuildConfig {
        file: ncl_path.clone(),
        import_paths: stage3_import_paths,
        output_dir: output_dir.to_path_buf(),
        state_dir: state_dir.to_path_buf(),
        store_dir: store_dir.to_string(),
        verbose,
        max_jobs,
        substituter_url: if no_substitute {
            None
        } else {
            Some("https://cache.nixos.org".to_string())
        },
        hermeticity_mode,
        keypair,
        trusted_keys,
        trust_unsigned,
        root_retention_source: Some(crunch_store::GcRootSource::SelfBuild),
    };

    let result = run_build(&config)?;
    report_build_result(&config, &result, false, crate::build_cmd::BuildOutputMode::Human)?;
    emit_progress_marker("crunch-build-done");

    let output_root_dir = resolve_single_root_output_dir(&result, &config.store_dir, output_dir, "crunch")?;
    let output_binary = output_root_dir.join("bin").join("crunch");
    ensure_executable_file(&output_binary, "self-built crunch binary")?;
    Ok(output_binary)
}

/// Build a single bootstrap tool (.ncl file) as a root derivation.
///
/// This exports the tool's output to the `--store` directory on disk,
/// making it available for subsequent self-build stages to discover
/// via `find_crunch_bwrap` / `find_crunch_busybox`.
#[allow(clippy::too_many_arguments)]
fn build_bootstrap_tool(
    tool_ncl: &Path,
    import_paths: &[std::ffi::OsString],
    output_dir: &Path,
    state_dir: &Path,
    store_dir: &str,
    verbose: bool,
    max_jobs: u32,
    no_substitute: bool,
    keypair: &crunch_build::KeyPair,
    trusted_keys: &[nix_compat::narinfo::VerifyingKey],
    trust_unsigned: bool,
    hermeticity_mode: crunch_pipeline::HermeticityMode,
) -> Result<PathBuf, RunError> {
    assert!(tool_ncl.exists(), "tool NCL must exist: {}", tool_ncl.display());
    assert!(!import_paths.is_empty(), "import_paths must not be empty");

    let config = crunch_pipeline::BuildConfig {
        file: tool_ncl.to_path_buf(),
        import_paths: import_paths.to_vec(),
        output_dir: output_dir.to_path_buf(),
        state_dir: state_dir.to_path_buf(),
        store_dir: store_dir.to_string(),
        verbose,
        max_jobs,
        substituter_url: if no_substitute {
            None
        } else {
            Some("https://cache.nixos.org".to_string())
        },
        hermeticity_mode,
        keypair: keypair.clone(),
        trusted_keys: trusted_keys.to_vec(),
        trust_unsigned,
        root_retention_source: None,
    };

    let result = run_build(&config)?;
    report_build_result(&config, &result, false, crate::build_cmd::BuildOutputMode::Human)?;
    let tool_label = tool_ncl
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| RunError::Internal(format!("tool NCL has no valid stem: {}", tool_ncl.display())))?;
    resolve_single_root_output_dir(&result, &config.store_dir, output_dir, tool_label)
}

fn resolve_single_root_output_dir(
    result: &crunch_pipeline::PipelineResult,
    store_dir: &str,
    output_dir: &Path,
    expected_label: &str,
) -> Result<PathBuf, RunError> {
    assert!(!expected_label.is_empty(), "expected_label must not be empty");

    let mut matched_output_dirs: Vec<PathBuf> = Vec::new();
    let output_dir_str = output_dir.to_str().unwrap_or(store_dir);
    for outcome in &result.outcomes {
        let drv_key = crunch_pipeline::drv_key_for(store_dir, &outcome.drv_path);
        let Some(label) = crunch_pipeline::label_for_key(result, &drv_key) else {
            continue;
        };
        if label != expected_label {
            continue;
        }
        let path_info = outcome
            .outputs
            .get("out")
            .ok_or_else(|| RunError::Internal(format!("root '{expected_label}' did not produce an 'out' output")))?;
        let output_path = path_info.store_path.to_absolute_path_with_prefix(output_dir_str);
        matched_output_dirs.push(PathBuf::from(output_path));
    }

    if matched_output_dirs.is_empty() {
        return Err(RunError::Internal(format!("could not find root output directory for '{expected_label}'",)));
    }
    matched_output_dirs.sort();
    matched_output_dirs.dedup();
    if matched_output_dirs.len() != 1 {
        return Err(RunError::Internal(format!(
            "expected exactly 1 root output directory for '{expected_label}', found {}",
            matched_output_dirs.len(),
        )));
    }
    Ok(matched_output_dirs.pop().expect("checked len == 1"))
}

fn ensure_executable_file(path: &Path, label: &str) -> Result<(), RunError> {
    assert!(!label.is_empty(), "label must not be empty");
    if !path.is_file() {
        return Err(RunError::Internal(format!("{label} is not on disk at {}", path.display())));
    }
    if !is_executable(path) {
        return Err(RunError::Internal(format!("{label} is not executable at {}", path.display())));
    }
    Ok(())
}

fn validate_staged_source_dir(source_dir: &Path, output_dir: &Path) -> Result<(), RunError> {
    assert!(output_dir.is_dir(), "output dir must exist: {}", output_dir.display());
    if !source_dir.is_dir() {
        return Err(RunError::Internal(format!("staged source directory does not exist: {}", source_dir.display(),)));
    }
    let source_parent = source_dir
        .parent()
        .ok_or_else(|| RunError::Internal(format!("staged source path has no parent: {}", source_dir.display())))?;
    let normalized_parent = std::fs::canonicalize(source_parent)
        .map_err(|e| RunError::Internal(format!("canonicalize {}: {e}", source_parent.display())))?;
    let normalized_output_dir = std::fs::canonicalize(output_dir)
        .map_err(|e| RunError::Internal(format!("canonicalize {}: {e}", output_dir.display())))?;
    if normalized_parent != normalized_output_dir {
        return Err(RunError::Internal(format!(
            "staged source {} must live directly under the self-build store {}",
            source_dir.display(),
            output_dir.display(),
        )));
    }
    let cargo_toml = source_dir.join("Cargo.toml");
    let bootstrap_dir = source_dir.join("bootstrap");
    let lib_dir = source_dir.join("lib");
    let vendor_config = source_dir.join(".cargo").join("vendor-config.toml");
    if !cargo_toml.is_file() {
        return Err(RunError::Internal(format!("staged source missing Cargo.toml: {}", cargo_toml.display())));
    }
    if !bootstrap_dir.is_dir() {
        return Err(RunError::Internal(format!("staged source missing bootstrap/: {}", bootstrap_dir.display())));
    }
    if !lib_dir.is_dir() {
        return Err(RunError::Internal(format!("staged source missing lib/: {}", lib_dir.display())));
    }
    if !vendor_config.is_file() {
        return Err(RunError::Internal(format!(
            "staged source missing .cargo/vendor-config.toml: {}",
            vendor_config.display(),
        )));
    }
    require_checked_vendor_inputs(source_dir)?;
    let expected_store_name = expected_staged_source_store_name(source_dir)?;
    let actual_store_name = source_dir.file_name().and_then(|name| name.to_str()).ok_or_else(|| {
        RunError::Internal(format!("staged source path has no UTF-8 file name: {}", source_dir.display()))
    })?;
    if actual_store_name != expected_store_name {
        return Err(RunError::Internal(format!(
            "staged source contents no longer match its store name: expected {}, got {}",
            expected_store_name, actual_store_name,
        )));
    }
    Ok(())
}

fn prepare_self_build_source(
    source_dir: &Path,
    output_dir: &Path,
    source_store_path: Option<&Path>,
) -> Result<(String, PathBuf), RunError> {
    match source_store_path {
        Some(existing_source) => {
            let absolute_source = absolutize_path(existing_source)?;
            validate_staged_source_dir(&absolute_source, output_dir)?;
            let store_name = absolute_source
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| {
                    RunError::Internal(format!(
                        "staged source path has no UTF-8 file name: {}",
                        absolute_source.display(),
                    ))
                })?
                .to_string();
            assert!(!store_name.is_empty(), "staged source store name must not be empty");
            eprintln!("  reusing staged source: {}", absolute_source.display());
            Ok((store_name, absolute_source))
        }
        None => {
            let store_name = stage_source(source_dir, output_dir)?;
            let staged_source = output_dir.join(&store_name);
            assert!(staged_source.is_dir(), "new staged source must exist: {}", staged_source.display());
            Ok((store_name, staged_source))
        }
    }
}

fn resolve_store_entry_name(entry_path: &Path, output_dir: &Path, label: &str) -> Result<String, RunError> {
    assert!(!label.is_empty(), "label must not be empty");
    let parent = entry_path
        .parent()
        .ok_or_else(|| RunError::Internal(format!("{label} path has no parent: {}", entry_path.display())))?;
    if parent != output_dir {
        return Err(RunError::Internal(format!(
            "{label} output {} must live directly under the self-build store {}",
            entry_path.display(),
            output_dir.display(),
        )));
    }
    let file_name = entry_path.file_name().and_then(|name| name.to_str()).ok_or_else(|| {
        RunError::Internal(format!("{label} output has no UTF-8 file name: {}", entry_path.display()))
    })?;
    assert!(!file_name.is_empty(), "{label} output store name must not be empty");
    Ok(file_name.to_string())
}

fn absolutize_path(path: &Path) -> Result<PathBuf, RunError> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    let cwd = std::env::current_dir().map_err(|e| RunError::Internal(format!("cwd: {e}")))?;
    Ok(cwd.join(path))
}

fn resolve_explicit_bootstrap_output_dir(
    binary_path: &Path,
    output_dir: &Path,
    label: &str,
    expected_file_name: &str,
) -> Result<(PathBuf, PathBuf), RunError> {
    assert!(!label.is_empty(), "label must not be empty");
    assert!(!expected_file_name.is_empty(), "expected file name must not be empty");
    let absolute_binary = absolutize_path(binary_path)?;
    ensure_executable_file(&absolute_binary, &format!("exact {label}"))?;
    let actual_file_name = absolute_binary.file_name().and_then(|name| name.to_str()).ok_or_else(|| {
        RunError::Internal(format!("exact {label} path has no UTF-8 file name: {}", absolute_binary.display()))
    })?;
    if actual_file_name != expected_file_name {
        return Err(RunError::Internal(format!(
            "exact {label} path {} must end with {expected_file_name}",
            absolute_binary.display(),
        )));
    }
    let bin_dir = absolute_binary.parent().ok_or_else(|| {
        RunError::Internal(format!("exact {label} path has no parent directory: {}", absolute_binary.display()))
    })?;
    let output_root = bin_dir.parent().ok_or_else(|| {
        RunError::Internal(format!("exact {label} path has no store root parent: {}", absolute_binary.display()))
    })?;
    resolve_store_entry_name(output_root, output_dir, label)?;
    Ok((output_root.to_path_buf(), absolute_binary))
}

fn resolve_explicit_bootstrap_bwrap_source(
    output_dir: &Path,
    bootstrap_bwrap_path: Option<&Path>,
) -> Result<Option<BwrapSource>, RunError> {
    let Some(path) = bootstrap_bwrap_path else {
        return Ok(None);
    };
    let (_output_root, binary_path) = resolve_explicit_bootstrap_output_dir(path, output_dir, "bwrap", "bwrap")?;
    let bin_dir = binary_path.parent().ok_or_else(|| {
        RunError::Internal(format!("exact bwrap path has no parent directory: {}", binary_path.display()))
    })?;
    Ok(Some(BwrapSource::CrunchBuilt(bin_dir.to_path_buf())))
}

fn resolve_explicit_bootstrap_busybox_path(
    output_dir: &Path,
    bootstrap_busybox_path: Option<&Path>,
) -> Result<Option<PathBuf>, RunError> {
    let Some(path) = bootstrap_busybox_path else {
        return Ok(None);
    };
    let (_output_root, binary_path) = resolve_explicit_bootstrap_output_dir(path, output_dir, "busybox", "busybox")?;
    Ok(Some(binary_path))
}

fn enforce_source_resolution_policy(
    output_dir: &Path,
    hermeticity_mode: crunch_pipeline::HermeticityMode,
    presence: BootstrapToolPresence,
    source_store_path: Option<&Path>,
) -> Result<(), RunError> {
    if source_store_path.is_some() {
        return Ok(());
    }
    if !strict_later_stage_active(hermeticity_mode, presence) {
        return Ok(());
    }
    Err(RunError::Internal(format!(
        "strict self-build later stage requires --source-store-path once bootstrap roots already exist in {} \
         (bwrap_root={}, busybox_root={}). Refusing checkout source discovery fallback.",
        output_dir.display(),
        presence.has_bwrap_root,
        presence.has_busybox_root,
    )))
}

fn fallback_event_for_bwrap_source(source: &BwrapSource) -> Option<SelfBuildFallbackEvent> {
    match source {
        BwrapSource::CrunchBuilt(_) => None,
        BwrapSource::HostFallback(path) => Some(SelfBuildFallbackEvent::BwrapHostFallback(path.clone())),
    }
}

fn resolve_self_build_source_dir(
    output_dir: &Path,
    source_store_path: Option<&Path>,
    hermeticity_mode: crunch_pipeline::HermeticityMode,
) -> Result<(PathBuf, Vec<SelfBuildFallbackEvent>), RunError> {
    let presence = inspect_bootstrap_tool_presence(output_dir);
    enforce_source_resolution_policy(output_dir, hermeticity_mode, presence, source_store_path)?;

    match source_store_path {
        Some(existing_source) => {
            let absolute_source = absolutize_path(existing_source)?;
            validate_staged_source_dir(&absolute_source, output_dir)?;
            Ok((absolute_source, Vec::new()))
        }
        None => {
            let source_dir = find_source_dir()?;
            let event = SelfBuildFallbackEvent::SourceHostDiscovery(source_dir.clone());
            Ok((source_dir, vec![event]))
        }
    }
}

struct SelfBuildSetup {
    invoking_binary: PathBuf,
    output_dir: PathBuf,
    src_dir: PathBuf,
    bootstrap_dir: PathBuf,
    fallback_events: Vec<SelfBuildFallbackEvent>,
    bootstrap_bwrap_source: Option<BwrapSource>,
    bootstrap_busybox_path: Option<PathBuf>,
}

struct SelfBuildShared {
    store_name: String,
    staged_source: PathBuf,
    import_paths: Vec<std::ffi::OsString>,
    keypair: crunch_build::KeyPair,
    trusted_keys: Vec<nix_compat::narinfo::VerifyingKey>,
}

fn initialize_self_build(
    output_dir: &Path,
    source_store_path: Option<&Path>,
    hermeticity_mode: crunch_pipeline::HermeticityMode,
    bootstrap_bwrap_path: Option<&Path>,
    bootstrap_busybox_path: Option<&Path>,
) -> Result<SelfBuildSetup, RunError> {
    eprintln!("=== crunch self-build ===");

    let invoking_binary = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("crunch"));
    eprintln!("  invoking binary: {}", invoking_binary.display());
    let output_dir = absolutize_path(output_dir)?;

    let explicit_bwrap_source = resolve_explicit_bootstrap_bwrap_source(&output_dir, bootstrap_bwrap_path)?;
    let explicit_busybox_path = resolve_explicit_bootstrap_busybox_path(&output_dir, bootstrap_busybox_path)?;

    let initial_bwrap = match &explicit_bwrap_source {
        Some(source) => source.clone(),
        None => resolve_bwrap_source(&output_dir, hermeticity_mode)?,
    };
    let mut fallback_events = Vec::new();
    if explicit_bwrap_source.is_none()
        && let Some(event) = fallback_event_for_bwrap_source(&initial_bwrap)
    {
        fallback_events.push(event);
    }
    activate_bwrap_source(&initial_bwrap)?;

    let (src_dir, source_events) = resolve_self_build_source_dir(&output_dir, source_store_path, hermeticity_mode)?;
    fallback_events.extend(source_events);
    eprintln!("source: {}", src_dir.display());
    let bootstrap_dir = src_dir.join("bootstrap");
    if !bootstrap_dir.exists() {
        return Err(RunError::Internal(format!("bootstrap/ directory not found at {}", bootstrap_dir.display(),)));
    }

    Ok(SelfBuildSetup {
        invoking_binary,
        output_dir,
        src_dir,
        bootstrap_dir,
        fallback_events,
        bootstrap_bwrap_source: explicit_bwrap_source,
        bootstrap_busybox_path: explicit_busybox_path,
    })
}

fn build_self_build_import_paths(src_dir: &Path, bootstrap_dir: &Path) -> Result<Vec<std::ffi::OsString>, RunError> {
    let lib_dir = src_dir.join("lib");
    build_import_paths(&[lib_dir, bootstrap_dir.to_path_buf()])
}

fn prepare_self_build_shared(
    setup: &SelfBuildSetup,
    state_dir: &Path,
    signing_key_path: Option<&Path>,
    trusted_public_keys: Option<&[nix_compat::narinfo::VerifyingKey]>,
    source_store_path: Option<&Path>,
) -> Result<SelfBuildShared, RunError> {
    eprintln!("\n[1/{SELF_BUILD_STEP_COUNT}] Staging source...");
    let (store_name, staged_source) = prepare_self_build_source(&setup.src_dir, &setup.output_dir, source_store_path)?;
    let import_paths = build_self_build_import_paths(&setup.src_dir, &setup.bootstrap_dir)?;
    let keypair = load_or_generate_signing_keypair(signing_key_path, state_dir, true)?;
    let configured_trusted_keys = load_configured_trusted_public_keys(trusted_public_keys, state_dir)?;
    let trusted_keys = crunch_build::build_trusted_keys(&keypair, configured_trusted_keys.as_deref());

    Ok(SelfBuildShared {
        store_name,
        staged_source,
        import_paths,
        keypair,
        trusted_keys,
    })
}

fn verify_self_build_output(output_binary: &Path, no_verify: bool) -> Result<(), RunError> {
    if no_verify {
        eprintln!("\n[4/{SELF_BUILD_STEP_COUNT}] Verification skipped (--no-verify).");
        return Ok(());
    }
    eprintln!("\n[4/{SELF_BUILD_STEP_COUNT}] Verifying output...");
    verify_binary(output_binary)
}

fn emit_self_build_completion(report: &SelfBuildReport) {
    eprint!("{}", report.format_proof_lines());
    eprintln!("\n=== self-build complete ===");
}

#[allow(clippy::too_many_arguments)]
pub fn cmd_self_build(
    output_dir: &Path,
    state_dir: &Path,
    store_dir: &str,
    verbose: bool,
    max_jobs: u32,
    no_substitute: bool,
    no_verify: bool,
    signing_key_path: Option<&Path>,
    trusted_public_keys: Option<&[nix_compat::narinfo::VerifyingKey]>,
    trust_unsigned: bool,
    hermeticity_mode: crunch_pipeline::HermeticityMode,
    source_store_path: Option<&Path>,
    bootstrap_bwrap_path: Option<&Path>,
    bootstrap_busybox_path: Option<&Path>,
) -> Result<SelfBuildReport, RunError> {
    let setup = initialize_self_build(
        output_dir,
        source_store_path,
        hermeticity_mode,
        bootstrap_bwrap_path,
        bootstrap_busybox_path,
    )?;
    let shared =
        prepare_self_build_shared(&setup, state_dir, signing_key_path, trusted_public_keys, source_store_path)?;

    eprintln!("\n[2/{SELF_BUILD_STEP_COUNT}] Building bootstrap tools...");
    let tools = build_all_bootstrap_tools(
        &setup.bootstrap_dir,
        &shared.import_paths,
        &setup.output_dir,
        state_dir,
        store_dir,
        verbose,
        max_jobs,
        no_substitute,
        &shared.keypair,
        &shared.trusted_keys,
        trust_unsigned,
        hermeticity_mode,
        setup.bootstrap_bwrap_source.as_ref(),
        setup.bootstrap_busybox_path.as_deref(),
    )?;

    eprintln!("\n[3/{SELF_BUILD_STEP_COUNT}] Building crunch...");
    emit_progress_marker("crunch-build-start");
    let output_binary = build_crunch_binary(
        &setup.src_dir,
        &shared.store_name,
        &tools.bwrap_store_name,
        &tools.busybox_store_name,
        &setup.output_dir,
        state_dir,
        store_dir,
        verbose,
        max_jobs,
        no_substitute,
        shared.keypair,
        shared.trusted_keys,
        trust_unsigned,
        hermeticity_mode,
    )?;
    verify_self_build_output(&output_binary, no_verify)?;

    let report = SelfBuildReport {
        hermeticity_mode,
        invoking_binary: setup.invoking_binary,
        staged_source: shared.staged_source,
        bwrap_source: tools.bwrap_source,
        fallback_events: setup.fallback_events,
        busybox_path: tools.busybox_path,
        output_binary,
    };
    emit_self_build_completion(&report);
    Ok(report)
}

fn find_source_dir() -> Result<PathBuf, RunError> {
    let cwd = std::env::current_dir().map_err(|e| RunError::Internal(format!("cwd: {e}")))?;

    if cwd.join("Cargo.toml").exists() && cwd.join("bootstrap").exists() {
        return Ok(cwd);
    }

    let mut dir = cwd.as_path();
    for _ in 0..8_u32 {
        if dir.join("Cargo.toml").exists() && dir.join("bootstrap").exists() {
            return Ok(dir.to_path_buf());
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => break,
        }
    }

    Err(RunError::Internal(
        "could not locate crunch source directory.\nRun `crunch self-build` from the crunch repo root.".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use nix_compat::store_path::StorePath;

    use super::*;

    fn test_self_build_ncl() -> String {
        generate_self_build_ncl("abc123-crunch-src", "bwrap123-bwrap", "busybox123-busybox", "/nix/store")
    }

    fn write_minimal_staged_source(dir: &Path) {
        std::fs::create_dir_all(dir.join("bootstrap")).unwrap();
        std::fs::create_dir_all(dir.join("lib")).unwrap();
        std::fs::create_dir_all(dir.join(".cargo")).unwrap();
        let vendor_dep = dir.join("vendor-deps").join("dep-a");
        std::fs::create_dir_all(&vendor_dep).unwrap();
        std::fs::write(dir.join("Cargo.toml"), "[package]\nname = \"crunch\"\nversion = \"0.0.0\"\n").unwrap();
        let package_checksum = sample_cargo_sha256('a');
        std::fs::write(
            dir.join("Cargo.lock"),
            format!(
                "[[package]]\nname = \"dep-a\"\nversion = \"0.0.0\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = \"{package_checksum}\"\n",
            ),
        )
        .unwrap();
        std::fs::write(
            dir.join(".cargo").join("vendor-config.toml"),
            "[source.crates-io]\nreplace-with = \"vendored-sources\"\n\n[source.vendored-sources]\ndirectory = \"vendor-deps\"\n",
        )
        .unwrap();
        std::fs::write(vendor_dep.join("Cargo.toml"), "[package]\nname=\"dep-a\"\nversion=\"0.0.0\"\n").unwrap();
        std::fs::write(vendor_dep.join("lib.rs"), "pub fn dep_a() {}\n").unwrap();
        write_vendor_checksum_manifest(&vendor_dep, Some(&package_checksum));
    }

    fn write_stageable_checkout(dir: &Path) {
        std::fs::create_dir_all(dir.join(".cargo")).unwrap();
        std::fs::create_dir_all(dir.join("bootstrap")).unwrap();
        std::fs::create_dir_all(dir.join("builders")).unwrap();
        std::fs::create_dir_all(dir.join("crates").join("crate-a")).unwrap();
        std::fs::create_dir_all(dir.join("lib")).unwrap();
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::create_dir_all(dir.join("vendor").join("patched")).unwrap();
        let vendor_dep = dir.join("vendor-deps").join("dep-a");
        std::fs::create_dir_all(&vendor_dep).unwrap();
        std::fs::write(dir.join("Cargo.toml"), "[workspace]\nmembers = []\n").unwrap();
        let package_checksum = sample_cargo_sha256('a');
        std::fs::write(
            dir.join("Cargo.lock"),
            format!(
                "[[package]]\nname = \"dep-a\"\nversion = \"0.0.0\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = \"{package_checksum}\"\n",
            ),
        )
        .unwrap();
        std::fs::write(dir.join("rust-toolchain.toml"), "[toolchain]\nchannel = \"nightly\"\n").unwrap();
        std::fs::write(
            dir.join(".cargo").join("vendor-config.toml"),
            "[source.crates-io]\nreplace-with = \"vendored-sources\"\n\n[source.vendored-sources]\ndirectory = \"vendor-deps\"\n",
        )
        .unwrap();
        std::fs::write(dir.join(".cargo").join("config.toml"), "[build]\n").unwrap();
        std::fs::write(dir.join("bootstrap").join("seed.ncl"), "{}").unwrap();
        std::fs::write(dir.join("builders").join("mk.ncl"), "{}").unwrap();
        std::fs::write(dir.join("crates").join("crate-a").join("lib.rs"), "pub fn x() {}\n").unwrap();
        std::fs::write(dir.join("lib").join("lib.ncl"), "{}").unwrap();
        std::fs::write(dir.join("src").join("main.rs"), "fn main() {}\n").unwrap();
        std::fs::write(dir.join("vendor").join("patched").join("README"), "vendor patch\n").unwrap();
        std::fs::write(vendor_dep.join("Cargo.toml"), "[package]\nname=\"dep-a\"\nversion=\"0.0.0\"\n").unwrap();
        std::fs::write(vendor_dep.join("lib.rs"), "pub fn dep_a() {}\n").unwrap();
        write_vendor_checksum_manifest(&vendor_dep, Some(&package_checksum));
    }

    fn sample_cargo_sha256(ch: char) -> String {
        assert!(ch.is_ascii_hexdigit(), "sample checksum char must be hex");
        assert!(!ch.is_ascii_uppercase(), "sample checksum char must be lowercase");
        std::iter::repeat_n(ch, CARGO_SHA256_HEX_LEN).collect()
    }

    fn write_vendor_checksum_manifest(package_dir: &Path, package_checksum: Option<&str>) {
        let cargo_toml_digest = hash_file_cargo_sha256_hex(&package_dir.join("Cargo.toml")).unwrap();
        let lib_digest = hash_file_cargo_sha256_hex(&package_dir.join("lib.rs")).unwrap();
        let package_json = match package_checksum {
            Some(checksum) => format!("\"{checksum}\""),
            None => "null".to_string(),
        };
        let manifest = format!(
            "{{\"files\":{{\"Cargo.toml\":\"{cargo_toml_digest}\",\"lib.rs\":\"{lib_digest}\"}},\"package\":{package_json}}}",
        );
        std::fs::write(package_dir.join(".cargo-checksum.json"), manifest).unwrap();
    }

    fn make_valid_staged_source(output_dir: &Path) -> PathBuf {
        let scratch = output_dir.join("scratch-staged-source");
        write_minimal_staged_source(&scratch);
        let store_name = expected_staged_source_store_name(&scratch).unwrap();
        let staged_source = output_dir.join(store_name);
        std::fs::rename(&scratch, &staged_source).unwrap();
        staged_source
    }

    #[test]
    fn generate_ncl_has_source_path() {
        let ncl = test_self_build_ncl();
        assert!(ncl.contains("/nix/store/abc123-crunch-src"));
        assert!(ncl.contains("crunch.Derivation"));
    }

    #[test]
    fn generate_ncl_source_is_plain_input() {
        // The source tree should be a string input (source path),
        // not a fetchTarball FOD.
        let ncl = test_self_build_ncl();
        assert!(!ncl.contains("crunch-src\",\n  hash"));
        assert!(ncl.contains("\"/nix/store/abc123-crunch-src\""));
    }

    #[test]
    fn generate_ncl_has_all_bootstrap_deps() {
        let ncl = test_self_build_ncl();
        assert!(ncl.contains("import \"bootstrap/seed.ncl\""));
        assert!(ncl.contains("import \"bootstrap/make.ncl\""));
        assert!(ncl.contains("import \"bootstrap/dash.ncl\""));
        assert!(ncl.contains("import \"bootstrap/binutils.ncl\""));
        assert!(ncl.contains("import \"bootstrap/musl.ncl\""));
        assert!(ncl.contains("import \"bootstrap/gcc.ncl\""));
        assert!(ncl.contains("import \"bootstrap/rust.ncl\""));
        assert!(ncl.contains("import \"lib/lib.ncl\""));
        assert!(!ncl.contains("import \"bootstrap/busybox.ncl\""));
        assert!(!ncl.contains("import \"bootstrap/bwrap.ncl\""));
    }

    #[test]
    fn generate_ncl_has_build_essentials() {
        let ncl = test_self_build_ncl();
        assert!(ncl.contains("cargo build"));
        assert!(ncl.contains("--release"));
        assert!(ncl.contains("$out/bin/crunch"));
        assert!(ncl.contains("SNIX_BUILD_SANDBOX_SHELL"));
        assert!(ncl.contains("export RUSTC_BOOTSTRAP=1"));
    }

    #[test]
    fn generate_ncl_uses_staged_vendor_config() {
        let ncl = test_self_build_ncl();
        assert!(ncl.contains(".cargo/vendor-config.toml"));
        assert!(!ncl.contains("git+https://github.com/tvlfyi/wu-manber.git"));
    }

    #[test]
    fn generate_ncl_uses_nix_store_env_var() {
        let ncl = test_self_build_ncl();
        // Shell globs must use $NIX_STORE for toolchain discovery.
        assert!(ncl.contains("$NIX_STORE/*-gcc"));
        assert!(ncl.contains("$NIX_STORE/*-rust"));
        assert!(ncl.contains("$NIX_STORE/*-%{seed_name}"));
        // Exact bootstrap tool paths are threaded directly.
        assert!(!ncl.contains("$NIX_STORE/*-busybox"));
        assert!(!ncl.contains("$NIX_STORE/*-bwrap"));
        // No hardcoded /nix/store in shell globs.
        assert!(!ncl.contains("for d in /nix/store/"));
    }

    #[test]
    fn generate_ncl_respects_store_prefix() {
        let ncl = generate_self_build_ncl("abc-src", "uvw-bwrap", "xyz-busybox", "/crunch/store");
        // The input path in the Nickel record should use the prefix.
        assert!(ncl.contains("\"/crunch/store/abc-src\""));
        assert!(ncl.contains("\"/crunch/store/uvw-bwrap\""));
        assert!(ncl.contains("\"/crunch/store/xyz-busybox\""));
        assert!(!ncl.contains("/nix/store/abc-src"));
    }

    #[test]
    fn tree_fingerprint_deterministic() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.txt"), "hello").unwrap();
        std::fs::create_dir_all(dir.path().join("sub")).unwrap();
        std::fs::write(dir.path().join("sub/b.txt"), "world").unwrap();

        let h1 = tree_fingerprint(dir.path()).unwrap();
        let h2 = tree_fingerprint(dir.path()).unwrap();
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64); // blake3 hex
    }

    #[test]
    fn tree_fingerprint_changes_with_content() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.txt"), "hello").unwrap();
        let h1 = tree_fingerprint(dir.path()).unwrap();

        std::fs::write(dir.path().join("a.txt"), "hello world").unwrap();
        let h2 = tree_fingerprint(dir.path()).unwrap();
        assert_ne!(h1, h2);
    }

    #[test]
    fn tree_fingerprint_changes_with_same_size_content() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.txt"), "hello").unwrap();
        let h1 = tree_fingerprint(dir.path()).unwrap();

        std::fs::write(dir.path().join("a.txt"), "jello").unwrap();
        let h2 = tree_fingerprint(dir.path()).unwrap();
        assert_ne!(h1, h2);
    }

    #[cfg(unix)]
    #[test]
    fn tree_fingerprint_changes_with_mode_bits() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("a.txt");
        std::fs::write(&file_path, "hello").unwrap();
        std::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o644)).unwrap();
        let h1 = tree_fingerprint(dir.path()).unwrap();

        std::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o755)).unwrap();
        let h2 = tree_fingerprint(dir.path()).unwrap();
        assert_ne!(h1, h2);
    }

    fn test_store_path(name: &str) -> StorePath<String> {
        let digest = *blake3::hash(name.as_bytes()).as_bytes();
        let mut store_digest = [0u8; 20];
        store_digest.copy_from_slice(&digest[..20]);
        StorePath::from_name_and_digest_fixed(name, store_digest).unwrap()
    }

    fn test_path_info(name: &str) -> snix_store::path_info::PathInfo {
        snix_store::path_info::PathInfo {
            store_path: test_store_path(name),
            node: snix_castore::Node::Symlink {
                target: snix_castore::SymlinkTarget::try_from("target").unwrap(),
            },
            references: vec![],
            nar_size: 1,
            nar_sha256: [7u8; 32],
            signatures: vec![],
            deriver: None,
            ca: None,
        }
    }

    #[test]
    fn resolve_single_root_output_dir_uses_root_labels() {
        let drv_path = test_store_path("busybox.drv");
        let output = test_path_info("busybox");
        let expected = PathBuf::from(output.store_path.to_absolute_path_with_prefix("/tmp/test-store"));
        let result = crunch_pipeline::PipelineResult {
            outcomes: vec![crunch_build::BuildOutcome {
                drv_path: drv_path.clone(),
                outputs: std::collections::BTreeMap::from([("out".to_string(), output)]),
                substitutions: std::collections::BTreeMap::new(),
                cached: false,
                log: None,
            }],
            failed: vec![],
            fod_mismatches: vec![],
            root_labels: std::collections::HashMap::from([(
                crunch_pipeline::drv_key_for("/nix/store", &drv_path),
                "busybox".to_string(),
            )]),
            hermeticity_mode: crunch_pipeline::HermeticityMode::Practical,
            hermeticity_audit_events: Vec::new(),
        };

        let actual =
            resolve_single_root_output_dir(&result, "/nix/store", Path::new("/tmp/test-store"), "busybox").unwrap();
        assert_eq!(actual, expected);
        assert!(actual.file_name().unwrap().to_string_lossy().ends_with("busybox"));
    }

    #[test]
    fn resolve_single_root_output_dir_rejects_multiple_matches() {
        let drv_a = test_store_path("crunch-a.drv");
        let drv_b = test_store_path("crunch-b.drv");
        let result = crunch_pipeline::PipelineResult {
            outcomes: vec![
                crunch_build::BuildOutcome {
                    drv_path: drv_a.clone(),
                    outputs: std::collections::BTreeMap::from([("out".to_string(), test_path_info("crunch-a"))]),
                    substitutions: std::collections::BTreeMap::new(),
                    cached: false,
                    log: None,
                },
                crunch_build::BuildOutcome {
                    drv_path: drv_b.clone(),
                    outputs: std::collections::BTreeMap::from([("out".to_string(), test_path_info("crunch-b"))]),
                    substitutions: std::collections::BTreeMap::new(),
                    cached: false,
                    log: None,
                },
            ],
            failed: vec![],
            fod_mismatches: vec![],
            root_labels: std::collections::HashMap::from([
                (crunch_pipeline::drv_key_for("/nix/store", &drv_a), "crunch".to_string()),
                (crunch_pipeline::drv_key_for("/nix/store", &drv_b), "crunch".to_string()),
            ]),
            hermeticity_mode: crunch_pipeline::HermeticityMode::Practical,
            hermeticity_audit_events: Vec::new(),
        };

        let err =
            resolve_single_root_output_dir(&result, "/nix/store", Path::new("/tmp/test-store"), "crunch").unwrap_err();
        assert!(err.message().contains("expected exactly 1 root output directory"));
    }

    #[test]
    fn run_cmd_reports_failure() {
        let err = run_cmd(Command::new("false").arg(""), "test-false").unwrap_err();
        assert!(err.message().contains("test-false"));
    }

    #[test]
    fn copy_selected_source_tree_copies_only_allowlisted_entries() {
        let repo = tempfile::tempdir().unwrap();
        write_stageable_checkout(repo.path());
        std::fs::create_dir_all(repo.path().join("target").join("debug")).unwrap();
        std::fs::write(repo.path().join("target").join("debug").join("junk"), "skip me\n").unwrap();
        std::fs::write(repo.path().join("scratch.txt"), "skip me too\n").unwrap();

        let stage = tempfile::tempdir().unwrap();
        copy_selected_source_tree(repo.path(), stage.path()).unwrap();

        assert!(stage.path().join("Cargo.toml").is_file());
        assert!(stage.path().join("vendor-deps").join("dep-a").join("Cargo.toml").is_file());
        assert!(stage.path().join(".cargo").join("vendor-config.toml").is_file());
        assert!(!stage.path().join("target").exists());
        assert!(!stage.path().join("scratch.txt").exists());
    }

    #[test]
    fn require_checked_vendor_inputs_rejects_missing_vendor_config() {
        let repo = tempfile::tempdir().unwrap();
        write_stageable_checkout(repo.path());
        std::fs::remove_file(repo.path().join(".cargo").join("vendor-config.toml")).unwrap();

        let err = require_checked_vendor_inputs(repo.path()).unwrap_err();
        assert!(err.to_string().contains("vendor-config.toml"), "unexpected error: {err}");
    }

    #[test]
    fn require_checked_vendor_inputs_rejects_non_vendored_directory_target() {
        let repo = tempfile::tempdir().unwrap();
        write_stageable_checkout(repo.path());
        std::fs::write(
            repo.path().join(".cargo").join("vendor-config.toml"),
            "[source.vendored-sources]\ndirectory = \"/tmp/not-vendor-deps\"\n",
        )
        .unwrap();

        let err = require_checked_vendor_inputs(repo.path()).unwrap_err();
        assert!(err.to_string().contains("must point at source-tree vendor-deps"), "unexpected error: {err}");
    }

    #[test]
    fn require_checked_vendor_inputs_accepts_matching_vendor_tree() {
        let repo = tempfile::tempdir().unwrap();
        write_stageable_checkout(repo.path());

        require_checked_vendor_inputs(repo.path()).unwrap();
    }

    #[test]
    fn require_checked_vendor_inputs_rejects_missing_locked_package() {
        let repo = tempfile::tempdir().unwrap();
        write_stageable_checkout(repo.path());
        std::fs::remove_dir_all(repo.path().join("vendor-deps").join("dep-a")).unwrap();

        let err = require_checked_vendor_inputs(repo.path()).unwrap_err();
        assert!(err.to_string().contains("missing from vendor-deps"), "unexpected error: {err}");
    }

    #[test]
    fn require_checked_vendor_inputs_rejects_extra_vendored_package() {
        let repo = tempfile::tempdir().unwrap();
        write_stageable_checkout(repo.path());
        let extra = repo.path().join("vendor-deps").join("dep-extra");
        std::fs::create_dir_all(&extra).unwrap();
        std::fs::write(extra.join("Cargo.toml"), "[package]\nname=\"dep-extra\"\nversion=\"0.0.0\"\n").unwrap();
        std::fs::write(extra.join("lib.rs"), "pub fn dep_extra() {}\n").unwrap();
        write_vendor_checksum_manifest(&extra, Some(&sample_cargo_sha256('b')));

        let err = require_checked_vendor_inputs(repo.path()).unwrap_err();
        assert!(err.to_string().contains("not present in Cargo.lock"), "unexpected error: {err}");
    }

    #[test]
    fn require_checked_vendor_inputs_rejects_package_checksum_mismatch() {
        let repo = tempfile::tempdir().unwrap();
        write_stageable_checkout(repo.path());
        let vendor_dep = repo.path().join("vendor-deps").join("dep-a");
        write_vendor_checksum_manifest(&vendor_dep, Some(&sample_cargo_sha256('b')));

        let err = require_checked_vendor_inputs(repo.path()).unwrap_err();
        assert!(err.to_string().contains("package checksum mismatch"), "unexpected error: {err}");
    }

    #[test]
    fn require_checked_vendor_inputs_rejects_file_checksum_mismatch() {
        let repo = tempfile::tempdir().unwrap();
        write_stageable_checkout(repo.path());
        std::fs::write(repo.path().join("vendor-deps").join("dep-a").join("lib.rs"), "pub fn tampered() {}\n").unwrap();

        let err = require_checked_vendor_inputs(repo.path()).unwrap_err();
        assert!(err.to_string().contains("vendor file checksum mismatch"), "unexpected error: {err}");
    }

    #[test]
    fn require_checked_vendor_inputs_accepts_git_package_without_registry_checksum() {
        let repo = tempfile::tempdir().unwrap();
        write_stageable_checkout(repo.path());
        let dep_dir = repo.path().join("vendor-deps").join("dep-a");
        write_vendor_checksum_manifest(&dep_dir, None);
        std::fs::write(
            repo.path().join("Cargo.lock"),
            "[[package]]\nname = \"dep-a\"\nversion = \"0.0.0\"\nsource = \"git+https://example.invalid/dep-a.git#0123456789abcdef\"\n",
        )
        .unwrap();

        require_checked_vendor_inputs(repo.path()).unwrap();
    }

    #[test]
    fn require_checked_vendor_inputs_rejects_registry_package_without_lock_checksum() {
        let repo = tempfile::tempdir().unwrap();
        write_stageable_checkout(repo.path());
        std::fs::write(
            repo.path().join("Cargo.lock"),
            "[[package]]\nname = \"dep-a\"\nversion = \"0.0.0\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\n",
        )
        .unwrap();

        let err = require_checked_vendor_inputs(repo.path()).unwrap_err();
        assert!(err.to_string().contains("missing Cargo.lock checksum"), "unexpected error: {err}");
    }

    #[test]
    fn require_checked_vendor_inputs_rejects_malformed_lock_quote_without_panic() {
        let repo = tempfile::tempdir().unwrap();
        write_stageable_checkout(repo.path());
        std::fs::write(
            repo.path().join("Cargo.lock"),
            "[[package]]\nname = \"\nversion = \"0.0.0\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\n",
        )
        .unwrap();

        let err = require_checked_vendor_inputs(repo.path()).unwrap_err();
        assert!(err.to_string().contains("quoted Cargo"), "unexpected error: {err}");
    }

    #[test]
    fn prepare_self_build_source_reuses_exact_staged_source() {
        let output_dir = tempfile::tempdir().unwrap();
        let staged_source = make_valid_staged_source(output_dir.path());

        let (store_name, resolved_path) =
            prepare_self_build_source(staged_source.as_path(), output_dir.path(), Some(staged_source.as_path()))
                .unwrap();

        assert_eq!(store_name, staged_source.file_name().unwrap().to_string_lossy());
        assert_eq!(resolved_path, staged_source);
    }

    #[test]
    fn prepare_self_build_source_rejects_source_outside_store_dir() {
        let output_dir = tempfile::tempdir().unwrap();
        let outside_dir = tempfile::tempdir().unwrap();
        let staged_source = make_valid_staged_source(outside_dir.path());

        let err = prepare_self_build_source(staged_source.as_path(), output_dir.path(), Some(staged_source.as_path()))
            .unwrap_err();
        assert!(err.to_string().contains("must live directly under the self-build store"), "unexpected error: {err}",);
    }

    #[test]
    fn validate_staged_source_dir_rejects_missing_lib_dir() {
        let output_dir = tempfile::tempdir().unwrap();
        let staged_source = output_dir.path().join("abc123-crunch-src");
        std::fs::create_dir_all(staged_source.join("bootstrap")).unwrap();
        std::fs::create_dir_all(staged_source.join(".cargo")).unwrap();
        std::fs::write(staged_source.join("Cargo.toml"), "[package]\nname = \"crunch\"\nversion = \"0.0.0\"\n")
            .unwrap();
        std::fs::write(
            staged_source.join(".cargo").join("vendor-config.toml"),
            "[source.crates-io]\nreplace-with = \"vendored-sources\"\n",
        )
        .unwrap();

        let err = validate_staged_source_dir(&staged_source, output_dir.path()).unwrap_err();
        assert!(err.to_string().contains("staged source missing lib/"), "unexpected error: {err}");
    }

    #[test]
    fn validate_staged_source_dir_rejects_tampered_contents() {
        let output_dir = tempfile::tempdir().unwrap();
        let staged_source = make_valid_staged_source(output_dir.path());
        std::fs::write(staged_source.join("Cargo.toml"), "[package]\nname = \"crunch\"\nversion = \"0.0.1\"\n")
            .unwrap();

        let err = validate_staged_source_dir(&staged_source, output_dir.path()).unwrap_err();
        assert!(err.to_string().contains("contents no longer match its store name"), "unexpected error: {err}",);
    }

    #[test]
    fn resolve_store_entry_name_uses_exact_store_child() {
        let output_dir = tempfile::tempdir().unwrap();
        let entry_path = output_dir.path().join("abc123-bwrap");
        std::fs::create_dir_all(&entry_path).unwrap();

        let store_name = resolve_store_entry_name(&entry_path, output_dir.path(), "bwrap").unwrap();
        assert_eq!(store_name, "abc123-bwrap");
    }

    #[test]
    fn resolve_explicit_bootstrap_bwrap_source_accepts_exact_store_binary() {
        let output_dir = tempfile::tempdir().unwrap();
        let bwrap_bin = output_dir.path().join("abc123-bwrap").join("bin").join("bwrap");
        std::fs::create_dir_all(bwrap_bin.parent().unwrap()).unwrap();
        std::fs::write(&bwrap_bin, "#!/bin/sh\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&bwrap_bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        }

        let source = resolve_explicit_bootstrap_bwrap_source(output_dir.path(), Some(&bwrap_bin)).unwrap();
        assert!(source.is_some(), "explicit bwrap source should resolve");
        match source.unwrap() {
            BwrapSource::CrunchBuilt(dir) => assert_eq!(dir, output_dir.path().join("abc123-bwrap").join("bin")),
            BwrapSource::HostFallback(path) => {
                panic!("expected crunch-built bwrap, got host fallback {}", path.display())
            }
        }
    }

    #[test]
    fn resolve_explicit_bootstrap_bwrap_source_ignores_stale_sibling() {
        let output_dir = tempfile::tempdir().unwrap();
        let stage0_bwrap_bin = output_dir.path().join("aaa-stage0-bwrap").join("bin").join("bwrap");
        std::fs::create_dir_all(stage0_bwrap_bin.parent().unwrap()).unwrap();
        std::fs::write(&stage0_bwrap_bin, "#!/bin/sh\n").unwrap();
        let stale_bwrap_bin = output_dir.path().join("zzz-stale-bwrap").join("bin").join("bwrap");
        std::fs::create_dir_all(stale_bwrap_bin.parent().unwrap()).unwrap();
        std::fs::write(&stale_bwrap_bin, "#!/bin/sh\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&stage0_bwrap_bin, std::fs::Permissions::from_mode(0o755)).unwrap();
            std::fs::set_permissions(&stale_bwrap_bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        }

        let source = resolve_explicit_bootstrap_bwrap_source(output_dir.path(), Some(&stage0_bwrap_bin)).unwrap();
        match source.unwrap() {
            BwrapSource::CrunchBuilt(dir) => {
                assert_eq!(dir, output_dir.path().join("aaa-stage0-bwrap").join("bin"));
                assert_ne!(dir, output_dir.path().join("zzz-stale-bwrap").join("bin"));
            }
            BwrapSource::HostFallback(path) => {
                panic!("expected crunch-built bwrap, got host fallback {}", path.display())
            }
        }
    }

    #[test]
    fn resolve_explicit_bootstrap_busybox_path_rejects_path_outside_store() {
        let output_dir = tempfile::tempdir().unwrap();
        let outside_dir = tempfile::tempdir().unwrap();
        let busybox_bin = outside_dir.path().join("outside-busybox").join("bin").join("busybox");
        std::fs::create_dir_all(busybox_bin.parent().unwrap()).unwrap();
        std::fs::write(&busybox_bin, "#!/bin/sh\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&busybox_bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        }

        let err = resolve_explicit_bootstrap_busybox_path(output_dir.path(), Some(&busybox_bin)).unwrap_err();
        let message = err.to_string();
        assert!(message.contains("must live directly under the self-build store"), "unexpected error: {message}");
    }

    #[test]
    fn resolve_explicit_bootstrap_busybox_path_ignores_stale_sibling() {
        let output_dir = tempfile::tempdir().unwrap();
        let stage0_busybox_bin = output_dir.path().join("aaa-stage0-busybox").join("bin").join("busybox");
        std::fs::create_dir_all(stage0_busybox_bin.parent().unwrap()).unwrap();
        std::fs::write(&stage0_busybox_bin, "#!/bin/sh\n").unwrap();
        let stale_busybox_bin = output_dir.path().join("zzz-stale-busybox").join("bin").join("busybox");
        std::fs::create_dir_all(stale_busybox_bin.parent().unwrap()).unwrap();
        std::fs::write(&stale_busybox_bin, "#!/bin/sh\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&stage0_busybox_bin, std::fs::Permissions::from_mode(0o755)).unwrap();
            std::fs::set_permissions(&stale_busybox_bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        }

        let busybox_path =
            resolve_explicit_bootstrap_busybox_path(output_dir.path(), Some(&stage0_busybox_bin)).unwrap();
        assert_eq!(busybox_path, Some(stage0_busybox_bin.clone()));
        assert_ne!(busybox_path, Some(stale_busybox_bin));
    }

    #[test]
    fn absolutize_path_makes_relative_path_absolute() {
        let relative = Path::new("relative-store");
        let absolute = absolutize_path(relative).unwrap();
        assert!(absolute.is_absolute());
        assert!(absolute.ends_with(relative));
    }

    #[test]
    fn dir_size_basic() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a"), "12345").unwrap();
        std::fs::write(dir.path().join("b"), "67").unwrap();
        let s = dir_size(dir.path());
        assert_eq!(s, 7);
    }

    // ── NCL bwrap branch tests ──────────────────────────────────

    #[test]
    fn generate_ncl_has_exact_bwrap_and_busybox_paths() {
        let ncl = test_self_build_ncl();
        assert!(ncl.contains("BWRAP_BIN=\"/nix/store/bwrap123-bwrap/bin\""));
        assert!(ncl.contains("BUSYBOX_BIN=\"/nix/store/busybox123-busybox/bin/busybox\""));
        assert!(ncl.contains("Using crunch-built bwrap"));
        assert!(ncl.contains("Using crunch-built busybox"));
    }

    #[test]
    fn generate_ncl_uses_exact_bootstrap_inputs_without_glob_discovery() {
        let ncl = test_self_build_ncl();
        assert!(ncl.contains("\"/nix/store/bwrap123-bwrap\""));
        assert!(ncl.contains("\"/nix/store/busybox123-busybox\""));
        assert!(!ncl.contains("$NIX_STORE/*-bwrap"));
        assert!(!ncl.contains("$NIX_STORE/*-busybox"));
    }

    // ── Host-side bwrap resolution tests ───────────────────────
    //
    // These tests use temp dirs to avoid mutating global PATH.

    #[test]
    fn find_crunch_bwrap_finds_executable_in_store() {
        let store = tempfile::tempdir().unwrap();
        let bwrap_dir = store.path().join("abc123-bwrap").join("bin");
        std::fs::create_dir_all(&bwrap_dir).unwrap();
        let bwrap_bin = bwrap_dir.join("bwrap");
        std::fs::write(&bwrap_bin, "#!/bin/sh\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&bwrap_bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        }

        let result = find_crunch_bwrap(store.path());
        assert!(result.is_some(), "should find bwrap in store");
        assert_eq!(result.unwrap(), bwrap_dir);
    }

    #[test]
    fn find_crunch_bwrap_ignores_non_executable() {
        let store = tempfile::tempdir().unwrap();
        let bwrap_dir = store.path().join("abc123-bwrap").join("bin");
        std::fs::create_dir_all(&bwrap_dir).unwrap();
        let bwrap_bin = bwrap_dir.join("bwrap");
        std::fs::write(&bwrap_bin, "not executable").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&bwrap_bin, std::fs::Permissions::from_mode(0o644)).unwrap();
        }

        let result = find_crunch_bwrap(store.path());
        assert!(result.is_none(), "should skip non-executable bwrap");
    }

    #[test]
    fn find_crunch_bwrap_returns_none_for_empty_store() {
        let store = tempfile::tempdir().unwrap();
        assert!(find_crunch_bwrap(store.path()).is_none());
    }

    #[test]
    fn find_crunch_bwrap_ignores_wrong_name_suffix() {
        let store = tempfile::tempdir().unwrap();
        // Name doesn't end with "-bwrap"
        let dir = store.path().join("abc123-notbwrap").join("bin");
        std::fs::create_dir_all(&dir).unwrap();
        let bin = dir.join("bwrap");
        std::fs::write(&bin, "#!/bin/sh\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        }

        assert!(find_crunch_bwrap(store.path()).is_none());
    }

    #[test]
    fn resolve_bwrap_source_prefers_crunch_built() {
        let store = tempfile::tempdir().unwrap();
        let bwrap_dir = store.path().join("abc123-bwrap").join("bin");
        std::fs::create_dir_all(&bwrap_dir).unwrap();
        let bwrap_bin = bwrap_dir.join("bwrap");
        std::fs::write(&bwrap_bin, "#!/bin/sh\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&bwrap_bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        }

        let result = resolve_bwrap_source(store.path(), crunch_pipeline::HermeticityMode::Practical).unwrap();
        assert!(result.is_crunch_built(), "should prefer crunch-built bwrap",);
        match result {
            BwrapSource::CrunchBuilt(dir) => assert_eq!(dir, bwrap_dir),
            _ => panic!("expected CrunchBuilt"),
        }
    }

    #[test]
    fn resolve_bwrap_source_falls_back_to_controlled_host_path() {
        let _lock = PATH_MUTEX.lock().unwrap();
        let fake_dir = tempfile::tempdir().unwrap();
        let fake_dir_path = make_fake_executable(fake_dir.path(), "bwrap");
        let fake_bwrap_path = fake_dir_path.join("bwrap");
        let _path_guard = PathGuard::set(fake_dir.path());

        assert_eq!(find_executable_on_path("bwrap"), Some(fake_bwrap_path.clone()));

        let store = tempfile::tempdir().unwrap();
        let result = resolve_bwrap_source(store.path(), crunch_pipeline::HermeticityMode::Practical).unwrap();

        assert!(!result.is_crunch_built(), "should be host fallback");
        match result {
            BwrapSource::HostFallback(path) => assert_eq!(path, fake_bwrap_path),
            _ => panic!("expected HostFallback"),
        }
    }

    #[test]
    fn resolve_bwrap_source_errors_without_host_bwrap_on_controlled_path() {
        let _lock = PATH_MUTEX.lock().unwrap();
        let empty_path = tempfile::tempdir().unwrap();
        let _path_guard = PathGuard::set(empty_path.path());

        let empty_entries = std::fs::read_dir(empty_path.path()).unwrap().count();
        assert_eq!(empty_entries, 0, "temp PATH dir must start empty");
        assert!(find_executable_on_path("bwrap").is_none(), "empty temp PATH must not expose bwrap");

        let store = tempfile::tempdir().unwrap();
        let result = resolve_bwrap_source(store.path(), crunch_pipeline::HermeticityMode::Practical);

        assert!(result.is_err(), "should error when no bwrap");
        assert_eq!(
            result.unwrap_err().message(),
            "bwrap (bubblewrap) not found. The first self-build requires bwrap on PATH. \
             Install it from https://github.com/containers/bubblewrap",
        );
    }

    #[test]
    fn resolve_bwrap_source_strict_rejects_host_fallback_once_bootstrap_root_exists() {
        let _lock = PATH_MUTEX.lock().unwrap();
        let fake_dir = tempfile::tempdir().unwrap();
        let fake_dir_path = make_fake_executable(fake_dir.path(), "bwrap");
        let fake_bwrap_path = fake_dir_path.join("bwrap");
        let _path_guard = PathGuard::set(fake_dir.path());

        assert_eq!(find_executable_on_path("bwrap"), Some(fake_bwrap_path));

        let store = tempfile::tempdir().unwrap();
        let broken_bwrap_dir = store.path().join("abc-bwrap").join("bin");
        std::fs::create_dir_all(&broken_bwrap_dir).unwrap();
        std::fs::write(broken_bwrap_dir.join("bwrap"), "not executable").unwrap();

        let result = resolve_bwrap_source(store.path(), crunch_pipeline::HermeticityMode::Strict);

        assert!(result.is_err(), "strict later stage must reject host fallback");
        let message = result.unwrap_err().message().to_string();
        assert!(message.contains("refuses host bwrap fallback"), "unexpected error: {message}");
        assert!(message.contains("bwrap_root=true"), "unexpected error: {message}");
        assert!(message.contains("busybox_root=false"), "unexpected error: {message}");
    }

    #[test]
    fn enforce_source_resolution_policy_rejects_checkout_fallback_in_strict_later_stage() {
        let presence = BootstrapToolPresence {
            has_bwrap_root: true,
            has_busybox_root: true,
        };
        let output_dir = Path::new("/tmp/proof-store");

        let result =
            enforce_source_resolution_policy(output_dir, crunch_pipeline::HermeticityMode::Strict, presence, None);

        assert!(result.is_err(), "strict later stage must require --source-store-path");
        let message = result.unwrap_err().message().to_string();
        assert!(message.contains("requires --source-store-path"), "unexpected error: {message}");
        assert!(message.contains("Refusing checkout source discovery fallback"), "unexpected error: {message}");
    }

    #[test]
    fn choose_host_bwrap_path_prefers_wrapper() {
        let wrapper = PathBuf::from("/run/wrappers/bin/bwrap");
        let path = PathBuf::from("/nix/store/abc-bubblewrap/bin/bwrap");
        let chosen = choose_host_bwrap_path(Some(wrapper.clone()), Some(path));
        assert_eq!(chosen, Some(wrapper));
    }

    #[test]
    fn choose_host_bwrap_path_falls_back_to_path() {
        let path = PathBuf::from("/usr/bin/bwrap");
        let chosen = choose_host_bwrap_path(None, Some(path.clone()));
        assert_eq!(chosen, Some(path));
        assert!(choose_host_bwrap_path(None, None).is_none());
    }

    #[test]
    fn find_executable_on_path_returns_none_for_nonexistent() {
        assert!(find_executable_on_path("this-binary-does-not-exist-crunch-test").is_none());
    }

    #[test]
    fn is_executable_true_for_real_binary() {
        // /bin/sh should be executable on any Unix host.
        #[cfg(unix)]
        {
            let sh = Path::new("/bin/sh");
            if sh.exists() {
                assert!(is_executable(sh), "/bin/sh should be executable");
            }
        }
    }

    #[test]
    fn is_executable_false_for_plain_file() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("plain.txt");
        std::fs::write(&f, "hello").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&f, std::fs::Permissions::from_mode(0o644)).unwrap();
        }
        assert!(!is_executable(&f), "plain file should not be executable");
    }

    // ── Deterministic PATH tests (tempdir-based) ──────────────
    //
    // Tests that call prepend_to_path() mutate the process-global PATH.
    // They serialize on PATH_MUTEX and save/restore around assertions.
    static PATH_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// Create a fake executable in a temp dir. Returns the directory.
    fn make_fake_executable(dir: &Path, name: &str) -> PathBuf {
        std::fs::create_dir_all(dir).unwrap();
        let bin = dir.join(name);
        std::fs::write(&bin, "#!/bin/sh\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        dir.to_path_buf()
    }

    struct PathGuard {
        original_path: Option<std::ffi::OsString>,
    }

    impl PathGuard {
        fn set(controlled_path: &Path) -> Self {
            let original_path = std::env::var_os("PATH");
            unsafe { std::env::set_var("PATH", controlled_path) };
            Self { original_path }
        }
    }

    impl Drop for PathGuard {
        fn drop(&mut self) {
            match &self.original_path {
                Some(path) => unsafe { std::env::set_var("PATH", path) },
                None => unsafe { std::env::remove_var("PATH") },
            }
        }
    }

    #[test]
    fn find_executable_on_path_finds_binary_in_tempdir() {
        let _lock = PATH_MUTEX.lock().unwrap();
        let orig = std::env::var_os("PATH");

        let dir = tempfile::tempdir().unwrap();
        make_fake_executable(dir.path(), "crunch-test-bwrap");

        // Set PATH to only our temp dir so the real function's
        // var_os + split_paths + ordering logic is exercised.
        unsafe { std::env::set_var("PATH", dir.path()) };

        let found = find_executable_on_path("crunch-test-bwrap");

        // Restore before asserting.
        match &orig {
            Some(p) => unsafe { std::env::set_var("PATH", p) },
            None => unsafe { std::env::remove_var("PATH") },
        }

        assert!(found.is_some(), "should find the fake executable");
        assert_eq!(found.unwrap(), dir.path().join("crunch-test-bwrap"));
    }

    #[cfg(unix)]
    #[test]
    fn find_executable_on_path_skips_non_executable_in_tempdir() {
        let _lock = PATH_MUTEX.lock().unwrap();
        let orig = std::env::var_os("PATH");

        let dir = tempfile::tempdir().unwrap();
        let bin = dir.path().join("crunch-test-noexec");
        std::fs::write(&bin, "not executable").unwrap();
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o644)).unwrap();
        }

        unsafe { std::env::set_var("PATH", dir.path()) };

        let found = find_executable_on_path("crunch-test-noexec");

        match &orig {
            Some(p) => unsafe { std::env::set_var("PATH", p) },
            None => unsafe { std::env::remove_var("PATH") },
        }

        assert!(found.is_none(), "should skip non-executable file");
    }

    #[test]
    fn prepend_to_path_adds_dir_first() {
        let _lock = PATH_MUTEX.lock().unwrap();
        let orig = std::env::var_os("PATH");
        let dir = tempfile::tempdir().unwrap();
        make_fake_executable(dir.path(), "bwrap");

        prepend_to_path(dir.path()).unwrap();

        let new_path = std::env::var_os("PATH").unwrap();
        let dirs: Vec<PathBuf> = std::env::split_paths(&new_path).collect();

        // Restore before assertions so parallel tests aren't affected.
        match &orig {
            Some(p) => unsafe { std::env::set_var("PATH", p) },
            None => unsafe { std::env::remove_var("PATH") },
        }

        assert!(!dirs.is_empty(), "PATH should not be empty after prepend",);
        assert_eq!(dirs[0], dir.path().to_path_buf(), "prepended dir should be first in PATH",);
        // No trailing empty component (the old manual concat bug).
        assert!(dirs.iter().all(|d| !d.as_os_str().is_empty()), "PATH should have no empty components: {dirs:?}",);
    }

    #[test]
    fn prepend_to_path_handles_empty_path() {
        let _lock = PATH_MUTEX.lock().unwrap();
        let orig = std::env::var_os("PATH");
        unsafe { std::env::set_var("PATH", "") };

        let dir = tempfile::tempdir().unwrap();
        prepend_to_path(dir.path()).unwrap();

        let new_path = std::env::var_os("PATH").unwrap();
        let dirs: Vec<PathBuf> = std::env::split_paths(&new_path).collect();

        // Restore.
        match &orig {
            Some(p) => unsafe { std::env::set_var("PATH", p) },
            None => unsafe { std::env::remove_var("PATH") },
        }

        // Should have exactly one entry, not a trailing colon.
        assert_eq!(dirs.len(), 1, "empty PATH + prepend = 1 entry: {dirs:?}");
        assert_eq!(dirs[0], dir.path().to_path_buf());
    }

    #[test]
    fn activate_bwrap_source_prepends_host_parent_dir() {
        let _lock = PATH_MUTEX.lock().unwrap();
        let orig = std::env::var_os("PATH");
        unsafe { std::env::set_var("PATH", "") };

        let dir = tempfile::tempdir().unwrap();
        make_fake_executable(dir.path(), "bwrap");
        let source = BwrapSource::HostFallback(dir.path().join("bwrap"));

        activate_bwrap_source(&source).unwrap();

        let new_path = std::env::var_os("PATH").unwrap();
        let dirs: Vec<PathBuf> = std::env::split_paths(&new_path).collect();

        match &orig {
            Some(p) => unsafe { std::env::set_var("PATH", p) },
            None => unsafe { std::env::remove_var("PATH") },
        }

        assert!(!dirs.is_empty(), "host fallback activation should prepend at least one dir",);
        assert_eq!(dirs[0], dir.path().to_path_buf());
        if let Some(wrapper_dir) = find_nixos_wrapper_dir()
            && wrapper_dir != dir.path()
        {
            assert!(dirs.len() >= 2, "wrapper dir should be present after source dir");
            assert_eq!(dirs[1], wrapper_dir);
        }
    }

    #[test]
    fn build_bwrap_path_entries_adds_wrapper_after_source() {
        let source_dir = PathBuf::from("/nix/store/abc-bubblewrap/bin");
        let wrapper_dir = PathBuf::from("/run/wrappers/bin");
        let dirs = build_bwrap_path_entries(Some(wrapper_dir.clone()), source_dir.clone());
        assert_eq!(dirs, vec![source_dir, wrapper_dir]);
    }

    #[test]
    fn build_bwrap_path_entries_deduplicates_wrapper() {
        let source_dir = PathBuf::from("/run/wrappers/bin");
        let dirs = build_bwrap_path_entries(Some(source_dir.clone()), source_dir.clone());
        assert_eq!(dirs, vec![source_dir]);
        let dirs = build_bwrap_path_entries(None, PathBuf::from("/usr/bin"));
        assert_eq!(dirs, vec![PathBuf::from("/usr/bin")]);
    }

    // ── BwrapSource tests ─────────────────────────────────────────

    #[test]
    fn bwrap_source_display_roundtrip_crunch_built() {
        let src = BwrapSource::CrunchBuilt(PathBuf::from("/tmp/store/abc-bwrap/bin"));
        let s = src.to_string();
        assert!(s.starts_with("crunch-built:"));
        let parsed = BwrapSource::parse(&s).unwrap();
        assert_eq!(parsed, src);
    }

    #[test]
    fn bwrap_source_display_roundtrip_host_fallback() {
        let src = BwrapSource::HostFallback(PathBuf::from("/usr/bin/bwrap"));
        let s = src.to_string();
        assert!(s.starts_with("host-fallback:"));
        let parsed = BwrapSource::parse(&s).unwrap();
        assert_eq!(parsed, src);
    }

    #[test]
    fn bwrap_source_parse_rejects_garbage() {
        assert!(BwrapSource::parse("").is_none());
        assert!(BwrapSource::parse("something-else:/bin/bwrap").is_none());
    }

    #[test]
    fn bwrap_source_is_crunch_built_predicate() {
        assert!(BwrapSource::CrunchBuilt(PathBuf::from("/x")).is_crunch_built());
        assert!(!BwrapSource::HostFallback(PathBuf::from("/x")).is_crunch_built());
    }

    #[test]
    fn bwrap_source_bin_dir_uses_parent_for_host_fallback() {
        let source = BwrapSource::HostFallback(PathBuf::from("/run/wrappers/bin/bwrap"));
        let dir = source.bin_dir().unwrap();
        assert_eq!(dir, PathBuf::from("/run/wrappers/bin"));
    }

    // ── SelfBuildReport tests ────────────────────────────────────

    #[test]
    fn report_format_roundtrip_with_busybox() {
        let report = SelfBuildReport {
            hermeticity_mode: crunch_pipeline::HermeticityMode::Strict,
            invoking_binary: PathBuf::from("/tmp/checkout/target/debug/crunch"),
            staged_source: PathBuf::from("/tmp/store/src-crunch-src"),
            bwrap_source: BwrapSource::CrunchBuilt(PathBuf::from("/tmp/store/abc-bwrap/bin")),
            fallback_events: vec![
                SelfBuildFallbackEvent::BwrapHostFallback(PathBuf::from("/run/wrappers/bin/bwrap")),
                SelfBuildFallbackEvent::SourceHostDiscovery(PathBuf::from("/work/crunch")),
            ],
            busybox_path: Some(PathBuf::from("/tmp/store/xyz-busybox/bin/busybox")),
            output_binary: PathBuf::from("/tmp/store/def-crunch/bin/crunch"),
        };
        let lines = report.format_proof_lines();

        // Each line starts with the prefix.
        for line in lines.lines() {
            assert!(line.starts_with(PROOF_PREFIX), "bad line: {line}");
        }

        let parsed = SelfBuildReport::parse_proof_lines(&lines).expect("should parse back");
        assert_eq!(parsed.hermeticity_mode, report.hermeticity_mode);
        assert_eq!(parsed.invoking_binary, report.invoking_binary);
        assert_eq!(parsed.staged_source, report.staged_source);
        assert_eq!(parsed.bwrap_source, report.bwrap_source);
        assert_eq!(parsed.fallback_events, report.fallback_events);
        assert_eq!(parsed.busybox_path, report.busybox_path);
        assert_eq!(parsed.output_binary, report.output_binary);
    }

    #[test]
    fn report_format_roundtrip_without_busybox() {
        let report = SelfBuildReport {
            hermeticity_mode: crunch_pipeline::HermeticityMode::Practical,
            invoking_binary: PathBuf::from("/usr/bin/crunch"),
            staged_source: PathBuf::from("/tmp/store/src-crunch-src"),
            bwrap_source: BwrapSource::HostFallback(PathBuf::from("/usr/bin/bwrap")),
            fallback_events: vec![SelfBuildFallbackEvent::BwrapHostFallback(PathBuf::from(
                "/usr/bin/bwrap",
            ))],
            busybox_path: None,
            output_binary: PathBuf::from("/tmp/store/out-crunch/bin/crunch"),
        };
        let lines = report.format_proof_lines();
        assert!(lines.contains("busybox-path=none"));
        assert!(lines.contains("fallback-event=bwrap-host-fallback:/usr/bin/bwrap"));

        let parsed = SelfBuildReport::parse_proof_lines(&lines).expect("should parse back");
        assert_eq!(parsed.fallback_events, report.fallback_events);
        assert!(parsed.busybox_path.is_none());
        assert!(!parsed.bwrap_source.is_crunch_built());
    }

    #[test]
    fn report_parse_returns_none_for_empty_input() {
        assert!(SelfBuildReport::parse_proof_lines("").is_none());
    }

    #[test]
    fn report_parse_returns_none_for_partial_input() {
        let partial = format!(
            "{PROOF_PREFIX} hermeticity-mode=strict\n\
             {PROOF_PREFIX} invoking-binary=/bin/crunch\n\
             {PROOF_PREFIX} staged-source=/store/src-crunch-src\n\
             {PROOF_PREFIX} bwrap-source=host-fallback:/usr/bin/bwrap\n\
             {PROOF_PREFIX} fallback-event=none\n"
        );
        // Missing busybox-path and output-binary.
        assert!(SelfBuildReport::parse_proof_lines(&partial).is_none());
    }

    #[test]
    fn report_parse_ignores_non_report_proof_lines() {
        let mixed = format!(
            "some random log line\n\
             {PROOF_PREFIX} {PROGRESS_KEY}bootstrap-tool-start:bwrap.ncl\n\
             {PROOF_PREFIX} hermeticity-mode=strict\n\
             {PROOF_PREFIX} invoking-binary=/bin/crunch\n\
             {PROOF_PREFIX} staged-source=/store/src-crunch-src\n\
             another log line\n\
             {PROOF_PREFIX} {PROGRESS_KEY}bootstrap-tool-done:bwrap.ncl\n\
             {PROOF_PREFIX} bwrap-source=crunch-built:/store/x-bwrap/bin\n\
             {PROOF_PREFIX} fallback-event=source-host-discovery:/work/crunch\n\
             {PROOF_PREFIX} busybox-path=/store/y-busybox/bin/busybox\n\
             {PROOF_PREFIX} output-binary=/store/z-crunch/bin/crunch\n\
             {PROOF_PREFIX} {PROGRESS_KEY}crunch-build-done\n"
        );
        let parsed = SelfBuildReport::parse_proof_lines(&mixed).expect("should parse despite progress markers");
        assert_eq!(parsed.hermeticity_mode, crunch_pipeline::HermeticityMode::Strict);
        assert_eq!(parsed.invoking_binary, PathBuf::from("/bin/crunch"));
        assert_eq!(parsed.staged_source, PathBuf::from("/store/src-crunch-src"));
        assert!(parsed.bwrap_source.is_crunch_built());
        assert_eq!(parsed.fallback_events, vec![SelfBuildFallbackEvent::SourceHostDiscovery(PathBuf::from(
            "/work/crunch"
        ))]);
    }

    // ── find_crunch_busybox tests ───────────────────────────────

    #[test]
    fn find_crunch_busybox_finds_executable() {
        let store = tempfile::tempdir().unwrap();
        let bb_dir = store.path().join("abc-busybox").join("bin");
        std::fs::create_dir_all(&bb_dir).unwrap();
        let bb = bb_dir.join("busybox");
        std::fs::write(&bb, "#!/bin/sh\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&bb, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        let result = find_crunch_busybox(store.path());
        assert_eq!(result, Some(bb));
    }

    #[test]
    fn find_crunch_busybox_returns_none_for_empty_store() {
        let store = tempfile::tempdir().unwrap();
        assert!(find_crunch_busybox(store.path()).is_none());
    }

    #[test]
    fn find_crunch_busybox_ignores_wrong_suffix() {
        let store = tempfile::tempdir().unwrap();
        let dir = store.path().join("abc-notbusybox").join("bin");
        std::fs::create_dir_all(&dir).unwrap();
        let bb = dir.join("busybox");
        std::fs::write(&bb, "#!/bin/sh\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&bb, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        assert!(find_crunch_busybox(store.path()).is_none());
    }

    // ── find_crunch_outputs / invalidate tests ────────────────

    #[test]
    fn find_crunch_outputs_finds_matching_dirs() {
        let store = tempfile::tempdir().unwrap();
        // Create two *-crunch entries and one non-matching entry.
        for name in ["aaa-crunch", "bbb-crunch", "ccc-notcrunch"] {
            let bin_dir = store.path().join(name).join("bin");
            std::fs::create_dir_all(&bin_dir).unwrap();
            std::fs::write(bin_dir.join("crunch"), "fake").unwrap();
        }
        let found = find_crunch_outputs(store.path());
        assert_eq!(found.len(), 2, "should find exactly 2 *-crunch dirs");
        let names: Vec<&str> = found.iter().map(|(n, _)| n.as_str()).collect();
        assert!(names.contains(&"aaa-crunch"));
        assert!(names.contains(&"bbb-crunch"));
    }

    #[test]
    fn find_crunch_outputs_empty_store() {
        let store = tempfile::tempdir().unwrap();
        assert!(find_crunch_outputs(store.path()).is_empty());
    }

    #[test]
    fn find_crunch_outputs_skips_without_binary() {
        let store = tempfile::tempdir().unwrap();
        // Directory named *-crunch but no bin/crunch file.
        std::fs::create_dir_all(store.path().join("aaa-crunch")).unwrap();
        assert!(find_crunch_outputs(store.path()).is_empty());
    }

    #[test]
    fn invalidate_crunch_outputs_removes_dirs() {
        let store = tempfile::tempdir().unwrap();
        let dir = store.path().join("abc-crunch");
        let bin_dir = dir.join("bin");
        std::fs::create_dir_all(&bin_dir).unwrap();
        std::fs::write(bin_dir.join("crunch"), "fake").unwrap();

        let removed = invalidate_crunch_outputs(store.path()).unwrap();
        assert_eq!(removed, 1);
        assert!(!dir.exists(), "directory should be removed");
    }

    #[test]
    fn invalidate_crunch_outputs_noop_on_empty_store() {
        let store = tempfile::tempdir().unwrap();
        let removed = invalidate_crunch_outputs(store.path()).unwrap();
        assert_eq!(removed, 0);
    }

    // ── Behavioral invariant tests ─────────────────────────────
    //
    // These tests enforce the self-build invariants documented in
    // AGENTS.md by testing constants and calling real functions,
    // not scanning source text.

    /// REQUIRED_BOOTSTRAP_TOOLS must include both bwrap and busybox.
    #[test]
    fn required_tools_includes_bwrap_and_busybox() {
        assert!(REQUIRED_BOOTSTRAP_TOOLS.contains(&"bwrap.ncl"), "REQUIRED_BOOTSTRAP_TOOLS must include bwrap.ncl",);
        assert!(
            REQUIRED_BOOTSTRAP_TOOLS.contains(&"busybox.ncl"),
            "REQUIRED_BOOTSTRAP_TOOLS must include busybox.ncl",
        );
    }

    /// REQUIRED_BOOTSTRAP_TOOLS must have exactly 2 entries.
    /// Adding a new tool is fine but requires updating this test
    /// and the AGENTS.md docs.
    #[test]
    fn required_tools_count_is_exact() {
        assert_eq!(
            REQUIRED_BOOTSTRAP_TOOLS.len(),
            2,
            "REQUIRED_BOOTSTRAP_TOOLS changed size; update this test and AGENTS.md",
        );
    }

    /// Self-build step count must be exactly 4.
    #[test]
    fn self_build_step_count_is_four() {
        assert_eq!(SELF_BUILD_STEP_COUNT, 4, "SELF_BUILD_STEP_COUNT changed; update this test and AGENTS.md",);
    }

    /// validate_bootstrap_tools must error when a required file is missing.
    #[test]
    fn validate_bootstrap_tools_errors_on_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        // Empty dir — no .ncl files exist.
        let result = validate_bootstrap_tools(dir.path());
        assert!(result.is_err(), "must error when bootstrap tools are missing");
        let msg = result.unwrap_err().message().to_string();
        assert!(msg.contains("not found"), "error must mention missing file: {msg}",);
    }

    /// validate_bootstrap_tools must pass when all required files exist.
    #[test]
    fn validate_bootstrap_tools_passes_when_present() {
        let dir = tempfile::tempdir().unwrap();
        for name in REQUIRED_BOOTSTRAP_TOOLS {
            std::fs::write(dir.path().join(name), "# placeholder").unwrap();
        }
        let result = validate_bootstrap_tools(dir.path());
        assert!(result.is_ok(), "must pass when all tools exist");
    }

    /// validate_bootstrap_tools must error if only one tool exists.
    #[test]
    fn validate_bootstrap_tools_errors_on_partial() {
        let dir = tempfile::tempdir().unwrap();
        // Only create the first tool.
        std::fs::write(dir.path().join(REQUIRED_BOOTSTRAP_TOOLS[0]), "# placeholder").unwrap();
        let result = validate_bootstrap_tools(dir.path());
        assert!(result.is_err(), "must error when only one bootstrap tool exists",);
    }

    /// verify_tools_on_disk must error when the store is empty and no
    /// bwrap is on PATH. Controls PATH with an empty temp dir so the
    /// test does not depend on host search semantics.
    #[test]
    fn verify_tools_on_disk_errors_on_empty_store() {
        let _lock = PATH_MUTEX.lock().unwrap();
        let empty_path = tempfile::tempdir().unwrap();
        let _path_guard = PathGuard::set(empty_path.path());

        let empty_entries = std::fs::read_dir(empty_path.path()).unwrap().count();
        assert_eq!(empty_entries, 0, "temp PATH dir must start empty");
        assert!(find_executable_on_path("bwrap").is_none(), "empty temp PATH must not expose bwrap",);

        let store = tempfile::tempdir().unwrap();
        let result = verify_tools_on_disk(store.path());

        assert!(result.is_err(), "must error when no tools on disk");
        assert_eq!(
            result.unwrap_err().message(),
            "bwrap (bubblewrap) not found. The first self-build requires bwrap \
             on PATH. Install it from https://github.com/containers/bubblewrap",
        );
    }

    /// When host bwrap exists but store has no crunch-built bwrap,
    /// verify_tools_on_disk must produce the exact bwrap.ncl error.
    /// Uses a fake bwrap on PATH so the test is deterministic.
    #[test]
    fn verify_tools_on_disk_error_names_bwrap_ncl() {
        let _lock = PATH_MUTEX.lock().unwrap();

        // Put a fake bwrap on PATH so resolve_bwrap_source returns
        // HostFallback instead of erroring with "not found".
        let fake_dir = tempfile::tempdir().unwrap();
        let fake_dir_path = make_fake_executable(fake_dir.path(), "bwrap");
        let fake_bwrap_path = fake_dir_path.join("bwrap");
        let _path_guard = PathGuard::set(fake_dir.path());

        assert_eq!(find_executable_on_path("bwrap"), Some(fake_bwrap_path));

        let store = tempfile::tempdir().unwrap();
        let result = verify_tools_on_disk(store.path());

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().message(),
            "bwrap was not found on disk after building bwrap.ncl. \
             The bootstrap tool build may have failed silently.",
        );
    }

    /// verify_tools_on_disk succeeds when both tools are on disk.
    #[test]
    fn verify_tools_on_disk_succeeds_with_both() {
        let store = tempfile::tempdir().unwrap();
        // Create fake bwrap.
        let bwrap_dir = store.path().join("abc-bwrap").join("bin");
        std::fs::create_dir_all(&bwrap_dir).unwrap();
        let bwrap_bin = bwrap_dir.join("bwrap");
        std::fs::write(&bwrap_bin, "#!/bin/sh\n").unwrap();
        // Create fake busybox.
        let bb_dir = store.path().join("xyz-busybox").join("bin");
        std::fs::create_dir_all(&bb_dir).unwrap();
        let bb_bin = bb_dir.join("busybox");
        std::fs::write(&bb_bin, "#!/bin/sh\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&bwrap_bin, std::fs::Permissions::from_mode(0o755)).unwrap();
            std::fs::set_permissions(&bb_bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        let result = verify_tools_on_disk(store.path());
        assert!(result.is_ok(), "must succeed with both tools: {:?}", result.err());
        let (bwrap_source, busybox_path) = result.unwrap();
        assert!(bwrap_source.is_crunch_built());
        assert!(busybox_path.ends_with("busybox"));
    }

    /// verify_tools_on_disk must error when bwrap exists but busybox doesn't.
    #[test]
    fn verify_tools_on_disk_errors_without_busybox() {
        let store = tempfile::tempdir().unwrap();
        // Create a fake bwrap.
        let bwrap_dir = store.path().join("abc-bwrap").join("bin");
        std::fs::create_dir_all(&bwrap_dir).unwrap();
        let bwrap_bin = bwrap_dir.join("bwrap");
        std::fs::write(&bwrap_bin, "#!/bin/sh\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&bwrap_bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        let result = verify_tools_on_disk(store.path());
        assert!(result.is_err(), "must error when busybox is missing");
        assert_eq!(
            result.unwrap_err().message(),
            "busybox was not found on disk after building busybox.ncl. \
             The bootstrap tool build may have failed silently.",
        );
    }
}
