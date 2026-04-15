use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::path::PathBuf;

use crunch_pipeline::BuildConfig;
use crunch_pipeline::build;

fn sha256_sri(bytes: &[u8]) -> String {
    use sha2::Digest;

    let digest: [u8; 32] = sha2::Sha256::digest(bytes).into();
    format!("sha256-{}", data_encoding::BASE64.encode(&digest))
}

fn can_build() -> bool {
    Path::new("/nix/store").exists()
        && std::process::Command::new("bwrap")
            .arg("--version")
            .output()
            .is_ok_and(|output| output.status.success())
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root should exist")
}

fn import_paths() -> Vec<OsString> {
    let stdlib = crunch_eval::stdlib::stdlib_import_path().expect("stdlib import path should resolve");
    vec![stdlib.into(), repo_root().into_os_string()]
}

fn host_store_path() -> Option<String> {
    let entries = std::fs::read_dir("/nix/store").ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if nix_compat::store_path::StorePath::<String>::from_absolute_path(path.as_os_str().as_bytes()).is_ok() {
            return Some(path.display().to_string());
        }
    }
    None
}

fn build_config(file: PathBuf, output_dir: &Path, state_dir: &Path) -> BuildConfig {
    let keypair = crunch_build::load_keypair(
        "cache.example.com-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==",
    )
    .unwrap();
    let trusted_keys = crunch_build::build_trusted_keys(&keypair, None);
    BuildConfig {
        file,
        import_paths: import_paths(),
        output_dir: output_dir.to_path_buf(),
        state_dir: state_dir.to_path_buf(),
        store_dir: nix_compat::store_path::STORE_DIR.to_string(),
        verbose: false,
        max_jobs: 2,
        substituter_url: None,
        hermeticity_mode: crunch_pipeline::HermeticityMode::Practical,
        keypair,
        trusted_keys,
        trust_unsigned: false,
    }
}

const DETERMINISM_PROBE_PREFIX: &str = "determinism-probe:";

#[derive(Debug, Clone, Copy)]
struct AmbientCase {
    name: &'static str,
    user: &'static str,
    logname: &'static str,
    tz: &'static str,
    lang: &'static str,
    umask: &'static str,
}

