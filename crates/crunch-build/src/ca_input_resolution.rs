//! Pure, bounded resolution of floating content-addressed input edges.
//!
//! The caller supplies already-admitted signed realisation facts. This module
//! never discovers records, verifies signatures, or reads the store.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::sync::Arc;

use bstr::BString;
use bstr::ByteSlice;
use nix_compat::derivation::Derivation;
use nix_compat::store_path::StorePath;

pub const MAX_CA_INPUT_EDGES: usize = 256;
pub const MAX_CA_INPUT_OUTPUTS: usize = 1024;
pub const MAX_CA_PATH_BYTES: usize = 4096;
pub const MAX_RESOLVED_VALUE_BYTES: usize = 1_048_576;
const RESOLVED_IDENTITY_PREFIX: &str = "mantle-resolved-derivation://blake3/";
const RESOLVED_IDENTITY_DOMAIN: &[u8] = b"mantle.resolved-derivation.v1";
const ACTION_REF_PREFIX: &str = "mantle-action://blake3/";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedResolvedInput {
    pub derivation: StorePath<String>,
    pub output: String,
    pub provisional: String,
    pub realized: StorePath<String>,
    pub signed_action_ref: String,
}

#[derive(Debug, Clone)]
pub struct ResolvedDerivation {
    pub derivation: Arc<Derivation>,
    pub path: StorePath<String>,
    pub identity: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ResolutionFailure {
    #[error("ca-input-unrealized")]
    Unrealized,
    #[error("ca-realisation-conflict")]
    Conflict,
    #[error("ca-realisation-untrusted")]
    Untrusted,
    #[error("ca-resolution-limit")]
    Limit,
}

fn substitute(value: &[u8], replacements: &[(Vec<u8>, Vec<u8>)]) -> Result<Option<Vec<u8>>, ResolutionFailure> {
    if value.len() > MAX_RESOLVED_VALUE_BYTES {
        return Err(ResolutionFailure::Limit);
    }
    let mut rewritten: Option<Vec<u8>> = None;
    let mut cursor = 0;
    while cursor < value.len() {
        let next = replacements
            .iter()
            .filter_map(|(old, new)| value[cursor..].find(old.as_slice()).map(|offset| (cursor + offset, old, new)))
            .min_by(|left, right| left.0.cmp(&right.0).then(right.1.len().cmp(&left.1.len())));
        let Some((start, old, new)) = next else {
            break;
        };
        let result = rewritten.get_or_insert_with(|| Vec::with_capacity(value.len()));
        result.extend_from_slice(&value[cursor..start]);
        result.extend_from_slice(new);
        if result.len() > MAX_RESOLVED_VALUE_BYTES {
            return Err(ResolutionFailure::Limit);
        }
        cursor = start + old.len();
    }
    if let Some(result) = &mut rewritten {
        result.extend_from_slice(&value[cursor..]);
        if result.len() > MAX_RESOLVED_VALUE_BYTES {
            return Err(ResolutionFailure::Limit);
        }
    }
    Ok(rewritten)
}

fn valid_action_ref(action_ref: &str) -> bool {
    action_ref.strip_prefix(ACTION_REF_PREFIX).is_some_and(|digest| {
        digest.len() == blake3::OUT_LEN * 2
            && digest.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    })
}

/// Resolve admitted direct CA inputs and previously CA-resolved intermediates.
/// CA-independent inputs return the same Arc and path, preserving their identities.
/// The `admitted` facts must be authenticated by the shell before this call.
// r[impl mantle.ca_input_resolution.resolved_derivation]
pub fn resolve_ca_inputs(
    original: Arc<Derivation>,
    original_path: &StorePath<String>,
    admitted: &[AdmittedResolvedInput],
    remaining_input_hdms: &BTreeMap<StorePath<&str>, [u8; 32]>,
    store_dir: &str,
) -> Result<ResolvedDerivation, ResolutionFailure> {
    assert!(!store_dir.is_empty(), "store prefix must not be empty");
    assert!(store_dir.starts_with('/'), "store prefix must be absolute");
    if admitted.is_empty() {
        return Ok(ResolvedDerivation {
            derivation: original,
            path: original_path.clone(),
            identity: None,
        });
    }
    if admitted.len() > MAX_CA_INPUT_OUTPUTS {
        return Err(ResolutionFailure::Limit);
    }
    let mut replaced_outputs = BTreeMap::<StorePath<String>, BTreeSet<String>>::new();
    let mut replacements = BTreeMap::<String, String>::new();
    let mut resolved = (*original).clone();
    for fact in admitted {
        if !valid_action_ref(&fact.signed_action_ref) {
            return Err(ResolutionFailure::Untrusted);
        }
        if fact.provisional.is_empty() {
            return Err(ResolutionFailure::Unrealized);
        }
        if fact.provisional.len() > MAX_CA_PATH_BYTES {
            return Err(ResolutionFailure::Limit);
        }
        if StorePath::<String>::from_absolute_path_with_prefix(fact.provisional.as_bytes(), store_dir).is_err() {
            return Err(ResolutionFailure::Untrusted);
        }
        let realized_abs = fact.realized.to_absolute_path_with_prefix(store_dir);
        if realized_abs.len() > MAX_CA_PATH_BYTES {
            return Err(ResolutionFailure::Limit);
        }
        let Some(names) = original.input_derivations.get(&fact.derivation) else {
            return Err(ResolutionFailure::Unrealized);
        };
        if !names.contains(&fact.output) {
            return Err(ResolutionFailure::Unrealized);
        }
        let previous = replacements.insert(fact.provisional.clone(), realized_abs);
        if previous.is_some_and(|old| old != replacements[&fact.provisional]) {
            return Err(ResolutionFailure::Conflict);
        }
        if !replaced_outputs.contains_key(&fact.derivation) && replaced_outputs.len() >= MAX_CA_INPUT_EDGES {
            return Err(ResolutionFailure::Limit);
        }
        replaced_outputs.entry(fact.derivation.clone()).or_default().insert(fact.output.clone());
        resolved.input_sources.insert(fact.realized.clone());
    }
    for (drv_path, output_names) in &replaced_outputs {
        if original.input_derivations.get(drv_path) != Some(output_names) {
            return Err(ResolutionFailure::Unrealized);
        }
        resolved.input_derivations.remove(drv_path);
    }
    for parent in resolved.input_derivations.keys() {
        if !remaining_input_hdms.contains_key(&parent.as_ref()) {
            return Err(ResolutionFailure::Unrealized);
        }
    }
    let pairs = replacements
        .into_iter()
        .filter(|(old, new)| old != new)
        .map(|(old, new)| (old.into_bytes(), new.into_bytes()))
        .collect::<Vec<_>>();
    for argument in &mut resolved.arguments {
        if let Some(bytes) = substitute(argument.as_bytes(), &pairs)? {
            *argument = String::from_utf8(bytes).map_err(|_| ResolutionFailure::Untrusted)?;
        }
    }
    for value in resolved.environment.values_mut() {
        if let Some(bytes) = substitute(value.as_slice(), &pairs)? {
            *value = BString::from(bytes);
        }
    }
    let is_floating_ca = resolved.outputs.values().all(|output| output.path.is_none() && output.ca_hash.is_none());
    if !is_floating_ca {
        for (name, output) in &mut resolved.outputs {
            output.path = None;
            resolved.environment.insert(name.clone(), BString::from(""));
        }
    } else {
        // The provisional paths of this CA action also depend on its HDM.
        for name in resolved.outputs.keys() {
            resolved.environment.insert(name.clone(), BString::from(""));
        }
    }
    let hdm = resolved.hash_derivation_modulo_with_store_dir(
        |parent| *remaining_input_hdms.get(parent).expect("remaining input HDMs validated above"),
        store_dir,
    );
    let name = original_path.name().strip_suffix(".drv").ok_or(ResolutionFailure::Untrusted)?;
    resolved
        .calculate_output_paths_with_store_dir(name, &hdm, store_dir)
        .map_err(|_| ResolutionFailure::Untrusted)?;
    if is_floating_ca {
        for output in resolved.outputs.values_mut() {
            output.path = None;
        }
    }
    let path = resolved
        .calculate_derivation_path_with_store_dir(name, store_dir)
        .map_err(|_| ResolutionFailure::Untrusted)?;
    let mut hash = blake3::Hasher::new();
    hash.update(RESOLVED_IDENTITY_DOMAIN);
    hash.update(&[0]);
    hash.update(&resolved.to_aterm_bytes_with_store_dir(store_dir));
    let identity = format!("{RESOLVED_IDENTITY_PREFIX}{}", hash.finalize().to_hex());
    debug_assert!(resolved.input_derivations.len() < original.input_derivations.len());
    debug_assert!(identity.starts_with(RESOLVED_IDENTITY_PREFIX));
    Ok(ResolvedDerivation {
        derivation: Arc::new(resolved),
        path,
        identity: Some(identity),
    })
}

#[cfg(test)]
mod tests {
    use nix_compat::derivation::Output;

