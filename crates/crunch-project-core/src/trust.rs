use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;

use crate::error::Error;
use crate::manifest::HashAlgo;

pub const TRUST_POLICY_SCHEMA: &str = "mantle-project-input-trust-v1";
pub const PROJECT_INPUT_TRUST_NON_CLAIM: &str = "project input trust proves only the configured verifier evidence for a bound source digest; it does not prove build reproducibility, compiler correctness, release validity, forge trust, or global upstream authenticity";
pub const MAX_TRUST_SIGNATURE_REFS: u32 = 16;
pub const MAX_TRUSTED_PUBLIC_KEYS: u32 = 32;
pub const MAX_REQUIRED_SIGNERS: u32 = 32;
pub const MAX_TRUST_REF_BYTES: u32 = 1024;
const MIN_TRUST_QUORUM: u32 = 1;
const TRUST_DIGEST_PREFIX: &str = "blake3:";
const TRUST_POLICY_SERIALIZATION_FAILURE: &[u8] = b"mantle-project-input-trust-v1:serialization-failure";
const MAX_LOCKED_TRUST_PROBLEMS: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TrustVerifierKind {
    Ed25519Detached,
}

impl TrustVerifierKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ed25519Detached => "ed25519-detached",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum TrustDigestBinding {
    #[default]
    ContentHash,
}

impl TrustDigestBinding {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ContentHash => "content-hash",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TrustSubjectKind {
    Input,
    Patch,
}

impl TrustSubjectKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Input => "input",
            Self::Patch => "patch",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustSubject {
    pub kind: TrustSubjectKind,
    pub name: String,
}

impl TrustSubject {
    pub fn input(name: String) -> Self {
        Self {
            kind: TrustSubjectKind::Input,
            name,
        }
    }

    pub fn patch(name: String) -> Self {
        Self {
            kind: TrustSubjectKind::Patch,
            name,
        }
    }

