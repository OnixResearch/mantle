#!/usr/bin/env -S cargo +nightly -Zscript
---
[package]
edition = "2024"
[dependencies]
---

//! Deterministic bootstrap blocker inventory and promotion-drift gate.
//!
//! The gate is intentionally source-derived and lightweight. It does not prove
//! the bootstrap chain; it prevents an empty OpenSpec queue or a normalized
//! provider artifact from being mistaken for full-source bootstrap promotion.

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MarkerClass {
    id: &'static str,
    description: &'static str,
    needles: &'static [&'static str],
}

impl MarkerClass {
    fn matches_line(&self, lower_line: &str) -> bool {
        self.needles.iter().any(|needle| lower_line.contains(needle))
    }
}

const MARKERS: &[MarkerClass] = &[
    MarkerClass {
        id: "bridge-output",
        description: "Bootstrap stage uses or documents a bridge output rather than end-to-end source-built proof.",
        needles: &["bridge", "bridge input", "bridge copy", "bridge smoke"],
    },
    MarkerClass {
        id: "compiler-runtime-crash-boundary",
        description: "Known compiler/runtime crash, timeout, signal-derived exit, or static-link boundary still gates promotion evidence.",
        needles: &[
            "segfault",
            "segmentation fault",
            "rc=139",
            "exit 139",
            "timeout",
            "signal-derived",
            "static link",
            "static-link",
        ],
    },
    MarkerClass {
        id: "legacy-provider-fallback",
        description: "Legacy musl.cc or host-provider fallback remains part of the bootstrap path or documentation.",
        needles: &[
            "legacy musl.cc",
            "seed-legacy",
            "legacy provider",
            "crunch_legacy_seed",
            "host fallback",
            "host-bwrap fallback",
        ],
    },
    MarkerClass {
        id: "normalization-only-provider",
        description: "Provider contract is normalized but not yet accepted as full source-built proof.",
        needles: &[
            "normalization contract only",
            "normalization complete",
            "normalized seed contract",
            "not yet functional",
        ],
    },
    MarkerClass {
        id: "placeholder-deferred",
        description: "Placeholder, TODO, or deferred full-source work remains in a bootstrap-critical surface.",
        needles: &["placeholder", "todo:", "deferred task", "not yet implemented"],
    },
    MarkerClass {
        id: "prerequisite-gated-evidence",
        description: "Evidence is explicitly prerequisite-gated or records a blocked status rather than promotion.",
        needles: &[
            "prerequisite-gated",
            "remains blocked",
            "status=blocked",
            "still gated",
            "gated by",
            "full-source bootstrap status remains blocked",
        ],
    },
];

const PROMOTION_CLAIM_NEEDLES: &[&str] = &[
    "full-source bootstrap status: promoted",
    "full-source bootstrap status = promoted",
    "full-source bootstrap status: ready",
    "full-source bootstrap status = ready",
    "full-source-ready = true",
    "full_source_ready = true",
    "provider_status = \"promoted\"",
    "provider_status: promoted",
    "seed-full promoted",
    "full-source proof complete",
];

#[derive(Debug, Clone)]
struct Finding {
    class_id: &'static str,
    path: String,
    line: usize,
    excerpt: String,
}

#[derive(Debug, Clone)]
struct Suppression {
    class_id: &'static str,
    path: String,
    line: usize,
    reason: &'static str,
    excerpt: String,
}

#[derive(Debug, Clone)]
struct PromotionClaim {
    path: String,
    line: usize,
    excerpt: String,
}

#[derive(Default)]
struct Config {
    paths: Vec<PathBuf>,
    json_path: Option<PathBuf>,
    markdown_path: Option<PathBuf>,
    enforce: bool,
    self_test: bool,
}