    use super::*;

    fn path(name: &str, byte: u8) -> StorePath<String> {
        StorePath::from_name_and_digest_fixed(name, [byte; 20]).unwrap()
    }

    fn dependent(parent: StorePath<String>, provisional: &str) -> (Arc<Derivation>, StorePath<String>) {
        let mut derivation = Derivation {
            arguments: vec!["-c".into(), format!("cp {provisional}/result $out")],
            builder: "/bin/sh".into(),
            environment: BTreeMap::from([
                ("out".into(), BString::from("")),
                ("source".into(), BString::from(provisional)),
            ]),
            input_derivations: BTreeMap::from([(parent, BTreeSet::from(["out".into()]))]),
            input_sources: BTreeSet::new(),
            outputs: BTreeMap::from([("out".into(), Output {
                path: None,
                ca_hash: None,
            })]),
            system: "x86_64-linux".into(),
        };
        let hdm = derivation.hash_derivation_modulo_with_store_dir(|_| [7; 32], "/nix/store");
        derivation.calculate_output_paths_with_store_dir("dependent", &hdm, "/nix/store").unwrap();
        let drv_path = derivation.calculate_derivation_path_with_store_dir("dependent", "/nix/store").unwrap();
        (Arc::new(derivation), drv_path)
    }

    fn fact(parent: StorePath<String>, provisional: &str) -> AdmittedResolvedInput {
        AdmittedResolvedInput {
            derivation: parent,
            output: "out".into(),
            provisional: provisional.into(),
            realized: path("same-content", 42),
            signed_action_ref: format!("{ACTION_REF_PREFIX}{}", "a".repeat(64)),
        }
    }

