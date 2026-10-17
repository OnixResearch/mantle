// machine-artifact-public: bootstrap.source-pin-plans
//! Batched bootstrap pin resolution and preimage-bound, pin-only application.
use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use crunch_project::HashAlgo;
use crunch_project::HashResolutionMode;
use crunch_project::RefreshResolver;
use crunch_project_core::bootstrap_pins::BootstrapPin;
use crunch_project_core::bootstrap_pins::HashKind;
use crunch_project_core::bootstrap_pins::PinCandidate;
use crunch_project_core::bootstrap_pins::PinDecision;
use crunch_project_core::bootstrap_pins::PinPlan;
use crunch_project_core::bootstrap_pins::PinPlanEntry;
use crunch_project_core::bootstrap_pins::ResolveHook;
use crunch_project_core::bootstrap_pins::decide_pin;
use crunch_project_core::bootstrap_pins::parse_pin;
use crunch_project_core::bootstrap_pins::pin_preimage;
use crunch_project_core::bootstrap_pins::render_pin_url;
use crunch_project_core::bootstrap_pins::seal_plan;
use crunch_project_core::bootstrap_pins::validate_pin;
use crunch_project_core::bootstrap_pins::verify_plan;
use fs2::FileExt;
use serde::Deserialize;
use serde::Serialize;

use crate::project_resolve::LiveResolver;

const MAX_PIN_FILES: usize = 256;
const MAX_CACHE_BYTES: u64 = 65_536;
const MAX_CACHE_ENTRIES: usize = 256;
const MAX_RELEASE_BYTES: u64 = 262_144;
const MAX_PLAN_BYTES: u64 = 1_048_576;
const MAX_REQUESTS_PER_HOST: u32 = 64;
const USER_AGENT: &str = "mantle-bootstrap-pin/1";

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CachedRelease {
    hook_identity: String,
    etag: Option<String>,
    last_modified: Option<String>,
    version: String,
    release_date: String,
}

// r[impl mantle.bootstrap_source_pins.batched_resolution]
pub fn check(root: &Path, cache_dir: &Path, plan_path: &Path) -> Result<bool, String> {
    let pins = load_pins(root)?;
    fs::create_dir_all(cache_dir).map_err(|e| format!("creating cache: {e}"))?;
    for (index, entry) in fs::read_dir(cache_dir).map_err(|e| format!("reading cache: {e}"))?.enumerate() {
        entry.map_err(|e| format!("reading cache entry: {e}"))?;
        if index >= MAX_CACHE_ENTRIES {
            return Err(format!("cache exceeds {MAX_CACHE_ENTRIES} entries; retire obsolete source caches explicitly"));
        }
    }
    let agent = release_agent();
    let mut host_counts = BTreeMap::new();
    let resolver = LiveResolver::new(root);
    let mut entries = Vec::with_capacity(pins.len());
    for (source, (pin, preimage)) in &pins {
        let result = resolve_release(pin, cache_dir, &agent, &mut host_counts).and_then(|(version, release_date)| {
            let pair = crunch_project_core::bootstrap_pins::VersionPair {
                current: &pin.version,
                candidate: &version,
            };
            if !crunch_project_core::bootstrap_pins::compare_versions(pair).is_lt() {
                return Ok(PinCandidate {
                    version,
                    release_date,
                    hashes: BTreeMap::new(),
                });
            }
            let mut hashes = BTreeMap::new();
            for (role, artifact) in &pin.artifacts {
                let url = render_pin_url(artifact, &version)?;
                bound_host(&url, &mut host_counts)?;
                let mode = match artifact.hash_kind {
                    HashKind::Flat => HashResolutionMode::Flat,
                    HashKind::Tree => HashResolutionMode::Recursive,
                };
                let hash = resolver
                    .hash_url_content(&url, &HashAlgo::Sha256, mode)
                    .map_err(|e| format!("{source}: fixed-output prefetch {url}: {e}"))?
                    .ok_or_else(|| format!("{source}: fixed-output prefetch returned no hash"))?;
                hashes.insert(role.clone(), hash);
            }
            Ok(PinCandidate {
                version,
                release_date,
                hashes,
            })
        });
        entries.push(PinPlanEntry {
            source: source.clone(),
            preimage: preimage.clone(),
            current: pin.version.clone(),
            decision: decide_pin(pin, result),
        });
    }
    let plan = seal_plan(entries)?;
    let pending = plan.entries.iter().any(|entry| !matches!(entry.decision, PinDecision::Current));
    let mut bytes = serde_json::to_vec_pretty(&plan).map_err(|e| format!("encoding plan: {e}"))?;
    bytes.push(b'\n');
    atomic_write(plan_path, &bytes)?;
    std::io::stdout().write_all(&bytes).map_err(|e| format!("reporting pin plan: {e}"))?;
    Ok(pending)
}