fn main() -> ExitCode {
    match run() {
        Ok(outcome) => {
            println!(
                "bootstrap blocker inventory: {} findings across {} classes, {} evidence-backed suppressions, {} promotion claims, enforce={}",
                outcome.findings.len(),
                outcome.by_class.len(),
                outcome.suppressions.len(),
                outcome.promotion_claims.len(),
                outcome.enforce,
            );
            if outcome.enforce && !outcome.promotion_claims.is_empty() && !outcome.findings.is_empty() {
                eprintln!("FAIL: promotion claim conflicts with remaining bootstrap blockers; see report for details");
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(2)
        }
    }
}

struct Outcome {
    findings: Vec<Finding>,
    suppressions: Vec<Suppression>,
    by_class: BTreeMap<&'static str, usize>,
    promotion_claims: Vec<PromotionClaim>,
    enforce: bool,
}

fn run() -> Result<Outcome, String> {
    let config = parse_args()?;
    if config.self_test {
        run_self_tests()?;
    }
    let paths = if config.paths.is_empty() {
        vec![
            PathBuf::from("bootstrap"),
            PathBuf::from("openspec/specs/bootstrap/spec.md"),
        ]
    } else {
        config.paths.clone()
    };

    let files = collect_files(&paths)?;
    let mut findings = Vec::new();
    let mut suppressions = Vec::new();
    let mut promotion_claims = Vec::new();
    let evidence = EvidenceState::load();

    for file in files {
        scan_file(&file, &evidence, &mut findings, &mut suppressions, &mut promotion_claims)?;
    }

    findings
        .sort_by(|a, b| (&a.class_id, &a.path, a.line, &a.excerpt).cmp(&(&b.class_id, &b.path, b.line, &b.excerpt)));
    suppressions
        .sort_by(|a, b| (&a.class_id, &a.path, a.line, &a.excerpt).cmp(&(&b.class_id, &b.path, b.line, &b.excerpt)));
    promotion_claims.sort_by(|a, b| (&a.path, a.line, &a.excerpt).cmp(&(&b.path, b.line, &b.excerpt)));

    let mut by_class: BTreeMap<&'static str, usize> = BTreeMap::new();
    for finding in &findings {
        *by_class.entry(finding.class_id).or_insert(0) += 1;
    }

    let outcome = Outcome {
        findings,
        suppressions,
        by_class,
        promotion_claims,
        enforce: config.enforce,
    };

    if let Some(path) = &config.json_path {
        fs::write(path, render_json(&outcome)).map_err(|e| format!("write {}: {e}", path.display()))?;
    }
    if let Some(path) = &config.markdown_path {
        fs::write(path, render_markdown(&outcome)).map_err(|e| format!("write {}: {e}", path.display()))?;
    }

    Ok(outcome)
}

fn parse_args() -> Result<Config, String> {
    let mut config = Config::default();
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--json" => config.json_path = Some(PathBuf::from(args.next().ok_or("--json requires a path")?)),
            "--markdown" => {
                config.markdown_path = Some(PathBuf::from(args.next().ok_or("--markdown requires a path")?))
            }
            "--enforce" => config.enforce = true,
            "--self-test" => config.self_test = true,
            "--help" | "-h" => return Err(help()),
            _ if arg.starts_with('-') => return Err(format!("unknown option: {arg}\n{}", help())),
            _ => config.paths.push(PathBuf::from(arg)),
        }
    }
    Ok(config)
}

fn help() -> String {
    "usage: check-bootstrap-blocker-inventory.rs [--enforce] [--self-test] [--json PATH] [--markdown PATH] [PATH ...]"
        .to_string()
}

