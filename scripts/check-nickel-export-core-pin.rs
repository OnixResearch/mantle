//! Verify Mantle's standalone Nickel export core uses one immutable source.

use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

const REVISION: &str = "257fafc1c746f1faf156207043a4c826bfb16d49";
const REPOSITORY: &str = "https://github.com/OnixResearch/nickel-export";
const FLAKE_SOURCE: &str = "github:OnixResearch/nickel-export/257fafc1c746f1faf156207043a4c826bfb16d49";
const CARGO_DEPENDENCY: &str = "nickel-export-core = { git = \"https://github.com/OnixResearch/nickel-export\", rev = \"257fafc1c746f1faf156207043a4c826bfb16d49\" }";
const CARGO_LOCK_SOURCE: &str = "git+https://github.com/OnixResearch/nickel-export?rev=257fafc1c746f1faf156207043a4c826bfb16d49#257fafc1c746f1faf156207043a4c826bfb16d49";
const VENDOR_SOURCE_HEADER: &str =
    "[source.\"git+https://github.com/OnixResearch/nickel-export?rev=257fafc1c746f1faf156207043a4c826bfb16d49\"]";
const VENDOR_GIT: &str = "git = \"https://github.com/OnixResearch/nickel-export\"";
const VENDOR_REVISION: &str = "rev = \"257fafc1c746f1faf156207043a4c826bfb16d49\"";
const FLAKE_INPUT_MARKER: &str = "    nickelExportCore = {";
const FLAKE_INPUT_END: &str = "    };";
const FLAKE_LOCK_MARKER: &str = "\"nickelExportCore\": {";
const RELEASE_SCHEMA: &str = "mantle-nickel-export-core-source-v1";
const EXPECTED_SINGLE_MATCH: usize = 1;

struct PinMaterial<'a> {
    cargo_manifest: &'a str,
    cargo_lock: &'a str,
    vendor_config: &'a str,
    flake: &'a str,
    flake_lock: &'a str,
    release_nickel: &'a str,
    release_json: &'a str,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    if arguments.as_slice() == ["--self-test"] {
        return self_test();
    }
    let root = parse_root(&arguments)?;
    let owned = read_material(&root)?;
    let material = owned.as_borrowed();
    let issues = validate_pin_material(&material);
    if !issues.is_empty() {
        return Err(issues.join("\n"));
    }
    println!("nickel-export-core pin verified: repository={REPOSITORY} revision={REVISION}");
    Ok(())
}

fn parse_root(arguments: &[String]) -> Result<PathBuf, String> {
    match arguments {
        [] => env::current_dir().map_err(|error| format!("resolve current directory: {error}")),
        [flag, root] if flag == "--root" => Ok(PathBuf::from(root)),
        _ => Err("usage: check-nickel-export-core-pin.rs [--root PATH] | --self-test".to_string()),
    }
}

struct OwnedPinMaterial {
    cargo_manifest: String,
    cargo_lock: String,
    vendor_config: String,
    flake: String,
    flake_lock: String,
    release_nickel: String,
    release_json: String,
}

impl OwnedPinMaterial {
    fn as_borrowed(&self) -> PinMaterial<'_> {
        PinMaterial {
            cargo_manifest: &self.cargo_manifest,
            cargo_lock: &self.cargo_lock,
            vendor_config: &self.vendor_config,
            flake: &self.flake,
            flake_lock: &self.flake_lock,
            release_nickel: &self.release_nickel,
            release_json: &self.release_json,
        }
    }
}

fn read_material(root: &Path) -> Result<OwnedPinMaterial, String> {
    Ok(OwnedPinMaterial {
        cargo_manifest: read(root, "Cargo.toml")?,
        cargo_lock: read(root, "Cargo.lock")?,
        vendor_config: read(root, ".cargo/vendor-config.toml")?,
        flake: read(root, "flake.nix")?,
        flake_lock: read(root, "flake.lock")?,
        release_nickel: read(root, "config/nickel-export-core-source.ncl")?,
        release_json: read(root, "config/generated/nickel-export-core-source.json")?,
    })
}

fn read(root: &Path, relative: &str) -> Result<String, String> {
    let path = root.join(relative);
    fs::read_to_string(&path).map_err(|error| format!("read {}: {error}", path.display()))
}