// r[impl mantle.bootstrap_source_pins.apply_writes_pins_only]
pub fn apply(root: &Path, plan_path: &Path, reviewed: bool) -> Result<(), String> {
    if !reviewed {
        return Err("apply requires explicit --reviewed acknowledgement".into());
    }
    let directory = fs::File::open(root.join("bootstrap/pins")).map_err(|e| format!("opening pin directory: {e}"))?;
    directory.try_lock_exclusive().map_err(|e| format!("bootstrap pin mutation lock busy: {e}"))?;
    let pins = load_pins(root)?;
    let bytes = read_limited(plan_path, MAX_PLAN_BYTES)?;
    let plan: PinPlan = serde_json::from_slice(&bytes).map_err(|e| format!("invalid plan: {e}"))?;
    verify_plan(&plan, &pins)?;
    // Verify *all* remote fixed-output hashes before staging *any* write.
    let resolver = LiveResolver::new(root);
    let mut host_counts = BTreeMap::new();
    let mut replacements = Vec::new();
    for entry in &plan.entries {
        let PinDecision::Candidate { candidate } = &entry.decision else {
            continue;
        };
        let pin = &pins[&entry.source].0;
        let mut next = pin.clone();
        next.version.clone_from(&candidate.version);
        next.release_date.clone_from(&candidate.release_date);
        for (role, artifact) in &mut next.artifacts {
            artifact.url = render_pin_url(artifact, &next.version)?;
            bound_host(&artifact.url, &mut host_counts)?;
            let expected = &candidate.hashes[role];
            let mode = match artifact.hash_kind {
                HashKind::Flat => HashResolutionMode::Flat,
                HashKind::Tree => HashResolutionMode::Recursive,
            };
            let actual = resolver
                .hash_url_content(&artifact.url, &HashAlgo::Sha256, mode)
                .map_err(|e| format!("{}: fixed-output prefetch: {e}", entry.source))?
                .ok_or_else(|| format!("{}: fixed-output prefetch returned no hash", entry.source))?;
            if &actual != expected {
                return Err(format!("{}: upstream hash mismatch for {role}", entry.source));
            }
            artifact.hash = actual;
        }
        validate_pin(&next)?;
        let toml = toml::to_string_pretty(&next).map_err(|e| format!("serialize pin: {e}"))?;
        let mut json = serde_json::to_vec_pretty(&next).map_err(|e| format!("serialize reader: {e}"))?;
        json.push(b'\n');
        replacements.push((root.join("bootstrap/pins").join(format!("{}.toml", entry.source)), toml.into_bytes()));
        replacements.push((root.join("bootstrap/pins/generated").join(format!("{}.json", entry.source)), json));
    }
    // Recheck after potentially long downloads while the directory mutation lock is held.
    verify_plan(&plan, &load_pins(root)?)?;
    // Stage every replacement before the first rename; rollback committed paths on failure.
    publish_replacements(&replacements)?;
    println!(r#"{{"schema":"mantle-bootstrap-pin-apply-v1","applied_sources":{}}}"#, replacements.len() / 2);
    Ok(())
}

fn load_pins(root: &Path) -> Result<BTreeMap<String, (BootstrapPin, String)>, String> {
    let dir = root.join("bootstrap/pins");
    let mut paths = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|e| format!("reading {}: {e}", dir.display()))? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().is_some_and(|ext| ext == "toml") {
            if paths.len() >= MAX_PIN_FILES {
                return Err("pin directory exceeds 256 TOML records".into());
            }
            paths.push(path);
        }
    }
    paths.sort();
    if paths.is_empty() {
        return Err("pin directory has no TOML records".into());
    }
    let mut pins = BTreeMap::new();
    for path in paths {
        if !fs::symlink_metadata(&path).map_err(|e| e.to_string())?.file_type().is_file() {
            return Err(format!("pin is not a regular file: {}", path.display()));
        }
        let bytes = read_limited(&path, MAX_CACHE_BYTES)?;
        let pin: BootstrapPin = toml::from_str(std::str::from_utf8(&bytes).map_err(|e| e.to_string())?)
            .map_err(|e| format!("{}: {e}; known fields: schema, source, package_url, version, release_date, artifacts, resolve, recipes; artifact fields: url_template, url, hash_kind, hash", path.display()))?;
        validate_pin(&pin)?;
        if path.file_stem().and_then(|stem| stem.to_str()) != Some(&pin.source) {
            return Err(format!("{}: source differs from filename", path.display()));
        }
        let derived = root.join("bootstrap/pins/generated").join(format!("{}.json", pin.source));
        let reader = read_limited(&derived, MAX_CACHE_BYTES)?;
        let mut expected = serde_json::to_vec_pretty(&pin).map_err(|e| format!("rendering reader: {e}"))?;
        expected.push(b'\n');
        if parse_pin(&reader)? != pin || reader != expected {
            return Err(format!(
                "{}: stale generated Nickel reader; run bootstrap/pins/generate_readers.py",
                pin.source
            ));
        }
        if pins.insert(pin.source.clone(), (pin, pin_preimage(&bytes))).is_some() {
            return Err("duplicate pin source".into());
        }
    }
    Ok(pins)
}