fn run_self_tests() -> Result<(), String> {
    let fixture_cases: &[(&str, &[&str])] = &[
        ("bridge-output", &[
            "GCC pass1 bridge smoke remains the current proof boundary",
            "bridge input copied into the derivation output",
        ]),
        ("compiler-runtime-crash-boundary", &[
            "TinyCC static link still exits 139",
            "diagnostic rc=139 under libtcc.c",
            "simple Makefile execution still segfaults",
            "bounded timeout while compiling the compiler",
            "exit status is not signal-derived",
        ]),
        ("legacy-provider-fallback", &[
            "legacy musl.cc provider remains the bootstrap seed",
            "CRUNCH_LEGACY_SEED keeps host fallback available",
        ]),
        ("normalization-only-provider", &[
            "normalized seed contract is not yet functional",
            "normalization contract only for the provider",
        ]),
        ("placeholder-deferred", &[
            "placeholder libgcc member remains in the archive",
            "TODO: replace this deferred task with native source proof",
        ]),
        ("prerequisite-gated-evidence", &[
            "full-source bootstrap status remains blocked",
            "promotion is still gated by GCC correctness evidence",
        ]),
    ];

    for (marker_id, cases) in fixture_cases {
        let marker = MARKERS
            .iter()
            .find(|marker| marker.id == *marker_id)
            .ok_or_else(|| format!("missing marker fixture class: {marker_id}"))?;
        for case in *cases {
            let lower = case.to_lowercase();
            if !marker.matches_line(&lower) {
                return Err(format!("self-test expected {marker_id} marker match for: {case}"));
            }
        }
    }

    let compiler_marker = MARKERS
        .iter()
        .find(|marker| marker.id == "compiler-runtime-crash-boundary")
        .ok_or("missing compiler-runtime-crash-boundary marker")?;
    let compiler_negative_cases = [
        "#define HAVE_SIGNAL_H 1",
        "#include <signal.h>",
        "char *strsignal(int sig);",
        "signal names are available in this bootstrap shell",
        "static int helper(void) { return 0; }",
    ];
    for case in compiler_negative_cases {
        let lower = case.to_lowercase();
        if compiler_marker.matches_line(&lower) {
            return Err(format!("self-test expected no compiler marker match for: {case}"));
        }
    }

    let evidence = EvidenceState::load();
    if evidence.binutils_tcc_tool_smoke_checked
        && suppression_reason(Path::new("bootstrap/binutils-tcc.ncl"), 382, MARKERS[0], &evidence).is_none()
    {
        return Err("self-test expected binutils-tcc bridge-output suppression with checked evidence".to_string());
    }
    if evidence.gcc40_placeholder_inventory_checked
        && suppression_reason(
            Path::new("bootstrap/evidence/gcc-4.0-placeholder-inventory.json"),
            9,
            MARKERS[0],
            &evidence,
        )
        .is_none()
    {
        return Err("self-test expected gcc-4.0 placeholder inventory bridge-output suppression".to_string());
    }
    if evidence.gcc40_native_boundary_checked
        && suppression_reason(Path::new("bootstrap/evidence/gcc-4.0-native-boundary.json"), 5, MARKERS[0], &evidence)
            .is_none()
    {
        return Err("self-test expected gcc-4.0 native boundary bridge-output suppression".to_string());
    }
    if evidence.gcc40_placeholder_inventory_checked
        && suppression_reason(Path::new("bootstrap/gcc-4.0.ncl"), 1255, MARKERS[0], &evidence).is_none()
    {
        return Err("self-test expected checked gcc-4.0 pass1 marker suppression".to_string());
    }
    if evidence.gcc40_native_boundary_checked
        && suppression_reason(Path::new("bootstrap/gcc-4.0.ncl"), 101, MARKERS[0], &evidence).is_none()
    {
        return Err("self-test expected checked gcc-4.0 mechanical bridge identifier suppression".to_string());
    }
    if is_gcc40_mechanical_bridge_identifier(1103) {
        return Err("self-test expected gcc-4.0 prose bridge blocker line to remain unsuppressed".to_string());
    }
    if evidence.gcc40_native_boundary_checked
        && suppression_reason(Path::new("bootstrap/diag-gcc40-c-parse-boundary.ncl"), 719, MARKERS[1], &evidence)
            .is_none()
    {
        return Err("self-test expected gcc-4.0 diagnostic boundary duplication suppression".to_string());
    }
    if suppression_reason(Path::new("openspec/specs/bootstrap/spec.md"), 2547, MARKERS[0], &evidence).is_none() {
        return Err("self-test expected bootstrap OpenSpec policy text suppression".to_string());
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, Default)]
struct EvidenceState {
    binutils_tcc_tool_smoke_checked: bool,
    gcc40_placeholder_inventory_checked: bool,
    gcc40_native_boundary_checked: bool,
}

