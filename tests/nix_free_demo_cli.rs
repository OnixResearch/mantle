use assert_cmd::Command;
use serde_json::Value;
use tempfile::TempDir;

const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const SUMMARY_FILE_NAME: &str = "summary.json";
const MISSING_FIXED_POINT: &str = "missing-fixed-point-evidence";
const MISSING_GUARD: &str = "missing-guard-evidence";
const MISSING_NON_CLAIMS: &str = "missing-non-claims";
const SUCCESS_VERDICT: &str = "success";
const DEMO_PROFILE: &str = "source-root-cargo-free-fixed-point";
const CLAIMABLE_TEXT: &str = "Nix-free fixed-point demo: claimable";
const TRANSCRIPT_FILE_NAME: &str = "proof.log";
const RECEIPT_DIGEST_ARG: &str = "receipt:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const ARTIFACT_DIGEST_ARG: &str = "stage2-mantle:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const BAD_RECEIPT_DIGEST_ARG: &str = "receipt:not-a-digest";
const MISSING_TRANSCRIPT: &str = "missing-transcript";
const CONTRADICTORY_STATUS: &str = "contradictory-proof-status";
const OUTPUT_CONFLICT: &str = "output-conflict";

fn mantle_cmd() -> Command {
    Command::cargo_bin("mantle").expect("mantle binary should be built")
}