    // r[verify mantle.ca_input_resolution.resolved_derivation]
    // r[verify mantle.ca_input_resolution.resolved_identity]
    #[test]
    fn identical_realized_ca_content_unifies_distinct_unresolved_derivations() {
        let p1 = path("producer-a.drv", 1);
        let p2 = path("producer-b.drv", 2);
        let provisional_a = path("provisional-a", 3).to_absolute_path();
        let provisional_b = path("provisional-b", 4).to_absolute_path();
        let (left, left_path) = dependent(p1.clone(), &provisional_a);
        let (right, right_path) = dependent(p2.clone(), &provisional_b);
        assert_ne!(left_path, right_path);
        let a =
            resolve_ca_inputs(left, &left_path, &[fact(p1, &provisional_a)], &BTreeMap::new(), "/nix/store").unwrap();
        let b =
            resolve_ca_inputs(right, &right_path, &[fact(p2, &provisional_b)], &BTreeMap::new(), "/nix/store").unwrap();
        assert_eq!(a.path, b.path);
        assert_eq!(a.identity, b.identity);
        assert_eq!(a.derivation.arguments, b.derivation.arguments);
        assert_eq!(a.derivation.environment["source"], b.derivation.environment["source"]);
        assert!(a.derivation.input_derivations.is_empty());
        assert!(a.derivation.input_sources.contains(&path("same-content", 42)));
        let again = resolve_ca_inputs(a.derivation.clone(), &a.path, &[], &BTreeMap::new(), "/nix/store").unwrap();
        assert!(Arc::ptr_eq(&again.derivation, &a.derivation));
        assert_eq!(again.path, a.path);
    }