impl EvidenceState {
    fn load() -> Self {
        Self {
            binutils_tcc_tool_smoke_checked: checked_binutils_tcc_tool_smoke(),
            gcc40_placeholder_inventory_checked: checked_gcc40_placeholder_inventory(),
            gcc40_native_boundary_checked: checked_gcc40_native_boundary(),
        }
    }
}

fn checked_evidence_file(path: &str, needles: &[&str]) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    needles.iter().all(|needle| content.contains(needle)).then_some(content)
}

fn checked_binutils_tcc_tool_smoke() -> bool {
    let Some(content) = checked_evidence_file("bootstrap/evidence/binutils-tcc-tool-smoke.json", &[
        "\"schema\": \"mantle-binutils-tcc-tool-smoke-v1\"",
        "\"derivation\": \"bootstrap/binutils-tcc.ncl\"",
        "\"host_fallback\": false",
        "\"fallback_markers\": []",
        "\"provider_kind\": \"source-root\"",
        "\"as\"",
        "\"ld\"",
        "\"ar\"",
        "\"ranlib\"",
        "\"nm\"",
        "\"objcopy\"",
    ]) else {
        return false;
    };
    content.matches("\"exit_status\": 0").count() >= 6
}

fn checked_gcc40_placeholder_inventory() -> bool {
    checked_evidence_file("bootstrap/evidence/gcc-4.0-placeholder-inventory.json", &[
        "\"schema\": \"mantle-gcc40-placeholder-inventory-v1\"",
        "\"derivation\": \"bootstrap/gcc-4.0.ncl\"",
        "\"status\": \"inventory-only\"",
        "\"marker_count\": 4",
        "\"classification\": \"pass1-generator-or-driver-bridge\"",
        "\"classification\": \"pass1-driver-boundary\"",
    ])
    .is_some()
}

fn checked_gcc40_native_boundary() -> bool {
    checked_evidence_file("bootstrap/evidence/gcc-4.0-native-boundary.json", &[
        "\"schema\": \"mantle-gcc40-native-boundary-v1\"",
        "\"derivation\": \"bootstrap/gcc-4.0.ncl\"",
        "\"status\": \"boundary-only\"",
        "\"boundary\": \"native-gcc-make-to-pass1-bridge\"",
        "\"id\": \"cc1-arithmetic-control-flow\"",
        "\"parity_effect\": \"evidence-backed partial; does not prove native gcc.4.0 correctness\"",
    ])
    .is_some()
}

fn suppression_reason(path: &Path, line: usize, marker: MarkerClass, evidence: &EvidenceState) -> Option<&'static str> {
    let path_s = path.to_string_lossy();
    if marker.id == "bridge-output"
        && path_s.ends_with("bootstrap/binutils-tcc.ncl")
        && evidence.binutils_tcc_tool_smoke_checked
    {
        return Some(
            "binutils-tcc has checked source-root tool-smoke evidence and remains partial, not unproved bridge output",
        );
    }
    if marker.id == "bridge-output"
        && path_s.ends_with("bootstrap/gcc-4.0.ncl")
        && evidence.gcc40_native_boundary_checked
        && is_gcc40_mechanical_bridge_identifier(line)
    {
        return Some("gcc-4.0 mechanical bridge identifier is covered by checked native-boundary metadata");
    }
    if marker.id == "bridge-output"
        && path_s.ends_with("bootstrap/gcc-4.0.ncl")
        && evidence.gcc40_placeholder_inventory_checked
        && matches!(line, 1255 | 1428 | 1443 | 1520)
    {
        return Some("gcc-4.0 pass1 marker is covered by the checked placeholder inventory receipt");
    }
    if marker.id == "bridge-output"
        && path_s.ends_with("bootstrap/evidence/gcc-4.0-placeholder-inventory.json")
        && evidence.gcc40_placeholder_inventory_checked
    {
        return Some("gcc-4.0 placeholder inventory is checked blocker metadata, not an additional bridge blocker");
    }
    if marker.id == "bridge-output"
        && path_s.ends_with("bootstrap/evidence/gcc-4.0-native-boundary.json")
        && evidence.gcc40_native_boundary_checked
    {
        return Some("gcc-4.0 native boundary receipt is checked boundary metadata, not an additional bridge blocker");
    }
    if matches!(marker.id, "bridge-output" | "compiler-runtime-crash-boundary")
        && path_s.ends_with("bootstrap/diag-gcc40-c-parse-boundary.ncl")
        && evidence.gcc40_native_boundary_checked
    {
        return Some(
            "gcc-4.0 c-parse diagnostic duplicates the checked native-boundary receipt; production blockers remain counted",
        );
    }
    if path_s.ends_with("openspec/specs/bootstrap/spec.md") {
        return Some(
            "bootstrap OpenSpec text is policy/control-plane metadata; source and evidence blockers remain counted",
        );
    }
    None
}