    pub fn label(&self) -> String {
        format!("{} '{}'", self.kind.as_str(), self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TrustSignatureRef {
    #[serde(rename = "local-file")]
    LocalFile { path: String },
}

impl TrustSignatureRef {
    pub fn ref_text(&self) -> &str {
        match self {
            Self::LocalFile { path } => path.as_str(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InputTrustPolicy {
    pub verifier: TrustVerifierKind,
    pub signatures: Vec<TrustSignatureRef>,
    pub trusted_public_keys: Vec<String>,
    pub required_signers: Vec<String>,
    pub quorum: u32,
    pub digest_binding: TrustDigestBinding,
}

#[derive(Deserialize)]
struct RawInputTrustPolicy {
    verifier: TrustVerifierKind,
    signatures: Option<Vec<TrustSignatureRef>>,
    trusted_public_keys: Option<Vec<String>>,
    required_signers: Option<Vec<String>>,
    quorum: Option<u32>,
    digest_binding: Option<TrustDigestBinding>,
}

impl<'de> Deserialize<'de> for InputTrustPolicy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        let raw = RawInputTrustPolicy::deserialize(deserializer)?;
        Ok(Self {
            verifier: raw.verifier,
            signatures: raw.signatures.unwrap_or_default(),
            trusted_public_keys: raw.trusted_public_keys.unwrap_or_default(),
            required_signers: raw.required_signers.unwrap_or_default(),
            quorum: raw.quorum.unwrap_or(MIN_TRUST_QUORUM),
            digest_binding: raw.digest_binding.unwrap_or_default(),
        })
    }
}

impl InputTrustPolicy {
    pub fn validate(&self, context: &str) -> Vec<String> {
        let mut problems = Vec::new();
        push_signature_ref_problems(self, context, &mut problems);
        push_trusted_key_problems(self, context, &mut problems);
        push_required_signer_problems(self, context, &mut problems);
        push_quorum_problems(self, context, &mut problems);
        problems
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifiedTrustFact {
    pub subject: TrustSubject,
    pub verifier: TrustVerifierKind,
    pub digest_binding: TrustDigestBinding,
    pub hash_algo: HashAlgo,
    pub hash_value: String,
    pub signer: String,
    pub key_ref: String,
    pub signature_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockedTrust {
    pub schema: String,
    pub subject: TrustSubject,
    pub policy_digest: String,
    pub verifier: TrustVerifierKind,
    pub digest_binding: TrustDigestBinding,
    pub hash_algo: HashAlgo,
    pub hash_value: String,
    pub signers: Vec<String>,
    pub key_refs: Vec<String>,
    pub signature_refs: Vec<String>,
    pub claim: String,
}

pub fn trust_signature_payload(
    subject: &TrustSubject,
    digest_binding: TrustDigestBinding,
    hash_algo: &HashAlgo,
    hash_value: &str,
) -> String {
    assert!(!subject.name.is_empty(), "trust subject name must not be empty");
    assert!(!hash_value.is_empty(), "trust-bound hash value must not be empty");
    format!(
        "{TRUST_POLICY_SCHEMA}\nsubject-kind={}\nsubject-name={}\ndigest-binding={}\nhash-algo={}\nhash-value={}\n",
        subject.kind.as_str(),
        subject.name,
        digest_binding.as_str(),
        hash_algo,
        hash_value
    )
}

pub fn trust_policy_digest(policy: &InputTrustPolicy) -> String {
    let bytes = match serde_json::to_vec(policy) {
        Ok(bytes) => bytes,
        Err(_) => TRUST_POLICY_SERIALIZATION_FAILURE.to_vec(),
    };
    let digest = blake3::hash(&bytes).to_hex().to_string();
    format!("{TRUST_DIGEST_PREFIX}{digest}")
}

pub fn evaluate_trust_policy(
    subject: TrustSubject,
    policy: &InputTrustPolicy,
    hash_algo: &HashAlgo,
    hash_value: &str,
    facts: &[VerifiedTrustFact],
) -> Result<LockedTrust, Error> {
    assert!(!subject.name.is_empty(), "trust subject name must not be empty");
    assert!(!hash_value.is_empty(), "trust hash value must not be empty");
    let policy_problems = policy.validate(&subject.label());
    if let Some(problem) = policy_problems.first() {
        return Err(Error::Validation(problem.clone()));
    }
    let accepted = accepted_facts(&subject, policy, hash_algo, hash_value, facts);
    let signers = unique_sorted(accepted.iter().map(|fact| fact.signer.clone()).collect());
    let missing_required = missing_required_signers(&policy.required_signers, &signers);
    if let Some(signer) = missing_required.first() {
        return Err(Error::Validation(format!("{} missing required trust signer '{signer}'", subject.label())));
    }
    if signers.len() as u64 <= policy.quorum.saturating_sub(MIN_TRUST_QUORUM) as u64 {
        return Err(Error::Validation(format!(
            "{} trust quorum not satisfied: {} accepted signer(s), require {}",
            subject.label(),
            signers.len(),
            policy.quorum
        )));
    }
    if accepted.is_empty() {
        return Err(Error::Validation(format!("{} has no accepted trust evidence", subject.label())));
    }
    Ok(LockedTrust {
        schema: TRUST_POLICY_SCHEMA.to_string(),
        subject: subject.clone(),
        policy_digest: trust_policy_digest(policy),
        verifier: policy.verifier,
        digest_binding: policy.digest_binding,
        hash_algo: hash_algo.clone(),
        hash_value: hash_value.to_string(),
        signers,
        key_refs: unique_sorted(accepted.iter().map(|fact| fact.key_ref.clone()).collect()),
        signature_refs: unique_sorted(accepted.iter().map(|fact| fact.signature_ref.clone()).collect()),
        claim: bounded_trust_claim(&subject, policy.verifier, hash_algo, hash_value),
    })
}

pub fn validate_locked_trust(
    trust: &LockedTrust,
    subject: &TrustSubject,
    hash_algo: &HashAlgo,
    hash_value: &str,
    context: impl AsRef<str>,
) -> Vec<String> {
    validate_locked_trust_fields(LockedTrustValidation {
        trust,
        subject,
        hash_algo,
        hash_value,
        context: context.as_ref(),
    })
}

struct LockedTrustValidation<'a> {
    trust: &'a LockedTrust,
    subject: &'a TrustSubject,
    hash_algo: &'a HashAlgo,
    hash_value: &'a str,
    context: &'a str,
}

fn validate_locked_trust_fields(validation: LockedTrustValidation<'_>) -> Vec<String> {
    let mut problems = Vec::new();
    let trust = validation.trust;
    let context = validation.context;
    if trust.schema != TRUST_POLICY_SCHEMA {
        problems.push(format!("{context}: trust schema '{}' is unsupported", trust.schema));
    }
    if trust.subject != *validation.subject {
        problems.push(format!("{context}: trust subject does not match {}", validation.subject.label()));
    }
    if &trust.hash_algo != validation.hash_algo {
        problems.push(format!("{context}: trust hash algorithm does not match locked hash"));
    }
    if trust.hash_value != validation.hash_value {
        problems.push(format!("{context}: trust hash value does not match locked hash"));
    }
    if trust.signers.is_empty() {
        problems.push(format!("{context}: trust evidence has no signers"));
    }
    if trust.policy_digest.is_empty() {
        problems.push(format!("{context}: trust evidence has empty policy digest"));
    }
    if !trust.policy_digest.starts_with(TRUST_DIGEST_PREFIX) {
        problems.push(format!("{context}: trust policy digest must be BLAKE3-prefixed"));
    }
    if !trust.claim.contains(PROJECT_INPUT_TRUST_NON_CLAIM) {
        problems.push(format!("{context}: trust claim is missing bounded non-claim text"));
    }
    debug_assert!(problems.len() <= MAX_LOCKED_TRUST_PROBLEMS);
    debug_assert!(problems.iter().all(|problem| problem.starts_with(context)));
    problems
}

pub fn trust_policy_matches_locked(policy: &InputTrustPolicy, locked: &LockedTrust) -> bool {
    locked.schema == TRUST_POLICY_SCHEMA && locked.policy_digest == trust_policy_digest(policy)
}

fn accepted_facts<'a>(
    subject: &TrustSubject,
    policy: &InputTrustPolicy,
    hash_algo: &HashAlgo,
    hash_value: &str,
    facts: &'a [VerifiedTrustFact],
) -> Vec<&'a VerifiedTrustFact> {
    let trusted_keys: BTreeSet<String> = policy.trusted_public_keys.iter().cloned().collect();
    let trusted_names: BTreeSet<String> =
        policy.trusted_public_keys.iter().filter_map(|key_ref| key_name_from_key_ref(key_ref)).collect();
    facts
        .iter()
        .filter(|fact| fact.subject == *subject)
        .filter(|fact| fact.verifier == policy.verifier)
        .filter(|fact| fact.digest_binding == policy.digest_binding)
        .filter(|fact| &fact.hash_algo == hash_algo)
        .filter(|fact| fact.hash_value == hash_value)
        .filter(|fact| trusted_keys.contains(&fact.key_ref))
        .filter(|fact| trusted_names.contains(&fact.signer))
        .collect()
}

fn push_signature_ref_problems(policy: &InputTrustPolicy, context: &str, problems: &mut Vec<String>) {
    if policy.signatures.is_empty() {
        problems.push(format!("{context}: trust policy requires at least one signature reference"));
    }
    if policy.signatures.len() as u64 > MAX_TRUST_SIGNATURE_REFS as u64 {
        problems.push(format!("{context}: trust policy has more than {MAX_TRUST_SIGNATURE_REFS} signature references"));
    }
    for signature in &policy.signatures {
        push_ref_text_problem("signature reference", signature.ref_text(), context, problems);
    }
}

fn push_trusted_key_problems(policy: &InputTrustPolicy, context: &str, problems: &mut Vec<String>) {
    let problem_count_before = problems.len();
    if policy.trusted_public_keys.is_empty() {
        problems.push(format!("{context}: trust policy requires at least one trusted public key"));
    }
    if policy.trusted_public_keys.len() as u64 > MAX_TRUSTED_PUBLIC_KEYS as u64 {
        problems.push(format!("{context}: trust policy has more than {MAX_TRUSTED_PUBLIC_KEYS} trusted public keys"));
    }
    let mut names = BTreeSet::new();
    for key_ref in &policy.trusted_public_keys {
        push_ref_text_problem("trusted public key", key_ref, context, problems);
        match key_name_from_key_ref(key_ref) {
            Some(name) => {
                if !names.insert(name.clone()) {
                    problems.push(format!("{context}: trust policy repeats key name '{name}'"));
                }
            }
            None => problems.push(format!("{context}: trusted public key must be name:base64")),
        }
    }
    debug_assert!(problems.len() >= problem_count_before);
    debug_assert!(names.len() <= policy.trusted_public_keys.len());
}

fn push_required_signer_problems(policy: &InputTrustPolicy, context: &str, problems: &mut Vec<String>) {
    if policy.required_signers.len() as u64 > MAX_REQUIRED_SIGNERS as u64 {
        problems.push(format!("{context}: trust policy has more than {MAX_REQUIRED_SIGNERS} required signers"));
    }
    let trusted_names: BTreeSet<String> =
        policy.trusted_public_keys.iter().filter_map(|key_ref| key_name_from_key_ref(key_ref)).collect();
    let mut seen = BTreeSet::new();
    for signer in &policy.required_signers {
        push_ref_text_problem("required signer", signer, context, problems);
        if !seen.insert(signer.clone()) {
            problems.push(format!("{context}: trust policy repeats required signer '{signer}'"));
        }
        if !trusted_names.contains(signer) {
            problems.push(format!("{context}: required signer '{signer}' is not in trusted public keys"));
        }
    }
}

fn push_quorum_problems(policy: &InputTrustPolicy, context: &str, problems: &mut Vec<String>) {
    if policy.quorum < MIN_TRUST_QUORUM {
        problems.push(format!("{context}: trust quorum must be at least {MIN_TRUST_QUORUM}"));
    }
    if policy.quorum as u64 > policy.trusted_public_keys.len() as u64 {
        problems.push(format!(
            "{context}: trust quorum {} exceeds trusted key count {}",
            policy.quorum,
            policy.trusted_public_keys.len()
        ));
    }
}

fn push_ref_text_problem(label: impl AsRef<str>, value: &str, context: impl AsRef<str>, problems: &mut Vec<String>) {
    let label = label.as_ref();
    let context = context.as_ref();
    if value.is_empty() {
        problems.push(format!("{context}: {label} must not be empty"));
    }
    if value.len() as u64 > MAX_TRUST_REF_BYTES as u64 {
        problems.push(format!("{context}: {label} exceeds {MAX_TRUST_REF_BYTES} bytes"));
    }
    if value.chars().any(|ch| ch.is_control()) {
        problems.push(format!("{context}: {label} contains a control character"));
    }
}

fn key_name_from_key_ref(key_ref: &str) -> Option<String> {
    let (name, material) = key_ref.split_once(':')?;
    if name.is_empty() || material.is_empty() {
        return None;
    }
    if name.chars().any(|ch| ch.is_control() || ch.is_whitespace()) {
        return None;
    }
    Some(name.to_string())
}

fn missing_required_signers(required: &[String], signers: &[String]) -> Vec<String> {
    let accepted: BTreeSet<String> = signers.iter().cloned().collect();
    required.iter().filter(|signer| !accepted.contains(*signer)).cloned().collect()
}

fn unique_sorted(values: Vec<String>) -> Vec<String> {
    let set: BTreeSet<String> = values.into_iter().collect();
    set.into_iter().collect()
}

fn bounded_trust_claim(
    subject: &TrustSubject,
    verifier: TrustVerifierKind,
    hash_algo: &HashAlgo,
    hash_value: &str,
) -> String {
    format!(
        "{} satisfied {} trust evidence for {}:{}; {}",
        subject.label(),
        verifier.as_str(),
        hash_algo,
        hash_value,
        PROJECT_INPUT_TRUST_NON_CLAIM
    )
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::vec;

    use super::*;

    const TEST_HASH: &str = "0123456789abcdef";
    const TEST_KEY_ONE: &str = "alice:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
    const TEST_KEY_TWO: &str = "bob:BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB=";

    fn policy() -> InputTrustPolicy {
        InputTrustPolicy {
            verifier: TrustVerifierKind::Ed25519Detached,
            signatures: vec![TrustSignatureRef::LocalFile {
                path: "pkg.sig".to_string(),
            }],
            trusted_public_keys: vec![TEST_KEY_ONE.to_string()],
            required_signers: vec!["alice".to_string()],
            quorum: 1,
            digest_binding: TrustDigestBinding::ContentHash,
        }
    }

    fn fact(signer: &str, key_ref: &str, hash_value: &str) -> VerifiedTrustFact {
        VerifiedTrustFact {
            subject: TrustSubject::input("pkg".to_string()),
            verifier: TrustVerifierKind::Ed25519Detached,
            digest_binding: TrustDigestBinding::ContentHash,
            hash_algo: HashAlgo::Sha256,
            hash_value: hash_value.to_string(),
            signer: signer.to_string(),
            key_ref: key_ref.to_string(),
            signature_ref: "pkg.sig".to_string(),
        }
    }

    #[test]
    fn trust_policy_accepts_matching_required_signer() {
        let subject = TrustSubject::input("pkg".to_string());
        let locked = evaluate_trust_policy(subject.clone(), &policy(), &HashAlgo::Sha256, TEST_HASH, &[fact(
            "alice",
            TEST_KEY_ONE,
            TEST_HASH,
        )])
        .expect("matching fact should satisfy policy");

        assert_eq!(locked.subject, subject);
        assert_eq!(locked.signers, vec!["alice".to_string()]);
        assert!(locked.claim.contains(PROJECT_INPUT_TRUST_NON_CLAIM));
        assert!(locked.policy_digest.starts_with(TRUST_DIGEST_PREFIX));
    }

    #[test]
    fn trust_policy_rejects_untrusted_signer_and_bad_quorum() {
        let subject = TrustSubject::input("pkg".to_string());
        let err = evaluate_trust_policy(subject, &policy(), &HashAlgo::Sha256, TEST_HASH, &[fact(
            "mallory",
            TEST_KEY_ONE,
            TEST_HASH,
        )])
        .expect_err("wrong signer must not satisfy policy");

        assert!(err.message().contains("missing required trust signer"));
        assert!(!err.message().contains("build reproducibility"));
    }

    #[test]
    fn trust_policy_requires_declared_evidence_and_bounds_quorum() {
        let mut invalid = policy();
        invalid.signatures.clear();
        invalid.trusted_public_keys = vec![TEST_KEY_ONE.to_string()];
        invalid.quorum = 2;

        let problems = invalid.validate("input 'pkg'");
        assert!(problems.iter().any(|problem| problem.contains("at least one signature reference")));
        assert!(problems.iter().any(|problem| problem.contains("exceeds trusted key count")));
    }

    #[test]
    fn trust_policy_rejects_malformed_key_refs_and_unknown_verifier() {
        let mut invalid = policy();
        invalid.trusted_public_keys = vec!["not-a-key-ref".to_string()];
        invalid.required_signers = vec!["alice".to_string()];

        let problems = invalid.validate("input 'pkg'");
        assert!(problems.iter().any(|problem| problem.contains("trusted public key must be name:base64")));
        assert!(
            problems
                .iter()
                .any(|problem| problem.contains("required signer 'alice' is not in trusted public keys"))
        );

        let unsupported = r#"{"verifier":"pgp-detached","signatures":[],"trusted_public_keys":[]}"#;
        let err = serde_json::from_str::<InputTrustPolicy>(unsupported).expect_err("unsupported verifier must fail");
        assert!(err.to_string().contains("unknown variant"));
    }

    #[test]
    fn locked_trust_validation_rejects_wrong_digest_binding() {
        let subject = TrustSubject::input("pkg".to_string());
        let mut locked = evaluate_trust_policy(subject.clone(), &policy(), &HashAlgo::Sha256, TEST_HASH, &[fact(
            "alice",
            TEST_KEY_ONE,
            TEST_HASH,
        )])
        .expect("matching fact should satisfy policy");
        locked.hash_value = "different".to_string();

        let problems = validate_locked_trust(&locked, &subject, &HashAlgo::Sha256, TEST_HASH, "pkg lock entry");
        assert!(problems.iter().any(|problem| problem.contains("hash value does not match")));
        assert!(!problems.is_empty());
    }

    #[test]
    fn locked_trust_validation_rejects_overbroad_claim() {
        let subject = TrustSubject::input("pkg".to_string());
        let mut locked = evaluate_trust_policy(subject.clone(), &policy(), &HashAlgo::Sha256, TEST_HASH, &[fact(
            "alice",
            TEST_KEY_ONE,
            TEST_HASH,
        )])
        .expect("matching fact should satisfy policy");
        locked.claim = "input is globally authentic".to_string();

        let problems = validate_locked_trust(&locked, &subject, &HashAlgo::Sha256, TEST_HASH, "pkg lock entry");
        assert!(problems.iter().any(|problem| problem.contains("bounded non-claim")));
        assert!(!problems.is_empty());
    }

    #[test]
    fn quorum_accepts_independent_trusted_signers() {
        let mut two_of_two = policy();
        two_of_two.trusted_public_keys = vec![TEST_KEY_ONE.to_string(), TEST_KEY_TWO.to_string()];
        two_of_two.required_signers.clear();
        two_of_two.quorum = 2;

        let locked = evaluate_trust_policy(
            TrustSubject::input("pkg".to_string()),
            &two_of_two,
            &HashAlgo::Sha256,
            TEST_HASH,
            &[
                fact("alice", TEST_KEY_ONE, TEST_HASH),
                fact("bob", TEST_KEY_TWO, TEST_HASH),
            ],
        )
        .expect("two trusted facts should satisfy quorum");

        assert_eq!(locked.signers, vec!["alice".to_string(), "bob".to_string()]);
        assert!(trust_policy_matches_locked(&two_of_two, &locked));
    }
}