fn validate_pin_material(material: &PinMaterial<'_>) -> Vec<String> {
    let mut issues = Vec::new();
    require_exact_count(material.cargo_manifest, CARGO_DEPENDENCY, "Cargo dependency", &mut issues);
    reject_tokens(
        material.cargo_manifest,
        &[
            "nickel-export-core = { path =",
            "nickel-export-core = { version =",
            "branch =",
            "tag =",
        ],
        "Cargo floating or path override",
        &mut issues,
    );
    require_exact_count(material.cargo_lock, CARGO_LOCK_SOURCE, "Cargo lock source", &mut issues);
    validate_vendor_config(material.vendor_config, &mut issues);
    validate_flake(material.flake, &mut issues);
    validate_flake_lock(material.flake_lock, &mut issues);
    validate_release_material(material.release_nickel, "Nickel release input", &mut issues);
    validate_release_material(material.release_json, "generated release input", &mut issues);
    issues
}

fn validate_vendor_config(vendor_config: &str, issues: &mut Vec<String>) {
    require_exact_count(vendor_config, VENDOR_SOURCE_HEADER, "vendored Cargo source", issues);
    require_exact_count(vendor_config, VENDOR_GIT, "vendored Cargo repository", issues);
    require_exact_count(vendor_config, VENDOR_REVISION, "vendored Cargo revision", issues);
    require_contains(vendor_config, "replace-with = \"vendored-sources\"", "vendored Cargo replacement", issues);
    reject_tokens(
        vendor_config,
        &["nickel-export?branch=", "nickel-export?tag=", "path ="],
        "vendored Cargo floating or path override",
        issues,
    );
}

fn validate_flake(flake: &str, issues: &mut Vec<String>) {
    let Some(block) = delimited_block(flake, FLAKE_INPUT_MARKER, FLAKE_INPUT_END) else {
        issues.push("Nix input block is missing".to_string());
        return;
    };
    require_contains(block, FLAKE_SOURCE, "Nix exact source", issues);
    require_contains(block, "flake = false;", "Nix non-flake source", issues);
    reject_tokens(block, &["path:", "?ref=", "?rev="], "Nix floating or path override", issues);
}

fn validate_flake_lock(flake_lock: &str, issues: &mut Vec<String>) {
    let Some(block) = braced_block(flake_lock, FLAKE_LOCK_MARKER) else {
        issues.push("flake.lock nickelExportCore node is missing".to_string());
        return;
    };
    require_contains(block, REVISION, "flake.lock revision", issues);
    require_contains(block, "\"owner\": \"OnixResearch\"", "flake.lock owner", issues);
    require_contains(block, "\"repo\": \"nickel-export\"", "flake.lock repository", issues);
    require_contains(block, "\"type\": \"github\"", "flake.lock source type", issues);
    reject_tokens(block, &["\"type\": \"path\"", "\"ref\":"], "flake.lock floating or path override", issues);
}

fn validate_release_material(text: &str, label: &str, issues: &mut Vec<String>) {
    require_contains(text, RELEASE_SCHEMA, &format!("{label} schema"), issues);
    require_contains(text, REPOSITORY, &format!("{label} repository"), issues);
    require_contains(text, REVISION, &format!("{label} revision"), issues);
    require_contains(text, FLAKE_SOURCE, &format!("{label} Nix source"), issues);
    reject_tokens(
        text,
        &[
            "path_override_allowed = true",
            "\"path_override_allowed\": true",
            "floating_ref_allowed = true",
            "\"floating_ref_allowed\": true",
        ],
        &format!("{label} override policy"),
        issues,
    );
}

fn require_exact_count(text: &str, needle: &str, label: &str, issues: &mut Vec<String>) {
    let count = text.matches(needle).count();
    if count != EXPECTED_SINGLE_MATCH {
        issues.push(format!("{label} count is {count}, expected {EXPECTED_SINGLE_MATCH}"));
    }
}

fn require_contains(text: &str, needle: &str, label: &str, issues: &mut Vec<String>) {
    if !text.contains(needle) {
        issues.push(format!("{label} is missing"));
    }
}

fn reject_tokens(text: &str, tokens: &[&str], label: &str, issues: &mut Vec<String>) {
    for token in tokens {
        if text.contains(token) {
            issues.push(format!("{label} contains forbidden token `{token}`"));
        }
    }
}

fn delimited_block<'a>(text: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let start_offset = text.find(start)?;
    let tail = &text[start_offset..];
    let end_offset = tail.find(end)?;
    Some(&tail[..end_offset + end.len()])
}

fn braced_block<'a>(text: &'a str, marker: &str) -> Option<&'a str> {
    let start = text.find(marker)?;
    let tail = &text[start..];
    let mut depth = 0_u32;
    for (offset, character) in tail.char_indices() {
        if character == '{' {
            depth = depth.checked_add(1)?;
        } else if character == '}' {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(&tail[..=offset]);
            }
        }
    }
    None
}

