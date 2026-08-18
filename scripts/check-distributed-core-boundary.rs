//! r[impl realization_routing.distributed_core_boundary]
//!
//! Static guard for the pure distributed decision core boundary.
//!
//! The guard proves that `crates/crunch-build/src/distributed/core.rs` stays
//! infrastructure-free. A future regression that imports a vendor type, an
//! async runtime, or a host effect into the core fails this guard before any
//! runtime test can report false confidence.
//!
//! Run from the repo root:
//!   nix develop -c cargo -Zscript scripts/check-distributed-core-boundary.rs
//!
//! Self-check:
//!   scripts/check-distributed-core-boundary.rs --self-test

const CORE_PATH: &str = "crates/crunch-build/src/distributed/core.rs";

/// Tokens that must never appear in the pure core. Each entry carries the
/// reason it would break the boundary.
const FORBIDDEN_TOKENS: &[(&str, &str)] = &[
    // Vendor and store types.
    ("snix", "vendor Snix type or crate reached the core"),
    ("PathInfo", "store PathInfo type reached the core"),
    ("StoreHandle", "store handle reached the core"),
    ("nix_compat", "vendor Nix-compat type reached the core"),
    // Async runtime and provider ports.
    ("async fn", "async call reached the core"),
    ("async_trait", "async provider port reached the core"),
    ("tokio::", "tokio runtime reached the core"),
    ("futures::", "futures type reached the core"),
    // Host effects.
    ("std::fs", "filesystem I/O reached the core"),
    ("std::env", "environment read reached the core"),
    ("std::process", "process spawn reached the core"),
    ("std::thread", "thread spawn reached the core"),
    ("std::time::Instant", "clock read reached the core"),
    ("std::time::SystemTime", "clock read reached the core"),
    ("std::net::", "network I/O reached the core"),
    ("println!", "printing reached the core"),
    ("eprintln!", "printing reached the core"),
];

/// Markers the core must carry so Tracey can see the implementation edge.
const REQUIRED_MARKERS: &[&str] = &[
    "r[impl realization_routing.distributed_core_boundary]",
    "r[verify realization_routing.distributed_core_boundary]",
];

/// Strip `//` line comments and `/* */` block comments so prose that *names*
/// a forbidden concept does not count as a real reference. The markers check
/// runs on the raw source because those markers are comments.
fn strip_comments(source: &str) -> String {
    let mut result_lines: Vec<String> = Vec::with_capacity(source.lines().count());
    let mut in_block_comment = false;
    for line in source.lines() {
        let line = line.trim_end();
        if in_block_comment {
            if let Some(end) = line.find("*/") {
                in_block_comment = false;
                result_lines.push(line[end + 2..].to_string());
            }
            continue;
        }
        let line = match line.find("/*") {
            Some(block_start) => {
                if let Some(block_end) = line[block_start + 2..].find("*/") {
                    let end = block_start + 2 + block_end + 2;
                    let mut cleaned = line[..block_start].to_string();
                    cleaned.push_str(&line[end..]);
                    cleaned
                } else {
                    in_block_comment = true;
                    line[..block_start].to_string()
                }
            }
            None => line.to_string(),
        };
        let line = match line_comment_start(&line) {
            Some(comment_start) => line[..comment_start].to_string(),
            None => line,
        };
        result_lines.push(line);
    }
    result_lines.join("\n")
}

/// Find a `//` that is not part of a string literal. This simple scanner
/// tracks double-quoted strings; the guarded module never uses raw strings or
/// char literals containing `//`.
fn line_comment_start(line: &str) -> Option<usize> {
    let mut in_string = false;
    let bytes = line.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' if in_string => index += 2,
            b'"' => {
                in_string = !in_string;
                index += 1;
            }
            b'/' if !in_string && index + 1 < bytes.len() && bytes[index + 1] == b'/' => return Some(index),
            _ => index += 1,
        }
    }
    None
}

