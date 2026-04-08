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

use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::build_cmd::{
    build_import_paths,
    load_configured_trusted_public_keys,
    load_or_generate_signing_keypair,
    report_build_result,
    run_build,
};
use crate::errors::RunError;

/// Maximum source tree size: 2 GiB.
const MAX_SOURCE_BYTES: u64 = 2 * 1024 * 1024 * 1024;

/// Maximum number of `*-crunch` output directories to scan before giving up.
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
    pub fn parse(s: &str) -> Option<Self> {
        if let Some(rest) = s.strip_prefix("crunch-built:") {
            Some(BwrapSource::CrunchBuilt(PathBuf::from(rest)))
        } else if let Some(rest) = s.strip_prefix("host-fallback:") {
            Some(BwrapSource::HostFallback(PathBuf::from(rest)))
        } else {
            None
        }
    }

    /// True when this stage used a crunch-built bwrap.
    pub fn is_crunch_built(&self) -> bool {
        matches!(self, BwrapSource::CrunchBuilt(_))
    }
}

/// Structured report from a self-build run.
///
/// Captures the facts a proof runner needs to verify that self-hosting
/// works: which binary drove the build, which sandbox tools were
/// selected, and where the output landed.
#[derive(Debug, Clone)]
pub struct SelfBuildReport {
    /// Path to the crunch binary that drove this self-build.
    pub invoking_binary: PathBuf,
    /// How bwrap was resolved (crunch-built vs host fallback).
    pub bwrap_source: BwrapSource,
    /// Path to the busybox binary that the NCL script will use for
    /// `SNIX_BUILD_SANDBOX_SHELL`. `None` if no crunch-built busybox
    /// was found in the output store (falls back to `/bin/sh`).
    pub busybox_path: Option<PathBuf>,
    /// Path to the produced output binary.
    pub output_binary: PathBuf,
}

impl SelfBuildReport {
    /// Format the report as stable `self-build-proof:` lines.
    ///
    /// Each line is `self-build-proof: key=value`. A proof runner can
    /// filter stderr for this prefix and parse the key-value pairs.
    pub fn format_proof_lines(&self) -> String {
        let mut out = String::with_capacity(512);
        out.push_str(&format!(
            "{PROOF_PREFIX} invoking-binary={}\n",
            self.invoking_binary.display(),
        ));
        out.push_str(&format!(
            "{PROOF_PREFIX} bwrap-source={}\n",
            self.bwrap_source,
        ));
        match &self.busybox_path {
            Some(p) => out.push_str(&format!(
                "{PROOF_PREFIX} busybox-path={}\n",
                p.display(),
            )),
            None => out.push_str(&format!(
                "{PROOF_PREFIX} busybox-path=none\n",
            )),
        }
        out.push_str(&format!(
            "{PROOF_PREFIX} output-binary={}\n",
            self.output_binary.display(),
        ));
        out
    }

    /// Parse a report from lines previously produced by
    /// `format_proof_lines`. Returns `None` when any required field
    /// is missing.
    pub fn parse_proof_lines(text: &str) -> Option<Self> {
        let mut invoking_binary: Option<PathBuf> = None;
        let mut bwrap_source: Option<BwrapSource> = None;
        let mut busybox_path: Option<Option<PathBuf>> = None;
        let mut output_binary: Option<PathBuf> = None;

        for line in text.lines() {
            let trimmed = line.trim();
            let rest = match trimmed.strip_prefix(PROOF_PREFIX) {
                Some(r) => r.trim(),
                None => continue,
            };
            if let Some(val) = rest.strip_prefix("invoking-binary=") {
                invoking_binary = Some(PathBuf::from(val));
            } else if let Some(val) = rest.strip_prefix("bwrap-source=") {
                bwrap_source = BwrapSource::parse(val);
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
            invoking_binary: invoking_binary?,
            bwrap_source: bwrap_source?,
            busybox_path: busybox_path?,
            output_binary: output_binary?,
        })
    }
}