fn self_test() -> Result<(), String> {
    let valid = valid_fixture();
    if !validate_pin_material(&valid.as_borrowed()).is_empty() {
        return Err("positive pin fixture failed".to_string());
    }
    assert_rejected(
        valid.with_cargo_manifest("nickel-export-core = { path = \"../nickel-export\" }"),
        "Cargo path override",
    )?;
    assert_rejected(
        valid.with_cargo_manifest(
            "nickel-export-core = { git = \"https://github.com/OnixResearch/nickel-export\", branch = \"main\" }",
        ),
        "Cargo branch",
    )?;
    assert_rejected(
        valid.with_cargo_lock("source = \"git+https://github.com/OnixResearch/nickel-export#deadbeef\""),
        "lock drift",
    )?;
    assert_rejected(
        valid.with_vendor_config(
            "[source.\"git+https://github.com/OnixResearch/nickel-export?branch=main\"]\npath = \"../nickel-export\"",
        ),
        "vendored Cargo path or branch override",
    )?;
    assert_rejected(
        valid.with_flake(
            "    nickelExportCore = {\n      url = \"path:../nickel-export\";\n      flake = false;\n    };",
        ),
        "Nix path override",
    )?;
    assert_rejected(valid.with_release_nickel("path_override_allowed = true"), "release path override")?;
    println!("nickel-export-core pin self-test passed");
    Ok(())
}

fn assert_rejected(material: OwnedPinMaterial, label: &str) -> Result<(), String> {
    if validate_pin_material(&material.as_borrowed()).is_empty() {
        return Err(format!("negative fixture `{label}` was accepted"));
    }
    Ok(())
}

fn valid_fixture() -> OwnedPinMaterial {
    OwnedPinMaterial {
        cargo_manifest: CARGO_DEPENDENCY.to_string(),
        cargo_lock: CARGO_LOCK_SOURCE.to_string(),
        vendor_config: format!(
            "{VENDOR_SOURCE_HEADER}\n{VENDOR_GIT}\n{VENDOR_REVISION}\nreplace-with = \"vendored-sources\""
        ),
        flake: format!(
            "{FLAKE_INPUT_MARKER}\n      url = \"{FLAKE_SOURCE}\";\n      flake = false;\n{FLAKE_INPUT_END}"
        ),
        flake_lock: format!(
            "{{\n  {FLAKE_LOCK_MARKER}\n    \"locked\": {{ \"owner\": \"OnixResearch\", \"repo\": \"nickel-export\", \"rev\": \"{REVISION}\", \"type\": \"github\" }}\n  }}\n}}"
        ),
        release_nickel: format!(
            "{RELEASE_SCHEMA}\n{REPOSITORY}\n{REVISION}\n{FLAKE_SOURCE}\npath_override_allowed = false\nfloating_ref_allowed = false"
        ),
        release_json: format!(
            "{RELEASE_SCHEMA}\n{REPOSITORY}\n{REVISION}\n{FLAKE_SOURCE}\n\"path_override_allowed\": false\n\"floating_ref_allowed\": false"
        ),
    }
}

impl OwnedPinMaterial {
    fn with_cargo_manifest(&self, cargo_manifest: &str) -> Self {
        Self {
            cargo_manifest: cargo_manifest.to_string(),
            ..self.clone()
        }
    }

    fn with_cargo_lock(&self, cargo_lock: &str) -> Self {
        Self {
            cargo_lock: cargo_lock.to_string(),
            ..self.clone()
        }
    }

    fn with_vendor_config(&self, vendor_config: &str) -> Self {
        Self {
            vendor_config: vendor_config.to_string(),
            ..self.clone()
        }
    }

    fn with_flake(&self, flake: &str) -> Self {
        Self {
            flake: flake.to_string(),
            ..self.clone()
        }
    }

    fn with_release_nickel(&self, release_nickel: &str) -> Self {
        Self {
            release_nickel: release_nickel.to_string(),
            ..self.clone()
        }
    }
}

impl Clone for OwnedPinMaterial {
    fn clone(&self) -> Self {
        Self {
            cargo_manifest: self.cargo_manifest.clone(),
            cargo_lock: self.cargo_lock.clone(),
            vendor_config: self.vendor_config.clone(),
            flake: self.flake.clone(),
            flake_lock: self.flake_lock.clone(),
            release_nickel: self.release_nickel.clone(),
            release_json: self.release_json.clone(),
        }
    }
}
