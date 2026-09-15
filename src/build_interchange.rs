//! Owner-emitted build interchange records for one completed build.
//!
//! The pure mapping takes explicit facts and returns the contract's own request
//! and observation values, validated by the contract crate. The writer adds the
//! file effects. Emission proves no execution authority, custody, or promotion;
//! it only makes the owner's own build facts available in the consumer's
//! contract shape.
//!
//! ```compile_fail
//! fn forge(value: mantle_build_contract::Identity) -> mantle_build_contract::Identity {
//!     let _ = crate::build_interchange::identity("domain", &[]);
//!     value
//! }
//! ```

use std::path::{Path, PathBuf};

use mantle_build_contract::{
    BuildObservation, BuildOutcome, BuildRequest, CacheKind, CacheObservation, ContractError,
    Identity, Label, OBSERVATION_SCHEMA, ProductObservation, REQUEST_SCHEMA, REQUIRED_NON_CLAIMS,
    ValueError, observation_identity, request_identity, validate_observation, validate_request,
};

const ARTIFACT_DOMAIN: &str = "mantle.build-artifact.identity.v1";
const ATTEMPT_DOMAIN: &str = "mantle.build-attempt.identity.v1";
const CACHE_SOURCE_DOMAIN: &str = "mantle.build-cache-source.identity.v1";
const CANDIDATE_DOMAIN: &str = "mantle.build-candidate.identity.v1";
const EFFECT_DOMAIN: &str = "mantle.build-effect.identity.v1";
const ENGINE_DOMAIN: &str = "mantle.build-engine.identity.v1";
const IDEMPOTENCY_DOMAIN: &str = "mantle.build-idempotency.identity.v1";
const LOG_DOMAIN: &str = "mantle.build-log.identity.v1";
const PIPELINE_DOMAIN: &str = "mantle.build-pipeline.identity.v1";
const PLAN_DOMAIN: &str = "mantle.build-plan.identity.v1";
const POLICY_DOMAIN: &str = "mantle.build-policy.identity.v1";
const RECEIPT_DOMAIN: &str = "mantle.build-receipt.identity.v1";
const STORE_DOMAIN: &str = "mantle.build-store.identity.v1";
const WORKER_DOMAIN: &str = "mantle.build-worker.identity.v1";

/// One observed product with the bytes and attestation the owner measured.
pub struct Product {
    pub name: String,
    pub bytes: [u8; 32],
    pub attestation: [u8; 32],
}

/// Explicit facts for one completed local build.
pub struct Facts<'a> {
    pub platform: &'a str,
    pub engine_revision: &'a str,
    pub store_dir: &'a str,
    pub hermeticity_mode: &'a str,
    pub substitution_allowed: bool,
    pub jobs: u64,
    pub derivation_key: &'a str,
    pub attempt: u64,
    pub outcome: BuildOutcome,
    pub cache_kind: CacheKind,
    pub cache_source_leaf: Option<&'a str>,
    pub products: Vec<Product>,
    pub logs: Vec<[u8; 32]>,
}