/// Stage the crunch source tree into the output store.
///
/// 1. `cargo vendor --locked vendor-deps`
/// 2. `git archive HEAD | tar -x` into staging
/// 3. Copy vendor-deps/ into staging
/// 4. Move staging into `$store_dir/$hash-crunch-src/`
///
/// Returns the store path name (e.g., "abcdef...-crunch-src").
pub fn stage_source(
    src_dir: &Path,
    store_dir: &Path,
) -> Result<String, RunError> {
    let staging = tempfile::tempdir()
        .map_err(|e| RunError::Internal(format!("tmpdir: {e}")))?;
    let stage_root = staging.path().join("crunch-src");
    std::fs::create_dir_all(&stage_root)
        .map_err(|e| RunError::Internal(format!("mkdir staging: {e}")))?;

    // 1. Vendor dependencies.
    eprintln!("  vendoring cargo dependencies...");
    run_cmd(
        Command::new("cargo")
            .args(["vendor", "--locked", "vendor-deps"])
            .current_dir(src_dir),
        "cargo vendor",
    )?;

    // 2. Git-tracked files → staging.
    eprintln!("  exporting git-tracked files...");
    run_cmd(
        Command::new("sh")
            .args(["-c", &format!(
                "cd '{}' && git archive HEAD | tar -x -C '{}'",
                src_dir.display(),
                stage_root.display(),
            )]),
        "git archive | tar extract",
    )?;

    // 3. Copy vendor-deps.
    eprintln!("  copying vendored deps...");
    assert!(
        src_dir.join("vendor-deps").exists(),
        "vendor-deps/ must exist after cargo vendor"
    );
    run_cmd(
        Command::new("cp")
            .args(["-a"])
            .arg(src_dir.join("vendor-deps"))
            .arg(stage_root.join("vendor-deps")),
        "cp vendor-deps",
    )?;

    // Size check.
    let size = dir_size(&stage_root);
    assert!(
        size <= MAX_SOURCE_BYTES,
        "source tree {} MiB exceeds {} MiB limit",
        size / (1024 * 1024),
        MAX_SOURCE_BYTES / (1024 * 1024),
    );
    eprintln!("  source tree: {} MiB", size / (1024 * 1024));

    // 4. Compute a content fingerprint for the store path name.
    //
    // Not a NAR hash — just a quick fingerprint for a unique name.
    // Hash the file listing (paths + sizes) with blake3, then encode
    // the first 20 bytes as nix-base32 (the store path digest format).
    let fingerprint = tree_fingerprint(&stage_root);
    let digest_bytes = data_encoding::HEXLOWER.decode(fingerprint.as_bytes())
        .unwrap_or_else(|_| vec![0u8; 32]);
    let store_hash = nix_compat::nixbase32::encode(&digest_bytes[..20]);
    let store_name = format!("{store_hash}-crunch-src");
    let dest = store_dir.join(&store_name);

    if dest.exists() {
        eprintln!("  source already staged: {}", dest.display());
        return Ok(store_name);
    }

    // Move staging into store. Use cp + rm since rename() doesn't
    // work across filesystems (tmpfs → disk).
    run_cmd(
        Command::new("cp")
            .args(["-a"])
            .arg(&stage_root)
            .arg(&dest),
        "cp to store",
    )?;

    eprintln!("  staged: {}", dest.display());
    Ok(store_name)
}