const AMBIENT_CASES: [AmbientCase; 3] = [
    AmbientCase {
        name: "new-york-077",
        user: "hostile-user-a",
        logname: "hostile-logname-a",
        tz: "America/New_York",
        lang: "en_US.UTF-8",
        umask: "077",
    },
    AmbientCase {
        name: "tokyo-022",
        user: "hostile-user-b",
        logname: "hostile-logname-b",
        tz: "Asia/Tokyo",
        lang: "ja_JP.UTF-8",
        umask: "022",
    },
    AmbientCase {
        name: "berlin-027",
        user: "hostile-user-c",
        logname: "hostile-logname-c",
        tz: "Europe/Berlin",
        lang: "de_DE.UTF-8",
        umask: "027",
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProbeAuditEvent {
    kind: String,
    detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DeterminismProbe {
    output_digest_hex: Option<String>,
    audit_events: Vec<ProbeAuditEvent>,
    blocker_class: Option<String>,
}

fn build_poisoned_path(prefix_dir: &Path) -> OsString {
    let mut path_entries = vec![prefix_dir.to_path_buf()];
    let current_path = std::env::var_os("PATH").unwrap_or_default();
    path_entries.extend(std::env::split_paths(&current_path).filter(|entry| !entry.as_os_str().is_empty()));
    std::env::join_paths(path_entries).expect("poisoned PATH should be valid")
}

fn parse_probe(stdout: &[u8]) -> DeterminismProbe {
    let stdout = String::from_utf8_lossy(stdout);
    let start = stdout
        .find(DETERMINISM_PROBE_PREFIX)
        .unwrap_or_else(|| panic!("missing probe line in stdout:\n{stdout}"));
    let payload = stdout[start + DETERMINISM_PROBE_PREFIX.len()..]
        .lines()
        .next()
        .expect("probe payload line must exist");
    let mut fields = payload.split('\t');
    let digest = decode_optional_field(fields.next().expect("probe digest field must exist"));
    let blocker = decode_optional_field(fields.next().expect("probe blocker field must exist"));
    let audit_events = decode_audit_events(fields.next().expect("probe audit field must exist"));
    assert!(fields.next().is_none(), "probe payload must have exactly three fields");
    DeterminismProbe {
        output_digest_hex: digest,
        audit_events,
        blocker_class: blocker,
    }
}

fn hex_bytes(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn encode_field(value: &str) -> String {
    data_encoding::BASE64.encode(value.as_bytes())
}

fn decode_field(value: &str) -> String {
    let decoded = data_encoding::BASE64.decode(value.as_bytes()).expect("probe field should be valid base64");
    String::from_utf8(decoded).expect("probe field should be valid UTF-8")
}

fn encode_optional_field(value: Option<&str>) -> String {
    value.map_or_else(|| "-".to_string(), encode_field)
}

fn decode_optional_field(value: &str) -> Option<String> {
    if value == "-" {
        return None;
    }
    Some(decode_field(value))
}

fn encode_audit_events(events: &[ProbeAuditEvent]) -> String {
    if events.is_empty() {
        return "-".to_string();
    }
    events
        .iter()
        .map(|event| format!("{}:{}", encode_field(&event.kind), encode_field(&event.detail)))
        .collect::<Vec<_>>()
        .join(",")
}

fn decode_audit_events(value: &str) -> Vec<ProbeAuditEvent> {
    if value == "-" {
        return Vec::new();
    }
    value
        .split(',')
        .map(|entry| {
            let (kind, detail) = entry.split_once(':').expect("audit entry should contain kind/detail separator");
            ProbeAuditEvent {
                kind: decode_field(kind),
                detail: decode_field(detail),
            }
        })
        .collect()
}

fn summarize_audit_events(events: &[crunch_pipeline::HermeticityAuditEvent]) -> Vec<ProbeAuditEvent> {
    events
        .iter()
        .map(|event| ProbeAuditEvent {
            kind: event.kind.as_str().to_string(),
            detail: event.detail.clone(),
        })
        .collect()
}

fn emit_probe(probe: &DeterminismProbe) {
    println!(
        "{DETERMINISM_PROBE_PREFIX}{}\t{}\t{}",
        encode_optional_field(probe.output_digest_hex.as_deref()),
        encode_optional_field(probe.blocker_class.as_deref()),
        encode_audit_events(&probe.audit_events),
    );
}

fn success_probe(result: &crunch_pipeline::PipelineResult) -> DeterminismProbe {
    assert!(result.failed.is_empty(), "successful probe must not fail: {:?}", result.failed);
    assert_eq!(result.outcomes.len(), 1, "successful probe must have one outcome");
    let out = result.outcomes[0].outputs.get("out").expect("successful probe must have out output");
    DeterminismProbe {
        output_digest_hex: Some(hex_bytes(&out.nar_sha256)),
        audit_events: summarize_audit_events(&result.hermeticity_audit_events),
        blocker_class: None,
    }
}

fn blocker_probe(result: &crunch_pipeline::PipelineResult, blocker_class: &str) -> DeterminismProbe {
    assert!(result.outcomes.is_empty(), "blocker probe must not succeed: {:?}", result.outcomes);
    assert_eq!(result.failed.len(), 1, "blocker probe must report one failure");
    DeterminismProbe {
        output_digest_hex: None,
        audit_events: summarize_audit_events(&result.hermeticity_audit_events),
        blocker_class: Some(blocker_class.to_string()),
    }
}

fn run_ambient_probe(test_name: &str, case: AmbientCase) -> DeterminismProbe {
    let current_exe = std::env::current_exe().expect("current test binary should exist");
    let root = tempfile::tempdir().expect("ambient temp root should exist");
    let home_dir = root.path().join("home");
    let tmp_dir = root.path().join("tmp");
    let cwd_dir = root.path().join("cwd");
    let path_prefix = root.path().join("path-prefix");
    std::fs::create_dir_all(&home_dir).unwrap();
    std::fs::create_dir_all(&tmp_dir).unwrap();
    std::fs::create_dir_all(&cwd_dir).unwrap();
    std::fs::create_dir_all(&path_prefix).unwrap();

    let output = std::process::Command::new("/bin/sh")
        .arg("-c")
        .arg("cd \"$3\" && umask \"$4\" && exec \"$1\" --ignored --exact --nocapture --test-threads=1 \"$2\"")
        .arg("sh")
        .arg(&current_exe)
        .arg(test_name)
        .arg(&cwd_dir)
        .arg(case.umask)
        .env("HOME", &home_dir)
        .env("PATH", build_poisoned_path(&path_prefix))
        .env("USER", case.user)
        .env("LOGNAME", case.logname)
        .env("TZ", case.tz)
        .env("LANG", case.lang)
        .env("LC_ALL", case.lang)
        .env("TMPDIR", &tmp_dir)
        .env("TEMP", &tmp_dir)
        .env("TMP", &tmp_dir)
        .env("TEMPDIR", &tmp_dir)
        .env("SHELL", "/tmp/hostile-shell")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "ambient case {} failed for {test_name}\nstdout:\n{}\nstderr:\n{}",
        case.name,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let mut combined = output.stdout;
    combined.extend_from_slice(&output.stderr);
    parse_probe(&combined)
}

fn assert_success_probe_stability(test_name: &str) {
    let mut probes = AMBIENT_CASES.iter().map(|case| (case.name, run_ambient_probe(test_name, *case)));
    let (baseline_name, baseline_probe) = probes.next().expect("ambient cases must not be empty");
    let baseline_digest = baseline_probe.output_digest_hex.clone().expect("baseline success probe must emit digest");
    assert!(baseline_probe.blocker_class.is_none(), "baseline success probe must not emit blocker");
    for (case_name, probe) in probes {
        let digest = probe.output_digest_hex.clone().expect("success probe must emit digest");
        assert_eq!(digest, baseline_digest, "digest changed for {case_name}");
        assert_eq!(probe.audit_events, baseline_probe.audit_events, "audit events changed for {case_name}");
        assert!(probe.blocker_class.is_none(), "success probe must not emit blocker for {case_name}");
    }
    assert!(!baseline_name.is_empty(), "baseline case name must not be empty");
}

fn assert_blocker_probe_stability(test_name: &str, expected_blocker: &str) {
    let mut probes = AMBIENT_CASES.iter().map(|case| (case.name, run_ambient_probe(test_name, *case)));
    let (_baseline_name, baseline_probe) = probes.next().expect("ambient cases must not be empty");
    assert_eq!(baseline_probe.output_digest_hex, None, "blocker probe must not emit digest");
    assert_eq!(baseline_probe.blocker_class.as_deref(), Some(expected_blocker));
    for (case_name, probe) in probes {
        assert_eq!(probe.output_digest_hex, None, "blocker probe must not emit digest for {case_name}");
        assert_eq!(probe.audit_events, baseline_probe.audit_events, "blocker audit events changed for {case_name}");
        assert_eq!(probe.blocker_class, baseline_probe.blocker_class, "blocker class changed for {case_name}");
    }
}

#[tokio::test]
async fn pipeline_builds_trivial_derivation_end_to_end() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let work = tempfile::tempdir().unwrap();
    let output_dir = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    let ncl_file = work.path().join("test.ncl");

    std::fs::write(
        &ncl_file,
        r#"let crunch = import "lib.ncl" in
{
  name = "pipeline-e2e",
  builder = "/bin/sh",
  args = ["-c", "echo 'pipeline works' > $out"],
  addressing_mode = 'input-addressed,
} | crunch.Derivation"#,
    )
    .unwrap();

    let mut config = build_config(ncl_file, output_dir.path(), state_dir.path());
    config.max_jobs = 1;

    let result = build(&config).await.unwrap();
    assert!(result.failed.is_empty(), "pipeline failures: {:?}", result.failed);
    assert!(result.fod_mismatches.is_empty(), "unexpected FOD mismatches");
    assert_eq!(result.hermeticity_mode, crunch_pipeline::HermeticityMode::Practical);
    assert!(result.hermeticity_audit_events.is_empty(), "unexpected hermeticity audit events");
    assert_eq!(result.outcomes.len(), 1);

    let outcome = &result.outcomes[0];
    let path_info = outcome.outputs.get("out").expect("out output should exist");
    let output_path = path_info.store_path.to_absolute_path_with_prefix(output_dir.path().to_str().unwrap());
    let output_path = PathBuf::from(output_path);

    assert!(output_path.exists(), "output should exist on disk: {}", output_path.display());
    let content = std::fs::read_to_string(&output_path).unwrap();
    assert_eq!(content.trim(), "pipeline works");
}

#[tokio::test]
async fn pipeline_reports_fod_mismatch_without_aborting_other_roots() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let work = tempfile::tempdir().unwrap();
    let output_dir = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    let good_src = work.path().join("good.txt");
    let bad_src = work.path().join("bad.txt");
    let ncl_file = work.path().join("fetches.ncl");

    let good_content = b"good pipeline fetch";
    let bad_content = b"bad pipeline fetch";
    let wrong_hash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

    std::fs::write(&good_src, good_content).unwrap();
    std::fs::write(&bad_src, bad_content).unwrap();
    std::fs::write(
        &ncl_file,
        format!(
            r#"let crunch = import "lib.ncl" in
{{
  good = crunch.fetchurl {{
    name = "good-src",
    url = "file://{}",
    hash = "{}",
  }},
  bad = crunch.fetchurl {{
    name = "bad-src",
    url = "file://{}",
    hash = "{}",
  }},
}}"#,
            good_src.display(),
            sha256_sri(good_content),
            bad_src.display(),
            wrong_hash,
        ),
    )
    .unwrap();

    let config = build_config(ncl_file, output_dir.path(), state_dir.path());
    let result = build(&config).await.unwrap();

    assert_eq!(result.outcomes.len(), 1, "successful roots: {:?}", result.outcomes);
    assert_eq!(result.failed.len(), 1, "failed roots: {:?}", result.failed);
    assert_eq!(result.fod_mismatches.len(), 1, "FOD mismatches: {:?}", result.fod_mismatches);

    let mismatch = &result.fod_mismatches[0];
    assert_eq!(mismatch.name, "bad-src");
    assert_eq!(mismatch.expected_sri, wrong_hash);
    assert_ne!(mismatch.actual_sri, wrong_hash);
    assert!(mismatch.actual_sri.starts_with("sha256-"));
    assert!(result.failed[0].error.contains("FOD hash mismatch"));
    assert!(result.root_labels.values().any(|label| label == "good"));
    assert!(result.root_labels.values().any(|label| label == "bad"));

    let good_outcome = &result.outcomes[0];
    let good_output = good_outcome.outputs.get("out").expect("good root should have out");
    let good_path =
        PathBuf::from(good_output.store_path.to_absolute_path_with_prefix(output_dir.path().to_str().unwrap()));
    assert!(good_path.exists(), "good fetch should land on disk: {}", good_path.display());
    assert_eq!(std::fs::read(&good_path).unwrap(), good_content);
}

#[tokio::test]
async fn pipeline_normalizes_runtime_environment_and_umask() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let work = tempfile::tempdir().unwrap();
    let output_dir = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    let ncl_file = work.path().join("env-envelope.ncl");

    std::fs::write(
        &ncl_file,
        r#"let crunch = import "lib.ncl" in
{
  name = "pipeline-env-envelope",
  builder = "/bin/sh",
  args = ["-c", "set -eu; [ \"$HOME\" = \"/homeless-shelter\" ]; [ \"$PATH\" = \"/path-not-set\" ]; [ \"$USER\" = \"nixbld\" ]; [ \"$TZ\" = \"UTC\" ]; [ \"$LANG\" = \"C\" ]; [ \"$TMPDIR\" = \"/build\" ]; [ \"$SHELL\" = \"/bin/sh\" ]; [ -x \"$SHELL\" ]; [ -x /bin/sh ]; printf '%s\n' \"$HOME\" \"$PATH\" \"$USER\" \"$TZ\" \"$LANG\" \"$TMPDIR\" \"$SHELL\" \"$(umask)\" > \"$out\""],
  addressing_mode = 'input-addressed,
} | crunch.Derivation"#,
    )
    .unwrap();

    let mut config = build_config(ncl_file, output_dir.path(), state_dir.path());
    config.hermeticity_mode = crunch_pipeline::HermeticityMode::Strict;

    let result = build(&config).await.unwrap();

    assert_eq!(result.hermeticity_mode, crunch_pipeline::HermeticityMode::Strict);
    assert!(result.failed.is_empty(), "pipeline failures: {:?}", result.failed);
    assert!(result.hermeticity_audit_events.is_empty(), "unexpected hermeticity audit events");
    assert_eq!(result.outcomes.len(), 1);

    let outcome = &result.outcomes[0];
    let output_path = PathBuf::from(
        outcome.outputs["out"].store_path.to_absolute_path_with_prefix(output_dir.path().to_str().unwrap()),
    );
    let lines: Vec<String> = std::fs::read_to_string(&output_path).unwrap().lines().map(ToOwned::to_owned).collect();

    assert_eq!(lines, vec![
        "/homeless-shelter".to_string(),
        "/path-not-set".to_string(),
        "nixbld".to_string(),
        "UTC".to_string(),
        "C".to_string(),
        "/build".to_string(),
        "/bin/sh".to_string(),
        "0022".to_string(),
    ]);
}

#[test]
fn pipeline_host_ambient_state_does_not_leak_into_strict_build() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let current_exe = std::env::current_exe().unwrap();
    let current_path = std::env::var_os("PATH").unwrap_or_default();
    let poisoned_path =
        std::env::join_paths([PathBuf::from("/tmp/hostile-path").into_os_string(), current_path]).unwrap();

    let output = std::process::Command::new("/bin/sh")
        .arg("-c")
        .arg("umask 077 && exec \"$1\" --exact --test-threads=1 \"$2\"")
        .arg("sh")
        .arg(&current_exe)
        .arg("pipeline_normalizes_runtime_environment_and_umask")
        .env("HOME", "/tmp/hostile-home")
        .env("PATH", &poisoned_path)
        .env("USER", "hostile-user")
        .env("LOGNAME", "hostile-logname")
        .env("TZ", "America/New_York")
        .env("LANG", "en_US.UTF-8")
        .env("LC_ALL", "en_US.UTF-8")
        .env("TMPDIR", "/tmp/hostile-tmpdir")
        .env("TEMP", "/tmp/hostile-temp")
        .env("TMP", "/tmp/hostile-tmp")
        .env("TEMPDIR", "/tmp/hostile-tempdir")
        .env("SHELL", "/tmp/hostile-shell")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "hostile child run failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn pipeline_determinism_normal_derivation_stable_across_ambient_state() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    assert_success_probe_stability("pipeline_determinism_probe_normal_derivation");
}

