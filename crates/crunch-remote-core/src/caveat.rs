//! Bounded application filters, not authentication or transferable authority.
//!
//! The shell must verify UCAN holder/proof preservation and current local policy
//! before effects (ADR 0090). Strict payload decoding belongs to that shell:
//! reject duplicate/unknown fields and alternate codecs before constructing the
//! borrowed wire DTOs below. `from_records` bounds the actual serialized bytes
//! and checks their ordinal order; DTO fields must describe those exact bytes.
//! This module neither verifies that correspondence nor invents a token format.
//!
//! All values are absolute, slash-delimited ASCII segments. Matching consumes
//! the whole value; `:name` captures exactly one segment. Alternatives choose
//! the first matching pattern in declaration order, never a later rewrite after
//! an output-bound failure. Appending a filter runs it first, but every ancestor
//! prefix must independently admit the ORIGINAL input under the same pure site
//! policy. Denial is `None`; callers must send no protocol-level feedback.

use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;

// Keep aligned with config/authority-caveats/default.ncl.
pub const MAX_CHAIN: usize = 8;
pub const MAX_ALTERNATIVES: usize = 8;
pub const MAX_PATTERN_BYTES: usize = 256;
pub const MAX_CAVEAT_BYTES: usize = 8192;
pub const MAX_VALUE_BYTES: usize = 512;
pub const MAX_SEGMENTS: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidCaveat {
    Malformed,
    Oversized,
    DuplicateBinding,
    UnboundTemplate,
    ChainTooLong,
    InvalidOrdinal,
}

/// Already decoded fields, borrowed without copying untrusted text.
#[derive(Debug, Clone, Copy)]
pub struct WireRewrite<'a> {
    pub pattern: &'a str,
    pub template: &'a str,
}

#[derive(Debug, Clone, Copy)]
pub struct WireCaveat<'a> {
    pub kind: &'a str,
    pub pattern: Option<&'a str>,
    pub template: Option<&'a str>,
    pub alternatives: &'a [WireRewrite<'a>],
}

/// Only records from the same verified capability may be assembled together.
/// The shell validates the registered UCAN domain/type before this boundary.
#[derive(Debug, Clone, Copy)]
pub struct WireCaveatRecord<'a> {
    pub key: &'a str,
    pub serialized_payload: &'a [u8],
    pub caveat: WireCaveat<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Segment<'a> {
    Literal(&'a str),
    Binding(&'a str),
}

/// An admitted pattern. Private fields prevent bypassing parsing invariants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pattern<'a> {
    segments: Vec<Segment<'a>>,
}

impl<'a> Pattern<'a> {
    pub fn parse(source: &'a str) -> Result<Self, InvalidCaveat> {
        if source.len() > MAX_PATTERN_BYTES {
            return Err(InvalidCaveat::Oversized);
        }
        let count = validate_value(source)?;
        let mut segments = Vec::with_capacity(count);
        for part in source[1..].split('/') {
            if let Some(name) = part.strip_prefix(':') {
                if !binding_name_is_valid(name) {
                    return Err(InvalidCaveat::Malformed);
                }
                if segments.contains(&Segment::Binding(name)) {
                    return Err(InvalidCaveat::DuplicateBinding);
                }
                segments.push(Segment::Binding(name));
            } else {
                segments.push(Segment::Literal(part));
            }
        }
        Ok(Self { segments })
    }

    pub fn matches(&self, value: &str) -> bool {
        validate_value(value).is_ok() && self.bindings(value).is_some()
    }

    fn bindings<'v>(&self, value: &'v str) -> Option<[Option<&'v str>; MAX_SEGMENTS]> {
        let mut captures = [None; MAX_SEGMENTS];
        let mut values = value.strip_prefix('/')?.split('/');
        for (index, segment) in self.segments.iter().enumerate() {
            let candidate = values.next()?;
            match segment {
                Segment::Literal(literal) if *literal != candidate => return None,
                Segment::Literal(_) => {}
                Segment::Binding(_) => captures[index] = Some(candidate),
            }
        }
        if values.next().is_some() {
            return None;
        }
        Some(captures)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TemplateSegment<'a> {
    Literal(&'a str),
    Capture(usize),
}

/// An admitted rewrite with all template references resolved during parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rewrite<'a> {
    pattern: Pattern<'a>,
    template: Vec<TemplateSegment<'a>>,
}