fn is_gcc40_mechanical_bridge_identifier(line: usize) -> bool {
    matches!(
        line,
        101 | 102
            | 103
            | 144
            | 146
            | 147
            | 148
            | 267
            | 280
            | 288
            | 300
            | 345
            | 352
            | 360
            | 376
            | 389
            | 395
            | 396
            | 401
            | 405
            | 406
            | 410
            | 413
    )
}

fn collect_files(paths: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    for path in paths {
        if path.is_file() {
            files.push(path.clone());
        } else if path.is_dir() {
            collect_dir(path, &mut files)?;
        } else {
            return Err(format!("path not found: {}", path.display()));
        }
    }
    files.sort();
    Ok(files)
}

fn collect_dir(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("read {}: {e}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("read entry in {}: {e}", dir.display()))?;
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name == ".git" || name == "target" || name == "vendor" || name == "archive" {
            continue;
        }
        if path.is_dir() {
            collect_dir(&path, files)?;
        } else if is_scanned_file(&path) {
            files.push(path);
        }
    }
    Ok(())
}

fn is_scanned_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|s| s.to_str()),
        Some("ncl" | "md" | "json" | "toml" | "yaml" | "yml" | "txt")
    )
}

fn scan_file(
    path: &Path,
    evidence: &EvidenceState,
    findings: &mut Vec<Finding>,
    suppressions: &mut Vec<Suppression>,
    promotion_claims: &mut Vec<PromotionClaim>,
) -> Result<(), String> {
    let content = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let path_s = path.display().to_string();
    for (idx, line) in content.lines().enumerate() {
        let lower = line.to_lowercase();
        for marker in MARKERS {
            if marker.matches_line(&lower) {
                if let Some(reason) = suppression_reason(path, idx + 1, *marker, evidence) {
                    suppressions.push(Suppression {
                        class_id: marker.id,
                        path: path_s.clone(),
                        line: idx + 1,
                        reason,
                        excerpt: compact(line),
                    });
                    continue;
                }
                findings.push(Finding {
                    class_id: marker.id,
                    path: path_s.clone(),
                    line: idx + 1,
                    excerpt: compact(line),
                });
            }
        }
        if PROMOTION_CLAIM_NEEDLES.iter().any(|needle| lower.contains(&needle.to_lowercase())) {
            promotion_claims.push(PromotionClaim {
                path: path_s.clone(),
                line: idx + 1,
                excerpt: compact(line),
            });
        }
    }
    Ok(())
}

fn compact(line: &str) -> String {
    let s = line.split_whitespace().collect::<Vec<_>>().join(" ");
    if s.len() > 180 { format!("{}…", &s[..180]) } else { s }
}

