//! Pure validation and reviewable decisions for bootstrap source pins.
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

pub const PIN_SCHEMA: &str = "mantle-bootstrap-pin-v1";
pub const PLAN_SCHEMA: &str = "mantle-bootstrap-pin-plan-v1";
const MAX_PIN_BYTES: usize = 65_536;
const MAX_PINS: usize = 256;
const MAX_ARTIFACTS: usize = 8;
const MAX_RECIPES: usize = 16;
const MAX_URL_BYTES: usize = 2048;
const MAX_VERSION_BYTES: usize = 80;
const MAX_VERSION_SEGMENTS: u32 = 64;
const MUTATED_PLAN: &str = "mutated or incomplete bootstrap pin plan";
const RECORD_KEYS: &str = "schema, source, package_url, version, release_date, artifacts, resolve, recipes";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BootstrapPin {
    pub schema: String,
    pub source: String,
    pub package_url: String,
    pub version: String,
    pub release_date: String,
    pub artifacts: BTreeMap<String, PinArtifact>,
    pub resolve: ResolveHook,
    pub recipes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PinArtifact {
    pub url_template: String,
    pub url: String,
    pub hash_kind: HashKind,
    pub hash: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HashKind {
    Flat,
    Tree,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ResolveHook {
    Json {
        url: String,
        version_field: String,
        date_field: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PinCandidate {
    pub version: String,
    pub release_date: String,
    pub hashes: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum PinDecision {
    Current,
    Candidate { candidate: PinCandidate },
    Error { message: String },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PinPlanEntry {
    pub source: String,
    pub preimage: String,
    pub current: String,
    pub decision: PinDecision,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PinPlan {
    pub schema: String,
    pub entries: Vec<PinPlanEntry>,
    pub digest: String,
}

// r[impl mantle.bootstrap_source_pins.pin_data_contract]
pub fn parse_pin(bytes: &[u8]) -> Result<BootstrapPin, String> {
    if bytes.is_empty() || bytes.len() > MAX_PIN_BYTES {
        return Err(format!("pin record must contain 1..={MAX_PIN_BYTES} bytes"));
    }
    let pin: BootstrapPin = serde_json::from_slice(bytes)
        .map_err(|e| format!("invalid pin record ({e}); known fields: {RECORD_KEYS}; artifact fields: url_template, url, hash_kind, hash; resolve fields: kind, url, version_field, date_field"))?;
    validate_pin(&pin)?;
    Ok(pin)
}

pub fn validate_pin(pin: &BootstrapPin) -> Result<(), String> {
    if pin.schema != PIN_SCHEMA {
        return Err(format!("pin schema must be {PIN_SCHEMA}"));
    }
    if !valid_source(&pin.source) || !pin.package_url.starts_with("pkg:") || pin.package_url.len() > 512 {
        return Err(format!("{}: invalid source or package URL identity", pin.source));
    }
    if !valid_version(&pin.version) || !valid_date(&pin.release_date) {
        return Err(format!("{}: invalid version or release date", pin.source));
    }
    if pin.artifacts.is_empty() || pin.artifacts.len() > MAX_ARTIFACTS {
        return Err(format!("{}: requires 1..={MAX_ARTIFACTS} artifacts", pin.source));
    }
    for (role, artifact) in &pin.artifacts {
        if !valid_artifact(role, artifact, &pin.version)? {
            return Err(format!("{}: invalid artifact {role} URL template or sha256 hash", pin.source));
        }
    }
    if pin.recipes.is_empty()
        || pin.recipes.len() > MAX_RECIPES
        || pin.recipes.iter().any(|name| !valid_source(name) || !name.ends_with(".ncl"))
    {
        return Err(format!("{}: requires 1..={MAX_RECIPES} declared bootstrap recipe filenames", pin.source));
    }
    if !valid_resolve_hook(&pin.resolve) {
        return Err(format!("{}: invalid JSON resolution hook", pin.source));
    }
    assert!(!pin.artifacts.is_empty(), "validated pin must declare an artifact");
    assert!(pin.recipes.len() <= MAX_RECIPES, "validated pin recipe list must stay bounded");
    Ok(())
}

fn valid_artifact(role: &str, artifact: &PinArtifact, version: &str) -> Result<bool, String> {
    if !valid_source(role) || !valid_sri(&artifact.hash) {
        return Ok(false);
    }
    let template = &artifact.url_template;
    if !template.starts_with("https://") || template.len() > MAX_URL_BYTES {
        return Ok(false);
    }
    if !template.contains("{version}") {
        return Ok(false);
    }
    Ok(render_pin_url(artifact, version)? == artifact.url)
}

fn valid_resolve_hook(hook: &ResolveHook) -> bool {
    let ResolveHook::Json {
        url,
        version_field,
        date_field,
    } = hook;
    if !url.starts_with("https://") || url.len() > MAX_URL_BYTES {
        return false;
    }
    valid_source(version_field) && valid_source(date_field)
}

pub fn render_pin_url(artifact: &PinArtifact, version: &str) -> Result<String, String> {
    if !valid_version(version) {
        return Err("invalid candidate version".to_string());
    }
    let url = artifact.url_template.replace("{version}", version);
    if !url.starts_with("https://") || url.len() > MAX_URL_BYTES {
        return Err("rendered pin URL is invalid".to_string());
    }
    Ok(url)
}

pub fn valid_sri(hash: &str) -> bool {
    let Some(encoded) = hash.strip_prefix("sha256-") else {
        return false;
    };
    encoded.len() == 44
        && encoded.ends_with('=')
        && encoded[..43].bytes().all(|b| b.is_ascii_alphanumeric() || b == b'+' || b == b'/')
}

fn valid_source(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-' || b == b'_')
}
fn valid_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes.iter().enumerate().all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit())
        && &value[5..7] >= "01"
        && &value[5..7] <= "12"
        && &value[8..] >= "01"
        && &value[8..] <= "31"
}
fn valid_version(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_VERSION_BYTES
        && value.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-' || b == b'_')
}

/// Current and candidate release versions, named so call sites cannot swap them.
#[derive(Clone, Copy, Debug)]
pub struct VersionPair<'a> {
    pub current: &'a str,
    pub candidate: &'a str,
}

/// Compare numeric segments numerically and remaining segments lexically.
/// At most `MAX_VERSION_SEGMENTS` segments are compared; longer equal prefixes compare equal.
pub fn compare_versions(pair: VersionPair<'_>) -> core::cmp::Ordering {
    let mut current_parts = pair.current.split(['.', '-', '_']);
    let mut candidate_parts = pair.candidate.split(['.', '-', '_']);
    for _ in 0..MAX_VERSION_SEGMENTS {
        match (current_parts.next(), candidate_parts.next()) {
            (Some(left), Some(right)) => {
                assert!(left.len() <= pair.current.len(), "segment lies within the current version");
                assert!(right.len() <= pair.candidate.len(), "segment lies within the candidate version");
                let cmp = match (left.parse::<u64>(), right.parse::<u64>()) {
                    (Ok(x), Ok(y)) => x.cmp(&y),
                    _ => left.cmp(right),
                };
                if !cmp.is_eq() {
                    return cmp;
                }
            }
            (None, Some(_)) => return core::cmp::Ordering::Less,
            (Some(_), None) => return core::cmp::Ordering::Greater,
            (None, None) => return core::cmp::Ordering::Equal,
        }
    }
    core::cmp::Ordering::Equal
}

pub fn decide_pin(pin: &BootstrapPin, observation: Result<PinCandidate, String>) -> PinDecision {
    assert!(!pin.artifacts.is_empty(), "decisions require a validated pin with artifacts");
    let candidate = match observation {
        Ok(candidate) => candidate,
        Err(message) => return PinDecision::Error { message },
    };
    if !valid_version(&candidate.version) || !valid_date(&candidate.release_date) {
        return PinDecision::Error {
            message: "invalid upstream version or date".to_string(),
        };
    }
    let pair = VersionPair {
        current: &pin.version,
        candidate: &candidate.version,
    };
    if !compare_versions(pair).is_lt() {
        return PinDecision::Current;
    }
    let is_complete = candidate.hashes.len() == pin.artifacts.len()
        && pin.artifacts.keys().all(|role| candidate.hashes.get(role).is_some_and(|hash| valid_sri(hash)));
    if !is_complete {
        return PinDecision::Error {
            message: "candidate requires an observed sha256 hash for every artifact".to_string(),
        };
    }
    assert_eq!(candidate.hashes.len(), pin.artifacts.len(), "candidate covers every artifact");
    PinDecision::Candidate { candidate }
}

pub fn pin_preimage(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

pub fn seal_plan(mut entries: Vec<PinPlanEntry>) -> Result<PinPlan, String> {
    if entries.is_empty() || entries.len() > MAX_PINS {
        return Err(format!("plan requires 1..={MAX_PINS} sources"));
    }
    entries.sort_by(|a, b| a.source.cmp(&b.source));
    if entries.windows(2).any(|pair| pair[0].source == pair[1].source) {
        return Err("duplicate plan source".to_string());
    }
    let digest = plan_digest(&entries)?;
    Ok(PinPlan {
        schema: PLAN_SCHEMA.to_string(),
        entries,
        digest,
    })
}
fn plan_digest(entries: &[PinPlanEntry]) -> Result<String, String> {
    let bytes = serde_json::to_vec(&(PLAN_SCHEMA, entries)).map_err(|e| format!("serialize plan: {e}"))?;
    Ok(pin_preimage(&bytes))
}

// r[impl mantle.bootstrap_source_pins.apply_writes_pins_only]
pub fn verify_plan(plan: &PinPlan, pins: &BTreeMap<String, (BootstrapPin, String)>) -> Result<(), String> {
    if plan.schema != PLAN_SCHEMA || plan.entries.is_empty() {
        return Err(MUTATED_PLAN.to_string());
    }
    if plan.entries.len() != pins.len() || plan.digest != plan_digest(&plan.entries)? {
        return Err(MUTATED_PLAN.to_string());
    }
    assert_eq!(plan.entries.len(), pins.len(), "plan covers every declared pin");
    for pair in plan.entries.windows(2) {
        if pair[0].source >= pair[1].source {
            return Err("duplicate or unsorted plan sources".to_string());
        }
    }
    for entry in &plan.entries {
        verify_plan_entry(entry, pins)?;
    }
    assert!(plan.entries.iter().all(|entry| pins.contains_key(&entry.source)), "every plan source is declared");
    Ok(())
}

fn verify_plan_entry(entry: &PinPlanEntry, pins: &BTreeMap<String, (BootstrapPin, String)>) -> Result<(), String> {
    let (pin, preimage) = pins.get(&entry.source).ok_or_else(|| format!("unknown source {}", entry.source))?;
    assert_eq!(pin.source, entry.source, "pin map is keyed by source");
    assert!(!preimage.is_empty(), "loaded pins carry a preimage digest");
    if &entry.preimage != preimage || entry.current != pin.version {
        return Err(format!("{}: source pin changed since check", entry.source));
    }
    match &entry.decision {
        PinDecision::Error { .. } => {
            Err(format!("{}: unresolved upstream observation; apply requires every source resolved", entry.source))
        }
        PinDecision::Candidate { candidate } => {
            if matches!(decide_pin(pin, Ok(candidate.clone())), PinDecision::Candidate { .. }) {
                Ok(())
            } else {
                Err(format!("{}: invalid candidate", entry.source))
            }
        }
        PinDecision::Current => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;
    const HASH: &str = "sha256-Gl7VQbtxjjyFy8rCJelqLhdl4TbO3L0FPumoBQb2+28=";
    fn example() -> BootstrapPin {
        BootstrapPin {
            schema: PIN_SCHEMA.to_string(),
            source: "autoconf".into(),
            package_url: "pkg:gnu/autoconf".into(),
            version: "2.69".into(),
            release_date: "2012-04-25".into(),
            artifacts: BTreeMap::from([("source".into(), PinArtifact {
                url_template: "https://example.org/autoconf-{version}.tar.xz".into(),
                url: "https://example.org/autoconf-2.69.tar.xz".into(),
                hash_kind: HashKind::Tree,
                hash: HASH.into(),
            })]),
            resolve: ResolveHook::Json {
                url: "https://example.org/releases.json".into(),
                version_field: "version".into(),
                date_field: "date".into(),
            },
            recipes: vec!["autoconf.ncl".into()],
        }
    }
    #[test]
    fn validates_roundtrip_and_rejects_unknown_missing_and_bad_hash() {
        let bytes = serde_json::to_vec(&example()).unwrap();
        assert_eq!(parse_pin(&bytes).unwrap(), example());
        let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        value["mystery"] = serde_json::json!(1);
        let error = parse_pin(&serde_json::to_vec(&value).unwrap()).unwrap_err();
        assert!(error.contains("mystery") && error.contains("known fields"));
        value.as_object_mut().unwrap().remove("mystery");
        value["artifacts"]["source"].as_object_mut().unwrap().remove("hash");
        assert!(parse_pin(&serde_json::to_vec(&value).unwrap()).is_err());
        let mut bad = example();
        bad.artifacts.get_mut("source").unwrap().hash = "flat:sha256".into();
        assert!(validate_pin(&bad).is_err());
    }
    #[test]
    fn decides_and_seals_reviewable_plan() {
        let pin = example();
        assert!(
            compare_versions(VersionPair {
                current: "2.9",
                candidate: "2.10"
            })
            .is_lt()
        );
        assert!(
            compare_versions(VersionPair {
                current: "2.10",
                candidate: "2.9"
            })
            .is_gt()
        );
        let candidate = PinCandidate {
            version: "2.70".into(),
            release_date: "2023-01-01".into(),
            hashes: BTreeMap::from([("source".into(), HASH.into())]),
        };
        assert!(matches!(decide_pin(&pin, Ok(candidate.clone())), PinDecision::Candidate { .. }));
        let bytes = serde_json::to_vec(&pin).unwrap();
        let preimage = pin_preimage(&bytes);
        let pins = BTreeMap::from([("autoconf".into(), (pin, preimage.clone()))]);
        let plan = seal_plan(vec![PinPlanEntry {
            source: "autoconf".into(),
            preimage,
            current: "2.69".into(),
            decision: PinDecision::Candidate { candidate },
        }])
        .unwrap();
        verify_plan(&plan, &pins).unwrap();
        let mut changed = plan.clone();
        changed.entries[0].current = "2.68".into();
        assert!(verify_plan(&changed, &pins).is_err());
        let mut unknown = plan;
        unknown.entries[0].source = "alien".into();
        unknown.digest = plan_digest(&unknown.entries).unwrap();
        assert!(verify_plan(&unknown, &pins).unwrap_err().contains("unknown source"));
    }
}