impl<'a> Rewrite<'a> {
    pub fn parse(pattern: &'a str, template: &'a str) -> Result<Self, InvalidCaveat> {
        let pattern = Pattern::parse(pattern)?;
        if template.len() > MAX_PATTERN_BYTES {
            return Err(InvalidCaveat::Oversized);
        }
        let count = validate_value(template)?;
        let mut compiled = Vec::with_capacity(count);
        let mut bound = [false; MAX_SEGMENTS];
        for segment in template[1..].split('/') {
            let part = if let Some(name) = segment.strip_prefix(':') {
                let index = pattern
                    .segments
                    .iter()
                    .position(|segment| *segment == Segment::Binding(name))
                    .ok_or(InvalidCaveat::UnboundTemplate)?;
                if bound[index] {
                    return Err(InvalidCaveat::DuplicateBinding);
                }
                bound[index] = true;
                TemplateSegment::Capture(index)
            } else {
                TemplateSegment::Literal(segment)
            };
            compiled.push(part);
        }
        Ok(Self {
            pattern,
            template: compiled,
        })
    }

    fn instantiate(&self, captures: &[Option<&str>; MAX_SEGMENTS]) -> Option<String> {
        let mut parts = [""; MAX_SEGMENTS];
        let mut bytes = 0_usize;
        for (index, segment) in self.template.iter().enumerate() {
            let part = match segment {
                TemplateSegment::Literal(literal) => *literal,
                TemplateSegment::Capture(index) => captures[*index]?,
            };
            bytes = bytes.checked_add(part.len().checked_add(1)?)?;
            if bytes > MAX_VALUE_BYTES {
                return None;
            }
            parts[index] = part;
        }
        // Allocate only after proving the complete output is bounded.
        let mut output = String::with_capacity(bytes);
        for part in &parts[..self.template.len()] {
            output.push('/');
            output.push_str(part);
        }
        debug_assert_eq!(output.len(), bytes);
        debug_assert!(validate_value(&output).is_ok());
        Some(output)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Filter<'a> {
    Rewrite(Rewrite<'a>),
    Reject(Pattern<'a>),
    Alternatives(Vec<Rewrite<'a>>),
    DenyAll,
}

/// Validated semantic caveat, not a raw wire variant or authorization proof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Caveat<'a>(Filter<'a>);

impl<'a> Caveat<'a> {
    // r[impl mantle.authority_attenuation.caveat_filter_semantics]
    pub fn from_wire(wire: WireCaveat<'a>) -> Result<Self, InvalidCaveat> {
        validate_wire_bounds(wire)?;
        let filter = match wire.kind {
            "rewrite" if wire.alternatives.is_empty() => Filter::Rewrite(Rewrite::parse(
                wire.pattern.ok_or(InvalidCaveat::Malformed)?,
                wire.template.ok_or(InvalidCaveat::Malformed)?,
            )?),
            "reject" if wire.template.is_none() && wire.alternatives.is_empty() => {
                Filter::Reject(Pattern::parse(wire.pattern.ok_or(InvalidCaveat::Malformed)?)?)
            }
            "alternatives" if wire.pattern.is_none() && wire.template.is_none() => {
                if wire.alternatives.is_empty() {
                    return Err(InvalidCaveat::Malformed);
                }
                let mut rewrites = Vec::with_capacity(wire.alternatives.len());
                for rule in wire.alternatives {
                    rewrites.push(Rewrite::parse(rule.pattern, rule.template)?);
                }
                Filter::Alternatives(rewrites)
            }
            "rewrite" | "reject" | "alternatives" => return Err(InvalidCaveat::Malformed),
            _ => Filter::DenyAll,
        };
        Ok(Self(filter))
    }

    fn apply<'v>(&self, value: Cow<'v, str>) -> Option<Cow<'v, str>> {
        match &self.0 {
            Filter::Rewrite(rule) => {
                let captures = rule.pattern.bindings(value.as_ref())?;
                rule.instantiate(&captures).map(Cow::Owned)
            }
            Filter::Reject(pattern) => {
                if pattern.bindings(value.as_ref()).is_some() {
                    None
                } else {
                    Some(value)
                }
            }
            Filter::Alternatives(rules) => {
                for rule in rules {
                    if let Some(captures) = rule.pattern.bindings(value.as_ref()) {
                        return rule.instantiate(&captures).map(Cow::Owned);
                    }
                }
                None
            }
            Filter::DenyAll => None,
        }
    }
}

fn validate_wire_bounds(wire: WireCaveat<'_>) -> Result<(), InvalidCaveat> {
    let is_beyond_kind_bound = wire.kind.len() > MAX_CAVEAT_BYTES;
    let is_beyond_pattern_bound = wire.pattern.is_some_and(|text| text.len() > MAX_PATTERN_BYTES);
    let is_beyond_template_bound = wire.template.is_some_and(|text| text.len() > MAX_PATTERN_BYTES);
    let is_beyond_alternative_bound = wire.alternatives.len() > MAX_ALTERNATIVES;
    let is_beyond_bounds =
        is_beyond_kind_bound || is_beyond_pattern_bound || is_beyond_template_bound || is_beyond_alternative_bound;
    if is_beyond_bounds {
        return Err(InvalidCaveat::Oversized);
    }
    for rule in wire.alternatives {
        if rule.pattern.len() > MAX_PATTERN_BYTES || rule.template.len() > MAX_PATTERN_BYTES {
            return Err(InvalidCaveat::Oversized);
        }
    }
    Ok(())
}

/// A locally parsed chain, ordered oldest to newest. Its lifetime keeps the
/// decoded payload storage alive; no raw text is copied merely to validate it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterChain<'a> {
    caveats: Vec<Caveat<'a>>,
}

impl<'a> FilterChain<'a> {
    pub const fn empty() -> Self {
        Self { caveats: Vec::new() }
    }