fn release_agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(30)))
        .timeout_connect(Some(Duration::from_secs(10)))
        .http_status_as_error(false)
        .build()
        .new_agent()
}

fn resolve_release(
    pin: &BootstrapPin,
    cache_dir: &Path,
    agent: &ureq::Agent,
    counts: &mut BTreeMap<String, u32>,
) -> Result<(String, String), String> {
    let ResolveHook::Json {
        url,
        version_field,
        date_field,
    } = &pin.resolve;
    bound_host(url, counts)?;
    let hook_identity = blake3::hash(format!("{url}\0{version_field}\0{date_field}").as_bytes()).to_hex().to_string();
    let path = cache_dir.join(format!("{}.json", pin.source));
    let cached: Option<CachedRelease> = if path.exists() {
        let cache: CachedRelease = serde_json::from_slice(&read_limited(&path, MAX_CACHE_BYTES)?)
            .map_err(|e| format!("{}: corrupt cache: {e}", pin.source))?;
        (cache.hook_identity == hook_identity).then_some(cache)
    } else {
        None
    };
    let mut request = agent.get(url).header("user-agent", USER_AGENT).header("accept", "application/json");
    if let Some(cache) = &cached {
        if let Some(etag) = &cache.etag {
            request = request.header("if-none-match", etag);
        }
        if let Some(modified) = &cache.last_modified {
            request = request.header("if-modified-since", modified);
        }
    }
    let response = request.call().map_err(|e| format!("{}: querying releases: {e}", pin.source))?;
    if response.status().as_u16() == 304 {
        let cache = cached.ok_or_else(|| format!("{}: 304 without cache", pin.source))?;
        return Ok((cache.version, cache.release_date));
    }
    if response.status().as_u16() != 200 {
        return Err(format!("{}: release HTTP {}", pin.source, response.status()));
    }
    let etag = response.headers().get("etag").and_then(|v| v.to_str().ok()).map(ToOwned::to_owned);
    let last_modified = response.headers().get("last-modified").and_then(|v| v.to_str().ok()).map(ToOwned::to_owned);
    let mut body = Vec::new();
    response
        .into_body()
        .with_config()
        .limit(MAX_RELEASE_BYTES + 1)
        .reader()
        .read_to_end(&mut body)
        .map_err(|e| format!("{}: reading release: {e}", pin.source))?;
    if body.len() as u64 > MAX_RELEASE_BYTES {
        return Err(format!("{}: release body exceeds limit", pin.source));
    }
    let value: serde_json::Value =
        serde_json::from_slice(&body).map_err(|e| format!("{}: malformed release: {e}", pin.source))?;
    let version = value
        .get(version_field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("{}: missing {version_field}", pin.source))?
        .trim_start_matches('v')
        .to_string();
    let date = value
        .get(date_field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("{}: missing {date_field}", pin.source))?;
    let release_date = date.get(..10).ok_or_else(|| format!("{}: invalid release date", pin.source))?.to_string();
    let cache = CachedRelease {
        hook_identity,
        etag,
        last_modified,
        version: version.clone(),
        release_date: release_date.clone(),
    };
    atomic_write(&path, &serde_json::to_vec(&cache).map_err(|e| e.to_string())?)?;
    Ok((version, release_date))
}

