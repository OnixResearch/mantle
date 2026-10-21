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
    let chain = FilterChain::from_records(&records).ok()?;
    chain.admit_with_policy(original, site_policy)
}

/// Gateway effects retain the exact signed target. Other attenuation sites can
/// consume `intersect` only after independently binding their rewritten effect.
pub fn admits(caveats: &[CaveatDocument], original: &str, site_policy: &(dyn Fn(&str) -> bool + Sync)) -> bool {
    intersect(caveats, original, site_policy).is_some_and(|output| output == original)
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
