//! Self-build: crunch builds itself from source.
//!
//! Creates a source tarball (with vendored Cargo deps), computes its
//! NAR hash, generates a temporary Nickel derivation, and delegates
//! to the normal build pipeline. The output is a statically-linked
//! crunch binary compiled inside a bwrap sandbox using only the
//! bootstrap toolchain (from-source GCC + fetched Rust).
//!
//! No Nix runtime is needed. The only host tools required are `git`
//! (for `git archive`) and `cargo` (for `cargo vendor`).

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::errors::RunError;

/// Maximum tarball size: 2 GiB. The crunch source + vendored deps
/// (668 crates) can reach ~800 MiB uncompressed.
const MAX_TARBALL_BYTES: u64 = 2 * 1024 * 1024 * 1024;

/// Create a source tarball with vendored dependencies.
///
/// 1. `cargo vendor --locked vendor-deps` (updates vendor dir)
/// 2. `git archive --prefix=crunch-src/ HEAD` (tracked files)
/// 3. Append vendor-deps/ into the tarball
/// 4. Compress with xz
///
/// Returns the path to the compressed tarball.
pub fn create_source_tarball(
    src_dir: &Path,
    out_dir: &Path,
) -> Result<PathBuf, RunError> {
    let tar_path = out_dir.join("crunch-src.tar");
    let xz_path = out_dir.join("crunch-src.tar.xz");

    // 1. Vendor dependencies.
    eprintln!("  vendoring cargo dependencies...");
    run_cmd(
        Command::new("cargo")
            .args(["vendor", "--locked", "vendor-deps"])
            .current_dir(src_dir),
        "cargo vendor",
    )?;

    // 2. Build a staging directory, then tar it.
    //
    // We stage into crunch-src/ so the tarball has a single top-level
    // directory. The fetch pipeline's strip logic removes this prefix,
    // leaving Cargo.toml etc at the output root.
    eprintln!("  staging source tree...");
    let staging = out_dir.join("crunch-src");
    let _ = std::fs::remove_dir_all(&staging);
    std::fs::create_dir_all(&staging)
        .map_err(|e| RunError::Internal(format!("mkdir staging: {e}")))?;

    // Git-tracked files → staging.
    run_cmd(
        Command::new("sh")
            .args(["-c", &format!(
                "cd '{}' && git archive HEAD | tar -x -C '{}'",
                src_dir.display(),
                staging.display(),
            )]),
        "git archive | tar extract",
    )?;

    // Vendor-deps → staging.
    assert!(
        src_dir.join("vendor-deps").exists(),
        "vendor-deps/ must exist after cargo vendor"
    );
    run_cmd(
        Command::new("cp")
            .args(["-a"])
            .arg(src_dir.join("vendor-deps"))
            .arg(staging.join("vendor-deps")),
        "cp vendor-deps",
    )?;

    // 3. Tar the staging directory.
    eprintln!("  creating tarball...");
    run_cmd(
        Command::new("tar")
            .arg("cf")
            .arg(&tar_path)
            .arg("crunch-src")
            .current_dir(out_dir),
        "tar create",
    )?;

    // Size check.
    let tar_size = std::fs::metadata(&tar_path)
        .map_err(|e| RunError::Internal(format!("stat tarball: {e}")))?
        .len();

    assert!(
        tar_size <= MAX_TARBALL_BYTES,
        "tarball {} MiB exceeds {} MiB limit",
        tar_size / (1024 * 1024),
        MAX_TARBALL_BYTES / (1024 * 1024),
    );

    // 4. Compress with xz.
    eprintln!("  compressing ({} MiB uncompressed)...", tar_size / (1024 * 1024));
    let _ = std::fs::remove_file(&xz_path);

    run_cmd(
        Command::new("xz").args(["-T0"]).arg(&tar_path),
        "xz",
    )?;

    assert!(xz_path.exists(), "xz did not produce output file");

    let xz_size = std::fs::metadata(&xz_path)
        .map_err(|e| RunError::Internal(format!("stat xz: {e}")))?
        .len();
    eprintln!(
        "  tarball: {} ({} MiB)",
        xz_path.display(),
        xz_size / (1024 * 1024),
    );

    Ok(xz_path)
}