    /// Bound bytes BEFORE shell decoding as well. This second bound checks the
    /// actual retained payload, not a caller-supplied numeric size assertion.
    /// Ordinals are exactly `00` through `07`, with no sorting or deduplication.
    pub fn from_records(records: &[WireCaveatRecord<'a>]) -> Result<Self, InvalidCaveat> {
        if records.len() > MAX_CHAIN {
            return Err(InvalidCaveat::ChainTooLong);
        }
        let mut caveats = Vec::with_capacity(records.len());
        for (index, record) in records.iter().enumerate() {
            if record.serialized_payload.is_empty() {
                return Err(InvalidCaveat::Malformed);
            }
            if record.serialized_payload.len() > MAX_CAVEAT_BYTES {
                return Err(InvalidCaveat::Oversized);
            }
            let ordinal_digit = u8::try_from(index).map_err(|_| InvalidCaveat::InvalidOrdinal)?;
            let ordinal = [
                b'0',
                b'0'.checked_add(ordinal_digit).ok_or(InvalidCaveat::InvalidOrdinal)?,
            ];
            if record.key.as_bytes() != ordinal.as_slice() {
                return Err(InvalidCaveat::InvalidOrdinal);
            }
            caveats.push(Caveat::from_wire(record.caveat)?);
        }
        Ok(Self { caveats })
    }

    /// Pure local composition, never cryptographic holder delegation. A shell
    /// must still verify every preserved parent payload in the signed proof.
    pub fn attenuate(mut self, caveat: Caveat<'a>) -> Result<Self, InvalidCaveat> {
        if self.caveats.len() >= MAX_CHAIN {
            return Err(InvalidCaveat::ChainTooLong);
        }
        self.caveats.push(caveat);
        Ok(self)
    }

    fn apply_prefix<'v>(&self, original: &'v str, end: usize) -> Option<Cow<'v, str>> {
        let mut current = Cow::Borrowed(original);
        for caveat in self.caveats[..end].iter().rev() {
            current = caveat.apply(current)?;
        }
        Some(current)
    }

    /// The callback must be a pure predicate over receiver-owned site facts,
    /// valid for original declarations and rewritten requests alike. A changing
    /// policy observation cannot establish the prefix/no-widening invariant.
    /// This result is an observation only: no redemption or other effect occurs.
    // r[impl mantle.authority_attenuation.composition_order]
    pub fn admit_with_policy<'v>(&self, original: &'v str, site_policy: impl Fn(&str) -> bool) -> Option<Cow<'v, str>> {
        validate_value(original).ok()?;
        if !site_policy(original) {
            return None;
        }
        let mut result = Cow::Borrowed(original);
        for end in 1..=self.caveats.len() {
            let output = self.apply_prefix(original, end)?;
            if !site_policy(output.as_ref()) {
                return None;
            }
            result = output;
        }
        Some(result)
    }
}

fn binding_name_is_valid(name: &str) -> bool {
    !name.is_empty() && name.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"_-".contains(&byte))
}

/// One canonical segment; no URI decoding, globbing or filesystem normalization
/// is ever performed here. This is the store-name-safe ASCII vocabulary, not a
/// claim that a supplied store digest or project identifier is authenticated.
fn literal_segment_is_valid(segment: &str) -> bool {
    !segment.is_empty()
        && segment != "."
        && segment != ".."
        && segment.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"+._?=-".contains(&byte))
}

fn validate_value(value: &str) -> Result<usize, InvalidCaveat> {
    if value.len() > MAX_VALUE_BYTES {
        return Err(InvalidCaveat::Oversized);
    }
    let body = value.strip_prefix('/').ok_or(InvalidCaveat::Malformed)?;
    let mut count = 0_usize;
    for segment in body.split('/') {
        // A leading colon is reserved in patterns but remains valid canonical
        // value text only when followed by a valid binding name.
        let is_valid_segment = if let Some(name) = segment.strip_prefix(':') {
            binding_name_is_valid(name)
        } else {
            literal_segment_is_valid(segment)
        };
        if !is_valid_segment {
            return Err(InvalidCaveat::Malformed);
        }
        count += 1;
        if count > MAX_SEGMENTS {
            return Err(InvalidCaveat::Oversized);
        }
    }
    debug_assert!(count >= 1);
    debug_assert!(count <= MAX_SEGMENTS);
    Ok(count)
}