fn bound_host(url: &str, counts: &mut BTreeMap<String, u32>) -> Result<(), String> {
    let parsed = url::Url::parse(url).map_err(|e| format!("invalid upstream URL: {e}"))?;
    if parsed.scheme() != "https"
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.fragment().is_some()
    {
        return Err("upstream URL must be HTTPS without credentials or fragment".into());
    }
    let host = parsed.host_str().ok_or("upstream URL has no host")?;
    let count = counts.entry(host.to_string()).or_default();
    if *count >= MAX_REQUESTS_PER_HOST {
        return Err(format!("{host}: per-host request bound reached"));
    }
    *count += 1;
    Ok(())
}

fn read_limited(path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    let file = fs::File::open(path).map_err(|e| format!("reading {}: {e}", path.display()))?;
    if file.metadata().map_err(|e| e.to_string())?.len() > limit {
        return Err(format!("{} exceeds {limit} bytes", path.display()));
    }
    let mut bytes = Vec::new();
    file.take(limit + 1).read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    if bytes.len() as u64 > limit {
        return Err(format!("{} exceeds {limit} bytes", path.display()));
    }
    Ok(bytes)
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path.parent().ok_or("output path lacks parent")?;
    fs::create_dir_all(parent).map_err(|e| format!("creating {}: {e}", parent.display()))?;
    let temp = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    fs::write(temp.path(), bytes).map_err(|e| e.to_string())?;
    temp.persist(path).map_err(|e| format!("publishing {}: {e}", path.display()))?;
    Ok(())
}

fn publish_replacements(replacements: &[(PathBuf, Vec<u8>)]) -> Result<(), String> {
    publish_replacements_with(replacements, |_, _| Ok(()))
}