/// Compute the sha256 NAR hash of an unpacked tarball.
///
/// Unpacks to a temp dir, ingests into castore, serializes to NAR,
/// hashes. Returns an SRI string like "sha256-base64...".
pub async fn hash_unpacked_tarball(
    tarball_path: &Path,
) -> Result<String, RunError> {
    use snix_castore::blobservice::ObjectStoreBlobService;
    use snix_castore::directoryservice::{RedbDirectoryService, RedbDirectoryServiceConfig};
    use snix_castore::import::fs::ingest_path;
    use snix_store::nar::NarCalculationService;
    use snix_store::nar::SimpleRenderer;

    // Unpack tarball to a temp dir (reuses crunch's own unpack logic).
    let unpack_dir = tempfile::tempdir()
        .map_err(|e| RunError::Internal(format!("creating temp dir: {e}")))?;

    let url_str = format!("file://{}", tarball_path.display());
    let out_path = unpack_dir.path().join("unpacked");

    tokio::task::spawn_blocking({
        let url = url_str.clone();
        let out = out_path.display().to_string();
        move || crunch_build::fetcher::fetch_and_unpack(&url, &out)
    })
    .await
    .map_err(|e| RunError::Internal(format!("spawn_blocking: {e}")))?
    .map_err(|e| RunError::Internal(format!("unpacking tarball: {e}")))?;

    assert!(out_path.exists(), "tarball unpacked but output dir missing");

    // Temporary castore services for hashing.
    let blob_dir = tempfile::tempdir()
        .map_err(|e| RunError::Internal(format!("creating blob temp dir: {e}")))?;

    let blob_service = std::sync::Arc::new(
        ObjectStoreBlobService::new_local(blob_dir.path())
            .map_err(|e| RunError::Internal(format!("blob service: {e}")))?
    );

    let directory_service = RedbDirectoryService::new_temporary(
        "hash".to_string(),
        RedbDirectoryServiceConfig {
            path: None,
            read_only: false,
            cache_size: None,
        },
    )
    .map_err(|e| RunError::Internal(format!("directory service: {e}")))?;

    // Ingest the unpacked tree.
    let node = ingest_path::<_, _, _, &[u8]>(
        blob_service.clone(),
        directory_service.clone(),
        out_path.as_path(),
        None,
    )
    .await
    .map_err(|e| RunError::Internal(format!("ingesting unpacked tarball: {e}")))?;

    // Compute NAR → sha256.
    let nar_renderer = SimpleRenderer::new(blob_service, directory_service);
    let (_nar_size, nar_sha256) = nar_renderer
        .calculate_nar(&node)
        .await
        .map_err(|e| RunError::Internal(format!("NAR hash: {e}")))?;

    let sri = format!(
        "sha256-{}",
        data_encoding::BASE64.encode(nar_sha256.as_slice()),
    );

    eprintln!("  NAR hash: {sri}");
    Ok(sri)
}

