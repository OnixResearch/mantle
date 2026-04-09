//! Self-hosting proof: stage0 -> stage1 -> stage2.
//!
//! This test is expensive (~30 min) and requires:
//!   - bwrap on PATH
//!   - git, cargo, tar, xz, cp on PATH
//!   - ~4 GiB free disk in /tmp
//!   - Internet access (for initial bootstrap fetch)
//!
//! Run with:
//!   cargo test -p crunch --test self_hosting -- --ignored --nocapture
//!
//! The test:
//! 1. Runs `crunch self-build` (stage0) using the checkout binary
//! 2. Finds the stage1 binary in the output store
//! 3. Invalidates the prior `*-crunch` output
//! 4. Runs `stage1/bin/crunch self-build` (stage2) with a fresh state dir
//! 5. Verifies stage2 produced a working binary
//! 6. Checks that stage2 used crunch-built bwrap (not host fallback)

mod audit_support;

use std::path::Path;
use std::path::PathBuf;

use assert_cmd::Command;
use audit_support::AuditArtifact;
use audit_support::write_command_audit;

/// Check prerequisites for the proof.
fn can_self_build() -> bool {
    // Need bwrap, git, cargo, tar on PATH.
    let tools = ["bwrap", "git", "cargo", "tar"];
    for tool in tools {
        if std::process::Command::new(tool).arg("--version").output().is_err() {
            eprintln!("SKIP: {tool} not on PATH");
            return false;
        }
    }
    // Need the crunch source tree (Cargo.toml + bootstrap/ in cwd or parents).
    let cwd = std::env::current_dir().unwrap();
    if !cwd.join("Cargo.toml").exists() || !cwd.join("bootstrap").exists() {
        eprintln!("SKIP: not in crunch source tree");
        return false;
    }
    true
}

/// Find the crunch binary in an output store (`*-crunch/bin/crunch`).
fn find_crunch_binary(store: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(store).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.ends_with("-crunch") {
            let binary = entry.path().join("bin").join("crunch");
            if binary.exists() {
                return Some(binary);
            }
        }
    }
    None
}

/// Remove all `*-crunch` output directories from a store.
fn remove_crunch_outputs(store: &Path) -> u32 {
    let mut removed: u32 = 0;
    let entries = match std::fs::read_dir(store) {
        Ok(e) => e,
        Err(_) => return 0,
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.ends_with("-crunch") {
            if std::fs::remove_dir_all(entry.path()).is_ok() {
                removed += 1;
            }
        }
    }
    removed
}

/// Scan store for `*-bwrap/bin/bwrap` and return the path if found.
fn find_bwrap_on_disk(store: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(store).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.ends_with("-bwrap") {
            let bin = entry.path().join("bin").join("bwrap");
            if bin.exists() {
                return Some(bin);
            }
        }
    }
    None
}

/// Scan store for `*-busybox/bin/busybox` and return the path if found.
fn find_busybox_on_disk(store: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(store).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.ends_with("-busybox") {
            let bin = entry.path().join("bin").join("busybox");
            if bin.exists() {
                return Some(bin);
            }
        }
    }
    None
}

/// Check a file is executable (unix).
#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata().map(|m| m.permissions().mode() & 0o111 != 0).unwrap_or(false)
}

/// Parse proof lines from stderr.
fn extract_proof_field<'a>(stderr: &'a str, key: &str) -> Option<&'a str> {
    let prefix = format!("self-build-proof: {key}=");
    for line in stderr.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(&prefix) {
            return Some(rest);
        }
    }
    None
}