#[derive(Debug)]
pub enum Error {
    /// A bounded value failed the contract's own value rules.
    Value(ValueError),
    /// The assembled record failed the contract's own admission.
    Contract(ContractError),
    /// The supplied facts contradict each other or this emitter's rules.
    Facts(&'static str),
    /// A record file could not be created or written.
    Io(std::io::Error),
}

impl core::fmt::Display for Error {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Value(error) => write!(formatter, "interchange value: {error}"),
            Self::Contract(error) => write!(formatter, "interchange admission: {error}"),
            Self::Facts(reason) => write!(formatter, "interchange facts: {reason}"),
            Self::Io(error) => write!(formatter, "interchange record io: {error}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<ValueError> for Error {
    fn from(error: ValueError) -> Self {
        Self::Value(error)
    }
}

impl From<ContractError> for Error {
    fn from(error: ContractError) -> Self {
        Self::Contract(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

/// Domain-separated identity over explicit text parts.
#[must_use]
pub fn identity(domain: &str, parts: &[&str]) -> Identity {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.as_bytes());
    for part in parts {
        hasher.update(b"|");
        hasher.update(part.as_bytes());
    }
    Identity::new(format!("b3:{}", hasher.finalize().to_hex())).unwrap_or_else(|_error| {
        unreachable!("a blake3 digest always formats as a canonical identity")
    })
}

fn artifact_identity(domain: &str, bytes: [u8; 32]) -> Identity {
    let hash = blake3::Hash::from_bytes(bytes);
    identity(domain, &[hash.to_hex().as_str()])
}

/// Plain BLAKE3 identity of complete bytes, without a domain frame.
///
/// A consumer that hashes the same bytes derives exactly this value, so this is
/// the only form used for products a consumer can measure itself.
fn byte_identity(bytes: [u8; 32]) -> Identity {
    let hash = blake3::Hash::from_bytes(bytes);
    Identity::new(format!("b3:{}", hash.to_hex()))
        .unwrap_or_else(|_error| unreachable!("a blake3 digest is a canonical identity"))
}

fn strict_labels(names: &[String]) -> Result<Vec<Label>, Error> {
    let mut labels = names
        .iter()
        .map(|name| Label::new(name.clone()).map_err(Error::Value))
        .collect::<Result<Vec<_>, _>>()?;
    labels.sort();
    for pair in labels.windows(2) {
        if pair[0] == pair[1] {
            return Err(Error::Facts("product names must be unique"));
        }
    }
    Ok(labels)
}

/// Assemble the request record for one completed build.
///
/// The plan identity binds the canonical derivation text. The remaining request
/// identities are deterministic correlation frames over that plan, the store,
/// and the attempt ordinal. They are not capabilities or authorization.
pub fn request(facts: &Facts<'_>) -> Result<BuildRequest, Error> {
    let required_products =
        strict_labels(&facts.products.iter().map(|product| product.name.clone()).collect::<Vec<_>>())?;
    let plan = identity(PLAN_DOMAIN, &[facts.derivation_key]);
    let store = identity(STORE_DOMAIN, &[facts.store_dir]);
    let substitution = if facts.substitution_allowed { "substitute" } else { "no-substitute" };
    let jobs = facts.jobs.to_string();
    let attempt = facts.attempt.to_string();
    let mut request = BuildRequest {
        schema: REQUEST_SCHEMA.to_owned(),
        identity: plan.clone(),
        idempotency_key: identity(IDEMPOTENCY_DOMAIN, &[plan.as_str(), store.as_str()]),
        effect_identity: identity(EFFECT_DOMAIN, &[plan.as_str()]),
        attempt_identity: identity(ATTEMPT_DOMAIN, &[plan.as_str(), attempt.as_str()]),
        candidate_identity: identity(CANDIDATE_DOMAIN, &[plan.as_str()]),
        pipeline_revision_identity: identity(PIPELINE_DOMAIN, &[facts.engine_revision]),
        plan_identity: plan,
        policy_identity: identity(POLICY_DOMAIN, &[facts.hermeticity_mode, substitution, jobs.as_str()]),
        platform: Label::new(facts.platform.to_owned())?,
        required_products,
    };
    request.identity = request_identity(&request);
    validate_request(&request)?;
    debug_assert_eq!(request.identity, request_identity(&request));
    Ok(request)
}

/// Assemble the observation record for one completed build.
///
/// `artifact_identity` is BLAKE3 over the complete exported product bytes, so a
/// consumer that hashes the same bytes derives the same identity. The receipt
/// identity binds the sorted per-product attestation digests. Cache hits name
/// their source; fresh builds name none.
pub fn observation(facts: &Facts<'_>, request: &BuildRequest) -> Result<BuildObservation, Error> {
    validate_request(request)?;
    let builder = identity(ENGINE_DOMAIN, &[facts.engine_revision]);
    let worker = identity(WORKER_DOMAIN, &[facts.platform]);
    let store = identity(STORE_DOMAIN, &[facts.store_dir]);
    let mut products = facts
        .products
        .iter()
        .map(|product| {
            Ok(ProductObservation {
                name: Label::new(product.name.clone())?,
                artifact_identity: byte_identity(product.bytes),
                store_identity: store.clone(),
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    products.sort_by(|left, right| left.name.as_str().cmp(right.name.as_str()));
    let cache_source = match (facts.cache_kind, facts.cache_source_leaf) {
        (CacheKind::None, None) => None,
        (CacheKind::Local | CacheKind::Substitution, Some(leaf)) => {
            Some(identity(CACHE_SOURCE_DOMAIN, &[facts.store_dir, leaf]))
        }
        (CacheKind::None, Some(_)) | (CacheKind::Local | CacheKind::Substitution, None) => {
            return Err(Error::Facts("cache kind and cache source disagree"));
        }
    };
    let receipt_identity = match facts.outcome {
        BuildOutcome::Unknown => None,
        _ => {
            let digests = facts
                .products
                .iter()
                .map(|product| blake3::Hash::from_bytes(product.attestation).to_hex().to_string())
                .collect::<Vec<_>>();
            let parts = digests.iter().map(String::as_str).collect::<Vec<_>>();
            Some(identity(RECEIPT_DOMAIN, &parts))
        }
    };
    let mut observation = BuildObservation {
        schema: OBSERVATION_SCHEMA.to_owned(),
        identity: request.identity.clone(),
        request_identity: request.identity.clone(),
        outcome: facts.outcome,
        products,
        builder_identity: builder.clone(),
        worker_identity: worker,
        store_identity: store,
        cache: CacheObservation {
            kind: facts.cache_kind,
            source_identity: cache_source,
        },
        logs: facts
            .logs
            .iter()
            .map(|digest| artifact_identity(LOG_DOMAIN, *digest))
            .collect(),
        metrics: Vec::new(),
        receipt_identity,
        non_claims: REQUIRED_NON_CLAIMS.iter().map(|value| (*value).to_owned()).collect(),
    };
    observation.identity = observation_identity(&observation);
    validate_observation(request, &builder, &observation)?;
    debug_assert_eq!(observation.identity, observation_identity(&observation));
    Ok(observation)
}

/// Write both records beneath an explicit directory without replacement.
pub fn write(
    directory: &Path,
    request: &BuildRequest,
    observation: &BuildObservation,
) -> Result<(PathBuf, PathBuf), Error> {
    validate_request(request)?;
    std::fs::create_dir_all(directory)?;
    let request_path = directory.join("request.json");
    let observation_path = directory.join("observation.json");
    write_new(&request_path, &serde_json::to_vec_pretty(request).map_err(|_error| Error::Facts("request record is not serializable"))?)?;
    write_new(&observation_path, &serde_json::to_vec_pretty(observation).map_err(|_error| Error::Facts("observation record is not serializable"))?)?;
    Ok((request_path, observation_path))
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), Error> {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new().create_new(true).write(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

/// Explicit interchange directory: the build configuration first, then
/// `MANTLE_INTERCHANGE_DIR`. Both are operator-supplied. The environment
/// trigger is an interim interface until the clap flag is threaded through the
/// prepared build command.
pub fn configured_directory(
    config: &crunch_pipeline::BuildConfig,
) -> Result<Option<PathBuf>, Error> {
    if let Some(directory) = config.interchange_dir.clone() {
        return Ok(Some(directory));
    }
    match std::env::var_os("MANTLE_INTERCHANGE_DIR") {
        Some(value) if !value.is_empty() => Ok(Some(PathBuf::from(value))),
        _ => Ok(None),
    }
}

/// The local host platform. A cross-system build must not use this emitter yet.
fn host_platform() -> &'static str {
    if cfg!(all(target_arch = "x86_64", target_os = "linux")) {
        "x86_64-linux"
    } else {
        "unsupported-local-platform"
    }
}

fn hash_file(path: &Path) -> Result<[u8; 32], Error> {
    let bytes = std::fs::read(path)?;
    Ok(*blake3::hash(&bytes).as_bytes())
}

/// Gather explicit facts from one completed build and write both records.
///
/// Exactly one build outcome is supported. The platform is the local host
/// platform, so a cross-system build must wait for derivation-system support.
/// Log digests are not bound in this first revision. A cache hit names the
/// derivation entry that supplied it. Emission establishes no authority.
pub fn emit_from_report(
    config: &crunch_pipeline::BuildConfig,
    result: &crunch_pipeline::PipelineResult,
    directory: &Path,
) -> Result<(PathBuf, PathBuf), Error> {
    let outcomes = result.outcomes.as_slice();
    let root_keys = result.root_labels.keys().collect::<Vec<_>>();
    let roots = outcomes
        .iter()
        .filter(|outcome| {
            root_keys.is_empty()
                || root_keys.iter().any(|key| {
                    **key == crunch_pipeline::drv_key_for(&config.store_dir, &outcome.drv_path)
                })
        })
        .collect::<Vec<_>>();
    let [outcome] = roots.as_slice() else {
        return Err(Error::Facts(
            "interchange emission supports exactly one requested build root",
        ));
    };
    let derivation_key = crunch_pipeline::drv_key_for(&config.store_dir, &outcome.drv_path);
    let output_dir = config
        .output_dir
        .to_str()
        .ok_or(Error::Facts("output directory is not UTF-8"))?;
    let mut products = Vec::with_capacity(outcome.outputs.len());
    for (name, path_info) in &outcome.outputs {
        let exported = path_info.store_path.to_absolute_path_with_prefix(output_dir);
        let bytes = hash_file(Path::new(&exported))?;
        let attestation_path = crunch_store::artifact_attestation_file_path(
            &config.state_dir,
            &config.store_dir,
            &path_info.store_path,
        );
        let attestation = hash_file(&attestation_path)?;
        products.push(Product {
            name: name.clone(),
            bytes,
            attestation,
        });
    }
    let hermeticity = config.hermeticity_mode.to_string();
    let outcome_kind = if result.failed.is_empty() {
        BuildOutcome::Success
    } else {
        BuildOutcome::Failed
    };
    let (cache_kind, cache_source) = if outcome.cached {
        (CacheKind::Local, Some(derivation_key.as_str()))
    } else {
        (CacheKind::None, None)
    };
    let facts = Facts {
        platform: host_platform(),
        engine_revision: env!("CARGO_PKG_VERSION"),
        store_dir: &config.store_dir,
        hermeticity_mode: hermeticity.as_str(),
        substitution_allowed: !config.substituter_urls.is_empty(),
        jobs: u64::from(config.max_jobs),
        derivation_key: derivation_key.as_str(),
        attempt: 1,
        outcome: outcome_kind,
        cache_kind,
        cache_source_leaf: cache_source,
        products,
        logs: Vec::new(),
    };
    debug_assert!(!facts.products.is_empty() || outcome_kind != BuildOutcome::Success);
    let request_record = request(&facts)?;
    let observation_record = observation(&facts, &request_record)?;
    write(directory, &request_record, &observation_record)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts(outcome: BuildOutcome, cache: (CacheKind, Option<&'static str>)) -> Facts<'static> {
        Facts {
            platform: "x86_64-linux",
            engine_revision: "mantle-test-revision",
            store_dir: "/nix/store",
            hermeticity_mode: "strict",
            substitution_allowed: false,
            jobs: 1,
            derivation_key: "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-neural-scalar-materialization.drv",
            attempt: 1,
            outcome,
            cache_kind: cache.0,
            cache_source_leaf: cache.1,
            products: vec![
                Product {
                    name: "tensor".to_owned(),
                    bytes: [7; 32],
                    attestation: [8; 32],
                },
                Product {
                    name: "transcript".to_owned(),
                    bytes: [9; 32],
                    attestation: [10; 32],
                },
            ],
            logs: vec![[11; 32]],
        }
    }

    #[test]
    fn records_admit_and_recompute_their_identities() {
        let facts = facts(BuildOutcome::Success, (CacheKind::None, None));
        let request_record = request(&facts).expect("request");
        let observation_record = observation(&facts, &request_record).expect("observation");
        assert_eq!(validate_request(&request_record), Ok(()));
        assert_eq!(request_record.identity, request_identity(&request_record));
        assert_eq!(
            validate_observation(
                &request_record,
                &observation_record.builder_identity,
                &observation_record
            ),
            Ok(())
        );
        assert_eq!(observation_record.identity, observation_identity(&observation_record));
        assert_eq!(observation_record.products.len(), 2);
        assert_eq!(observation_record.non_claims.len(), REQUIRED_NON_CLAIMS.len());
        assert!(observation_record.receipt_identity.is_some());
    }

    #[test]
    fn cache_and_receipt_follow_the_observed_facts() {
        let miss = facts(BuildOutcome::Success, (CacheKind::None, None));
        let miss_request = request(&miss).expect("request");
        let miss_observation = observation(&miss, &miss_request).expect("observation");
        assert!(miss_observation.cache.source_identity.is_none());

        let hit = facts(
            BuildOutcome::Success,
            (
                CacheKind::Local,
                Some("dddddddddddddddddddddddddddddddd-neural-scalar-materialization"),
            ),
        );
        let hit_request = request(&hit).expect("request");
        let hit_observation = observation(&hit, &hit_request).expect("observation");
        assert!(hit_observation.cache.source_identity.is_some());

        let unknown = facts(BuildOutcome::Unknown, (CacheKind::None, None));
        let unknown_request = request(&unknown).expect("request");
        let unknown_observation = observation(&unknown, &unknown_request).expect("observation");
        assert!(unknown_observation.receipt_identity.is_none());
    }

    #[test]
    fn contradictory_facts_fail_before_a_record_exists() {
        let mut duplicate = facts(BuildOutcome::Success, (CacheKind::None, None));
        duplicate.products[1].name = "tensor".to_owned();
        assert!(matches!(request(&duplicate), Err(Error::Facts(_))));

        let contradictory = facts(
            BuildOutcome::Success,
            (CacheKind::Local, None),
        );
        let contradictory_request = request(&contradictory).expect("request");
        assert!(matches!(
            observation(&contradictory, &contradictory_request),
            Err(Error::Facts(_))
        ));

        let bad_platform = Facts {
            platform: "X86_64-Linux",
            ..facts(BuildOutcome::Success, (CacheKind::None, None))
        };
        assert!(matches!(request(&bad_platform), Err(Error::Value(_))));
    }

    #[test]
    fn records_write_once_without_replacement() {
        let directory = std::env::temp_dir().join(format!(
            "mantle-interchange-{}-{}",
            std::process::id(),
            "write-once"
        ));
        let _ = std::fs::remove_dir_all(&directory);
        let facts = facts(BuildOutcome::Success, (CacheKind::None, None));
        let request_record = request(&facts).expect("request");
        let observation_record = observation(&facts, &request_record).expect("observation");
        let (request_path, observation_path) =
            write(&directory, &request_record, &observation_record).expect("records");
        assert!(request_path.is_file());
        assert!(observation_path.is_file());
        assert!(matches!(
            write(&directory, &request_record, &observation_record),
            Err(Error::Io(_))
        ));
        std::fs::remove_dir_all(&directory).expect("cleanup");
    }
}
