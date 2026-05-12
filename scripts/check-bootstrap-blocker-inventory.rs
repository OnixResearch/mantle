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
                "bootstrap blocker inventory: {} findings across {} classes, {} promotion claims, enforce={}",
                outcome.findings.len(),
                outcome.by_class.len(),
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
    let mut promotion_claims = Vec::new();

    for file in files {
        scan_file(&file, &mut findings, &mut promotion_claims)?;
    }

    findings
        .sort_by(|a, b| (&a.class_id, &a.path, a.line, &a.excerpt).cmp(&(&b.class_id, &b.path, b.line, &b.excerpt)));
    promotion_claims.sort_by(|a, b| (&a.path, a.line, &a.excerpt).cmp(&(&b.path, b.line, &b.excerpt)));

    let mut by_class: BTreeMap<&'static str, usize> = BTreeMap::new();
    for finding in &findings {
        *by_class.entry(finding.class_id).or_insert(0) += 1;
    }

    let outcome = Outcome {
        findings,
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

    Ok(())
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
    findings: &mut Vec<Finding>,
    promotion_claims: &mut Vec<PromotionClaim>,
) -> Result<(), String> {
    let content = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let path_s = path.display().to_string();
    for (idx, line) in content.lines().enumerate() {
        let lower = line.to_lowercase();
        for marker in MARKERS {
            if marker.matches_line(&lower) {
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
        "    \"finding_count\": {},\n    \"class_count\": {},\n    \"promotion_claim_count\": {}\n  }},\n",
        outcome.findings.len(),
        outcome.by_class.len(),
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
    s.push_str(&format!("- Schema version: 1\n- Enforcement mode: {}\n- Findings: {}\n- Marker classes present: {}\n- Promotion claims: {}\n\n", outcome.enforce, outcome.findings.len(), outcome.by_class.len(), outcome.promotion_claims.len()));
    s.push_str("## Marker classes\n\n");
    for marker in MARKERS {
        let count = outcome.by_class.get(marker.id).copied().unwrap_or(0);
        s.push_str(&format!("- `{}`: {} finding(s) — {}\n", marker.id, count, marker.description));
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
