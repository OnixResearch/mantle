//! Exact registered Mantle filter payload decoder and non-consuming UCAN policy.
//! JSON only: duplicate/unknown fields, malformed variants and ordinal gaps deny.
use basalt::ucan::AuthorizationRequest;
use basalt::ucan::CaveatDecision;
use basalt::ucan::CaveatDocument;
use basalt::ucan::CaveatIdentifier;
use basalt::ucan::CaveatPolicy;
use basalt::ucan::CaveatPolicySet;
use crunch_remote_core::caveat::FilterChain;
use crunch_remote_core::caveat::WireCaveat;
use crunch_remote_core::caveat::WireCaveatRecord;
use crunch_remote_core::caveat::WireRewrite;
use crunch_remote_core::caveat::{self};
use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Rewrite<'a> {
    #[serde(borrow)]
    pattern: &'a str,
    #[serde(borrow)]
    template: &'a str,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Payload<'a> {
    #[serde(borrow)]
    kind: &'a str,
    #[serde(default, borrow)]
    pattern: Option<&'a str>,
    #[serde(default, borrow)]
    template: Option<&'a str>,
    #[serde(default, borrow)]
    alternatives: Vec<Rewrite<'a>>,
}

/// The host supplies the ORIGINAL logical target and a pure receiver-owned site
/// policy. No caller assertion, credential verification, replay or effect here.
pub struct PatternPolicies<'a> {
    pub original: &'a str,
    /// Exact effect output chosen by the receiver; gateway sites use original.
    pub expected: &'a str,
    pub site_policy: &'a (dyn Fn(&str) -> bool + Sync),
    /// First authenticated capability selected by canonical UCAN containment.
    pub selected: &'a basalt::ucan::CapabilityDocument,
}
impl CaveatPolicySet for PatternPolicies<'_> {
    fn policy_for(&self, id: &CaveatIdentifier) -> Option<&dyn CaveatPolicy> {
        (id.domain() == "mantle" && id.caveat_type() == "filter-v1").then_some(self)
    }
}
impl CaveatPolicy for PatternPolicies<'_> {
    fn evaluate(&self, caveat: &CaveatDocument, _request: &AuthorizationRequest) -> CaveatDecision {
        if self.selected.caveats.contains(caveat)
            && intersect(&self.selected.caveats, self.original, self.site_policy)
                .is_some_and(|output| output == self.expected)
        {
            CaveatDecision::Satisfied
        } else {
            CaveatDecision::Rejected {
                message: "gateway-authority-denied".into(),
            }
        }
    }
}

/// Intersect the exact effective signed caveat list, in preserved ordinal order.
/// UCAN authenticates parent payload preservation before calling this policy.
/// Returns the bounded pure rewrite result, not a live grant or an effect.
pub fn intersect<'v>(
    caveats: &[CaveatDocument],
    original: &'v str,
    site_policy: &(dyn Fn(&str) -> bool + Sync),
) -> Option<std::borrow::Cow<'v, str>> {
    with_decoded_chain(caveats, |chain| chain.admit_with_policy(original, site_policy))
}

/// Strictly decode the registered payloads and hand the borrowed chain to `f`.
/// A chain that fails any bound, ordinal or payload check yields `None`.
fn with_decoded_chain<R>(caveats: &[CaveatDocument], f: impl FnOnce(&FilterChain<'_>) -> Option<R>) -> Option<R> {
    if caveats.len() > caveat::MAX_CHAIN {
        return None;
    }
    let mut payloads = Vec::with_capacity(caveats.len());
    for record in caveats {
        if record.domain() != "mantle"
            || record.caveat_type() != "filter-v1"
            || record.payload().is_empty()
            || record.payload().len() > caveat::MAX_CAVEAT_BYTES
        {
            return None;
        }
        let payload = serde_json::from_slice::<Payload<'_>>(record.payload()).ok()?;
        if payload.alternatives.len() > caveat::MAX_ALTERNATIVES {
            return None;
        }
        payloads.push(payload);
    }
    let rewrites: Vec<Vec<WireRewrite<'_>>> = payloads
        .iter()
        .map(|p| {
            p.alternatives
                .iter()
                .map(|r| WireRewrite {
                    pattern: r.pattern,
                    template: r.template,
                })
                .collect()
        })
        .collect();
    let records: Vec<_> = caveats
        .iter()
        .zip(&payloads)
        .zip(&rewrites)
        .map(|((record, payload), rules)| WireCaveatRecord {
            key: record.key(),
            serialized_payload: record.payload(),
            caveat: WireCaveat {
                kind: payload.kind,
                pattern: payload.pattern,
                template: payload.template,
                alternatives: rules,
            },
        })
        .collect();
    debug_assert_eq!(records.len(), caveats.len());
    debug_assert!(records.len() <= caveat::MAX_CHAIN);
    let chain = FilterChain::from_records(&records).ok()?;
    f(&chain)
}

/// Gateway effects retain the exact signed target. Other attenuation sites can
/// consume `intersect` only after independently binding their rewritten effect.
pub fn admits(caveats: &[CaveatDocument], original: &str, site_policy: &(dyn Fn(&str) -> bool + Sync)) -> bool {
    intersect(caveats, original, site_policy).is_some_and(|output| output == original)
}

const LEGACY_DELEGATION: &str = "legacy-bearer-not-delegated";
const UCAN_DELEGATION: &str = "holder-attenuated-ucan";
const STATUS_NO_CAVEATS: &str = "no-caveats";
const STATUS_FILTER_CHAIN: &str = "bounded-filter-chain";
const STATUS_DENIES_ALL: &str = "denies-everything";
const ORDINAL_INVALID: &str = "invalid";

/// One rewrite pair of a reported caveat. Patterns are grant content, bounded by
/// the registered policy; they are never rejected-request or bearer material.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EffectiveRewrite {
    pub pattern: String,
    pub template: String,
}