/// Restrictions retained from each authenticated parent grant. These are not
/// verified authority: the shell authenticates their exact signed payloads and
/// supplies receiver-local time before invoking this pure decision.
#[derive(Debug, Clone, Copy)]
pub struct RemoteGrantScope<'a> {
    pub jobs: &'a [&'a str],
    pub output_classes: &'a [&'a str],
    pub deadline_unix_s: u64,
}

pub const MAX_REMOTE_GRANT_JOBS: usize = 64;
pub const MAX_REMOTE_OUTPUT_CLASSES: usize = 32;

/// Check `/job/<request-id>/output/<validated-class>` against every retained
/// parent restriction, the original value and each rewritten prefix output.
/// A bool is a non-consuming decision, never a holder signature or ticket spend.
/// This boolean gate admits the ORIGINAL effect, so a rewrite to a different
/// allowed job or output class is not a grant to execute that original effect.
// r[impl mantle.authority_attenuation.attenuated_remote_grant]
pub fn admit_remote_job(
    chain: &FilterChain<'_>,
    original: &str,
    parents: &[RemoteGrantScope<'_>],
    receiver_now_unix_s: u64,
) -> bool {
    if parents.is_empty() || parents.len() > MAX_CHAIN {
        return false;
    }
    let has_unusable_parent = parents.iter().any(|parent| {
        parent.jobs.is_empty()
            || parent.jobs.len() > MAX_REMOTE_GRANT_JOBS
            || parent.output_classes.is_empty()
            || parent.output_classes.len() > MAX_REMOTE_OUTPUT_CLASSES
            || receiver_now_unix_s >= parent.deadline_unix_s
            || parent.jobs.iter().chain(parent.output_classes).any(|value| !literal_segment_is_valid(value))
    });
    if has_unusable_parent {
        return false;
    }
    debug_assert!(parents.len() <= MAX_CHAIN);
    debug_assert!(parents.iter().all(|parent| parent.jobs.len() <= MAX_REMOTE_GRANT_JOBS));
    debug_assert!(parents.iter().all(|parent| parent.output_classes.len() <= MAX_REMOTE_OUTPUT_CLASSES));
    chain
        .admit_with_policy(original, |value| {
            let Some(value) = value.strip_prefix("/job/") else {
                return false;
            };
            let Some((job, class)) = value.split_once("/output/") else {
                return false;
            };
            literal_segment_is_valid(job)
                && literal_segment_is_valid(class)
                && parents.iter().all(|parent| parent.jobs.contains(&job) && parent.output_classes.contains(&class))
        })
        .is_some_and(|output| output == original)
}

/// Exact logical-path gate inputs: the receiver's admitted store prefix, the
/// expected exact path and the requested path under the caller's filter chain.
#[derive(Debug, Clone, Copy)]
pub struct ExactStorePathRequest<'a> {
    pub prefix: &'a str,
    pub expected: &'a str,
    pub requested: &'a str,
}

/// One exact child path below an admitted prefix.
#[derive(Debug, Clone, Copy)]
struct ExactChild<'a> {
    prefix: &'a str,
    path: &'a str,
}

/// One project goal declaration lowered against its owning project.
#[derive(Debug, Clone, Copy)]
struct ProjectGoalInput<'a> {
    owner: &'a str,
    value: &'a str,
}

/// Project goal gate inputs: the owning project and the original declaration.
#[derive(Debug, Clone, Copy)]
pub struct ProjectGoalRequest<'a> {
    pub owner: &'a str,
    pub declaration: &'a str,
}

/// Exact logical-path gate: no prefix lookalikes, nested content paths or broad
/// listing. `expected` must come from the receiver's admitted StorePath codec;
/// this helper does not authenticate or reimplement the store digest format.
// r[impl mantle.authority_attenuation.attenuated_store_view]
pub fn admit_exact_store_path(chain: &FilterChain<'_>, request: ExactStorePathRequest<'_>) -> bool {
    let child = ExactChild {
        prefix: request.prefix,
        path: request.expected,
    };
    if validate_value(request.prefix).is_err() || !exact_child(child) {
        return false;
    }
    chain.admit_with_policy(request.requested, |path| path == request.expected).is_some()
}

fn exact_child(child: ExactChild<'_>) -> bool {
    child
        .path
        .strip_prefix(child.prefix)
        .and_then(|rest| rest.strip_prefix('/'))
        .is_some_and(literal_segment_is_valid)
}