#[test]
fn nix_free_demo_cli_validates_claimable_bundle_as_json() {
    let temp = TempDir::new().expect("tempdir should be created");
    let summary_path = write_summary(temp.path(), claimable_summary());

    let output = mantle_cmd()
        .args(["--json", "nix-free-demo", "validate", path_str(&summary_path)])
        .assert()
        .success()
        .get_output()
        .clone();
    let report: Value = serde_json::from_slice(&output.stdout).expect("stdout should be JSON");

    assert_eq!(report["schema"], "mantle-nix-free-demo-cli-v1");
    assert_eq!(report["profile"], DEMO_PROFILE);
    assert_eq!(report["verdict"], SUCCESS_VERDICT);
    assert_eq!(report["demo_claimable"], true);
    assert!(report["diagnostics"].as_array().unwrap().is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn nix_free_demo_cli_rejects_missing_fixed_point_without_success_claim() {
    let temp = TempDir::new().expect("tempdir should be created");
    let mut summary = claimable_summary();
    summary["stage2_binary_blake3"] = Value::String(DIGEST_B.to_string());
    let summary_path = write_summary(temp.path(), summary);

    let output = mantle_cmd()
        .args(["nix-free-demo", "validate", path_str(&summary_path)])
        .assert()
        .code(1)
        .get_output()
        .clone();
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");

    assert!(stdout.contains("Nix-free demo bundle: not claimable"));
    assert!(stdout.contains(MISSING_FIXED_POINT));
    assert!(!stdout.contains(CLAIMABLE_TEXT));
    assert!(output.stderr.is_empty());
}

#[test]
fn nix_free_demo_cli_rejects_missing_guard_as_json() {
    let temp = TempDir::new().expect("tempdir should be created");
    let mut summary = claimable_summary();
    let guards = summary["guards"].as_array_mut().expect("guards should be an array");
    guards.retain(|guard| guard["guard"] != "rustup");
    let summary_path = write_summary(temp.path(), summary);

    let output = mantle_cmd()
        .args(["--json", "nix-free-demo", "validate", path_str(&summary_path)])
        .assert()
        .code(1)
        .get_output()
        .clone();
    let report: Value = serde_json::from_slice(&output.stdout).expect("stdout should be JSON");
    let diagnostics = report["diagnostics"].as_array().expect("diagnostics should be an array");

    assert_eq!(report["demo_claimable"], false);
    assert!(diagnostics.iter().any(|diagnostic| diagnostic["code"] == MISSING_GUARD));
    assert!(!String::from_utf8(output.stdout).unwrap().contains(CLAIMABLE_TEXT));
    assert!(output.stderr.is_empty());
}

#[test]
fn nix_free_demo_cli_renders_readme_from_summary() {
    let temp = TempDir::new().expect("tempdir should be created");
    let summary_path = write_summary(temp.path(), claimable_summary());

    let output = mantle_cmd()
        .args(["nix-free-demo", "readme", path_str(&summary_path)])
        .assert()
        .success()
        .get_output()
        .clone();
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");

    assert!(stdout.contains("# Mantle fixed-point demo bundle"));
    assert!(stdout.contains(CLAIMABLE_TEXT));
    assert!(stdout.contains(DIGEST_A));
    assert!(stdout.contains("not release reproducibility"));
    assert!(output.stderr.is_empty());
}

#[test]
fn nix_free_demo_cli_readme_of_unclaimable_summary_renders_and_exits_one() {
    let temp = TempDir::new().expect("tempdir should be created");
    let mut summary = claimable_summary();
    summary["fixed_point_verdict"] = Value::String("blocked".to_string());
    let summary_path = write_summary(temp.path(), summary);

    let output = mantle_cmd()
        .args(["nix-free-demo", "readme", path_str(&summary_path)])
        .assert()
        .code(1)
        .get_output()
        .clone();
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");

    assert!(stdout.contains("# Mantle fixed-point demo bundle"));
    assert!(stdout.contains("Demo claim: not claimable"));
    assert!(stdout.contains(MISSING_FIXED_POINT));
    assert!(!stdout.contains(CLAIMABLE_TEXT));
    assert!(output.stderr.is_empty());
}

#[test]
fn nix_free_demo_cli_keeps_distinct_exit_classes_for_malformed_and_unreadable_summaries() {
    let temp = TempDir::new().expect("tempdir should be created");
    let malformed = temp.path().join("malformed.json");
    std::fs::write(&malformed, "not-json").expect("malformed summary should be written");
    let missing = temp.path().join("missing.json");

    let malformed_output = mantle_cmd()
        .args(["--json", "nix-free-demo", "validate", path_str(&malformed)])
        .assert()
        .code(1)
        .get_output()
        .clone();
    let report: Value = serde_json::from_slice(&malformed_output.stdout).expect("stdout should be JSON");
    assert_eq!(report["verdict"], "malformed");
    assert_eq!(report["demo_claimable"], false);
    assert_eq!(report["diagnostics"][0]["code"], "malformed-summary");
    assert!(malformed_output.stderr.is_empty());

    let missing_output = mantle_cmd()
        .args(["nix-free-demo", "validate", path_str(&missing)])
        .assert()
        .code(3)
        .get_output()
        .clone();
    assert!(missing_output.stdout.is_empty(), "an unreadable summary must not print a report");
    assert!(String::from_utf8_lossy(&missing_output.stderr).contains("reading Nix-free demo summary"));
}

#[test]
fn nix_free_demo_cli_generates_valid_deterministic_bundle() {
    let temp = TempDir::new().expect("tempdir should be created");
    let transcript = write_transcript(temp.path(), "stage1 ok\nstage2 ok\n");
    let first = temp.path().join("bundle-a");
    let second = temp.path().join("bundle-b");

    let first_report = run_generate(&first, &transcript).success_json();
    let second_report = run_generate(&second, &transcript).success_json();

    assert_eq!(first_report["schema"], "mantle-nix-free-demo-generate-cli-v1");
    assert_eq!(first_report["generated"], true);
    assert_eq!(first_report["demo_claimable"], true);
    assert!(first_report["files"].as_array().unwrap().contains(&Value::String("summary.json".to_string())));
    assert_eq!(second_report["demo_claimable"], true);
    assert_eq!(read_bundle_files(&first), read_bundle_files(&second));

    let validate_output = mantle_cmd()
        .args([
            "--json",
            "nix-free-demo",
            "validate",
            path_str(&first.join("summary.json")),
        ])
        .assert()
        .success()
        .get_output()
        .clone();
    let validate_report: Value = serde_json::from_slice(&validate_output.stdout).expect("stdout should be JSON");
    assert_eq!(validate_report["demo_claimable"], true);
    assert!(validate_output.stderr.is_empty());

    let readme = std::fs::read_to_string(first.join("README.md")).expect("README should be written");
    assert!(readme.contains(CLAIMABLE_TEXT));
    assert!(readme.contains("generated bundle"));
    assert!(readme.contains(TRANSCRIPT_FILE_NAME));
}

#[test]
fn nix_free_demo_cli_generate_rejects_bad_inputs_without_partial_valid_bundle() {
    let temp = TempDir::new().expect("tempdir should be created");
    let transcript = write_transcript(temp.path(), "blocked before binary\n");
    let missing_transcript = temp.path().join("missing.log");
    let missing_out = temp.path().join("missing-transcript");
    assert_generate_failure(&missing_out, &missing_transcript, MISSING_TRANSCRIPT);
    assert!(!missing_out.exists(), "a rejected request must not create its output");

    let blocked_out = temp.path().join("blocked-with-digests");
    let blocked = generate_args(&blocked_out, &transcript).with_proof_status("blocked").without_non_claims(false);
    assert_generate_args_failure(blocked, CONTRADICTORY_STATUS);
    assert!(!blocked_out.exists(), "a rejected request must not create its output");

    let no_claims_out = temp.path().join("blocked-no-claims");
    let no_claims = generate_args(&no_claims_out, &transcript)
        .with_proof_status("blocked")
        .without_stage_digests()
        .without_non_claims(true);
    assert_generate_args_failure(no_claims, MISSING_NON_CLAIMS);
    assert!(!no_claims_out.exists(), "a rejected request must not create its output");

    let bad_digest_out = temp.path().join("bad-digest");
    let bad_digest = generate_args(&bad_digest_out, &transcript).with_bad_receipt_digest();
    assert_generate_args_failure(bad_digest, "receipt-digest");
    assert!(!bad_digest_out.exists(), "a rejected request must not create its output");

    let conflict_out = temp.path().join("conflict");
    std::fs::create_dir(&conflict_out).expect("conflict dir should be created");
    std::fs::write(conflict_out.join("unrelated.txt"), "keep me").expect("conflict file should be written");
    assert_generate_failure(&conflict_out, &transcript, OUTPUT_CONFLICT);
    assert_eq!(dir_entry_names(&conflict_out), vec!["unrelated.txt".to_string()]);
}

#[test]
fn nix_free_demo_cli_generate_rejects_unreadable_inputs_first_without_writing() {
    let temp = TempDir::new().expect("tempdir should be created");
    let transcript = write_transcript(temp.path(), "stage1 ok\nstage2 ok\n");

    let directory_transcript = temp.path().join("transcript-dir");
    std::fs::create_dir(&directory_transcript).expect("transcript dir should be created");
    let directory_out = temp.path().join("directory-transcript");
    let report = generate_rejection(&generate_args(&directory_out, &directory_transcript));
    assert_eq!(report["diagnostics"][0]["code"], MISSING_TRANSCRIPT);
    let message = report["diagnostics"][0]["message"].as_str().expect("message should be a string");
    assert!(message.starts_with(path_str(&directory_transcript)), "{message}");
    assert!(!directory_out.exists(), "a rejected request must not create its output");

    let file_out = temp.path().join("out-is-a-file");
    std::fs::write(&file_out, "keep me").expect("output file should be written");
    let report = generate_rejection(&generate_args(&file_out, &transcript));
    assert_eq!(report["diagnostics"][0]["code"], OUTPUT_CONFLICT);
    let message = report["diagnostics"][0]["message"].as_str().expect("message should be a string");
    assert!(message.starts_with("read output dir: "), "{message}");
    assert_eq!(std::fs::read_to_string(&file_out).expect("output file should stay readable"), "keep me");

    let occupied_out = temp.path().join("occupied");
    std::fs::create_dir(&occupied_out).expect("occupied dir should be created");
    std::fs::write(occupied_out.join("unrelated.txt"), "keep me").expect("occupied file should be written");
    let occupied = generate_args(&occupied_out, &transcript).with_arg("--guard", "malformed");
    assert_eq!(generate_rejection(&occupied)["diagnostics"][0]["code"], OUTPUT_CONFLICT);
    assert_eq!(dir_entry_names(&occupied_out), vec!["unrelated.txt".to_string()]);

    let missing_out = temp.path().join("missing-before-guard");
    let missing = generate_args(&missing_out, &temp.path().join("missing.log")).with_arg("--guard", "malformed");
    assert_eq!(generate_rejection(&missing)["diagnostics"][0]["code"], MISSING_TRANSCRIPT);
    assert!(!missing_out.exists(), "a rejected request must not create its output");
}

#[test]
fn nix_free_demo_cli_generate_reports_exactly_the_files_it_wrote() {
    let temp = TempDir::new().expect("tempdir should be created");
    let transcript = write_transcript(temp.path(), "stage1 ok\nstage2 ok\n");
    let out = temp.path().join("bundle");

    let report = run_generate(&out, &transcript).success_json();

    let reported = report["files"]
        .as_array()
        .expect("files should be an array")
        .iter()
        .map(|file| file.as_str().expect("file should be a string").to_string())
        .collect::<Vec<_>>();
    assert_eq!(reported, bundle_file_paths(&out));
    let transcript_bytes = std::fs::read(&transcript).expect("transcript should be readable");
    let copied = std::fs::read(out.join("transcripts").join(TRANSCRIPT_FILE_NAME)).expect("copy should be readable");
    assert_eq!(copied, transcript_bytes);
    let manifest: Value = serde_json::from_slice(&std::fs::read(out.join("manifest.json")).expect("manifest"))
        .expect("manifest should be JSON");
    assert_eq!(manifest["transcripts"][0]["digest_blake3"], blake3::hash(&transcript_bytes).to_hex().as_str());
}

#[cfg(unix)]
#[test]
fn nix_free_demo_cli_generate_partial_write_failure_reports_no_bundle() {
    use std::os::unix::fs::PermissionsExt;

    let temp = TempDir::new().expect("tempdir should be created");
    let read_only_dir = temp.path().join("read-only");
    std::fs::create_dir(&read_only_dir).expect("read-only transcript dir should be created");
    let first = write_transcript(&read_only_dir, "first transcript\n");
    std::fs::set_permissions(&first, std::fs::Permissions::from_mode(0o444))
        .expect("transcript should become read-only");
    if std::fs::OpenOptions::new().write(true).open(&first).is_ok() {
        eprintln!("skipping: this user can write read-only files, so the second transcript copy cannot fail");
        return;
    }
    let second_dir = temp.path().join("second");
    std::fs::create_dir(&second_dir).expect("second transcript dir should be created");
    let second = write_transcript(&second_dir, "second transcript\n");
    let out = temp.path().join("bundle");
    let args = generate_args(&out, &first).with_arg("--transcript", path_str(&second));

    let output = mantle_cmd().args(&args.args).assert().code(3).get_output().clone();

    assert!(output.stdout.is_empty(), "a partial bundle must not print a generate report");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(&format!("copying transcript {}", path_str(&second))), "{stderr}");
    assert!(
        out.join("transcripts").join(TRANSCRIPT_FILE_NAME).exists(),
        "the first copy lands before the failure"
    );
    assert!(!out.join("summary.json").exists(), "no document is written after the failed copy");
}

#[test]
fn nix_free_demo_cli_generate_failed_write_exits_internal_without_report_or_bundle() {
    let temp = TempDir::new().expect("tempdir should be created");
    let transcript = write_transcript(temp.path(), "stage1 ok\nstage2 ok\n");
    let blocking_file = temp.path().join("bundle-parent-is-a-file");
    std::fs::write(&blocking_file, "not a directory").expect("blocking file should be written");
    let out = blocking_file.join("bundle");

    let output = mantle_cmd().args(&generate_args(&out, &transcript).args).assert().code(3).get_output().clone();

    assert!(output.stdout.is_empty(), "a failed bundle write must not print a generate report");
    assert!(!output.stderr.is_empty());
    assert!(!out.exists());
}

struct GenerateArgs {
    args: Vec<String>,
}

impl GenerateArgs {
    fn success_json(&self) -> Value {
        let output = mantle_cmd().args(&self.args).assert().success().get_output().clone();
        assert!(output.stderr.is_empty());
        serde_json::from_slice(&output.stdout).expect("stdout should be JSON")
    }

    fn with_proof_status(mut self, status: &str) -> Self {
        replace_arg_value(&mut self.args, "--proof-status", status);
        self
    }

    fn without_stage_digests(mut self) -> Self {
        remove_arg_pair(&mut self.args, "--stage1-binary-blake3");
        remove_arg_pair(&mut self.args, "--stage2-binary-blake3");
        self
    }

    fn without_non_claims(mut self, remove: bool) -> Self {
        if remove {
            remove_arg_pair(&mut self.args, "--non-claim");
        }
        self
    }

    fn with_bad_receipt_digest(mut self) -> Self {
        replace_arg_value(&mut self.args, "--receipt-digest", BAD_RECEIPT_DIGEST_ARG);
        self
    }

    fn with_arg(mut self, flag: &str, value: &str) -> Self {
        self.args.push(flag.to_string());
        self.args.push(value.to_string());
        self
    }
}

fn run_generate(out: &std::path::Path, transcript: &std::path::Path) -> GenerateArgs {
    generate_args(out, transcript)
}

fn generate_args(out: &std::path::Path, transcript: &std::path::Path) -> GenerateArgs {
    let mut args = vec![
        "--json".to_string(),
        "nix-free-demo".to_string(),
        "generate".to_string(),
        "--out".to_string(),
        path_str(out).to_string(),
        "--proof-status".to_string(),
        SUCCESS_VERDICT.to_string(),
        "--source-root-identity".to_string(),
        "source-root-v1".to_string(),
        "--toolchain-policy-digest-blake3".to_string(),
        DIGEST_B.to_string(),
        "--stage1-binary-blake3".to_string(),
        DIGEST_A.to_string(),
        "--stage2-binary-blake3".to_string(),
        DIGEST_A.to_string(),
        "--transcript".to_string(),
        path_str(transcript).to_string(),
        "--receipt-digest".to_string(),
        RECEIPT_DIGEST_ARG.to_string(),
        "--artifact-digest".to_string(),
        ARTIFACT_DIGEST_ARG.to_string(),
        "--replay-hint".to_string(),
        "copy bundle and rerun validator".to_string(),
        "--non-claim".to_string(),
        "not release reproducibility".to_string(),
    ];
    for guard_name in ["cargo", "nix", "rustup", "ambient-wrapper"] {
        args.push("--guard".to_string());
        args.push(format!("{guard_name}:denied:{guard_name} denied by fixture"));
    }
    GenerateArgs { args }
}

fn assert_generate_failure(out: &std::path::Path, transcript: &std::path::Path, expected_code: &str) {
    assert_generate_args_failure(generate_args(out, transcript), expected_code);
}

fn assert_generate_args_failure(args: GenerateArgs, expected_code: &str) {
    let output = mantle_cmd().args(&args.args).assert().code(1).get_output().clone();
    let report: Value = serde_json::from_slice(&output.stdout).expect("stdout should be JSON");
    assert_eq!(report["generated"], false);
    assert_eq!(report["diagnostics"][0]["code"], expected_code);
    assert!(report["files"].as_array().unwrap().is_empty());
    assert!(output.stderr.is_empty());
}

/// Run one rejected generation and return its report.
fn generate_rejection(args: &GenerateArgs) -> Value {
    let output = mantle_cmd().args(&args.args).assert().code(1).get_output().clone();
    let report: Value = serde_json::from_slice(&output.stdout).expect("stdout should be JSON");
    assert_eq!(report["generated"], false);
    assert!(report["files"].as_array().expect("files should be an array").is_empty());
    assert!(output.stderr.is_empty());
    report
}

fn dir_entry_names(dir: &std::path::Path) -> Vec<String> {
    let mut names = std::fs::read_dir(dir)
        .expect("dir should be listable")
        .map(|entry| entry.expect("entry should be readable").file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    names.sort();
    names
}

/// Every regular file under `root`, as sorted `/`-separated relative paths.
fn bundle_file_paths(root: &std::path::Path) -> Vec<String> {
    let mut files = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(&dir).expect("bundle dir should be listable") {
            let path = entry.expect("bundle entry should be readable").path();
            if path.is_dir() {
                pending.push(path);
            } else {
                let relative = path.strip_prefix(root).expect("entry should be under the bundle");
                files.push(relative.to_str().expect("bundle paths should be UTF-8").to_string());
            }
        }
    }
    files.sort();
    files
}

fn replace_arg_value(args: &mut [String], flag: &str, value: &str) {
    let Some(index) = args.iter().position(|arg| arg == flag) else {
        panic!("missing flag {flag}");
    };
    args[index + 1] = value.to_string();
}

fn remove_arg_pair(args: &mut Vec<String>, flag: &str) {
    while let Some(index) = args.iter().position(|arg| arg == flag) {
        args.remove(index);
        args.remove(index);
    }
}

fn write_transcript(root: &std::path::Path, text: &str) -> std::path::PathBuf {
    let path = root.join(TRANSCRIPT_FILE_NAME);
    std::fs::write(&path, text).expect("transcript should be written");
    path
}

fn read_bundle_files(root: &std::path::Path) -> std::collections::BTreeMap<String, String> {
    [
        "README.md",
        "manifest.json",
        "summary.json",
        "validation.json",
        "transcripts/proof.log",
    ]
    .iter()
    .map(|relative| {
        let text = std::fs::read_to_string(root.join(relative)).expect("bundle file should be readable");
        ((*relative).to_string(), text)
    })
    .collect()
}

fn write_summary(root: &std::path::Path, summary: Value) -> std::path::PathBuf {
    let path = root.join(SUMMARY_FILE_NAME);
    let bytes = serde_json::to_vec_pretty(&summary).expect("summary should serialize");
    std::fs::write(&path, bytes).expect("summary should be written");
    path
}

fn claimable_summary() -> Value {
    serde_json::json!({
        "schema": "mantle-nix-free-demo-summary-v1",
        "profile": DEMO_PROFILE,
        "fixed_point_verdict": SUCCESS_VERDICT,
        "stage1_binary_blake3": DIGEST_A,
        "stage2_binary_blake3": DIGEST_A,
        "source_root_identity": "source-root-v1",
        "toolchain_policy_digest_blake3": DIGEST_B,
        "command_owned_wrappers": [],
        "guards": [
            guard("cargo"),
            guard("nix"),
            guard("rustup"),
            guard("ambient-wrapper")
        ],
        "replay_hints": ["mantle self-build --cargo-free --fixed-point"],
        "non_claims": ["not release reproducibility"]
    })
}

fn guard(name: &str) -> Value {
    serde_json::json!({
        "guard": name,
        "status": "denied",
        "diagnostic": format!("{name} denied by fixture")
    })
}

fn path_str(path: &std::path::Path) -> &str {
    path.to_str().expect("test paths should be UTF-8")
}
