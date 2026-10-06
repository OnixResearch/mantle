#![no_std]
extern crate alloc;

pub mod sites;

use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;

pub const MAX_CHAIN: usize = 8;
pub const MAX_CAVEAT_BYTES: usize = 8192;
pub const MAX_ALTERNATIVES: usize = 8;
pub const MAX_PATTERN_BYTES: usize = 256;
pub const MAX_VALUE_BYTES: usize = 512;
pub const MAX_SEGMENTS: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidCaveat {
    Malformed,
    Oversized,
    UnboundTemplate,
    ChainTooLong,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Segment {
    Literal(String),
    Binding(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Pattern(Vec<Segment>);

#[derive(Debug, Clone, PartialEq, Eq)]
struct Rewrite {
    pattern: Pattern,
    template: Pattern,
    identity: bool,
}

// r[impl mantle.authority_attenuation.caveat_filter_semantics]
/// Filters operate on canonical, slash-separated assertion or message values.
/// Empty segments, dot segments, and non-ASCII bytes are never admitted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Caveat {
    Rewrite { pattern: String, template: String },
    Reject { pattern: String },
    Alternatives { rewrites: Vec<(String, String)> },
    Unknown,
}
/// Shell decoding must reject duplicate fields and oversized serialized bytes
/// before constructing this borrowed, bounded semantic input.
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
/// Existing UCAN caveat keys carry fixed-width ordinal identities. The
/// verified shell must pass only records from the same verified capability.
#[derive(Debug, Clone, Copy)]
pub struct WireCaveatRecord<'a> {
    pub key: &'a str,
    pub serialized_payload: &'a [u8],
    pub caveat: WireCaveat<'a>,
}

impl Caveat {
    /// Unknown variants become a deny-all caveat, never a permissive fallback.
    pub fn from_wire(wire: WireCaveat<'_>) -> Result<Self, InvalidCaveat> {
        if wire.kind.len() > 32
            || wire.pattern.is_some_and(|value| value.len() > MAX_PATTERN_BYTES)
            || wire.template.is_some_and(|value| value.len() > MAX_PATTERN_BYTES)
            || wire.alternatives.len() > MAX_ALTERNATIVES
            || wire
                .alternatives
                .iter()
                .any(|item| item.pattern.len() > MAX_PATTERN_BYTES || item.template.len() > MAX_PATTERN_BYTES)
        {
            return Err(InvalidCaveat::Oversized);
        }
        match wire.kind {
            "rewrite" if wire.alternatives.is_empty() => Ok(Self::Rewrite {
                pattern: String::from(wire.pattern.ok_or(InvalidCaveat::Malformed)?),
                template: String::from(wire.template.ok_or(InvalidCaveat::Malformed)?),
            }),
            "reject" if wire.template.is_none() && wire.alternatives.is_empty() => Ok(Self::Reject {
                pattern: String::from(wire.pattern.ok_or(InvalidCaveat::Malformed)?),
            }),
            "alternatives" if wire.pattern.is_none() && wire.template.is_none() => {
                if wire.alternatives.len() > MAX_ALTERNATIVES {
                    return Err(InvalidCaveat::Oversized);
                }
                Ok(Self::Alternatives {
                    rewrites: wire
                        .alternatives
                        .iter()
                        .map(|item| (String::from(item.pattern), String::from(item.template)))
                        .collect(),
                })
            }
            "rewrite" | "reject" | "alternatives" => Err(InvalidCaveat::Malformed),
            _ => Ok(Self::Unknown),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CompiledCaveat {
    Rewrite(Rewrite),
    Reject(Pattern),
    Alternatives(Vec<Rewrite>),
    DenyAll,
}

// r[impl mantle.authority_attenuation.composition_order]
/// Admission is an observation, not an authority token or an effect grant.
/// A receiver must authenticate the enclosing grant and recheck its original
/// site-specific policy on both the original and rewritten value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterChain(Vec<CompiledCaveat>);

fn text_is_valid(text: &str) -> bool {
    !text.is_empty()
        && text.len() <= MAX_VALUE_BYTES
        && text.is_ascii()
        && text.starts_with('/')
        && !text.ends_with('/')
        && text
            .split('/')
            .skip(1)
            .all(|s| !s.is_empty() && s != "." && s != ".." && !s.bytes().any(|b| b < 0x21 || b == 0x7f))
        && text.split('/').count() <= MAX_SEGMENTS + 1
}

fn segment_name_is_valid(text: &str) -> bool {
    !text.is_empty()
        && text != "."
        && text != ".."
        && text.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-' || b == b'.')
}

fn value_is_valid(value: &str) -> bool {
    text_is_valid(value) && value.split('/').skip(1).all(segment_name_is_valid)
}

fn parse_pattern(source: &str, bindings: bool) -> Result<Pattern, InvalidCaveat> {
    if source.len() > MAX_PATTERN_BYTES {
        return Err(InvalidCaveat::Oversized);
    }
    if !text_is_valid(source) {
        return Err(InvalidCaveat::Malformed);
    }
    let mut parts = Vec::new();
    for component in source.split('/').skip(1) {
        if let Some(name) = component.strip_prefix(':') {
            if !bindings || !segment_name_is_valid(name) {
                return Err(InvalidCaveat::Malformed);
            }
            if parts.iter().any(|part| matches!(part, Segment::Binding(old) if old == name)) {
                return Err(InvalidCaveat::Malformed);
            }
            parts.push(Segment::Binding(String::from(name)));
        } else if segment_name_is_valid(component) {
            parts.push(Segment::Literal(String::from(component)));
        } else {
            return Err(InvalidCaveat::Malformed);
        }
    }
    Ok(Pattern(parts))
}

fn compile_rewrite(pattern: &str, template: &str) -> Result<Rewrite, InvalidCaveat> {
    let pattern = parse_pattern(pattern, true)?;
    let template = parse_pattern(template, true)?;
    for part in &template.0 {
        if let Segment::Binding(name) = part
            && !pattern.0.iter().any(|p| matches!(p, Segment::Binding(source) if source == name))
        {
            return Err(InvalidCaveat::UnboundTemplate);
        }
    }
    let identity = pattern == template;
    Ok(Rewrite {
        pattern,
        template,
        identity,
    })
}

fn compile(caveat: Caveat) -> Result<CompiledCaveat, InvalidCaveat> {
    match caveat {
        Caveat::Rewrite { pattern, template } => Ok(CompiledCaveat::Rewrite(compile_rewrite(&pattern, &template)?)),
        Caveat::Reject { pattern } => Ok(CompiledCaveat::Reject(parse_pattern(&pattern, true)?)),
        Caveat::Alternatives { rewrites } => {
            if rewrites.is_empty() {
                return Err(InvalidCaveat::Malformed);
            }
            if rewrites.len() > MAX_ALTERNATIVES {
                return Err(InvalidCaveat::Oversized);
            }
            let mut compiled = Vec::with_capacity(rewrites.len());
            for (pattern, template) in rewrites {
                compiled.push(compile_rewrite(&pattern, &template)?);
            }
            Ok(CompiledCaveat::Alternatives(compiled))
        }
        Caveat::Unknown => Ok(CompiledCaveat::DenyAll),
    }
}

fn bindings_for<'a>(pattern: &Pattern, value: &'a str) -> Option<[Option<&'a str>; MAX_SEGMENTS]> {
    if !value_is_valid(value) {
        return None;
    }
    let mut values = [None; MAX_SEGMENTS];
    let mut segments = value.split('/').skip(1);
    for (index, part) in pattern.0.iter().enumerate() {
        let candidate = segments.next()?;
        match part {
            Segment::Literal(literal) if literal != candidate => return None,
            Segment::Literal(_) => {}
            Segment::Binding(_) => values[index] = Some(candidate),
        }
    }
    if segments.next().is_some() {
        return None;
    }
    Some(values)
}

fn rewrite(rule: &Rewrite, value: &str) -> Option<String> {
    let values = bindings_for(&rule.pattern, value)?;
    let mut output = String::with_capacity(MAX_VALUE_BYTES.min(value.len() + MAX_PATTERN_BYTES));
    for part in &rule.template.0 {
        let segment = match part {
            Segment::Literal(literal) => literal.as_str(),
            Segment::Binding(name) => {
                let index = rule.pattern.0.iter().position(|p| matches!(p, Segment::Binding(old) if old == name))?;
                values[index]?
            }
        };
        if output.len().checked_add(segment.len() + 1)? > MAX_VALUE_BYTES {
            return None;
        }
        output.push('/');
        output.push_str(segment);
    }
    text_is_valid(&output).then_some(output)
}

impl CompiledCaveat {
    fn apply<'a>(&self, value: Cow<'a, str>) -> Option<Cow<'a, str>> {
        match self {
            Self::Rewrite(rule) => {
                if rule.identity {
                    if bindings_for(&rule.pattern, value.as_ref()).is_some() {
                        Some(value)
                    } else {
                        None
                    }
                } else {
                    rewrite(rule, value.as_ref()).map(Cow::Owned)
                }
            }
            Self::Reject(pattern) => {
                if bindings_for(pattern, value.as_ref()).is_some() {
                    None
                } else {
                    Some(value)
                }
            }
            Self::Alternatives(rewrites) => {
                for rule in rewrites {
                    if rule.identity {
                        if bindings_for(&rule.pattern, value.as_ref()).is_some() {
                            return Some(value);
                        }
                    } else if let Some(rewritten) = rewrite(rule, value.as_ref()) {
                        return Some(Cow::Owned(rewritten));
                    }
                }
                None
            }
            Self::DenyAll => None,
        }
    }
}

impl FilterChain {
    /// An empty chain does not replace mandatory receiver-local policy.
    pub fn empty() -> Self {
        Self(Vec::new())
    }

    /// UCAN checks parent caveat preservation, but does not establish order.
    /// A missing, reordered or duplicate ordinal is not a valid chain.
    pub fn from_records(records: &[WireCaveatRecord<'_>]) -> Result<Self, InvalidCaveat> {
        if records.len() > MAX_CHAIN {
            return Err(InvalidCaveat::ChainTooLong);
        }
        let mut chain = Self::empty();
        for (index, record) in records.iter().enumerate() {
            if record.serialized_payload.is_empty() {
                return Err(InvalidCaveat::Malformed);
            }
            if record.serialized_payload.len() > MAX_CAVEAT_BYTES {
                return Err(InvalidCaveat::Oversized);
            }
            let key = record.key.as_bytes();
            if key.len() != 2
                || key[0] != b'0'
                || key[1] != b'0' + u8::try_from(index).map_err(|_| InvalidCaveat::ChainTooLong)?
            {
                return Err(InvalidCaveat::Malformed);
            }
            chain = chain.attenuate(Caveat::from_wire(record.caveat)?)?;
        }
        Ok(chain)
    }

    /// Appending a caveat means it runs first. The parent chain's result on
    /// the *original* input remains a mandatory independent admission gate.
    pub fn attenuate(mut self, caveat: Caveat) -> Result<Self, InvalidCaveat> {
        if self.0.len() >= MAX_CHAIN {
            return Err(InvalidCaveat::ChainTooLong);
        }
        self.0.push(compile(caveat)?);
        Ok(self)
    }

    fn apply_prefix<'a>(&self, value: &'a str, end: usize) -> Option<Cow<'a, str>> {
        let mut current = Cow::Borrowed(value);
        for caveat in self.0[..end].iter().rev() {
            current = caveat.apply(current)?;
        }
        Some(current)
    }

    /// For each prefix, the same original value must still be accepted.
    /// This guards against rewrites making an input acceptable to its parent
    /// when the parent's own decision on that input was a rejection.
    /// An empty chain accepts canonical input; callers still check site scope.
    pub fn admit<'a>(&self, value: &'a str) -> Option<Cow<'a, str>> {
        if !value_is_valid(value) || self.0.len() > MAX_CHAIN {
            return None;
        }
        for prefix_len in 1..self.0.len() {
            self.apply_prefix(value, prefix_len)?;
        }
        self.apply_prefix(value, self.0.len())
    }

    /// Site policy must also accept every ancestor's output on the original
    /// request; a later rewrite cannot repair a rejected parent output.
    /// The supplied site predicate must be deterministic for this request;
    /// it cannot be the caveat callback alone because an empty chain has none.
    pub fn admit_with_policy<'a>(&self, value: &'a str, policy: impl Fn(&str) -> bool) -> Option<Cow<'a, str>> {
        if !value_is_valid(value) || self.0.len() > MAX_CHAIN || !policy(value) {
            return None;
        }
        let mut output = Cow::Borrowed(value);
        for prefix_len in 1..=self.0.len() {
            output = self.apply_prefix(value, prefix_len)?;
            if !policy(output.as_ref()) {
                return None;
            }
        }
        Some(output)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn rewrite_rule(pattern: &str, template: &str) -> Caveat {
        Caveat::Rewrite {
            pattern: String::from(pattern),
            template: String::from(template),
        }
    }

    #[test]
    fn rewrite_bindings_and_silent_nonmatch() {
        let grant = FilterChain::empty()
            .attenuate(rewrite_rule("/job/:id/class/public", "/project/alpha/job/:id"))
            .unwrap();
        assert_eq!(grant.admit("/job/compile/class/public").as_deref(), Some("/project/alpha/job/compile"));
        assert_eq!(grant.admit("/job/compile/class/private"), None);
    }

    #[test]
    fn reject_alternatives_and_unknown() {
        let deny = FilterChain::empty()
            .attenuate(Caveat::Reject {
                pattern: String::from("/store/private/:path"),
            })
            .unwrap();
        assert_eq!(deny.admit("/store/private/secret"), None);
        assert_eq!(deny.admit("/store/public/visible").as_deref(), Some("/store/public/visible"));
        let alternatives = FilterChain::empty()
            .attenuate(Caveat::Alternatives {
                rewrites: alloc::vec![
                    (String::from("/job/:id"), String::from("/project/alpha/job/:id")),
                    (String::from("/store/:path"), String::from("/project/alpha/store/:path")),
                ],
            })
            .unwrap();
        assert_eq!(alternatives.admit("/store/a").as_deref(), Some("/project/alpha/store/a"));
        assert_eq!(alternatives.admit("/admin/a"), None);
        assert_eq!(FilterChain::empty().attenuate(Caveat::Unknown).unwrap().admit("/job/a"), None);
    }

    #[test]
    fn right_to_left_never_widens_parent() {
        let parent = FilterChain::empty()
            .attenuate(rewrite_rule("/project/alpha/goal/:goal", "/project/alpha/goal/:goal"))
            .unwrap();
        let child = parent
            .clone()
            .attenuate(rewrite_rule("/project/beta/goal/:goal", "/project/alpha/goal/:goal"))
            .unwrap();
        assert!(parent.admit("/project/alpha/goal/build").is_some());
        assert!(child.admit("/project/beta/goal/build").is_none());
        assert!(child.admit("/project/alpha/goal/build").is_none());
        let parent = FilterChain::empty()
            .attenuate(Caveat::Reject {
                pattern: String::from("/deny/:id"),
            })
            .unwrap();
        let child = parent.attenuate(rewrite_rule("/deny/:id", "/permit/:id")).unwrap();
        assert_eq!(child.admit("/deny/id"), None);
    }

    #[test]
    fn policy_rechecks_original_and_transformed() {
        let chain = FilterChain::empty()
            .attenuate(rewrite_rule("/project/alpha/goal/:goal", "/project/beta/goal/:goal"))
            .unwrap();
        assert_eq!(
            chain.admit_with_policy("/project/alpha/goal/build", |name| name.starts_with("/project/alpha/")),
            None
        );
        assert_eq!(
            chain.admit_with_policy("/project/alpha/goal/build", |name| name.starts_with("/project/beta/")),
            None
        );
        let parent = FilterChain::empty()
            .attenuate(Caveat::Alternatives {
                rewrites: alloc::vec![
                    (String::from("/a/:id"), String::from("/b/:id")),
                    (String::from("/c/:id"), String::from("/a/:id")),
                ],
            })
            .unwrap();
        let child = parent.attenuate(rewrite_rule("/a/:id", "/c/:id")).unwrap();
        assert_eq!(child.admit("/a/name").as_deref(), Some("/a/name"));
        assert_eq!(child.admit_with_policy("/a/name", |name| !name.starts_with("/b/")), None);
    }

    #[test]
    fn malformed_and_oversize_fail_closed() {
        for bad in ["/job//a", "/job/../a", "/job/:bad name", "/job/:same/:same", "/job/é"] {
            assert!(FilterChain::empty().attenuate(rewrite_rule(bad, "/ok")).is_err(), "{bad}");
        }
        assert_eq!(
            FilterChain::empty()
                .attenuate(rewrite_rule("/job/:id", "/ok/:id"))
                .unwrap()
                .admit("/job/a%2fprivate"),
            None,
        );
        assert_eq!(FilterChain::empty().admit("/job/a%2fprivate"), None);
        assert_eq!(
            FilterChain::empty().attenuate(rewrite_rule("/job/:id", "/job/:other")),
            Err(InvalidCaveat::UnboundTemplate)
        );
        assert_eq!(
            FilterChain::empty().attenuate(rewrite_rule(&alloc::format!("/{}", "a".repeat(MAX_PATTERN_BYTES)), "/a")),
            Err(InvalidCaveat::Oversized)
        );
        let mut chain = FilterChain::empty();
        for _ in 0..MAX_CHAIN {
            chain = chain
                .attenuate(Caveat::Reject {
                    pattern: String::from("/private"),
                })
                .unwrap();
        }
        assert_eq!(chain.attenuate(Caveat::Unknown), Err(InvalidCaveat::ChainTooLong));
    }
    #[test]
    fn wire_variant_shape_and_unknown_are_fail_closed() {
        let known = WireCaveat {
            kind: "rewrite",
            pattern: Some("/job/:id"),
            template: Some("/job/:id"),
            alternatives: &[],
        };
        assert_eq!(
            FilterChain::empty()
                .attenuate(Caveat::from_wire(known).unwrap())
                .unwrap()
                .admit("/job/a")
                .as_deref(),
            Some("/job/a")
        );
        assert_eq!(
            Caveat::from_wire(WireCaveat {
                template: None,
                ..known
            }),
            Err(InvalidCaveat::Malformed)
        );
        assert_eq!(
            Caveat::from_wire(WireCaveat {
                kind: "reject",
                template: Some("/bad"),
                ..known
            }),
            Err(InvalidCaveat::Malformed)
        );
        let unknown = Caveat::from_wire(WireCaveat {
            kind: "future-operation",
            ..known
        })
        .unwrap();
        assert_eq!(FilterChain::empty().attenuate(unknown).unwrap().admit("/job/a"), None);
        let long = alloc::format!("/{}", "a".repeat(MAX_PATTERN_BYTES));
        assert_eq!(
            Caveat::from_wire(WireCaveat {
                pattern: Some(&long),
                ..known
            }),
            Err(InvalidCaveat::Oversized)
        );
    }

    #[test]
    fn finite_inputs_keep_each_child_admission_inside_parent() {
        let parents = [
            Caveat::Reject {
                pattern: String::from("/private/:id"),
            },
            rewrite_rule("/project/alpha/:id", "/visible/:id"),
            Caveat::Alternatives {
                rewrites: alloc::vec![
                    (String::from("/visible/:id"), String::from("/visible/:id")),
                    (String::from("/private/:id"), String::from("/visible/:id")),
                ],
            },
        ];
        let children = [
            rewrite_rule("/private/:id", "/project/alpha/:id"),
            rewrite_rule("/project/alpha/:id", "/private/:id"),
            Caveat::Reject {
                pattern: String::from("/visible/:id"),
            },
            Caveat::Unknown,
        ];
        for parent_rule in &parents {
            let parent = FilterChain::empty().attenuate(parent_rule.clone()).unwrap();
            for child_rule in &children {
                let child = parent.clone().attenuate(child_rule.clone()).unwrap();
                for input in ["/private/a", "/visible/a", "/project/alpha/a", "/other/a"] {
                    if child.admit_with_policy(input, |_| true).is_some() {
                        assert!(parent.admit_with_policy(input, |_| true).is_some());
                    }
                }
            }
        }
    }
    #[test]
    fn record_ordinals_are_strict_and_unknown_denies() {
        let allow = WireCaveat {
            kind: "rewrite",
            pattern: Some("/a/:id"),
            template: Some("/a/:id"),
            alternatives: &[],
        };
        let deny = WireCaveat {
            kind: "future-restriction",
            pattern: None,
            template: None,
            alternatives: &[],
        };
        let valid = [
            WireCaveatRecord {
                key: "00",
                serialized_payload: b"rewrite",
                caveat: allow,
            },
            WireCaveatRecord {
                key: "01",
                serialized_payload: b"unknown",
                caveat: deny,
            },
        ];
        assert_eq!(FilterChain::from_records(&valid).unwrap().admit("/a/a"), None);
        assert_eq!(FilterChain::from_records(&valid[..1]).unwrap().admit("/a/a").as_deref(), Some("/a/a"));
        for invalid in [
            [
                WireCaveatRecord {
                    key: "01",
                    serialized_payload: b"rewrite",
                    caveat: allow,
                },
                valid[1],
            ],
            [valid[0], WireCaveatRecord {
                key: "00",
                serialized_payload: b"unknown",
                caveat: deny,
            }],
            [valid[1], valid[0]],
        ] {
            assert_eq!(FilterChain::from_records(&invalid), Err(InvalidCaveat::Malformed));
        }
        assert_eq!(
            FilterChain::from_records(&[WireCaveatRecord {
                serialized_payload: b"",
                ..valid[0]
            }]),
            Err(InvalidCaveat::Malformed)
        );
        let oversize = [b'a'; MAX_CAVEAT_BYTES + 1];
        assert_eq!(
            FilterChain::from_records(&[WireCaveatRecord {
                serialized_payload: &oversize,
                ..valid[0]
            }]),
            Err(InvalidCaveat::Oversized)
        );
        assert_eq!(FilterChain::from_records(&[valid[0]; MAX_CHAIN + 1]), Err(InvalidCaveat::ChainTooLong));
    }
}
