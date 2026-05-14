#!/usr/bin/env -S cargo +nightly -Zscript
---
[dependencies]
---

//! Audit bootstrap .ncl files for source-pin completeness.
//!
//! Validates that every `mantle.fetchTarball` and `mantle.fetchGit` block has
//! all required fields (url, hash, name; plus rev for fetchGit) and that hash
//! values use valid SRI format (e.g. `sha256-<base64>`).

use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

const REQUIRED_TARBALL_FIELDS: &[&str] = &["url", "hash", "name"];
const REQUIRED_GIT_FIELDS: &[&str] = &["url", "rev", "hash", "name"];
const VALID_SRI_PREFIXES: &[&str] = &["sha256-", "sha512-", "blake3-"];

#[derive(Debug)]
struct FetchBlock {
    file: String,
    line: usize,
    kind: FetchKind,
    fields: Vec<(String, String)>,
}

#[derive(Debug, Clone, Copy)]
enum FetchKind {
    Tarball,
    Git,
}

impl FetchKind {
    fn label(self) -> &'static str {
        match self {
            FetchKind::Tarball => "fetchTarball",
            FetchKind::Git => "fetchGit",
        }
    }

    fn required_fields(self) -> &'static [&'static str] {
        match self {
            FetchKind::Tarball => REQUIRED_TARBALL_FIELDS,
            FetchKind::Git => REQUIRED_GIT_FIELDS,
        }
    }
}

#[derive(Debug)]
struct Issue {
    file: String,
    line: usize,
    kind: String,
    message: String,
}

fn extract_fetch_blocks(path: &Path) -> Vec<FetchBlock> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let file = path.display().to_string();
    let lines: Vec<&str> = content.lines().collect();
    let mut blocks = Vec::new();

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        let fetch_kind = if trimmed.contains("mantle.fetchTarball")
            || trimmed.contains("crunch.fetchTarball")
            || trimmed.contains("fetchTarball {")
        {
            Some(FetchKind::Tarball)
        } else if trimmed.contains("mantle.fetchGit")
            || trimmed.contains("crunch.fetchGit")
            || trimmed.contains("fetchGit {")
        {
            Some(FetchKind::Git)
        } else {
            None
        };

        if let Some(kind) = fetch_kind {
            let block_start = i + 1;
            let mut fields = Vec::new();

            let mut j = i;
            while j < lines.len() {
                let bl = lines[j].trim();

                if let Some(field) = parse_field(bl) {
                    fields.push(field);
                }

                if bl.contains("} in") || (bl == "}" && j > i) {
                    break;
                }
                j += 1;
            }

            blocks.push(FetchBlock {
                file: file.clone(),
                line: block_start,
                kind,
                fields,
            });

            i = j + 1;
        } else {
            i += 1;
        }
    }

    blocks
}

fn parse_field(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim().trim_end_matches(',');

    let eq_pos = trimmed.find('=')?;
    let key = trimmed[..eq_pos].trim().to_string();
    let raw_val = trimmed[eq_pos + 1..].trim();

    if key.is_empty() || raw_val.is_empty() {
        return None;
    }

    let val = raw_val.trim_matches('"').trim_matches(',').to_string();

    if val.is_empty() {
        return None;
    }

    Some((key, val))
}

fn validate_block(block: &FetchBlock) -> Vec<Issue> {
    let mut issues = Vec::new();
    let required = block.kind.required_fields();
    let label = block.kind.label();

    let field_names: Vec<&str> = block.fields.iter().map(|(k, _)| k.as_str()).collect();

    for &req in required {
        if !field_names.contains(&req) {
            issues.push(Issue {
                file: block.file.clone(),
                line: block.line,
                kind: format!("missing-field/{label}"),
                message: format!("{label} block missing required field '{req}'"),
            });
        }
    }

    for (key, val) in &block.fields {
        if key == "hash" {
            let valid = VALID_SRI_PREFIXES.iter().any(|p| val.starts_with(p));
            if !valid {
                issues.push(Issue {
                    file: block.file.clone(),
                    line: block.line,
                    kind: format!("bad-hash-format/{label}"),
                    message: format!("{label} hash is not SRI format: '{}'", &val[..val.len().min(40)]),
                });
            }

            if val.starts_with("sha256-") {
                let b64 = &val["sha256-".len()..];
                if b64.len() < 40 || b64.len() > 48 {
                    issues.push(Issue {
                        file: block.file.clone(),
                        line: block.line,
                        kind: format!("bad-hash-length/{label}"),
                        message: format!("{label} sha256 base64 length {} looks wrong (expected ~44)", b64.len()),
                    });
                }
            }
        }

        if key == "url" && val.is_empty() {
            issues.push(Issue {
                file: block.file.clone(),
                line: block.line,
                kind: format!("empty-url/{label}"),
                message: format!("{label} has empty url"),
            });
        }

        if key == "rev" && val.len() != 40 {
            issues.push(Issue {
                file: block.file.clone(),
                line: block.line,
                kind: format!("bad-rev-length/{label}"),
                message: format!("{label} rev length {} (expected 40 hex chars)", val.len()),
            });
        }
    }

    issues
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!("usage: check-bootstrap-source-pins.rs <file.ncl>...");
        return ExitCode::from(2);
    }

    let mut total_blocks = 0u32;
    let mut total_issues = 0u32;
    let mut all_issues: Vec<Issue> = Vec::new();
    let mut file_count = 0u32;

    for arg in &args {
        let path = Path::new(arg);
        if !path.exists() {
            eprintln!("warning: file not found: {arg}");
            continue;
        }

        file_count += 1;
        let blocks = extract_fetch_blocks(path);
        total_blocks += blocks.len() as u32;

        for block in &blocks {
            let mut issues = validate_block(block);
            total_issues += issues.len() as u32;
            all_issues.append(&mut issues);
        }
    }

    if !all_issues.is_empty() {
        eprintln!("--- source pin issues ---");
        for issue in &all_issues {
            eprintln!("FAIL  {}:{}: [{}] {}", issue.file, issue.line, issue.kind, issue.message);
        }
        eprintln!("---");
    }

    println!("source-pin audit: {file_count} files, {total_blocks} fetch blocks, {total_issues} issues");

    if total_issues > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}