/// One signed caveat as the receiver interpreted it. A caveat that the filter
/// language would not accept reports only its digest, never its payload text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EffectiveCaveat {
    pub ordinal: String,
    pub kind: &'static str,
    pub payload_blake3: String,
    pub pattern: Option<String>,
    pub template: Option<String>,
    pub alternatives: Vec<EffectiveRewrite>,
}

/// Bounded, redacted description of the effective caveat chain of one grant.
/// It is an observation for reviewers and local logs: it never carries a bearer
/// secret, compact token, signature, or any denied request detail.
// r[impl mantle.authority_attenuation.composition_order]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EffectiveCaveatChain {
    pub delegation: &'static str,
    pub status: &'static str,
    pub caveat_count: u32,
    pub chain_blake3: String,
    pub caveats: Vec<EffectiveCaveat>,
}

impl EffectiveCaveatChain {
    /// A legacy bearer ticket carries no holder caveats and is never described
    /// as delegated or attenuated.
    pub fn legacy_bearer() -> Self {
        let chain = Self {
            delegation: LEGACY_DELEGATION,
            status: STATUS_NO_CAVEATS,
            caveat_count: 0,
            chain_blake3: chain_digest(&[]),
            caveats: Vec::new(),
        };
        debug_assert!(chain.caveats.is_empty());
        debug_assert_ne!(chain.delegation, UCAN_DELEGATION);
        chain
    }

    /// Describe the exact signed caveats of a verified capability, in preserved
    /// ordinal order. Over-bound chains report only their count: nothing is
    /// parsed beyond the registered bound.
    pub fn from_documents(caveats: &[CaveatDocument]) -> Self {
        let caveat_count = u32::try_from(caveats.len()).unwrap_or(u32::MAX);
        if caveats.len() > caveat::MAX_CHAIN {
            return Self {
                delegation: UCAN_DELEGATION,
                status: STATUS_DENIES_ALL,
                caveat_count,
                chain_blake3: chain_digest(&[]),
                caveats: Vec::new(),
            };
        }
        let described: Vec<EffectiveCaveat> = caveats.iter().map(describe_caveat).collect();
        let is_decodable = with_decoded_chain(caveats, |_| Some(())).is_some();
        let denies_all = described.iter().any(|entry| entry.kind.ends_with("deny-all"));
        let status = match (caveats.is_empty(), is_decodable && !denies_all) {
            (true, _) => STATUS_NO_CAVEATS,
            (false, true) => STATUS_FILTER_CHAIN,
            (false, false) => STATUS_DENIES_ALL,
        };
        debug_assert!(described.len() <= caveat::MAX_CHAIN);
        debug_assert_eq!(described.len(), caveats.len());
        Self {
            delegation: UCAN_DELEGATION,
            status,
            caveat_count,
            chain_blake3: chain_digest(caveats),
            caveats: described,
        }
    }
}