fn publish_replacements_with(
    replacements: &[(PathBuf, Vec<u8>)],
    mut before_publish: impl FnMut(usize, &Path) -> Result<(), String>,
) -> Result<(), String> {
    let mut staged = Vec::new();
    for (path, bytes) in replacements {
        let parent = path.parent().ok_or("pin path lacks parent")?;
        let temp = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
        fs::write(temp.path(), bytes).map_err(|e| e.to_string())?;
        staged.push(temp);
    }
    let originals: Vec<Vec<u8>> = replacements
        .iter()
        .map(|(p, _)| fs::read(p).map_err(|e| format!("reading {}: {e}", p.display())))
        .collect::<Result<_, _>>()?;
    for (published, (temp, (path, _))) in staged.into_iter().zip(replacements).enumerate() {
        let result =
            before_publish(published, path).and_then(|()| temp.persist(path).map(|_| ()).map_err(|e| e.to_string()));
        if let Err(error) = result {
            let mut rollback_errors = Vec::new();
            for (old, (written_path, _)) in originals.iter().zip(replacements).take(published) {
                if let Err(rollback) = atomic_write(written_path, old) {
                    rollback_errors.push(format!("{}: {rollback}", written_path.display()));
                }
            }
            if !rollback_errors.is_empty() {
                return Err(format!(
                    "publishing {} failed: {error}; ROLLBACK UNCERTAIN: {}",
                    path.display(),
                    rollback_errors.join("; ")
                ));
            }
            return Err(format!("publishing {} failed and prior writes rolled back: {error}", path.display()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unresolved_source_blocks_entire_mixed_candidate_plan_without_writes() {
        let dir = tempfile::tempdir().unwrap();
        let pin_dir = dir.path().join("bootstrap/pins");
        fs::create_dir_all(pin_dir.join("generated")).unwrap();
        let mut entries = Vec::new();
        let mut original = Vec::new();
        for (source, text) in [
            ("cmake", include_str!("../bootstrap/pins/cmake.toml")),
            ("picolibc", include_str!("../bootstrap/pins/picolibc.toml")),
        ] {
            let pin: BootstrapPin = toml::from_str(text).unwrap();
            let path = pin_dir.join(format!("{source}.toml"));
            let reader = pin_dir.join(format!("generated/{source}.json"));
            let mut reader_bytes = serde_json::to_vec_pretty(&pin).unwrap();
            reader_bytes.push(b'\n');
            fs::write(&path, text).unwrap();
            fs::write(&reader, &reader_bytes).unwrap();
            original.push((path, text.as_bytes().to_vec()));
            original.push((reader, reader_bytes));
            let decision = if source == "cmake" {
                PinDecision::Candidate {
                    candidate: PinCandidate {
                        version: "3.31.9".into(),
                        release_date: "2025-06-13".into(),
                        hashes: pin
                            .artifacts
                            .iter()
                            .map(|(role, artifact)| (role.clone(), artifact.hash.clone()))
                            .collect(),
                    },
                }
            } else {
                PinDecision::Error {
                    message: "upstream unavailable".into(),
                }
            };
            entries.push(PinPlanEntry {
                source: source.into(),
                preimage: pin_preimage(text.as_bytes()),
                current: pin.version,
                decision,
            });
        }
        let plan = seal_plan(entries).unwrap();
        let plan_path = dir.path().join("plan.json");
        fs::write(&plan_path, serde_json::to_vec(&plan).unwrap()).unwrap();
        assert!(apply(dir.path(), &plan_path, true).unwrap_err().contains("unresolved upstream observation"));
        for (path, bytes) in original {
            assert_eq!(fs::read(path).unwrap(), bytes);
        }
    }

    #[test]
    fn publication_rolls_back_first_pin_when_second_publication_fails() {
        let dir = tempfile::tempdir().unwrap();
        let first = dir.path().join("first.toml");
        let second = dir.path().join("second.json");
        fs::write(&first, b"original pin").unwrap();
        fs::write(&second, b"original reader").unwrap();
        let replacements = vec![
            (first.clone(), b"new pin".to_vec()),
            (second.clone(), b"new reader".to_vec()),
        ];
        let error = publish_replacements_with(&replacements, |index, _| {
            if index == 1 {
                Err("injected second publication failure".into())
            } else {
                Ok(())
            }
        })
        .unwrap_err();
        assert!(error.contains("prior writes rolled back"));
        assert_eq!(fs::read(&first).unwrap(), b"original pin");
        assert_eq!(fs::read(&second).unwrap(), b"original reader");
    }

    #[test]
    fn host_bounds_use_parsed_https_authority() {
        let mut counts = BTreeMap::new();
        assert!(bound_host("https://example.org?x", &mut counts).is_ok());
        assert_eq!(counts["example.org"], 1);
        assert!(bound_host("https://example.org@evil.test/a", &mut counts).is_err());
        assert!(bound_host("http://example.org/a", &mut counts).is_err());
    }
}
