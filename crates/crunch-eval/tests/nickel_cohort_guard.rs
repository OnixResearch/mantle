//! Cohort guard for the Nickel evaluator.
//!
//! Rejects old, mixed, or floating Nickel dependencies: the embedded crates
//! must be exactly the reviewed cohort, and the CLI Nix input must be the
//! exact pinned upstream revision.

use std::fs;

const EXPECTED_NICKEL_LANG_VERSION: &str = "2.2.0";
const EXPECTED_NICKEL_LANG_CORE_VERSION: &str = "0.18.0";
const EXPECTED_CLI_REVISION: &str = "1320a983e6c3d1e2fb53dd2464b084b4903b1426";
const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

fn repo_file(relative: &str) -> String {
    fs::read_to_string(format!("{ROOT}/{relative}")).expect("cohort guard reads a repo file")
}

/// Pure cohort check over in-memory Cargo.lock, flake.lock, and flake.nix
/// text. Returns a rejection reason on old, mixed, or floating cohorts.
fn check_cohort(cargo_lock: &str, flake_lock: &str, flake_nix: &str) -> Result<(String, String, String), String> {
    let nickel_lang_versions: Vec<String> = lock_versions_for(cargo_lock, "nickel-lang")
        .iter()
        .filter(|(name, _)| name == "nickel-lang")
        .map(|(_, version)| version.clone())
        .collect();
    if nickel_lang_versions != [EXPECTED_NICKEL_LANG_VERSION.to_string()] {
        return Err(format!(
            "nickel-lang cohort is {:?}, expected exactly [{}]",
            nickel_lang_versions, EXPECTED_NICKEL_LANG_VERSION
        ));
    }
    let core_versions: Vec<String> = lock_versions_for(cargo_lock, "nickel-lang")
        .iter()
        .filter(|(name, _)| name == "nickel-lang-core")
        .map(|(_, version)| version.clone())
        .collect();
    if core_versions != [EXPECTED_NICKEL_LANG_CORE_VERSION.to_string()] {
        return Err(format!(
            "nickel-lang-core cohort is {:?}, expected exactly [{}]",
            core_versions, EXPECTED_NICKEL_LANG_CORE_VERSION
        ));
    }
    if !flake_nix.contains(&format!("github:tweag/nickel/{EXPECTED_CLI_REVISION}")) {
        return Err(format!("flake.nix does not pin the Nickel CLI cohort at {EXPECTED_CLI_REVISION}"));
    }
    let locked_rev = flake_input_rev(flake_lock, "nickelCohort", "locked")
        .ok_or_else(|| "flake.lock has no locked nickelCohort revision".to_string())?;
    let original_rev = flake_input_rev(flake_lock, "nickelCohort", "original")
        .ok_or_else(|| "flake.lock has no original nickelCohort revision".to_string())?;
    if locked_rev != EXPECTED_CLI_REVISION || original_rev != EXPECTED_CLI_REVISION {
        return Err(format!("nickelCohort lock drifted: locked={locked_rev} original={original_rev}"));
    }
    Ok((
        EXPECTED_NICKEL_LANG_VERSION.to_string(),
        EXPECTED_NICKEL_LANG_CORE_VERSION.to_string(),
        EXPECTED_CLI_REVISION.to_string(),
    ))
}

/// Extract (name, version) pairs of every package whose name contains
/// `needle`, including exact-name and `-suffix` variants such as
/// `nickel-lang-core`.
fn lock_versions_for(cargo_lock: &str, needle: &str) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    let mut current_name: Option<String> = None;
    for line in cargo_lock.lines() {
        let line = line.trim();
        if let Some(name) = line.strip_prefix("name = \"").and_then(|v| v.strip_suffix('"')) {
            current_name = Some(String::from(name));
        } else if let Some(version) = line.strip_prefix("version = \"").and_then(|v| v.strip_suffix('"')) {
            if let Some(name) = &current_name
                && (name == needle || name.starts_with(&format!("{needle}-")))
            {
                pairs.push((name.clone(), String::from(version)));
            }
            current_name = None;
        }
    }
    pairs.sort();
    pairs
}

/// Extract the `rev` of the `locked` or `original` node of a flake input.
fn flake_input_rev(flake_lock: &str, input: &str, node: &str) -> Option<String> {
    let input_marker = format!("\"{input}\": {{");
    let start = flake_lock.find(&input_marker)? + input_marker.len();
    let rest = &flake_lock[start..];
    let node_marker = format!("\"{node}\": {{");
    let node_start = rest.find(&node_marker)? + node_marker.len();
    let tail = &rest[node_start..];
    let rev_marker = "\"rev\": \"";
    let rev_start = tail.find(rev_marker)? + rev_marker.len();
    let rev_tail = &tail[rev_start..];
    let end = rev_tail.find('"')?;
    Some(String::from(&rev_tail[..end]))
}

#[test]
fn committed_cohort_is_exact() {
    let outcome = check_cohort(&repo_file("Cargo.lock"), &repo_file("flake.lock"), &repo_file("flake.nix"))
        .expect("committed cohort is the reviewed 1.17 cohort");
    let (lang, core, cli) = outcome;
    assert_eq!(lang, "2.2.0");
    assert_eq!(core, "0.18.0");
    assert_eq!(cli, EXPECTED_CLI_REVISION);
}

fn old_lock() -> String {
    String::from(
        "name = \"nickel-lang\"\nversion = \"2.0.0\"\n\
         name = \"nickel-lang-core\"\nversion = \"0.16.1\"\n",
    )
}

fn good_lock() -> String {
    String::from(
        "name = \"nickel-lang\"\nversion = \"2.2.0\"\n\
         name = \"nickel-lang-core\"\nversion = \"0.18.0\"\n",
    )
}

fn pinned_nix() -> String {
    String::from("github:tweag/nickel/1320a983e6c3d1e2fb53dd2464b084b4903b1426")
}

#[test]
fn old_embedded_version_is_rejected() {
    let outcome = check_cohort(&old_lock(), "", &pinned_nix());
    let error = outcome.expect_err("old cohort must be rejected");
    assert!(error.contains("nickel-lang cohort"), "{error}");
}

#[test]
fn mixed_core_versions_are_rejected() {
    let lock = format!("{}name = \"nickel-lang-core\"\nversion = \"0.16.1\"\n", good_lock());
    let outcome = check_cohort(&lock, "", &pinned_nix());
    let error = outcome.expect_err("mixed cohort must be rejected");
    assert!(error.contains("nickel-lang-core cohort"), "{error}");
}

#[test]
fn floating_cli_input_is_rejected() {
    let outcome = check_cohort(&good_lock(), "", "nickel = pkgs.nickel;");
    let error = outcome.expect_err("floating CLI must be rejected");
    assert!(error.contains("does not pin"), "{error}");
}

#[test]
fn drifted_cli_lock_is_rejected() {
    let flake_lock =
        "\"nickelCohort\": { \"locked\": { \"rev\": \"aaaaaaaa\" }, \"original\": { \"rev\": \"aaaaaaaa\" } }";
    let outcome = check_cohort(&good_lock(), flake_lock, &pinned_nix());
    let error = outcome.expect_err("drifted lock must be rejected");
    assert!(error.contains("nickelCohort lock drifted"), "{error}");
}