fn chain_digest(caveats: &[CaveatDocument]) -> String {
    debug_assert!(caveats.len() <= caveat::MAX_CHAIN);
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mantle/authority-caveats/effective-chain/v1\0");
    for record in caveats {
        for field in [record.key().as_bytes(), record.payload()] {
            hasher.update(&u64::try_from(field.len()).unwrap_or(u64::MAX).to_be_bytes());
            hasher.update(field);
        }
    }
    let digest = hasher.finalize().to_hex().to_string();
    debug_assert_eq!(digest.len(), 64);
    digest
}

fn describe_caveat(record: &CaveatDocument) -> EffectiveCaveat {
    let ordinal_is_valid = record.key().len() == 2 && record.key().bytes().all(|byte| byte.is_ascii_digit());
    let mut entry = EffectiveCaveat {
        ordinal: if ordinal_is_valid {
            record.key().into()
        } else {
            ORDINAL_INVALID.into()
        },
        kind: "malformed-deny-all",
        payload_blake3: blake3::hash(record.payload()).to_hex().to_string(),
        pattern: None,
        template: None,
        alternatives: Vec::new(),
    };
    debug_assert_eq!(entry.payload_blake3.len(), 64);
    if record.domain() != "mantle" || record.caveat_type() != "filter-v1" {
        entry.kind = "unregistered-deny-all";
        return entry;
    }
    if record.payload().is_empty() || record.payload().len() > caveat::MAX_CAVEAT_BYTES {
        return entry;
    }
    let Ok(payload) = serde_json::from_slice::<Payload<'_>>(record.payload()) else {
        return entry;
    };
    let rules: Vec<WireRewrite<'_>> = payload
        .alternatives
        .iter()
        .map(|rule| WireRewrite {
            pattern: rule.pattern,
            template: rule.template,
        })
        .collect();
    let wire = WireCaveat {
        kind: payload.kind,
        pattern: payload.pattern,
        template: payload.template,
        alternatives: &rules,
    };
    if caveat::Caveat::from_wire(wire).is_err() {
        return entry;
    }
    entry.kind = match payload.kind {
        "rewrite" => "rewrite",
        "reject" => "reject",
        "alternatives" => "alternatives",
        _ => "unrecognized-deny-all",
    };
    debug_assert!(entry.alternatives.is_empty());
    if !entry.kind.ends_with("deny-all") {
        entry.pattern = payload.pattern.map(String::from);
        entry.template = payload.template.map(String::from);
        entry.alternatives = rules
            .iter()
            .map(|rule| EffectiveRewrite {
                pattern: rule.pattern.into(),
                template: rule.template.into(),
            })
            .collect();
    }
    entry
}

#[cfg(test)]
mod tests {
    use super::*;
    fn record(key: &str, payload: &[u8]) -> CaveatDocument {
        CaveatDocument::new("mantle".into(), "filter-v1".into(), key.into(), payload.to_vec()).unwrap()
    }
    #[test]
    fn registered_chain_rejects_parent_widening_unknown_and_ambiguous_payloads() {
        let parent = record("00", br#"{"kind":"reject","pattern":"/jobs/private"}"#);
        assert!(admits(std::slice::from_ref(&parent), "/jobs/public", &|_| true));
        assert!(!admits(std::slice::from_ref(&parent), "/jobs/private", &|_| true));
        let child = record("01", br#"{"kind":"rewrite","pattern":"/jobs/private","template":"/jobs/public"}"#);
        assert!(!admits(&[parent, child], "/jobs/private", &|_| true));
        for payload in [
            br#"{"kind":"future"}"#.as_slice(),
            br#"{"kind":"reject","kind":"rewrite","pattern":"/jobs/private"}"#,
            br#"{"kind":"reject","pattern":"/jobs/private","extra":true}"#,
        ] {
            assert!(!admits(&[record("00", payload)], "/jobs/public", &|_| true));
        }
        assert!(!admits(&[record("01", br#"{"kind":"reject","pattern":"/jobs/private"}"#)], "/jobs/public", &|_| {
            true
        }));
    }
    #[test]
    fn registered_rewrite_returns_only_receiver_admitted_project_goal() {
        let chain = [record(
            "00",
            br#"{"kind":"rewrite","pattern":"/project/owner/declared/:goal","template":"/project/owner/goal/:goal"}"#,
        )];
        let same_owner = |value: &str| value.starts_with("/project/owner/");
        assert_eq!(
            intersect(&chain, "/project/owner/declared/check", &same_owner).as_deref(),
            Some("/project/owner/goal/check")
        );
        assert!(intersect(&chain, "/project/other/declared/check", &same_owner).is_none());
        assert!(!admits(&chain, "/project/owner/declared/check", &same_owner));
    }
}