#[test]
fn pipeline_determinism_fetcher_root_stable_across_ambient_state() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    assert_success_probe_stability("pipeline_determinism_probe_fetcher_root");
}

#[test]
fn pipeline_determinism_self_build_friendly_path_stable_across_ambient_state() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    assert_success_probe_stability("pipeline_determinism_probe_self_build_friendly_path");
}

#[test]
fn pipeline_determinism_strict_blocker_stable_across_ambient_state() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    assert_blocker_probe_stability(
        "pipeline_determinism_probe_strict_environment_override_blocker",
        "unsafe-env-override:PATH",
    );
}

#[tokio::test]
#[ignore]
async fn pipeline_determinism_probe_normal_derivation() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let work = tempfile::tempdir().unwrap();
    let output_dir = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    let ncl_file = work.path().join("determinism-normal.ncl");

    std::fs::write(
        &ncl_file,
        r#"let crunch = import "lib.ncl" in
{
  name = "determinism-normal",
  builder = "/bin/sh",
  args = ["-c", "set -eu; printf 'stable normal build\n' > $out"],
  addressing_mode = 'input-addressed,
} | crunch.Derivation"#,
    )
    .unwrap();

    let mut config = build_config(ncl_file, output_dir.path(), state_dir.path());
    config.hermeticity_mode = crunch_pipeline::HermeticityMode::Strict;

    let result = build(&config).await.unwrap();
    assert_eq!(result.hermeticity_mode, crunch_pipeline::HermeticityMode::Strict);
    assert!(result.hermeticity_audit_events.is_empty(), "unexpected audit events: {:?}", result.hermeticity_audit_events);
    emit_probe(&success_probe(&result));
}