fn project_goal<'v>(input: ProjectGoalInput<'v>) -> Option<(&'v str, &'v str)> {
    let mut parts = input.value.strip_prefix('/')?.split('/');
    if parts.next()? != "project" || parts.next()? != input.owner {
        return None;
    }
    let kind = parts.next()?;
    if kind != "declared" && kind != "goal" {
        return None;
    }
    let goal = parts.next()?;
    if !literal_segment_is_valid(goal) || parts.next().is_some() {
        return None;
    }
    Some((kind, goal))
}

/// Lower only this owner's original declaration into an admitted, unnested
/// goal. The receiver supplies its own goal allow-set, checked on the original
/// and every ancestor-prefix output, never a project name from a rewrite.
// r[impl mantle.authority_attenuation.attenuated_remote_grant]
pub fn admit_project_goal<'v>(
    chain: &FilterChain<'_>,
    request: ProjectGoalRequest<'v>,
    goal_is_admitted: impl Fn(&str) -> bool,
) -> Option<Cow<'v, str>> {
    let declared = ProjectGoalInput {
        owner: request.owner,
        value: request.declaration,
    };
    if !literal_segment_is_valid(request.owner) || project_goal(declared)?.0 != "declared" {
        return None;
    }
    let output = chain.admit_with_policy(request.declaration, |value| {
        let candidate = ProjectGoalInput {
            owner: request.owner,
            value,
        };
        project_goal(candidate).is_some_and(|(_, goal)| goal_is_admitted(goal))
    })?;
    let lowered = ProjectGoalInput {
        owner: request.owner,
        value: output.as_ref(),
    };
    if project_goal(lowered)?.0 != "goal" {
        return None;
    }
    debug_assert!(literal_segment_is_valid(request.owner));
    debug_assert!(project_goal(declared).is_some());
    Some(output)
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    fn rewrite<'a>(pattern: &'a str, template: &'a str) -> Caveat<'a> {
        Caveat::from_wire(WireCaveat {
            kind: "rewrite",
            pattern: Some(pattern),
            template: Some(template),
            alternatives: &[],
        })
        .unwrap()
    }

    fn reject(pattern: &str) -> Caveat<'_> {
        Caveat::from_wire(WireCaveat {
            kind: "reject",
            pattern: Some(pattern),
            template: None,
            alternatives: &[],
        })
        .unwrap()
    }

    fn deny_all() -> Caveat<'static> {
        Caveat::from_wire(WireCaveat {
            kind: "future-variant",
            pattern: None,
            template: None,
            alternatives: &[],
        })
        .unwrap()
    }

    #[test]
    fn remote_site_conjoins_parent_jobs_classes_and_receiver_deadlines_before_rewrite() {
        let parent = RemoteGrantScope {
            jobs: &["job-a", "job-b"],
            output_classes: &["out", "dev"],
            deadline_unix_s: 100,
        };
        let child = RemoteGrantScope {
            jobs: &["job-a"],
            output_classes: &["out"],
            deadline_unix_s: 50,
        };
        let chain = FilterChain::empty();
        assert!(admit_remote_job(&chain, "/job/job-a/output/out", &[parent, child], 49));
        for input in [
            "/job/job-b/output/out",
            "/job/job-a/output/dev",
            "/job/job-a/output/out/extra",
        ] {
            assert!(!admit_remote_job(&chain, input, &[parent, child], 49));
        }
        assert!(!admit_remote_job(&chain, "/job/job-a/output/out", &[parent, child], 50));
        let laundering = chain.attenuate(rewrite("/job/job-b/output/out", "/job/job-a/output/out")).unwrap();
        assert!(!admit_remote_job(&laundering, "/job/job-b/output/out", &[parent, child], 49));
        let widened_output =
            FilterChain::empty().attenuate(rewrite("/job/job-a/output/out", "/job/job-b/output/out")).unwrap();
        assert!(!admit_remote_job(&widened_output, "/job/job-a/output/out", &[parent, child], 49));
        assert!(!admit_remote_job(&FilterChain::empty(), "/job/job-a/output/out", &[], 49));
        // Both targets are in the parent scope, but this boolean API cannot
        // forward a rewritten effect. Never execute the unfiltered original.
        let retargeted =
            FilterChain::empty().attenuate(rewrite("/job/job-a/output/out", "/job/job-b/output/dev")).unwrap();
        assert!(!admit_remote_job(&retargeted, "/job/job-a/output/out", &[parent], 49));
        let identity = FilterChain::empty()
            .attenuate(rewrite("/job/:job/output/:class", "/job/:job/output/:class"))
            .unwrap();
        assert!(admit_remote_job(&identity, "/job/job-a/output/out", &[parent, child], 49));
    }

    #[test]
    fn bound_rewrite_matches_complete_segments_and_preserves_binding() {
        let chain = FilterChain::empty()
            .attenuate(rewrite("/project/alpha/declared/:goal", "/project/alpha/goal/:goal"))
            .unwrap();
        assert_eq!(
            chain.admit_with_policy("/project/alpha/declared/build", |_| true).as_deref(),
            Some("/project/alpha/goal/build")
        );
        for input in [
            "/project/beta/declared/build",
            "/project/alpha/declared/build/extra",
            "/project/alpha/declared",
            "/project/alpha/declaredx/build",
        ] {
            assert_eq!(chain.admit_with_policy(input, |_| true), None);
        }
    }

    #[test]
    fn reject_is_identity_only_for_nonmatches_and_unknown_always_denies() {
        let chain = FilterChain::empty().attenuate(reject("/private/:name")).unwrap();
        assert_eq!(chain.admit_with_policy("/private/secret", |_| true), None);
        assert!(matches!(chain.admit_with_policy("/public/item", |_| true), Some(Cow::Borrowed("/public/item"))));
        let chain = chain.attenuate(deny_all()).unwrap();
        assert_eq!(chain.admit_with_policy("/public/item", |_| true), None);
        assert_eq!(FilterChain::empty().admit_with_policy("/public/item", |_| false), None);
    }

    #[test]
    fn alternatives_choose_first_match_and_deny_if_none_matches() {
        let rules = [
            WireRewrite {
                pattern: "/input/:name",
                template: "/first/:name",
            },
            WireRewrite {
                pattern: "/input/:name",
                template: "/second/:name",
            },
            WireRewrite {
                pattern: "/other/:name",
                template: "/third/:name",
            },
        ];
        let caveat = Caveat::from_wire(WireCaveat {
            kind: "alternatives",
            pattern: None,
            template: None,
            alternatives: &rules,
        })
        .unwrap();
        let chain = FilterChain::empty().attenuate(caveat).unwrap();
        assert_eq!(chain.admit_with_policy("/input/a", |_| true).as_deref(), Some("/first/a"));
        assert_eq!(chain.admit_with_policy("/other/a", |_| true).as_deref(), Some("/third/a"));
        assert_eq!(chain.admit_with_policy("/missing/a", |_| true), None);
    }

    #[test]
    fn known_variants_reject_missing_extra_and_invalid_fields() {
        let cases = [
            WireCaveat {
                kind: "rewrite",
                pattern: Some("/a"),
                template: None,
                alternatives: &[],
            },
            WireCaveat {
                kind: "reject",
                pattern: Some("/a"),
                template: Some("/b"),
                alternatives: &[],
            },
            WireCaveat {
                kind: "alternatives",
                pattern: None,
                template: None,
                alternatives: &[],
            },
            WireCaveat {
                kind: "alternatives",
                pattern: Some("/a"),
                template: None,
                alternatives: &[],
            },
        ];
        for case in cases {
            assert_eq!(Caveat::from_wire(case), Err(InvalidCaveat::Malformed));
        }
        assert_eq!(Rewrite::parse("/a/:same/:same", "/b/:same"), Err(InvalidCaveat::DuplicateBinding));
        assert_eq!(Rewrite::parse("/a/:name", "/b/:missing"), Err(InvalidCaveat::UnboundTemplate));
        assert_eq!(Rewrite::parse("/a/:name", "/b/:name/:name"), Err(InvalidCaveat::DuplicateBinding));
        for pattern in [
            "",
            "/",
            "relative",
            "/a//b",
            "/a/",
            "/a/./b",
            "/a/../b",
            "/a/:",
            "/a/:bad?",
            "/a/with space",
            "/a/é",
            "/a/%2e%2e",
            "/a/\\secret",
        ] {
            assert!(Pattern::parse(pattern).is_err(), "{pattern}");
        }
    }

    #[test]
    fn newest_runs_first_but_ancestor_rejection_cannot_be_rewritten_away() {
        let parent = FilterChain::empty().attenuate(rewrite("/allowed", "/allowed")).unwrap();
        assert_eq!(parent.admit_with_policy("/denied", |_| true), None);
        let child = parent.attenuate(rewrite("/denied", "/allowed")).unwrap();
        // Ordinary function composition WOULD accept this attack.
        assert_eq!(child.apply_prefix("/denied", 2).as_deref(), Some("/allowed"));
        assert_eq!(child.admit_with_policy("/denied", |_| true), None);
        let ordered = FilterChain::empty()
            .attenuate(rewrite("/:name", "/parent/:name"))
            .unwrap()
            .attenuate(rewrite("/start", "/child"))
            .unwrap();
        assert_eq!(ordered.admit_with_policy("/start", |_| true).as_deref(), Some("/parent/child"));
    }

    #[test]
    fn original_and_every_ancestor_output_must_pass_site_policy() {
        let chain = FilterChain::empty()
            .attenuate(rewrite("/:name", "/parent/:name"))
            .unwrap()
            .attenuate(rewrite("/start", "/child"))
            .unwrap();
        assert_eq!(chain.admit_with_policy("/start", |value| value != "/start"), None);
        assert_eq!(chain.admit_with_policy("/start", |value| value != "/parent/start"), None);
        assert_eq!(chain.admit_with_policy("/start", |value| value != "/parent/child"), None);
        assert_eq!(chain.admit_with_policy("/start", |_| true).as_deref(), Some("/parent/child"));
    }

    #[test]
    fn finite_input_space_never_widens_under_appended_rewrites() {
        let patterns = ["/a", "/b", "/:name"];
        for parent_pattern in patterns {
            for child_pattern in patterns {
                for child_template in ["/a", "/b"] {
                    let parent = FilterChain::empty().attenuate(rewrite(parent_pattern, "/a")).unwrap();
                    let child = parent.clone().attenuate(rewrite(child_pattern, child_template)).unwrap();
                    for input in ["/a", "/b", "/c", "/a/extra"] {
                        if child.admit_with_policy(input, |_| true).is_some() {
                            assert!(parent.admit_with_policy(input, |_| true).is_some());
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn pattern_value_segment_chain_and_alternative_bounds_fail_closed() {
        let max_pattern = alloc::format!("/{}", "x".repeat(MAX_PATTERN_BYTES - 1));
        assert!(Pattern::parse(&max_pattern).is_ok());
        let oversized_pattern = alloc::format!("{max_pattern}x");
        assert_eq!(Pattern::parse(&oversized_pattern), Err(InvalidCaveat::Oversized));
        let max_value = alloc::format!("/{}", "x".repeat(MAX_VALUE_BYTES - 1));
        assert_eq!(FilterChain::empty().admit_with_policy(&max_value, |_| true).as_deref(), Some(max_value.as_str()));
        assert_eq!(FilterChain::empty().admit_with_policy(&alloc::format!("{max_value}x"), |_| true), None);
        let max_segments = "/x".repeat(MAX_SEGMENTS);
        assert!(Pattern::parse(&max_segments).is_ok());
        assert_eq!(Pattern::parse(&alloc::format!("{max_segments}/x")), Err(InvalidCaveat::Oversized));
        let mut chain = FilterChain::empty();
        for _ in 0..MAX_CHAIN {
            chain = chain.attenuate(reject("/blocked")).unwrap();
        }
        assert_eq!(chain.attenuate(reject("/blocked")), Err(InvalidCaveat::ChainTooLong));
        let rules = [WireRewrite {
            pattern: "/a",
            template: "/a",
        }; MAX_ALTERNATIVES + 1];
        assert_eq!(
            Caveat::from_wire(WireCaveat {
                kind: "alternatives",
                pattern: None,
                template: None,
                alternatives: &rules,
            }),
            Err(InvalidCaveat::Oversized)
        );
    }

    #[test]
    fn expanded_output_over_bound_denies_without_alternative_fallback() {
        let long_literal = "x".repeat(MAX_PATTERN_BYTES - 5);
        let template = alloc::format!("/{long_literal}/:v");
        let rules = [
            WireRewrite {
                pattern: "/:v",
                template: &template,
            },
            WireRewrite {
                pattern: "/:v",
                template: "/safe",
            },
        ];
        let chain = FilterChain::empty()
            .attenuate(
                Caveat::from_wire(WireCaveat {
                    kind: "alternatives",
                    pattern: None,
                    template: None,
                    alternatives: &rules,
                })
                .unwrap(),
            )
            .unwrap();
        let input = alloc::format!("/{}", "v".repeat(300));
        assert_eq!(chain.admit_with_policy(&input, |_| true), None);
        assert!(chain.admit_with_policy("/small", |_| true).unwrap().ends_with("/small"));
    }

    #[test]
    fn record_ordinals_and_serialized_size_are_not_normalized_away() {
        let wire = WireCaveat {
            kind: "reject",
            pattern: Some("/blocked"),
            template: None,
            alternatives: &[],
        };
        let record = WireCaveatRecord {
            key: "00",
            serialized_payload: b"payload",
            caveat: wire,
        };
        assert_eq!(
            FilterChain::from_records(&[record]).unwrap().admit_with_policy("/ok", |_| true).as_deref(),
            Some("/ok")
        );
        for key in ["0", "000", "01", "-1", "00 ", "a0"] {
            assert_eq!(
                FilterChain::from_records(&[WireCaveatRecord { key, ..record }]),
                Err(InvalidCaveat::InvalidOrdinal)
            );
        }
        assert_eq!(FilterChain::from_records(&[record, record]), Err(InvalidCaveat::InvalidOrdinal));
        assert_eq!(
            FilterChain::from_records(&[WireCaveatRecord {
                serialized_payload: b"",
                ..record
            }]),
            Err(InvalidCaveat::Malformed)
        );
        let payload = vec![b'x'; MAX_CAVEAT_BYTES];
        assert!(
            FilterChain::from_records(&[WireCaveatRecord {
                serialized_payload: &payload,
                ..record
            }])
            .is_ok()
        );
        let payload = vec![b'x'; MAX_CAVEAT_BYTES + 1];
        assert_eq!(
            FilterChain::from_records(&[WireCaveatRecord {
                serialized_payload: &payload,
                ..record
            }]),
            Err(InvalidCaveat::Oversized)
        );
        assert_eq!(FilterChain::from_records(&[record; MAX_CHAIN + 1]), Err(InvalidCaveat::ChainTooLong));
    }

    #[test]
    fn exact_store_gate_denies_prefix_lookalikes_traversal_and_rewrites() {
        let expected = "/mantle/store/00000000000000000000000000000000-approved";
        let chain = FilterChain::empty();
        assert!(admit_exact_store_path(&chain, ExactStorePathRequest {
            prefix: "/mantle/store",
            expected,
            requested: expected,
        }));
        for request in [
            "/mantle/storeX/00000000000000000000000000000000-approved",
            "/mantle/store/00000000000000000000000000000000-private",
            "/mantle/store/00000000000000000000000000000000-approved/file",
            "/mantle/store/../private",
            "/mantle/store",
        ] {
            assert!(!admit_exact_store_path(&chain, ExactStorePathRequest {
                prefix: "/mantle/store",
                expected,
                requested: request,
            }));
        }
        let rewrite = chain.attenuate(rewrite("/mantle/store/:path", "/other/store/:path")).unwrap();
        assert!(!admit_exact_store_path(&rewrite, ExactStorePathRequest {
            prefix: "/mantle/store",
            expected,
            requested: expected,
        }));
        assert!(!admit_exact_store_path(&FilterChain::empty(), ExactStorePathRequest {
            prefix: "/mantle/store/",
            expected,
            requested: expected,
        }));
        assert!(!admit_exact_store_path(&FilterChain::empty(), ExactStorePathRequest {
            prefix: "/mantle/store",
            expected: "/mantle/store/a/b",
            requested: "/mantle/store/a/b",
        }));
    }

    #[test]
    fn project_gate_binds_original_owner_every_prefix_and_admitted_goal() {
        let chain = FilterChain::empty()
            .attenuate(rewrite("/project/alpha/declared/:goal", "/project/alpha/goal/:goal"))
            .unwrap();
        assert_eq!(
            admit_project_goal(
                &chain,
                ProjectGoalRequest {
                    owner: "alpha",
                    declaration: "/project/alpha/declared/build",
                },
                |goal| goal == "build"
            )
            .as_deref(),
            Some("/project/alpha/goal/build")
        );
        assert_eq!(
            admit_project_goal(
                &chain,
                ProjectGoalRequest {
                    owner: "alpha",
                    declaration: "/project/beta/declared/build",
                },
                |_| true
            ),
            None
        );
        assert_eq!(
            admit_project_goal(
                &chain,
                ProjectGoalRequest {
                    owner: "alpha",
                    declaration: "/project/alpha/declared/private",
                },
                |goal| goal == "build"
            ),
            None
        );
        for target in [
            "/project/beta/goal/build",
            "/project/alpha/goal/nested/build",
            "/project/alpha/goal/private",
        ] {
            let escape = FilterChain::empty().attenuate(rewrite("/project/alpha/declared/:goal", target)).unwrap();
            assert_eq!(
                admit_project_goal(
                    &escape,
                    ProjectGoalRequest {
                        owner: "alpha",
                        declaration: "/project/alpha/declared/build",
                    },
                    |goal| goal == "build"
                ),
                None
            );
        }
        assert_eq!(
            admit_project_goal(
                &FilterChain::empty(),
                ProjectGoalRequest {
                    owner: "alpha",
                    declaration: "/project/alpha/declared/build",
                },
                |_| true
            ),
            None
        );
        let unknown = chain.attenuate(deny_all()).unwrap();
        assert_eq!(
            admit_project_goal(
                &unknown,
                ProjectGoalRequest {
                    owner: "alpha",
                    declaration: "/project/alpha/declared/build",
                },
                |_| true
            ),
            None
        );
    }
}