    // r[verify mantle.ca_input_resolution.resolved_identity]
    #[test]
    fn unresolved_ia_input_keeps_its_hdm_when_ca_sibling_is_resolved() {
        let ca_parent = path("ca-parent.drv", 1);
        let ia_parent = path("ia-parent.drv", 2);
        let provisional = path("provisional", 3).to_absolute_path();
        let (single, _) = dependent(ca_parent.clone(), &provisional);
        let mut derivation = (*single).clone();
        derivation.input_derivations.insert(ia_parent.clone(), BTreeSet::from(["out".into()]));
        derivation.outputs.get_mut("out").unwrap().path = None;
        derivation.environment.insert("out".into(), BString::from(""));
        let original_hdm = derivation.hash_derivation_modulo_with_store_dir(|_| [7; 32], "/nix/store");
        derivation.calculate_output_paths_with_store_dir("dependent", &original_hdm, "/nix/store").unwrap();
        let original_path = derivation.calculate_derivation_path_with_store_dir("dependent", "/nix/store").unwrap();
        let derivation = Arc::new(derivation);
        let ia_key = ia_parent.as_ref();
        let first = resolve_ca_inputs(
            derivation.clone(),
            &original_path,
            &[fact(ca_parent.clone(), &provisional)],
            &BTreeMap::from([(ia_key.clone(), [9; 32])]),
            "/nix/store",
        )
        .unwrap();
        let second = resolve_ca_inputs(
            derivation.clone(),
            &original_path,
            &[fact(ca_parent.clone(), &provisional)],
            &BTreeMap::from([(ia_key, [10; 32])]),
            "/nix/store",
        )
        .unwrap();
        assert_ne!(first.path, second.path);
        assert_ne!(first.derivation.outputs["out"].path, second.derivation.outputs["out"].path);
        assert_eq!(first.derivation.input_derivations.len(), 1);
        assert!(first.derivation.input_derivations.contains_key(&ia_parent));
        assert!(first.derivation.input_sources.contains(&path("same-content", 42)));
        let unrelated = path("unrelated.drv", 4);
        assert_eq!(
            resolve_ca_inputs(
                derivation,
                &original_path,
                &[fact(ca_parent, &provisional)],
                &BTreeMap::from([(unrelated.as_ref(), [9; 32])]),
                "/nix/store",
            )
            .unwrap_err(),
            ResolutionFailure::Unrealized,
        );
    }

    #[test]
    fn substitution_uses_original_bytes_and_prefers_longest_overlapping_path() {
        let chained = [
            (b"/store/first".to_vec(), b"/store/second".to_vec()),
            (b"/store/second".to_vec(), b"/store/third".to_vec()),
        ];
        assert_eq!(
            substitute(b"/store/first and /store/second", &chained).unwrap().unwrap(),
            b"/store/second and /store/third",
        );
        let overlapping = [
            (b"/store/a".to_vec(), b"short".to_vec()),
            (b"/store/ab".to_vec(), b"long".to_vec()),
        ];
        assert_eq!(substitute(b"/store/ab:/store/a", &overlapping).unwrap().unwrap(), b"long:short");
    }

    // r[verify mantle.ca_input_resolution.negative_controls]
    #[test]
    fn missing_wrong_domain_and_over_limit_facts_fail_closed() {
        let parent = path("producer.drv", 1);
        let provisional = path("provisional", 3).to_absolute_path();
        let (derivation, original_path) = dependent(parent.clone(), &provisional);
        assert_eq!(
            resolve_ca_inputs(derivation.clone(), &original_path, &[], &BTreeMap::new(), "/nix/store")
                .unwrap()
                .path,
            original_path
        );
        let mut untrusted = fact(parent.clone(), &provisional);
        untrusted.signed_action_ref = "mantle-object://blake3/".to_string() + &"a".repeat(64);
        assert_eq!(
            resolve_ca_inputs(derivation.clone(), &original_path, &[untrusted], &BTreeMap::new(), "/nix/store")
                .unwrap_err(),
            ResolutionFailure::Untrusted
        );
        let mut malformed = fact(parent.clone(), "not-a-store-path");
        assert_eq!(
            resolve_ca_inputs(derivation.clone(), &original_path, &[malformed.clone()], &BTreeMap::new(), "/nix/store")
                .unwrap_err(),
            ResolutionFailure::Untrusted,
        );
        malformed.provisional = "x".repeat(MAX_CA_PATH_BYTES + 1);
        assert_eq!(
            resolve_ca_inputs(derivation.clone(), &original_path, &[malformed], &BTreeMap::new(), "/nix/store")
                .unwrap_err(),
            ResolutionFailure::Limit,
        );
        let missing = fact(path("wrong.drv", 8), &provisional);
        assert_eq!(
            resolve_ca_inputs(derivation.clone(), &original_path, &[missing], &BTreeMap::new(), "/nix/store")
                .unwrap_err(),
            ResolutionFailure::Unrealized
        );
        let too_many = vec![fact(parent, &provisional); MAX_CA_INPUT_OUTPUTS + 1];
        assert_eq!(
            resolve_ca_inputs(derivation, &original_path, &too_many, &BTreeMap::new(), "/nix/store").unwrap_err(),
            ResolutionFailure::Limit
        );
    }
}