fn render_json(outcome: &Outcome) -> String {
    let mut s = String::new();
    s.push_str("{\n");
    s.push_str(&format!("  \"schema_version\": 1,\n  \"enforce\": {},\n", outcome.enforce));
    s.push_str("  \"summary\": {\n");
    s.push_str(&format!(
        "    \"finding_count\": {},\n    \"class_count\": {},\n    \"evidence_suppression_count\": {},\n    \"promotion_claim_count\": {}\n  }},\n",
        outcome.findings.len(),
        outcome.by_class.len(),
        outcome.suppressions.len(),
        outcome.promotion_claims.len()
    ));
    s.push_str("  \"classes\": [\n");
    for (i, marker) in MARKERS.iter().enumerate() {
        let count = outcome.by_class.get(marker.id).copied().unwrap_or(0);
        s.push_str(&format!(
            "    {{\"id\": \"{}\", \"description\": \"{}\", \"count\": {}}}{}\n",
            json_escape(marker.id),
            json_escape(marker.description),
            count,
            comma(i, MARKERS.len())
        ));
    }
    s.push_str("  ],\n  \"evidence_suppressions\": [\n");
    for (i, suppression) in outcome.suppressions.iter().enumerate() {
        s.push_str(&format!(
            "    {{\"class\": \"{}\", \"path\": \"{}\", \"line\": {}, \"reason\": \"{}\", \"excerpt\": \"{}\"}}{}\n",
            json_escape(suppression.class_id),
            json_escape(&suppression.path),
            suppression.line,
            json_escape(suppression.reason),
            json_escape(&suppression.excerpt),
            comma(i, outcome.suppressions.len())
        ));
    }
    s.push_str("  ],\n  \"promotion_claims\": [\n");
    for (i, claim) in outcome.promotion_claims.iter().enumerate() {
        s.push_str(&format!(
            "    {{\"path\": \"{}\", \"line\": {}, \"excerpt\": \"{}\"}}{}\n",
            json_escape(&claim.path),
            claim.line,
            json_escape(&claim.excerpt),
            comma(i, outcome.promotion_claims.len())
        ));
    }
    s.push_str("  ],\n  \"findings\": [\n");
    for (i, finding) in outcome.findings.iter().enumerate() {
        s.push_str(&format!(
            "    {{\"class\": \"{}\", \"path\": \"{}\", \"line\": {}, \"excerpt\": \"{}\"}}{}\n",
            json_escape(finding.class_id),
            json_escape(&finding.path),
            finding.line,
            json_escape(&finding.excerpt),
            comma(i, outcome.findings.len())
        ));
    }
    s.push_str("  ]\n}\n");
    s
}

fn render_markdown(outcome: &Outcome) -> String {
    let mut s = String::new();
    s.push_str("# Bootstrap blocker inventory report\n\n");
    s.push_str(&format!("- Schema version: 1\n- Enforcement mode: {}\n- Findings: {}\n- Marker classes present: {}\n- Evidence-backed suppressions: {}\n- Promotion claims: {}\n\n", outcome.enforce, outcome.findings.len(), outcome.by_class.len(), outcome.suppressions.len(), outcome.promotion_claims.len()));
    s.push_str("## Marker classes\n\n");
    for marker in MARKERS {
        let count = outcome.by_class.get(marker.id).copied().unwrap_or(0);
        s.push_str(&format!("- `{}`: {} finding(s) — {}\n", marker.id, count, marker.description));
    }
    s.push_str("\n## Evidence-backed suppressions\n\n");
    if outcome.suppressions.is_empty() {
        s.push_str("No evidence-backed suppressions applied.\n");
    } else {
        for suppression in &outcome.suppressions {
            s.push_str(&format!(
                "- `{}` {}:{} — {} — `{}`\n",
                suppression.class_id, suppression.path, suppression.line, suppression.reason, suppression.excerpt
            ));
        }
    }
    s.push_str("\n## Promotion claims\n\n");
    if outcome.promotion_claims.is_empty() {
        s.push_str("No promotion claims detected.\n");
    } else {
        for claim in &outcome.promotion_claims {
            s.push_str(&format!("- {}:{} — `{}`\n", claim.path, claim.line, claim.excerpt));
        }
    }
    s.push_str("\n## Findings\n\n");
    for finding in &outcome.findings {
        s.push_str(&format!("- `{}` {}:{} — `{}`\n", finding.class_id, finding.path, finding.line, finding.excerpt));
    }
    s
}

fn comma(index: usize, len: usize) -> &'static str {
    if index + 1 == len { "" } else { "," }
}

fn json_escape(input: &str) -> String {
    let mut out = String::new();
    for ch in input.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}