/// Generate the Nickel derivation for building crunch.
///
/// The source tree is referenced as a plain input path (not a FOD).
pub fn generate_self_build_ncl(
    src_store_path: &str,
    store_prefix: &str,
) -> String {
    format!(
        r#"# Auto-generated by `crunch self-build`.
let crunch = import "lib.ncl" in

let toolchain = crunch.fetchTarball {{
  url = "https://musl.cc/x86_64-linux-musl-native.tgz",
  hash = "sha256-XpcI34j9YwAQj7qw4DpvXqT1CX00vHcUQbAk/do46jw=",
  name = "musl-gcc",
}} in

let gnumake = (import "make.ncl") in
let dash = (import "dash.ncl") in
let binutils = (import "binutils.ncl") in
let musl = (import "musl.ncl") in
let gcc = (import "gcc.ncl") in
let rust = (import "rust.ncl") in
let busybox = (import "busybox.ncl") in
let bwrap = (import "bwrap.ncl") in

{{
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

      CRUNCH_SRC="$NIX_STORE/{src_store_path}"
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
      $BB ln -sf $MUSL/lib/libc.so /lib/ld-musl-x86_64.so.1 2>/dev/null || true

      MUSL_GCC=""
      for d in $NIX_STORE/*-musl-gcc; do
        if [ -f "$d/lib/libgcc_s.so.1" ]; then MUSL_GCC="$d"; break; fi
      done
      if [ -n "$MUSL_GCC" ]; then
        export LD_LIBRARY_PATH="$MUSL_GCC/lib${{LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}}"
      elif [ -f "$GCC/lib/libgcc_s.so.1" ]; then
        export LD_LIBRARY_PATH="$GCC/lib${{LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}}"
      fi

      $BB ln -sf "$DASH/bin/dash" /tmp/tools/sh

      for tool in as ld ld.bfd ar nm objcopy objdump ranlib readelf strip; do
        if [ -x "$BINUTILS/bin/x86_64-linux-musl-$tool" ] && [ ! -e "/tmp/tools/$tool" ]; then
          $BB ln -sf "$BINUTILS/bin/x86_64-linux-musl-$tool" /tmp/tools/$tool
        fi
      done

      # Find crunch-built bwrap for PATH.
      BWRAP_BIN=""
      for d in $NIX_STORE/*-bwrap; do
        if [ -x "$d/bin/bwrap" ]; then BWRAP_BIN="$d/bin"; break; fi
      done

      # Add bwrap to PATH if available.
      # bwrap is a declared input, so the scheduler builds it before this
      # derivation. The else branch is a defensive guard. bwrap is not
      # needed inside this sandbox (we are compiling crunch, not running
      # builds), but having it on PATH is useful if the verify step ever
      # runs a crunch command that invokes bwrap.
      BWRAP_PATH=""
      if [ -n "$BWRAP_BIN" ]; then
        echo "Using crunch-built bwrap: $BWRAP_BIN"
        BWRAP_PATH="$BWRAP_BIN:"
      else
        echo "WARNING: crunch-built bwrap not found among inputs" >&2
      fi
      export PATH="/tmp/tools:${{BWRAP_PATH}}$RUST/bin:$GCC/bin:$BINUTILS/bin:$MAKE/bin"

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
      if [ -n "$MUSL_GCC" ]; then GCC_LIB="$MUSL_GCC/lib"
      elif [ -f "$GCC/lib/libgcc_s.so" ]; then GCC_LIB="$GCC/lib"
      fi

      $BB mkdir -p .cargo
      cat > .cargo/config.toml << CARGOEOF
[source.crates-io]
replace-with = "vendored-sources"

[source."git+https://github.com/tvlfyi/wu-manber.git"]
git = "https://github.com/tvlfyi/wu-manber.git"
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor-deps"

[build]
target = "x86_64-unknown-linux-musl"

[target.x86_64-unknown-linux-musl]
linker = "gcc"
rustflags = ["-C", "link-arg=-Wl,--allow-multiple-definition", "-C", "link-arg=-L${{GCC_LIB}}"]
CARGOEOF

      export CARGO_HOME=/tmp/cargo-home
      export CARGO_TARGET_DIR=/tmp/cargo-target
      # Use crunch-built busybox as the sandbox shell baked into the binary.
      # Find it from the inputs.
      BUSYBOX_BIN=""
      for d in $NIX_STORE/*-busybox; do
        if [ -x "$d/bin/busybox" ]; then BUSYBOX_BIN="$d/bin/busybox"; break; fi
      done
      if [ -n "$BUSYBOX_BIN" ]; then
        export SNIX_BUILD_SANDBOX_SHELL="$BUSYBOX_BIN"
      else
        export SNIX_BUILD_SANDBOX_SHELL=/bin/sh
      fi

      export CC=gcc
      export AR=ar
      export TARGET_CC=gcc
      export TARGET_AR=ar
      export HOST_CC=gcc

      if [ -n "$GCC_LIB" ]; then
        export LIBRARY_PATH="$GCC_LIB${{LIBRARY_PATH:+:$LIBRARY_PATH}}"
      fi

      echo "=== Building crunch ==="
      cargo build --release --locked -j 4 2>&1 || exit 1

      echo "=== Installing ==="
      mkdir -p $out/bin
      cp /tmp/cargo-target/x86_64-unknown-linux-musl/release/crunch $out/bin/

      echo "=== Verify ==="
      ls -la $out/bin/crunch
      file $out/bin/crunch 2>/dev/null || true
      $out/bin/crunch --version 2>/dev/null || $out/bin/crunch --help 2>&1 | head -3
    "%,
  ],
  inputs = [
    toolchain, gnumake, dash, binutils, musl, gcc, rust,
    busybox, bwrap,
    "{store_prefix}/{src_store_path}",
  ],
}} | crunch.Derivation
"#
    )
}

/// Verify the self-built binary by running `--help`.
pub fn verify_binary(binary_path: &Path) -> Result<(), RunError> {
    eprintln!("verifying {}...", binary_path.display());

    let out = Command::new(binary_path)
        .arg("--help")
        .output()
        .map_err(|e| RunError::Internal(format!(
            "failed to run self-built binary: {e}"
        )))?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(RunError::Build(format!(
            "self-built binary exited {} on --help:\n{stderr}",
            out.status.code().unwrap_or(-1),
        )));
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("crunch"),
        "self-built binary --help doesn't mention 'crunch'"
    );

    eprintln!("  binary OK");
    Ok(())
}

// ── helpers ────────────────────────────────────────────────────────────

/// Run a command, returning an error with stderr on failure.
fn run_cmd(cmd: &mut Command, label: &str) -> Result<(), RunError> {
    let out = cmd.output().map_err(|e| {
        RunError::Internal(format!("failed to run `{label}`: {e}"))
    })?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(RunError::Internal(format!(
            "{label} failed (exit {}):\n{stderr}",
            out.status.code().unwrap_or(-1),
        )));
    }

    Ok(())
}

/// Quick blake3 fingerprint of a directory tree (paths + sizes).
///
/// Not a NAR hash — just enough to get a unique store path name.
/// We created this tree ourselves, so integrity verification against
/// a known hash is pointless.
fn tree_fingerprint(dir: &Path) -> String {
    use std::io::Write;

    let mut hasher = blake3::Hasher::new();
    let mut entries: Vec<PathBuf> = Vec::new();

    collect_paths(dir, dir, &mut entries);
    entries.sort();

    for entry in &entries {
        let meta = std::fs::symlink_metadata(entry).ok();
        let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
        let rel = entry.strip_prefix(dir).unwrap_or(entry);
        let _ = write!(hasher, "{}:{}\n", rel.display(), size);
    }

    let hash = hasher.finalize();
    hash.to_hex().to_string()
}

fn collect_paths(base: &Path, dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        out.push(path.clone());
        if path.is_dir() && !path.is_symlink() {
            collect_paths(base, &path, out);
        }
    }
}

/// Total size of a directory tree in bytes.
fn dir_size(dir: &Path) -> u64 {
    let mut total: u64 = 0;
    let mut paths: Vec<PathBuf> = Vec::new();
    collect_paths(dir, dir, &mut paths);
    for p in &paths {
        if let Ok(meta) = std::fs::symlink_metadata(p) {
            total = total.saturating_add(meta.len());
        }
    }
    total
}

/// Locate the best bwrap binary for the self-build pipeline.
///
/// Preference order:
/// 1. Crunch-built bwrap in `output_dir` (from a prior self-build)
/// 2. Any bwrap on the host PATH (first bootstrap)
///
/// Returns `Some(dir)` when a crunch-built bwrap was found — the caller
/// must prepend it to PATH so `Command::new("bwrap")` picks it up.
/// Returns `Ok(None)` when falling back to an external bwrap on PATH.
/// Returns `Err` when no bwrap exists anywhere.
/// Resolve bwrap and return a typed source indicator.
///
/// Returns `CrunchBuilt(dir)` when a crunch-built bwrap was found in
/// the output store. The caller should prepend `dir` to PATH.
/// Returns `HostFallback(path)` when falling back to an external bwrap.
/// Returns `Err` when no bwrap exists anywhere.
fn resolve_bwrap_source(output_dir: &Path) -> Result<BwrapSource, RunError> {
    // 1. Prefer crunch-built bwrap from the output store.
    if let Some(bwrap_dir) = find_crunch_bwrap(output_dir) {
        eprintln!("  bwrap: {} (crunch-built)", bwrap_dir.display());
        return Ok(BwrapSource::CrunchBuilt(bwrap_dir));
    }

    // 2. Fall back to host PATH.
    match find_executable_on_path("bwrap") {
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
pub fn invalidate_crunch_outputs(
    output_dir: &Path,
) -> Result<u32, RunError> {
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
    path.metadata()
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
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
        .map(|p| std::env::split_paths(&p)
            .filter(|d| !d.as_os_str().is_empty())
            .collect())
        .unwrap_or_default();
    let mut dirs = vec![dir.to_path_buf()];
    dirs.extend(current);
    let new_path = std::env::join_paths(&dirs)
        .map_err(|e| RunError::Internal(format!("join_paths: {e}")))?;
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
fn verify_tools_on_disk(output_dir: &Path) -> Result<(BwrapSource, PathBuf), RunError> {
    let bwrap_source = resolve_bwrap_source(output_dir)?;
    if !bwrap_source.is_crunch_built() {
        return Err(RunError::Internal(
            "bwrap was not found on disk after building bwrap.ncl. \
             The bootstrap tool build may have failed silently."
                .to_string(),
        ));
    }
    let busybox_path = find_crunch_busybox(output_dir)
        .ok_or_else(|| RunError::Internal(
            "busybox was not found on disk after building busybox.ncl. \
             The bootstrap tool build may have failed silently."
                .to_string(),
        ))?;
    Ok((bwrap_source, busybox_path))
}

/// Build a single bootstrap tool (.ncl file) as a root derivation.
///
/// This exports the tool's output to the `--store` directory on disk,
/// making it available for subsequent self-build stages to discover
/// via `find_crunch_bwrap` / `find_crunch_busybox`.
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
) -> Result<(), RunError> {
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
        keypair: keypair.clone(),
        trusted_keys: trusted_keys.to_vec(),
        trust_unsigned,
    };

    let result = run_build(&config)?;
    report_build_result(&config, &result, false)?;
    Ok(())
}

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
) -> Result<SelfBuildReport, RunError> {
    eprintln!("=== crunch self-build ===");

    let invoking_binary = std::env::current_exe()
        .unwrap_or_else(|_| PathBuf::from("crunch"));
    eprintln!("  invoking binary: {}", invoking_binary.display());

    // Initial bwrap resolve — may use host fallback on first-ever build.
    let initial_bwrap = resolve_bwrap_source(output_dir)?;
    if let BwrapSource::CrunchBuilt(ref dir) = initial_bwrap {
        prepend_to_path(dir)?;
    }

    let src_dir = find_source_dir()?;
    eprintln!("source: {}", src_dir.display());

    let bootstrap_dir = src_dir.join("bootstrap");
    if !bootstrap_dir.exists() {
        return Err(RunError::Internal(format!(
            "bootstrap/ directory not found at {}",
            bootstrap_dir.display(),
        )));
    }

    eprintln!("\n[1/{SELF_BUILD_STEP_COUNT}] Staging source...");
    let store_name = stage_source(&src_dir, output_dir)?;

    // Set up import paths and signing keys (shared by all builds).
    let lib_dir = src_dir.join("lib");
    let import_paths: Vec<std::ffi::OsString> = {
        let mut paths = build_import_paths(&[lib_dir])?;
        paths.push(bootstrap_dir.clone().into());
        paths
    };

    let self_build_keypair = load_or_generate_signing_keypair(signing_key_path, state_dir)?;
    let configured_trusted_keys =
        load_configured_trusted_public_keys(trusted_public_keys, state_dir)?;
    let self_build_trusted =
        crunch_build::build_trusted_keys(&self_build_keypair, configured_trusted_keys.as_deref());

    // ── Step 2: Build bootstrap tools as separate roots ──────────
    //
    // bwrap and busybox are built as their own root derivations so
    // their outputs get exported to the --store directory on disk.
    // Without this, they only exist in castore/PathInfo (intermediates
    // are not exported). Exporting them lets the next self-build stage
    // find crunch-built tools via find_crunch_bwrap/find_crunch_busybox.

    eprintln!("\n[2/{SELF_BUILD_STEP_COUNT}] Building bootstrap tools...");
    validate_bootstrap_tools(&bootstrap_dir)?;
    for tool_name in REQUIRED_BOOTSTRAP_TOOLS {
        let tool_path = bootstrap_dir.join(tool_name);
        eprintln!("  building {tool_name}...");
        build_bootstrap_tool(
            &tool_path,
            &import_paths,
            output_dir,
            state_dir,
            store_dir,
            verbose,
            max_jobs,
            no_substitute,
            &self_build_keypair,
            &self_build_trusted,
            trust_unsigned,
        )?;
    }

    // Re-resolve bwrap and busybox now that they are on disk.
    let (bwrap_source, busybox_path) = verify_tools_on_disk(output_dir)?;
    if let BwrapSource::CrunchBuilt(ref dir) = bwrap_source {
        prepend_to_path(dir)?;
    }
    let busybox_path = Some(busybox_path);

    // ── Step 3: Build crunch ────────────────────────────────────

    eprintln!("\n[3/{SELF_BUILD_STEP_COUNT}] Building crunch...");
    let ncl_content = generate_self_build_ncl(&store_name, store_dir);

    let tmp_dir = tempfile::tempdir()
        .map_err(|e| RunError::Internal(format!("tmpdir: {e}")))?;
    let ncl_path = tmp_dir.path().join("self-build.ncl");
    std::fs::write(&ncl_path, &ncl_content)
        .map_err(|e| RunError::Internal(format!("writing ncl: {e}")))?;

    let config = crunch_pipeline::BuildConfig {
        file: ncl_path.clone(),
        import_paths,
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
        keypair: self_build_keypair,
        trusted_keys: self_build_trusted,
        trust_unsigned,
    };

    let result = run_build(&config)?;
    report_build_result(&config, &result, false)?;

    // ── Step 4: Verify ──────────────────────────────────────────

    let output_binary = if !no_verify {
        eprintln!("\n[4/{SELF_BUILD_STEP_COUNT}] Verifying output...");
        let crunch_bin = find_self_built_binary(output_dir)?;
        verify_binary(&crunch_bin)?;
        crunch_bin
    } else {
        eprintln!("\n[4/{SELF_BUILD_STEP_COUNT}] Verification skipped (--no-verify).");
        find_self_built_binary(output_dir)?
    };

    let report = SelfBuildReport {
        invoking_binary,
        bwrap_source,
        busybox_path,
        output_binary,
    };

    // Emit stable proof markers.
    eprint!("{}", report.format_proof_lines());

    eprintln!("\n=== self-build complete ===");
    Ok(report)
}

fn find_source_dir() -> Result<PathBuf, RunError> {
    let cwd = std::env::current_dir()
        .map_err(|e| RunError::Internal(format!("cwd: {e}")))?;

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
        "could not locate crunch source directory.\nRun `crunch self-build` from the crunch repo root."
            .to_string(),
    ))
}

fn find_self_built_binary(output_dir: &Path) -> Result<PathBuf, RunError> {
    let entries = std::fs::read_dir(output_dir).map_err(|e| {
        RunError::Internal(format!(
            "reading output dir {}: {e}",
            output_dir.display(),
        ))
    })?;

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.ends_with("-crunch") {
            let binary = entry.path().join("bin").join("crunch");
            if binary.exists() {
                return Ok(binary);
            }
        }
    }

    Err(RunError::Internal(format!(
        "could not find self-built crunch binary in {}",
        output_dir.display(),
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_ncl_has_source_path() {
        let ncl = generate_self_build_ncl("abc123-crunch-src", "/nix/store");
        assert!(ncl.contains("/nix/store/abc123-crunch-src"));
        assert!(ncl.contains("crunch.Derivation"));
    }

    #[test]
    fn generate_ncl_source_is_plain_input() {
        // The source tree should be a string input (source path),
        // not a fetchTarball FOD.
        let ncl = generate_self_build_ncl("abc123-crunch-src", "/nix/store");
        assert!(!ncl.contains("crunch-src\",\n  hash"));
        assert!(ncl.contains("\"/nix/store/abc123-crunch-src\""));
    }

    #[test]
    fn generate_ncl_has_all_bootstrap_deps() {
        let ncl = generate_self_build_ncl("x", "/nix/store");
        assert!(ncl.contains("import \"make.ncl\""));
        assert!(ncl.contains("import \"dash.ncl\""));
        assert!(ncl.contains("import \"binutils.ncl\""));
        assert!(ncl.contains("import \"musl.ncl\""));
        assert!(ncl.contains("import \"gcc.ncl\""));
        assert!(ncl.contains("import \"rust.ncl\""));
        assert!(ncl.contains("import \"busybox.ncl\""));
        assert!(ncl.contains("import \"bwrap.ncl\""));
    }

    #[test]
    fn generate_ncl_has_build_essentials() {
        let ncl = generate_self_build_ncl("x", "/nix/store");
        assert!(ncl.contains("cargo build"));
        assert!(ncl.contains("--release"));
        assert!(ncl.contains("$out/bin/crunch"));
        assert!(ncl.contains("SNIX_BUILD_SANDBOX_SHELL"));
    }

    #[test]
    fn generate_ncl_uses_nix_store_env_var() {
        let ncl = generate_self_build_ncl("x", "/nix/store");
        // Shell globs must use $NIX_STORE, not hardcoded /nix/store.
        assert!(ncl.contains("$NIX_STORE/*-gcc"));
        assert!(ncl.contains("$NIX_STORE/*-rust"));
        assert!(ncl.contains("$NIX_STORE/*-musl-gcc"));
        assert!(ncl.contains("$NIX_STORE/*-busybox"));
        // No hardcoded /nix/store in shell globs.
        assert!(!ncl.contains("for d in /nix/store/"));
    }

    #[test]
    fn generate_ncl_respects_store_prefix() {
        let ncl = generate_self_build_ncl("abc-src", "/crunch/store");
        // The input path in the Nickel record should use the prefix.
        assert!(ncl.contains("\"/crunch/store/abc-src\""));
        assert!(!ncl.contains("/nix/store/abc-src"));
    }

    #[test]
    fn tree_fingerprint_deterministic() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.txt"), "hello").unwrap();
        std::fs::create_dir_all(dir.path().join("sub")).unwrap();
        std::fs::write(dir.path().join("sub/b.txt"), "world").unwrap();

        let h1 = tree_fingerprint(dir.path());
        let h2 = tree_fingerprint(dir.path());
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64); // blake3 hex
    }

    #[test]
    fn tree_fingerprint_changes_with_content() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.txt"), "hello").unwrap();
        let h1 = tree_fingerprint(dir.path());

        // Different file content → different size → different hash.
        std::fs::write(dir.path().join("a.txt"), "hello world").unwrap();
        let h2 = tree_fingerprint(dir.path());
        assert_ne!(h1, h2);
    }

    #[test]
    fn run_cmd_reports_failure() {
        let err = run_cmd(
            Command::new("false").arg(""),
            "test-false",
        ).unwrap_err();
        assert!(err.message().contains("test-false"));
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
    fn generate_ncl_has_bwrap_warning_branch() {
        let ncl = generate_self_build_ncl("x", "/nix/store");
        assert!(
            ncl.contains("crunch-built bwrap not found"),
            "NCL must have a warning for missing bwrap",
        );
        assert!(
            ncl.contains("Using crunch-built bwrap"),
            "NCL must have a success message for found bwrap",
        );
    }

    #[test]
    fn generate_ncl_discovers_bwrap_before_path_export() {
        let ncl = generate_self_build_ncl("x", "/nix/store");
        let discover_pos = ncl.find("$NIX_STORE/*-bwrap")
            .expect("bwrap discovery loop missing");
        let path_export_pos = ncl.find("${BWRAP_PATH}")
            .expect("BWRAP_PATH in PATH export missing");
        assert!(
            discover_pos < path_export_pos,
            "bwrap discovery (pos {discover_pos}) must come before \
             PATH export (pos {path_export_pos})",
        );
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
            std::fs::set_permissions(&bwrap_bin,
                std::fs::Permissions::from_mode(0o755)).unwrap();
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
            std::fs::set_permissions(&bwrap_bin,
                std::fs::Permissions::from_mode(0o644)).unwrap();
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
            std::fs::set_permissions(&bin,
                std::fs::Permissions::from_mode(0o755)).unwrap();
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
            std::fs::set_permissions(&bwrap_bin,
                std::fs::Permissions::from_mode(0o755)).unwrap();
        }

        let result = resolve_bwrap_source(store.path()).unwrap();
        assert!(
            result.is_crunch_built(),
            "should prefer crunch-built bwrap",
        );
        match result {
            BwrapSource::CrunchBuilt(dir) => assert_eq!(dir, bwrap_dir),
            _ => panic!("expected CrunchBuilt"),
        }
    }

    #[test]
    fn resolve_bwrap_source_falls_back_to_path() {
        let store = tempfile::tempdir().unwrap();
        let result = resolve_bwrap_source(store.path());
        match find_executable_on_path("bwrap") {
            Some(host_path) => {
                let src = result.unwrap();
                assert!(!src.is_crunch_built(), "should be host fallback");
                match src {
                    BwrapSource::HostFallback(p) => assert_eq!(p, host_path),
                    _ => panic!("expected HostFallback"),
                }
            }
            None => {
                assert!(result.is_err(), "should error when no bwrap");
                let msg = result.unwrap_err().message().to_string();
                assert!(msg.contains("not found"), "error: {msg}");
            }
        }
    }

    #[test]
    fn find_executable_on_path_returns_none_for_nonexistent() {
        assert!(
            find_executable_on_path("this-binary-does-not-exist-crunch-test").is_none()
        );
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
            std::fs::set_permissions(&f,
                std::fs::Permissions::from_mode(0o644)).unwrap();
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
            std::fs::set_permissions(&bin,
                std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        dir.to_path_buf()
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
            std::fs::set_permissions(&bin,
                std::fs::Permissions::from_mode(0o644)).unwrap();
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

        assert!(
            !dirs.is_empty(),
            "PATH should not be empty after prepend",
        );
        assert_eq!(
            dirs[0], dir.path().to_path_buf(),
            "prepended dir should be first in PATH",
        );
        // No trailing empty component (the old manual concat bug).
        assert!(
            dirs.iter().all(|d| !d.as_os_str().is_empty()),
            "PATH should have no empty components: {dirs:?}",
        );
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

    // ── SelfBuildReport tests ────────────────────────────────────

    #[test]
    fn report_format_roundtrip_with_busybox() {
        let report = SelfBuildReport {
            invoking_binary: PathBuf::from("/tmp/checkout/target/debug/crunch"),
            bwrap_source: BwrapSource::CrunchBuilt(PathBuf::from("/tmp/store/abc-bwrap/bin")),
            busybox_path: Some(PathBuf::from("/tmp/store/xyz-busybox/bin/busybox")),
            output_binary: PathBuf::from("/tmp/store/def-crunch/bin/crunch"),
        };
        let lines = report.format_proof_lines();

        // Each line starts with the prefix.
        for line in lines.lines() {
            assert!(line.starts_with(PROOF_PREFIX), "bad line: {line}");
        }

        let parsed = SelfBuildReport::parse_proof_lines(&lines)
            .expect("should parse back");
        assert_eq!(parsed.invoking_binary, report.invoking_binary);
        assert_eq!(parsed.bwrap_source, report.bwrap_source);
        assert_eq!(parsed.busybox_path, report.busybox_path);
        assert_eq!(parsed.output_binary, report.output_binary);
    }

    #[test]
    fn report_format_roundtrip_without_busybox() {
        let report = SelfBuildReport {
            invoking_binary: PathBuf::from("/usr/bin/crunch"),
            bwrap_source: BwrapSource::HostFallback(PathBuf::from("/usr/bin/bwrap")),
            busybox_path: None,
            output_binary: PathBuf::from("/tmp/store/out-crunch/bin/crunch"),
        };
        let lines = report.format_proof_lines();
        assert!(lines.contains("busybox-path=none"));

        let parsed = SelfBuildReport::parse_proof_lines(&lines)
            .expect("should parse back");
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
            "{PROOF_PREFIX} invoking-binary=/bin/crunch\n\
             {PROOF_PREFIX} bwrap-source=host-fallback:/usr/bin/bwrap\n"
        );
        // Missing busybox-path and output-binary.
        assert!(SelfBuildReport::parse_proof_lines(&partial).is_none());
    }

    #[test]
    fn report_parse_ignores_non_proof_lines() {
        let mixed = format!(
            "some random log line\n\
             {PROOF_PREFIX} invoking-binary=/bin/crunch\n\
             another log line\n\
             {PROOF_PREFIX} bwrap-source=crunch-built:/store/x-bwrap/bin\n\
             {PROOF_PREFIX} busybox-path=/store/y-busybox/bin/busybox\n\
             {PROOF_PREFIX} output-binary=/store/z-crunch/bin/crunch\n"
        );
        let parsed = SelfBuildReport::parse_proof_lines(&mixed)
            .expect("should parse despite noise");
        assert_eq!(parsed.invoking_binary, PathBuf::from("/bin/crunch"));
        assert!(parsed.bwrap_source.is_crunch_built());
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
            std::fs::set_permissions(&bb,
                std::fs::Permissions::from_mode(0o755)).unwrap();
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
            std::fs::set_permissions(&bb,
                std::fs::Permissions::from_mode(0o755)).unwrap();
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
        assert!(
            REQUIRED_BOOTSTRAP_TOOLS.contains(&"bwrap.ncl"),
            "REQUIRED_BOOTSTRAP_TOOLS must include bwrap.ncl",
        );
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
        assert_eq!(
            SELF_BUILD_STEP_COUNT,
            4,
            "SELF_BUILD_STEP_COUNT changed; update this test and AGENTS.md",
        );
    }

    /// validate_bootstrap_tools must error when a required file is missing.
    #[test]
    fn validate_bootstrap_tools_errors_on_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        // Empty dir — no .ncl files exist.
        let result = validate_bootstrap_tools(dir.path());
        assert!(result.is_err(), "must error when bootstrap tools are missing");
        let msg = result.unwrap_err().message().to_string();
        assert!(
            msg.contains("not found"),
            "error must mention missing file: {msg}",
        );
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
        std::fs::write(
            dir.path().join(REQUIRED_BOOTSTRAP_TOOLS[0]),
            "# placeholder",
        ).unwrap();
        let result = validate_bootstrap_tools(dir.path());
        assert!(
            result.is_err(),
            "must error when only one bootstrap tool exists",
        );
    }

    /// verify_tools_on_disk must error when the store is empty and no
    /// bwrap is on PATH.  Controls PATH to make the test deterministic.
    #[test]
    fn verify_tools_on_disk_errors_on_empty_store() {
        let _lock = PATH_MUTEX.lock().unwrap();
        let orig = std::env::var_os("PATH");

        // Empty PATH so resolve_bwrap_source finds nothing.
        unsafe { std::env::set_var("PATH", "") };

        let store = tempfile::tempdir().unwrap();
        let result = verify_tools_on_disk(store.path());

        match &orig {
            Some(p) => unsafe { std::env::set_var("PATH", p) },
            None => unsafe { std::env::remove_var("PATH") },
        }

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
        let orig = std::env::var_os("PATH");

        // Put a fake bwrap on PATH so resolve_bwrap_source returns
        // HostFallback instead of erroring with "not found".
        let fake_dir = tempfile::tempdir().unwrap();
        make_fake_executable(fake_dir.path(), "bwrap");
        unsafe { std::env::set_var("PATH", fake_dir.path()) };

        let store = tempfile::tempdir().unwrap();
        let result = verify_tools_on_disk(store.path());

        match &orig {
            Some(p) => unsafe { std::env::set_var("PATH", p) },
            None => unsafe { std::env::remove_var("PATH") },
        }

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
            std::fs::set_permissions(&bwrap_bin,
                std::fs::Permissions::from_mode(0o755)).unwrap();
            std::fs::set_permissions(&bb_bin,
                std::fs::Permissions::from_mode(0o755)).unwrap();
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
            std::fs::set_permissions(&bwrap_bin,
                std::fs::Permissions::from_mode(0o755)).unwrap();
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