#[test]
#[ignore]
fn self_hosting_stage0_stage1_stage2() {
    if !can_self_build() {
        eprintln!("SKIP: prerequisites not met");
        return;
    }

    let proof_dir = tempfile::tempdir().expect("tempdir for proof");
    let store = proof_dir.path().join("store");
    std::fs::create_dir_all(&store).unwrap();

    // The store starts empty. Verify no stale outputs exist.
    assert!(find_bwrap_on_disk(&store).is_none(), "fresh store must not contain bwrap",);
    assert!(find_busybox_on_disk(&store).is_none(), "fresh store must not contain busybox",);
    assert!(find_crunch_binary(&store).is_none(), "fresh store must not contain crunch",);

    // ── Stage 0: checkout binary builds stage1 ──────────────────

    eprintln!("\n=== PROOF: Stage 0 (checkout -> stage1) ===\n");

    let stage0_state = proof_dir.path().join("state0");
    std::fs::create_dir_all(&stage0_state).unwrap();

    let stage0 = Command::cargo_bin("crunch")
        .expect("crunch binary built")
        .arg("--store")
        .arg(&store)
        .arg("--state-dir")
        .arg(&stage0_state)
        .arg("--nix-compat")
        .arg("self-build")
        .arg("--no-substitute")
        .arg("-j")
        .arg("4")
        .output()
        .expect("stage0 should execute");

    let stage0_stderr = String::from_utf8_lossy(&stage0.stderr);
    eprintln!("{stage0_stderr}");

    assert!(
        stage0.status.success(),
        "stage0 failed (exit {}):\n{stage0_stderr}",
        stage0.status.code().unwrap_or(-1),
    );

    // Find the stage1 binary.
    let stage1_binary = find_crunch_binary(&store).expect("stage0 should produce *-crunch/bin/crunch");
    eprintln!("stage1 binary: {}", stage1_binary.display());

    // Verify stage1 runs.
    let stage1_help =
        std::process::Command::new(&stage1_binary).arg("--help").output().expect("stage1 binary should run");
    assert!(stage1_help.status.success(), "stage1 --help failed",);

    // ── Verify bootstrap tools on disk (spec requirement) ─────

    let bwrap_bin = find_bwrap_on_disk(&store).expect("*-bwrap/bin/bwrap must exist on disk after stage0");
    #[cfg(unix)]
    assert!(is_executable(&bwrap_bin), "bwrap binary must be executable: {}", bwrap_bin.display(),);
    eprintln!("bwrap on disk: {}", bwrap_bin.display());

    let busybox_bin = find_busybox_on_disk(&store).expect("*-busybox/bin/busybox must exist on disk after stage0");
    #[cfg(unix)]
    assert!(is_executable(&busybox_bin), "busybox binary must be executable: {}", busybox_bin.display(),);
    eprintln!("busybox on disk: {}", busybox_bin.display());

    let stage0_command = vec![
        "crunch".to_string(),
        "--store".to_string(),
        store.display().to_string(),
        "--state-dir".to_string(),
        stage0_state.display().to_string(),
        "--nix-compat".to_string(),
        "self-build".to_string(),
        "--no-substitute".to_string(),
        "-j".to_string(),
        "4".to_string(),
    ];
    let _stage0_audit = write_command_audit(
        "self-hosting",
        "stage0",
        &std::env::current_dir().unwrap(),
        &stage0_command,
        &stage0,
        &[
            AuditArtifact {
                label: "stage1-binary",
                path: &stage1_binary,
            },
            AuditArtifact {
                label: "bwrap",
                path: &bwrap_bin,
            },
            AuditArtifact {
                label: "busybox",
                path: &busybox_bin,
            },
        ],
        &[
            ("CRUNCH_STATE_DIR", stage0_state.display().to_string()),
            ("CRUNCH_STORE_DIR", store.display().to_string()),
        ],
    )
    .unwrap();

    // ── Prepare for Stage 2 ─────────────────────────────────────

    // The store has the root output (*-crunch) on disk.
    // Intermediate bootstrap deps (bwrap, busybox, gcc, etc.) are in
    // castore/PathInfo but NOT exported to the --store directory
    // (only root outputs get exported to disk).
    //
    // Copy the stage1 binary out before invalidation, since it lives
    // inside the *-crunch directory we're about to remove.

    let stage1_copy = proof_dir.path().join("stage1-crunch");
    std::fs::copy(&stage1_binary, &stage1_copy).expect("copy stage1 binary out of store");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&stage1_copy, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let stage1_binary = stage1_copy;

    // Remove *-crunch so stage2 must rebuild the final binary.
    // Stage2 reuses the castore/PathInfo cache for intermediate deps.
    let removed = remove_crunch_outputs(&store);
    assert!(removed >= 1, "should have removed at least 1 *-crunch dir");
    assert!(find_crunch_binary(&store).is_none(), "crunch output should be gone after invalidation",);

    // Fresh state dir so pathinfo.redb doesn't give a false cache hit
    // on the final crunch output.
    let stage2_state = proof_dir.path().join("state2");
    std::fs::create_dir_all(&stage2_state).unwrap();

    // ── Stage 2: stage1 binary rebuilds crunch ──────────────────

    eprintln!("\n=== PROOF: Stage 2 (stage1 -> stage2) ===\n");

    let stage2 = std::process::Command::new(&stage1_binary)
        .arg("--store")
        .arg(&store)
        .arg("--state-dir")
        .arg(&stage2_state)
        .arg("--nix-compat")
        .arg("self-build")
        .arg("--no-substitute")
        .arg("-j")
        .arg("4")
        .output()
        .expect("stage2 should execute");

    let stage2_stderr = String::from_utf8_lossy(&stage2.stderr);
    eprintln!("{stage2_stderr}");

    assert!(
        stage2.status.success(),
        "stage2 failed (exit {}):\n{stage2_stderr}",
        stage2.status.code().unwrap_or(-1),
    );

    // ── Verify stage2 output ────────────────────────────────────

    let stage2_binary = find_crunch_binary(&store).expect("stage2 should produce *-crunch/bin/crunch");
    eprintln!("stage2 binary: {}", stage2_binary.display());

    // Stage2 binary should run.
    let stage2_help =
        std::process::Command::new(&stage2_binary).arg("--help").output().expect("stage2 binary should run");
    assert!(stage2_help.status.success(), "stage2 --help failed",);
    let stage2_stdout = String::from_utf8_lossy(&stage2_help.stdout);
    assert!(stage2_stdout.contains("crunch"), "stage2 --help should mention crunch",);

    // ── Verify proof markers ────────────────────────────────────

    // Stage2 was driven by stage1 binary, not the checkout binary.
    let s2_invoking = extract_proof_field(&stage2_stderr, "invoking-binary");
    assert!(s2_invoking.is_some(), "stage2 should emit invoking-binary proof line",);
    let s2_invoking_path = PathBuf::from(s2_invoking.unwrap());
    // The invoking binary MUST be the stage1 binary we found earlier.
    // current_exe() may resolve symlinks or return a different
    // representation, so canonicalize both before comparing.
    let stage1_canonical = std::fs::canonicalize(&stage1_binary).unwrap_or_else(|_| stage1_binary.clone());
    let invoking_canonical = std::fs::canonicalize(&s2_invoking_path).unwrap_or_else(|_| s2_invoking_path.clone());
    assert_eq!(
        invoking_canonical,
        stage1_canonical,
        "stage2 invoking binary must be the stage1 binary.\n\
         invoking: {}\n\
         stage1:   {}",
        invoking_canonical.display(),
        stage1_canonical.display(),
    );

    // Stage2 MUST use crunch-built bwrap. The self-build pipeline
    // now exports bwrap and busybox as separate root builds (step 2/4)
    // so they land on disk in the --store directory.
    let s2_bwrap = extract_proof_field(&stage2_stderr, "bwrap-source");
    assert!(s2_bwrap.is_some(), "stage2 should emit bwrap-source proof line",);
    let bwrap_val = s2_bwrap.unwrap();
    assert!(bwrap_val.starts_with("crunch-built:"), "stage2 bwrap must be crunch-built, got: {bwrap_val}",);

    // Stage2 MUST find crunch-built busybox on disk.
    let s2_busybox = extract_proof_field(&stage2_stderr, "busybox-path");
    assert!(s2_busybox.is_some(), "stage2 should emit busybox-path proof line",);
    assert_ne!(s2_busybox.unwrap(), "none", "stage2 must have a crunch-built busybox, not none",);

    // Output binary recorded.
    let s2_output = extract_proof_field(&stage2_stderr, "output-binary");
    assert!(s2_output.is_some(), "stage2 should emit output-binary proof line",);

    eprintln!("\n=== PROOF PASSED ===");
    eprintln!("stage1: {}", stage1_binary.display());
    eprintln!("stage2: {}", stage2_binary.display());
    eprintln!("bwrap:  {bwrap_val}");
    let stage2_command = vec![
        stage1_binary.display().to_string(),
        "--store".to_string(),
        store.display().to_string(),
        "--state-dir".to_string(),
        stage2_state.display().to_string(),
        "--nix-compat".to_string(),
        "self-build".to_string(),
        "--no-substitute".to_string(),
        "-j".to_string(),
        "4".to_string(),
    ];
    let _stage2_audit = write_command_audit(
        "self-hosting",
        "stage2",
        &std::env::current_dir().unwrap(),
        &stage2_command,
        &stage2,
        &[
            AuditArtifact {
                label: "stage1-driver",
                path: &stage1_binary,
            },
            AuditArtifact {
                label: "stage2-binary",
                path: &stage2_binary,
            },
            AuditArtifact {
                label: "store-bwrap",
                path: &bwrap_bin,
            },
            AuditArtifact {
                label: "store-busybox",
                path: &busybox_bin,
            },
        ],
        &[
            ("CRUNCH_STATE_DIR", stage2_state.display().to_string()),
            ("CRUNCH_STORE_DIR", store.display().to_string()),
        ],
    )
    .unwrap();

    eprintln!("busybox: {}", s2_busybox.unwrap());
}