#[tokio::test]
#[ignore]
async fn pipeline_determinism_probe_fetcher_root() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let work = tempfile::tempdir().unwrap();
    let output_dir = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    let source_file = work.path().join("fetch.txt");
    let ncl_file = work.path().join("determinism-fetch.ncl");
    let content = b"stable fetch root\n";

    std::fs::write(&source_file, content).unwrap();
    std::fs::write(
        &ncl_file,
        format!(
            r#"let crunch = import "lib.ncl" in
crunch.fetchurl {{
  name = "determinism-fetch-root",
  url = "file://{}",
  hash = "{}",
}}"#,
            source_file.display(),
            sha256_sri(content),
        ),
    )
    .unwrap();

    let mut config = build_config(ncl_file, output_dir.path(), state_dir.path());
    config.hermeticity_mode = crunch_pipeline::HermeticityMode::Strict;

    let result = build(&config).await.unwrap();
    assert_eq!(result.hermeticity_mode, crunch_pipeline::HermeticityMode::Strict);
    assert!(result.hermeticity_audit_events.is_empty(), "unexpected audit events: {:?}", result.hermeticity_audit_events);
    emit_probe(&success_probe(&result));
}

#[tokio::test]
#[ignore]
async fn pipeline_determinism_probe_self_build_friendly_path() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let work = tempfile::tempdir().unwrap();
    let output_dir = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    let ncl_file = work.path().join("determinism-self-build-friendly.ncl");

    std::fs::write(
        &ncl_file,
        r#"let crunch = import "lib.ncl" in
let bootstrap_tool = {
  name = "determinism-bootstrap-tool",
  builder = "/bin/sh",
  args = ["-c", "set -eu; printf 'bootstrap tool payload\n' > $out"],
  addressing_mode = 'input-addressed,
} | crunch.Derivation in
{
  name = "determinism-self-build-friendly",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      TOOL_PATH=""
      for d in $NIX_STORE/*-determinism-bootstrap-tool; do
        if [ -f "$d" ]; then TOOL_PATH="$d"; break; fi
      done
      test -n "$TOOL_PATH"
      $BB cat "$TOOL_PATH" > $out
    "%
  ],
  inputs = [bootstrap_tool],
  addressing_mode = 'input-addressed,
} | crunch.Derivation"#,
    )
    .unwrap();

    let mut config = build_config(ncl_file, output_dir.path(), state_dir.path());
    config.hermeticity_mode = crunch_pipeline::HermeticityMode::Strict;

    let result = build(&config).await.unwrap();
    assert_eq!(result.hermeticity_mode, crunch_pipeline::HermeticityMode::Strict);
    assert!(result.hermeticity_audit_events.is_empty(), "unexpected audit events: {:?}", result.hermeticity_audit_events);
    emit_probe(&success_probe(&result));
}

#[tokio::test]
#[ignore]
async fn pipeline_determinism_probe_strict_environment_override_blocker() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let work = tempfile::tempdir().unwrap();
    let output_dir = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    let ncl_file = work.path().join("determinism-blocker.ncl");

    std::fs::write(
        &ncl_file,
        r#"let crunch = import "lib.ncl" in
{
  name = "determinism-strict-blocker",
  builder = "/bin/sh",
  args = ["-c", "set -eu; printf '%s' \"$PATH\" > $out"],
  env = { PATH = "/override/bin" },
  addressing_mode = 'input-addressed,
} | crunch.Derivation"#,
    )
    .unwrap();

    let mut config = build_config(ncl_file, output_dir.path(), state_dir.path());
    config.hermeticity_mode = crunch_pipeline::HermeticityMode::Strict;

    let result = build(&config).await.unwrap();
    assert!(result.hermeticity_audit_events.is_empty(), "strict blocker must not degrade into audit events");
    assert!(result.failed[0].error.contains("unsafe sandbox environment override"));
    assert!(result.failed[0].error.contains("PATH"));
    emit_probe(&blocker_probe(&result, "unsafe-env-override:PATH"));
}

#[tokio::test]
async fn pipeline_practical_mode_audits_environment_override() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let work = tempfile::tempdir().unwrap();
    let output_dir = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    let ncl_file = work.path().join("env-override-practical.ncl");

    std::fs::write(
        &ncl_file,
        r#"let crunch = import "lib.ncl" in
{
  name = "pipeline-env-override-practical",
  builder = "/bin/sh",
  args = ["-c", "set -eu; [ \"$PATH\" = \"/override/bin\" ]; printf '%s' \"$PATH\" > \"$out\""],
  env = { PATH = "/override/bin" },
  addressing_mode = 'input-addressed,
} | crunch.Derivation"#,
    )
    .unwrap();

    let result = build(&build_config(ncl_file, output_dir.path(), state_dir.path())).await.unwrap();

    assert!(result.failed.is_empty(), "pipeline failures: {:?}", result.failed);
    assert_eq!(result.outcomes.len(), 1);
    assert_eq!(result.hermeticity_audit_events.len(), 1);
    assert_eq!(result.hermeticity_audit_events[0].kind, crunch_pipeline::HermeticityAuditKind::EnvironmentOverride);
    assert!(result.hermeticity_audit_events[0].detail.contains("PATH"));

    let outcome = &result.outcomes[0];
    let output_path = PathBuf::from(
        outcome.outputs["out"].store_path.to_absolute_path_with_prefix(output_dir.path().to_str().unwrap()),
    );
    assert_eq!(std::fs::read_to_string(&output_path).unwrap(), "/override/bin");
}

#[tokio::test]
async fn pipeline_strict_mode_rejects_environment_override() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let work = tempfile::tempdir().unwrap();
    let output_dir = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    let ncl_file = work.path().join("env-override-strict.ncl");

    std::fs::write(
        &ncl_file,
        r#"let crunch = import "lib.ncl" in
{
  name = "pipeline-env-override-strict",
  builder = "/bin/sh",
  args = ["-c", "set -eu; printf '%s' \"$PATH\" > \"$out\""],
  env = { PATH = "/override/bin" },
  addressing_mode = 'input-addressed,
} | crunch.Derivation"#,
    )
    .unwrap();

    let mut config = build_config(ncl_file, output_dir.path(), state_dir.path());
    config.hermeticity_mode = crunch_pipeline::HermeticityMode::Strict;

    let result = build(&config).await.unwrap();

    assert_eq!(result.hermeticity_mode, crunch_pipeline::HermeticityMode::Strict);
    assert!(result.outcomes.is_empty(), "strict mode should reject before build: {:?}", result.outcomes);
    assert_eq!(result.failed.len(), 1, "strict-mode failures: {:?}", result.failed);
    assert!(
        result.hermeticity_audit_events.is_empty(),
        "strict rejection should not downgrade to an audit event"
    );
    assert!(result.failed[0].error.contains("unsafe sandbox environment override"));
    assert!(result.failed[0].error.contains("PATH"));
}

#[tokio::test]
async fn pipeline_practical_mode_reports_pathinfo_startup_fallback() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let work = tempfile::tempdir().unwrap();
    let output_dir = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(state_dir.path().join("pathinfo.redb")).unwrap();
    let source = work.path().join("fetch.txt");
    let ncl_file = work.path().join("fetch.ncl");
    let content = b"pathinfo fallback\n";

    std::fs::write(&source, content).unwrap();
    std::fs::write(
        &ncl_file,
        format!(
            r#"let crunch = import "lib.ncl" in
crunch.fetchurl {{
  name = "pathinfo-fallback-src",
  url = "file://{}",
  hash = "{}",
}}"#,
            source.display(),
            sha256_sri(content),
        ),
    )
    .unwrap();

    let result = build(&build_config(ncl_file, output_dir.path(), state_dir.path())).await.unwrap();

    assert!(result.failed.is_empty(), "practical pathinfo fallback should still build: {:?}", result.failed);
    assert_eq!(result.outcomes.len(), 1);
    assert!(
        result
            .hermeticity_audit_events
            .iter()
            .any(|event| event.kind == crunch_pipeline::HermeticityAuditKind::PathInfoFallback)
    );
}

#[tokio::test]
async fn pipeline_strict_mode_rejects_pathinfo_startup_fallback() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let work = tempfile::tempdir().unwrap();
    let output_dir = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(state_dir.path().join("pathinfo.redb")).unwrap();
    let source = work.path().join("fetch.txt");
    let ncl_file = work.path().join("fetch.ncl");
    let content = b"pathinfo fallback strict\n";

    std::fs::write(&source, content).unwrap();
    std::fs::write(
        &ncl_file,
        format!(
            r#"let crunch = import "lib.ncl" in
crunch.fetchurl {{
  name = "pathinfo-fallback-src",
  url = "file://{}",
  hash = "{}",
}}"#,
            source.display(),
            sha256_sri(content),
        ),
    )
    .unwrap();

    let mut config = build_config(ncl_file, output_dir.path(), state_dir.path());
    config.hermeticity_mode = crunch_pipeline::HermeticityMode::Strict;

    let result = build(&config).await.unwrap();

    assert!(result.outcomes.is_empty(), "strict mode should reject before fetch build: {:?}", result.outcomes);
    assert_eq!(result.failed.len(), 1, "strict-mode failures: {:?}", result.failed);
    assert!(result.hermeticity_audit_events.is_empty());
    assert!(result.failed[0].error.contains("strict mode does not permit in-memory PathInfo fallback"));
    assert_eq!(result.root_labels.len(), 1);
}

#[tokio::test]
async fn pipeline_practical_mode_reports_degraded_closure_resolution() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }
    let Some(source_path) = host_store_path() else {
        eprintln!("skipping: no host /nix/store path available");
        return;
    };

    let work = tempfile::tempdir().unwrap();
    let output_dir = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    let ncl_file = work.path().join("closure-practical.ncl");

    std::fs::write(
        &ncl_file,
        format!(
            r#"let crunch = import "lib.ncl" in
{{
  name = "pipeline-closure-practical",
  builder = "/bin/sh",
  args = ["-c", "set -eu; test -e \"$src\"; printf 'closure ok' > \"$out\""],
  env = {{ src = "{}" }},
  inputs = ["{}"],
  addressing_mode = 'input-addressed,
}} | crunch.Derivation"#,
            source_path, source_path,
        ),
    )
    .unwrap();

    let result = build(&build_config(ncl_file, output_dir.path(), state_dir.path())).await.unwrap();

    assert!(
        result.failed.is_empty(),
        "practical degraded closure build should still succeed: {:?}",
        result.failed
    );
    assert_eq!(result.outcomes.len(), 1);
    let degraded = result
        .hermeticity_audit_events
        .iter()
        .find(|event| event.kind == crunch_pipeline::HermeticityAuditKind::ClosureResolutionDegraded)
        .expect("missing closure degraded audit event");
    assert!(degraded.detail.contains(&source_path));
}

#[tokio::test]
async fn pipeline_strict_mode_rejects_missing_closure_facts() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }
    let Some(source_path) = host_store_path() else {
        eprintln!("skipping: no host /nix/store path available");
        return;
    };

    let work = tempfile::tempdir().unwrap();
    let output_dir = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    let ncl_file = work.path().join("closure-strict.ncl");

    std::fs::write(
        &ncl_file,
        format!(
            r#"let crunch = import "lib.ncl" in
{{
  name = "pipeline-closure-strict",
  builder = "/bin/sh",
  args = ["-c", "set -eu; test -e \"$src\"; printf 'closure ok' > \"$out\""],
  env = {{ src = "{}" }},
  inputs = ["{}"],
  addressing_mode = 'input-addressed,
}} | crunch.Derivation"#,
            source_path, source_path,
        ),
    )
    .unwrap();

    let mut config = build_config(ncl_file, output_dir.path(), state_dir.path());
    config.hermeticity_mode = crunch_pipeline::HermeticityMode::Strict;

    let result = build(&config).await.unwrap();

    assert!(result.outcomes.is_empty(), "strict mode should reject before sandbox build: {:?}", result.outcomes);
    assert_eq!(result.failed.len(), 1, "strict-mode failures: {:?}", result.failed);
    assert!(result.hermeticity_audit_events.is_empty());
    assert!(result.failed[0].error.contains("missing closure facts for source input"));
    assert!(result.failed[0].error.contains(&source_path));
}
