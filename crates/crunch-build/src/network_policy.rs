use nix_compat::derivation::Derivation;
use nix_compat::nixhash::CAHash;
use nix_compat::nixhash::CAHashMode;

const FETCH_BUILDER_SELECTOR: &str = "builtin:fetchurl";
const OUTPUT_NAME_OUT: &str = "out";
const ENV_NAME: &str = "name";
const ENV_URL: &str = "url";
pub const ENV_NETWORK_CAPABILITY: &str = "__mantle_network_capability";
pub const ENV_NETWORK_POLICY_BASIS: &str = "__mantle_network_policy_basis";
pub const ENV_NETWORK_AUDIT_CLASS: &str = "__mantle_network_audit_class";
pub const NETWORK_CAPABILITY_BUILD_TIME: &str = "build-time-network";
pub const NETWORK_MODE_OFFLINE: &str = "offline";
pub const NETWORK_MODE_FIXED_OUTPUT_FETCHER: &str = "fixed-output-fetcher";
pub const NETWORK_MODE_COMPATIBILITY_CAPABILITY: &str = "compatibility-capability";
pub const NETWORK_RESULT_DENIED: &str = "denied";
pub const NETWORK_RESULT_ALLOWED: &str = "allowed";
pub const NETWORK_RESULT_BLOCKED: &str = "blocked";
const UNKNOWN_ACTION_NAME: &str = "<unnamed>";
pub const NETWORK_RETRY_POLICY_BOUNDED_TRANSIENT_FETCH: &str = "bounded-transient-fetch-retry";
const HASH_MODE_FLAT: &str = "flat";
const HASH_MODE_RECURSIVE: &str = "recursive";
const HASH_MODE_TEXT: &str = "text";
const DENY_BY_DEFAULT_DIAGNOSTIC: &str = "ordinary derivation network access is denied by default";
const MISSING_POLICY_BASIS_DIAGNOSTIC: &str = "declared network capability is missing policy basis";
const MISSING_AUDIT_CLASS_DIAGNOSTIC: &str = "declared network capability is missing audit class";
const INVALID_CAPABILITY_DIAGNOSTIC: &str = "declared network capability is unsupported";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatibilityNetworkPolicy {
    DenyAll,
    AllowDeclared,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixedOutputNetworkDeclaration {
    pub url: Option<String>,
    pub hash: Option<String>,
    pub mode: Option<String>,
    pub retry_policy: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildNetworkPolicyReport {
    pub action_name: String,
    pub mode: String,
    pub result: String,
    pub capability: Option<String>,
    pub policy_basis: Option<String>,
    pub audit_class: Option<String>,
    pub fixed_output: Option<FixedOutputNetworkDeclaration>,
    pub diagnostic: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildNetworkPolicyPlan {
    pub allow_network: bool,
    pub report: BuildNetworkPolicyReport,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkPolicyDenied {
    pub action_name: String,
    pub capability: Option<String>,
    pub diagnostic: String,
    pub report: Box<BuildNetworkPolicyReport>,
}

pub fn plan_network_policy(
    derivation: &Derivation,
    compatibility_policy: CompatibilityNetworkPolicy,
) -> Result<BuildNetworkPolicyPlan, NetworkPolicyDenied> {
    debug_assert!(!derivation.builder.is_empty(), "builder must not be empty");
    debug_assert!(!derivation.outputs.is_empty(), "derivation must declare outputs");

    let action_name = action_name(derivation);
    let capability = env_string(derivation, ENV_NETWORK_CAPABILITY)?;
    if is_fixed_output_fetcher(derivation) {
        return Ok(fixed_output_fetcher_plan(derivation, action_name, capability));
    }
    match capability {
        None => Ok(offline_plan(action_name)),
        Some(capability) => compatibility_plan(derivation, compatibility_policy, action_name, capability),
    }
}

pub fn is_fixed_output_fetcher(derivation: &Derivation) -> bool {
    let Some(output) = derivation.outputs.get(OUTPUT_NAME_OUT) else {
        return false;
    };
    derivation.builder == FETCH_BUILDER_SELECTOR && derivation.outputs.len() == 1 && output.is_fixed()
}

fn fixed_output_fetcher_plan(
    derivation: &Derivation,
    action_name: String,
    capability: Option<String>,
) -> BuildNetworkPolicyPlan {
    BuildNetworkPolicyPlan {
        allow_network: true,
        report: BuildNetworkPolicyReport {
            action_name,
            mode: NETWORK_MODE_FIXED_OUTPUT_FETCHER.to_string(),
            result: NETWORK_RESULT_ALLOWED.to_string(),
            capability,
            policy_basis: Some("declared fixed-output fetcher".to_string()),
            audit_class: Some("fixed-output-source-acquisition".to_string()),
            fixed_output: Some(fixed_output_declaration(derivation)),
            diagnostic: None,
        },
    }
}

fn offline_plan(action_name: String) -> BuildNetworkPolicyPlan {
    BuildNetworkPolicyPlan {
        allow_network: false,
        report: BuildNetworkPolicyReport {
            action_name,
            mode: NETWORK_MODE_OFFLINE.to_string(),
            result: NETWORK_RESULT_DENIED.to_string(),
            capability: None,
            policy_basis: Some(DENY_BY_DEFAULT_DIAGNOSTIC.to_string()),
            audit_class: None,
            fixed_output: None,
            diagnostic: None,
        },
    }
}

fn compatibility_plan(
    derivation: &Derivation,
    compatibility_policy: CompatibilityNetworkPolicy,
    action_name: String,
    capability: String,
) -> Result<BuildNetworkPolicyPlan, NetworkPolicyDenied> {
    assert!(!action_name.is_empty(), "network policy action name must not be empty");
    assert!(!capability.is_empty(), "network capability must not be empty");
    if capability != NETWORK_CAPABILITY_BUILD_TIME {
        return Err(denied(action_name, Some(capability), INVALID_CAPABILITY_DIAGNOSTIC));
    }
    let policy_basis = required_env_string(derivation, RequiredNetworkEnvironment {
        key: ENV_NETWORK_POLICY_BASIS,
        action_name: &action_name,
        capability: &capability,
        missing_diagnostic: MISSING_POLICY_BASIS_DIAGNOSTIC,
    })?;
    let audit_class = required_env_string(derivation, RequiredNetworkEnvironment {
        key: ENV_NETWORK_AUDIT_CLASS,
        action_name: &action_name,
        capability: &capability,
        missing_diagnostic: MISSING_AUDIT_CLASS_DIAGNOSTIC,
    })?;
    if compatibility_policy == CompatibilityNetworkPolicy::DenyAll {
        return Err(denied_with_scope(
            action_name,
            Some(capability),
            Some(policy_basis),
            Some(audit_class),
            DENY_BY_DEFAULT_DIAGNOSTIC,
        ));
    }
    Ok(BuildNetworkPolicyPlan {
        allow_network: true,
        report: BuildNetworkPolicyReport {
            action_name,
            mode: NETWORK_MODE_COMPATIBILITY_CAPABILITY.to_string(),
            result: NETWORK_RESULT_ALLOWED.to_string(),
            capability: Some(capability),
            policy_basis: Some(policy_basis),
            audit_class: Some(audit_class),
            fixed_output: None,
            diagnostic: None,
        },
    })
}

fn fixed_output_declaration(derivation: &Derivation) -> FixedOutputNetworkDeclaration {
    let ca_hash = derivation.outputs.get(OUTPUT_NAME_OUT).and_then(|output| output.ca_hash.as_ref());
    FixedOutputNetworkDeclaration {
        url: env_string(derivation, ENV_URL).ok().flatten(),
        hash: ca_hash.map(CAHash::to_nix_nixbase32_string),
        mode: ca_hash.map(ca_hash_mode_string),
        retry_policy: NETWORK_RETRY_POLICY_BOUNDED_TRANSIENT_FETCH.to_string(),
    }
}

fn ca_hash_mode_string(hash: &CAHash) -> String {
    match hash.mode() {
        CAHashMode::Flat => HASH_MODE_FLAT,
        CAHashMode::Nar => HASH_MODE_RECURSIVE,
        CAHashMode::Text => HASH_MODE_TEXT,
    }
    .to_string()
}

struct RequiredNetworkEnvironment<'a> {
    key: &'a str,
    action_name: &'a str,
    capability: &'a str,
    missing_diagnostic: &'a str,
}

fn required_env_string(
    derivation: &Derivation,
    required: RequiredNetworkEnvironment<'_>,
) -> Result<String, NetworkPolicyDenied> {
    match env_string(derivation, required.key)? {
        Some(value) => Ok(value),
        None => Err(denied(
            required.action_name.to_string(),
            Some(required.capability.to_string()),
            required.missing_diagnostic,
        )),
    }
}

fn env_string(derivation: &Derivation, key: &str) -> Result<Option<String>, NetworkPolicyDenied> {
    let Some(value) = derivation.environment.get(key) else {
        return Ok(None);
    };
    let bytes: &[u8] = value.as_ref();
    let Ok(value) = std::str::from_utf8(bytes) else {
        return Err(denied(action_name(derivation), Some(key.to_string()), "network policy field is not UTF-8"));
    };
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    Ok(Some(value.to_string()))
}

fn action_name(derivation: &Derivation) -> String {
    env_string(derivation, ENV_NAME).ok().flatten().unwrap_or_else(|| UNKNOWN_ACTION_NAME.to_string())
}

fn denied(action_name: String, capability: Option<String>, diagnostic: &str) -> NetworkPolicyDenied {
    denied_with_scope(action_name, capability, None, None, diagnostic)
}

fn denied_with_scope(
    action_name: String,
    capability: Option<String>,
    policy_basis: Option<String>,
    audit_class: Option<String>,
    diagnostic: &str,
) -> NetworkPolicyDenied {
    NetworkPolicyDenied {
        action_name: action_name.clone(),
        capability: capability.clone(),
        diagnostic: diagnostic.to_string(),
        report: Box::new(BuildNetworkPolicyReport {
            action_name,
            mode: NETWORK_MODE_COMPATIBILITY_CAPABILITY.to_string(),
            result: NETWORK_RESULT_BLOCKED.to_string(),
            capability,
            policy_basis,
            audit_class,
            fixed_output: None,
            diagnostic: Some(diagnostic.to_string()),
        }),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::collections::BTreeSet;

    use bstr::BString;
    use nix_compat::derivation::Derivation;
    use nix_compat::derivation::Output;
    use nix_compat::nixhash::CAHash;
    use nix_compat::nixhash::NixHash;

    use super::*;

    const SHA256_BYTE: u8 = 0xBB;

    fn ordinary_derivation() -> Derivation {
        let mut outputs = BTreeMap::new();
        outputs.insert("out".to_string(), Output {
            path: None,
            ca_hash: None,
        });
        let mut environment = BTreeMap::new();
        environment.insert("name".to_string(), "ordinary".into());
        environment.insert("builder".to_string(), "/bin/sh".into());
        environment.insert("out".to_string(), "".into());
        Derivation {
            arguments: vec!["-c".to_string(), "echo ok > $out".to_string()],
            builder: "/bin/sh".to_string(),
            environment,
            input_derivations: BTreeMap::new(),
            input_sources: BTreeSet::new(),
            outputs,
            system: "x86_64-linux".to_string(),
        }
    }

    fn fetcher_derivation() -> Derivation {
        let mut derivation = ordinary_derivation();
        derivation.builder = FETCH_BUILDER_SELECTOR.to_string();
        derivation.system = "builtin".to_string();
        derivation.arguments.clear();
        derivation.environment.insert("name".to_string(), "fetch".into());
        derivation.environment.insert("url".to_string(), "https://example.com/src.tar.gz".into());
        derivation.outputs.insert("out".to_string(), Output {
            path: None,
            ca_hash: Some(CAHash::Flat(NixHash::Sha256([SHA256_BYTE; 32]))),
        });
        derivation
    }

    fn declare_network_capability(derivation: &mut Derivation) {
        derivation
            .environment
            .insert(ENV_NETWORK_CAPABILITY.to_string(), NETWORK_CAPABILITY_BUILD_TIME.into());
        derivation
            .environment
            .insert(ENV_NETWORK_POLICY_BASIS.to_string(), "compat-policy:legacy-upstream".into());
        derivation.environment.insert(ENV_NETWORK_AUDIT_CLASS.to_string(), "legacy-network-build".into());
    }

    #[test]
    fn ordinary_derivation_is_offline_by_default() {
        let derivation = ordinary_derivation();
        let plan = plan_network_policy(&derivation, CompatibilityNetworkPolicy::DenyAll).unwrap();

        assert!(!plan.allow_network);
        assert_eq!(plan.report.mode, NETWORK_MODE_OFFLINE);
        assert_eq!(plan.report.result, NETWORK_RESULT_DENIED);
        assert_eq!(plan.report.policy_basis.as_deref(), Some(DENY_BY_DEFAULT_DIAGNOSTIC));
        assert!(plan.report.fixed_output.is_none());
    }

    #[test]
    fn fixed_output_fetcher_reports_declared_network_boundary() {
        let derivation = fetcher_derivation();
        let plan = plan_network_policy(&derivation, CompatibilityNetworkPolicy::DenyAll).unwrap();
        let fixed = plan.report.fixed_output.as_ref().expect("fixed output declaration");

        assert!(plan.allow_network);
        assert_eq!(plan.report.mode, NETWORK_MODE_FIXED_OUTPUT_FETCHER);
        assert_eq!(plan.report.result, NETWORK_RESULT_ALLOWED);
        assert_eq!(fixed.url.as_deref(), Some("https://example.com/src.tar.gz"));
        assert_eq!(fixed.mode.as_deref(), Some(HASH_MODE_FLAT));
        assert!(fixed.hash.as_deref().is_some_and(|hash| hash.starts_with("fixed:sha256:")));
        assert_eq!(fixed.retry_policy, NETWORK_RETRY_POLICY_BOUNDED_TRANSIENT_FETCH);
    }

    #[test]
    fn custom_fixed_output_builder_stays_offline() {
        let mut derivation = ordinary_derivation();
        derivation.outputs.insert("out".to_string(), Output {
            path: None,
            ca_hash: Some(CAHash::Flat(NixHash::Sha256([SHA256_BYTE; 32]))),
        });

        let plan = plan_network_policy(&derivation, CompatibilityNetworkPolicy::DenyAll).unwrap();

        assert!(!plan.allow_network);
        assert_eq!(plan.report.mode, NETWORK_MODE_OFFLINE);
        assert!(plan.report.fixed_output.is_none());
    }

    #[test]
    fn declared_network_capability_is_denied_without_compatibility_policy() {
        let mut derivation = ordinary_derivation();
        declare_network_capability(&mut derivation);

        let error = plan_network_policy(&derivation, CompatibilityNetworkPolicy::DenyAll).unwrap_err();

        assert_eq!(error.action_name, "ordinary");
        assert_eq!(error.capability.as_deref(), Some(NETWORK_CAPABILITY_BUILD_TIME));
        assert_eq!(error.diagnostic, DENY_BY_DEFAULT_DIAGNOSTIC);
        assert_eq!(error.report.action_name, "ordinary");
        assert_eq!(error.report.result, NETWORK_RESULT_BLOCKED);
        assert_eq!(error.report.capability.as_deref(), Some(NETWORK_CAPABILITY_BUILD_TIME));
        assert_eq!(error.report.policy_basis.as_deref(), Some("compat-policy:legacy-upstream"));
        assert_eq!(error.report.audit_class.as_deref(), Some("legacy-network-build"));
    }

    #[test]
    fn allowed_compatibility_capability_reports_scope() {
        let mut derivation = ordinary_derivation();
        declare_network_capability(&mut derivation);

        let plan = plan_network_policy(&derivation, CompatibilityNetworkPolicy::AllowDeclared).unwrap();

        assert!(plan.allow_network);
        assert_eq!(plan.report.mode, NETWORK_MODE_COMPATIBILITY_CAPABILITY);
        assert_eq!(plan.report.capability.as_deref(), Some(NETWORK_CAPABILITY_BUILD_TIME));
        assert_eq!(plan.report.policy_basis.as_deref(), Some("compat-policy:legacy-upstream"));
        assert_eq!(plan.report.audit_class.as_deref(), Some("legacy-network-build"));
    }

    #[test]
    fn compatibility_capability_requires_policy_basis() {
        let mut derivation = ordinary_derivation();
        declare_network_capability(&mut derivation);
        derivation.environment.remove(ENV_NETWORK_POLICY_BASIS);

        let error = plan_network_policy(&derivation, CompatibilityNetworkPolicy::AllowDeclared).unwrap_err();

        assert_eq!(error.action_name, "ordinary");
        assert_eq!(error.capability.as_deref(), Some(NETWORK_CAPABILITY_BUILD_TIME));
        assert_eq!(error.diagnostic, MISSING_POLICY_BASIS_DIAGNOSTIC);
    }

    #[test]
    fn invalid_utf8_network_policy_field_fails_closed() {
        let mut derivation = ordinary_derivation();
        derivation.environment.insert(ENV_NETWORK_CAPABILITY.to_string(), BString::from(vec![0xFF]));

        let error = plan_network_policy(&derivation, CompatibilityNetworkPolicy::DenyAll).unwrap_err();

        assert_eq!(error.action_name, "ordinary");
        assert_eq!(error.capability.as_deref(), Some(ENV_NETWORK_CAPABILITY));
        assert_eq!(error.diagnostic, "network policy field is not UTF-8");
    }
}