fn check_source(source: &str) -> Vec<String> {
    let mut findings: Vec<String> = Vec::new();
    for marker in REQUIRED_MARKERS {
        if !source.contains(marker) {
            findings.push(format!("core is missing marker `{marker}`"));
        }
    }
    let code = strip_comments(source);
    for (token, reason) in FORBIDDEN_TOKENS {
        if code.contains(token) {
            findings.push(format!("core contains `{token}`: {reason}"));
        }
    }
    debug_assert!(findings.len() <= REQUIRED_MARKERS.len() + FORBIDDEN_TOKENS.len(), "findings stay bounded");
    findings
}

fn check_file(path: &str) -> Result<(), String> {
    let source =
        std::fs::read_to_string(path).map_err(|error| format!("read {path}: {error}"))?;
    let source = source.trim();
    if source.is_empty() {
        return Err(format!("{path} is empty"));
    }
    let findings = check_source(source);
    if findings.is_empty() {
        let line_count = source.lines().count();
        println!("core-boundary-ok path={path} lines={line_count}");
        Ok(())
    } else {
        for finding in &findings {
            println!("core-boundary-finding {finding}");
        }
        Err(format!("core boundary violated: {} finding(s)", findings.len()))
    }
}

const POSITIVE_FIXTURE: &str = r#"
// r[impl realization_routing.distributed_core_boundary]
// r[verify realization_routing.distributed_core_boundary]
// A clean pure core: no vendor, async, or host-effect tokens.
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SampleFacts {
    pub count: u32,
}

pub fn decide(facts: &SampleFacts) -> bool {
    debug_assert!(facts.count > 0);
    facts.count % 2 == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decides() {
        assert!(decide(&SampleFacts { count: 2 }));
    }
}
"#;

const NEGATIVE_VENDOR_FIXTURE: &str = r#"
// A pathological core that imports a vendor Snix type.
use serde::Deserialize;
use serde::Serialize;
use snix_build::buildservice::BuildRequest;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SampleFacts {
    pub count: u32,
}

pub fn decide(facts: &SampleFacts, request: &BuildRequest) -> bool {
    facts.count > 0 && !request.command_args.is_empty()
}
"#;

const NEGATIVE_ASYNC_FIXTURE: &str = r#"
// A pathological core that awaits a provider port.
pub async fn decide(facts: &SampleFacts) -> bool {
    facts.count > 0
}
"#;

const NEGATIVE_EFFECT_FIXTURE: &str = r#"
// A pathological core that reads the process environment.
pub fn decide(facts: &SampleFacts) -> bool {
    let home = std::env::var("HOME").unwrap_or_default();
    facts.count > 0 && !home.is_empty()
}
"#;

/// The fixtures must classify exactly as expected: the positive passes, and
/// every negative fails on its named token.
fn run_self_test() -> Result<(), String> {
    let good = check_source(POSITIVE_FIXTURE);
    if !good.is_empty() {
        return Err(format!("positive fixture failed: {good:?}"));
    }
    let vendor = check_source(NEGATIVE_VENDOR_FIXTURE);
    if !vendor.iter().any(|finding| finding.contains("snix")) {
        return Err("negative vendor fixture must report the snix token".to_string());
    }
    let async_fixture = check_source(NEGATIVE_ASYNC_FIXTURE);
    if !async_fixture.iter().any(|finding| finding.contains("async fn")) {
        return Err("negative async fixture must report the async token".to_string());
    }
    let effect = check_source(NEGATIVE_EFFECT_FIXTURE);
    if !effect.iter().any(|finding| finding.contains("std::env")) {
        return Err("negative effect fixture must report the environment token".to_string());
    }
    println!("self-test-ok");
    Ok(())
}

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--self-test") {
        return match run_self_test() {
            Ok(()) => std::process::ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("error: {error}");
                std::process::ExitCode::from(2)
            }
        };
    }
    match check_file(CORE_PATH) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            std::process::ExitCode::from(2)
        }
    }
}