/// Generate the Nickel derivation source for building crunch.
///
/// Embeds the tarball URL and hash into the bootstrap template.
pub fn generate_self_build_ncl(
    tarball_url: &str,
    tarball_hash: &str,
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

let crunch_src = crunch.fetchTarball {{
  url = "{tarball_url}",
  hash = "{tarball_hash}",
  name = "crunch-src",
}} in

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
      for d in /nix/store/*-gcc; do
        if [ -x "$d/bin/gcc" ]; then GCC="$d"; break; fi
      done
      BINUTILS=""
      for d in /nix/store/*-binutils; do
        if [ -d "$d/bin" ]; then BINUTILS="$d"; break; fi
      done
      MUSL=""
      for d in /nix/store/*-musl; do
        if [ -d "$d/include" ]; then MUSL="$d"; break; fi
      done
      DASH=""
      for d in /nix/store/*-dash; do
        if [ -x "$d/bin/dash" ]; then DASH="$d"; break; fi
      done
      MAKE=""
      for d in /nix/store/*-gnumake; do
        if [ -x "$d/bin/make" ]; then MAKE="$d"; break; fi
      done
      RUST=""
      for d in /nix/store/*-rust; do
        if [ -x "$d/bin/rustc" ]; then RUST="$d"; break; fi
      done
      CRUNCH_SRC=""
      for d in /nix/store/*-crunch-src; do
        if [ -f "$d/Cargo.toml" ]; then CRUNCH_SRC="$d"; break; fi
      done

      for tool in GCC BINUTILS MUSL DASH MAKE RUST CRUNCH_SRC; do
        eval val=\$$tool
        if [ -z "$val" ]; then
          echo "ERROR: $tool not found" >&2; exit 1
        fi
        echo "$tool=$val"
      done

      $BB mkdir -p /lib 2>/dev/null || true
      $BB ln -sf $MUSL/lib/libc.so /lib/ld-musl-x86_64.so.1 2>/dev/null || true

      MUSL_GCC=""
      for d in /nix/store/*-musl-gcc; do
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

      export PATH="/tmp/tools:$RUST/bin:$GCC/bin:$BINUTILS/bin:$MAKE/bin"

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
      export SNIX_BUILD_SANDBOX_SHELL=/bin/sh
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
  inputs = [toolchain, gnumake, dash, binutils, musl, gcc, rust, crunch_src],
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_ncl_contains_url_and_hash() {
        let ncl = generate_self_build_ncl(
            "file:///tmp/test.tar.xz",
            "sha256-AAAA",
        );
        assert!(ncl.contains("file:///tmp/test.tar.xz"));
        assert!(ncl.contains("sha256-AAAA"));
        assert!(ncl.contains("crunch-src"));
        assert!(ncl.contains("crunch.Derivation"));
    }

    #[test]
    fn generate_ncl_has_all_bootstrap_deps() {
        let ncl = generate_self_build_ncl("file:///x", "sha256-X");
        assert!(ncl.contains("import \"make.ncl\""));
        assert!(ncl.contains("import \"dash.ncl\""));
        assert!(ncl.contains("import \"binutils.ncl\""));
        assert!(ncl.contains("import \"musl.ncl\""));
        assert!(ncl.contains("import \"gcc.ncl\""));
        assert!(ncl.contains("import \"rust.ncl\""));
    }

    #[test]
    fn generate_ncl_has_build_essentials() {
        let ncl = generate_self_build_ncl("file:///x", "sha256-X");
        assert!(ncl.contains("cargo build"));
        assert!(ncl.contains("--release"));
        assert!(ncl.contains("$out/bin/crunch"));
        assert!(ncl.contains("SNIX_BUILD_SANDBOX_SHELL"));
    }

    #[test]
    fn generate_ncl_shell_vars_escaped() {
        // format!() turns {{ → {, so the output has Nickel m%"..."
        // interpolation escapes: ${GCC_LIB} for heredocs (shell expands),
        // ${LD_LIBRARY_PATH...} for shell vars in Nickel multiline strings.
        let ncl = generate_self_build_ncl("file:///x", "sha256-X");
        assert!(ncl.contains("${GCC_LIB}"));
        assert!(ncl.contains("LD_LIBRARY_PATH"));
        // Build script essentials.
        assert!(ncl.contains("CARGO_HOME"));
    }

    #[test]
    fn run_cmd_reports_failure() {
        let err = run_cmd(
            Command::new("false").arg(""),
            "test-false",
        ).unwrap_err();
        assert!(err.message().contains("test-false"));
    }
}
