// r[impl mantlepkgs_domains.domain_partition]
// r[impl mantlepkgs_domains.deterministic_composition]
// r[impl mantlepkgs_domains.explicit_variants]
// r[impl mantlepkgs_domains.separate_validation_roots]
// r[impl mantlepkgs_domains.reference_corpus]
// r[impl mantlepkgs_domains.functional_core]
// r[impl mantlepkgs_domains.claim_boundary]
// r[verify mantlepkgs_domains.domain_partition]
// r[verify mantlepkgs_domains.deterministic_composition]
// r[verify mantlepkgs_domains.explicit_variants]
// r[verify mantlepkgs_domains.separate_validation_roots]
// r[verify mantlepkgs_domains.reference_corpus]
// r[verify mantlepkgs_domains.functional_core]
// r[verify mantlepkgs_domains.claim_boundary]

use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::BLAKE3_HEX_LENGTH;
use crate::CoreFailure;
use crate::Diagnostic;
use crate::MantlepkgsCatalog;
use crate::PackageDisposition;
use crate::PlannedPackage;
use crate::validate_catalog_identity;

pub const DOMAIN_MANIFEST_SCHEMA: &str = "mantlepkgs-domain-manifest-v1";
pub const DOMAIN_SHARD_SCHEMA: &str = "mantlepkgs-domain-shard-v1";
pub const DOMAIN_CATALOG_SCHEMA: &str = "mantlepkgs-domain-catalog-v1";
pub const VALIDATION_OBSERVATION_SCHEMA: &str = "mantlepkgs-validation-observation-v1";
pub const VALIDATION_RECEIPT_SCHEMA: &str = "mantlepkgs-validation-receipt-v1";
pub const CORPUS_EVIDENCE_SCHEMA: &str = "mantlepkgs-corpus-evidence-v1";
pub const DOMAIN_CLASS_CORE: &str = "core";
pub const DOMAIN_CLASS_ECOSYSTEM: &str = "ecosystem";
pub const CORPUS_LICENSE_MIT: &str = "MIT";
pub const CORPUS_ROLE_GRAPH: &str = "graph";
pub const CORPUS_ROLE_SOURCES: &str = "sources";
pub const CORPUS_ROLE_CATALOG: &str = "catalog";
pub const CORPUS_ROLE_BLOCKERS: &str = "blockers";
pub const CORPUS_ROLE_PRODUCER: &str = "producer";
pub const EXPECTED_VALIDATION_PASS: &str = "pass";
pub const OBSERVED_VALIDATION_PASS: &str = "pass";
pub const OBSERVED_VALIDATION_FAIL: &str = "fail";
pub const OBSERVED_VALIDATION_TIMEOUT: &str = "timeout";
pub const OBSERVED_VALIDATION_MALFORMED: &str = "malformed";

const DOMAIN_SHARD_IDENTITY_DOMAIN: &[u8] = b"mantle.mantlepkgs.domain-shard.v1";
const DOMAIN_PACKAGE_IDENTITY_DOMAIN: &[u8] = b"mantle.mantlepkgs.domain-package.v1";
const DOMAIN_VARIANT_IDENTITY_DOMAIN: &[u8] = b"mantle.mantlepkgs.domain-variant.v1";
const DOMAIN_VALIDATION_ROOT_IDENTITY_DOMAIN: &[u8] = b"mantle.mantlepkgs.validation-root.v1";
const DOMAIN_CATALOG_IDENTITY_DOMAIN: &[u8] = b"mantle.mantlepkgs.domain-catalog.v1";
const VALIDATION_RECEIPT_IDENTITY_DOMAIN: &[u8] = b"mantle.mantlepkgs.validation-receipt.v1";
const CORPUS_EVIDENCE_IDENTITY_DOMAIN: &[u8] = b"mantle.mantlepkgs.corpus-evidence.v1";
const DOMAIN_SEPARATOR: u8 = 0;
const MIN_ITEMS: u32 = 1;
const MAX_TEXT_BYTES: usize = 4_096;
const MAX_DOMAIN_SHARDS: u32 = 256;
const MAX_DOMAIN_PACKAGES: u32 = 65_536;
const MAX_DOMAIN_ALIASES: u32 = 65_536;
const MAX_DOMAIN_VARIANTS: u32 = 16_384;
const MAX_DOMAIN_VALIDATION_ROOTS: u32 = 16_384;
const MAX_DOMAIN_ARTIFACTS: u32 = 65_536;
const REQUIRED_CORPUS_ROLE_COUNT: usize = 5;
const REQUIRED_NON_CLAIM_COUNT: usize = 5;
const MILLISECONDS_PER_SECOND: u64 = 1_000;
const VALIDATION_DIAGNOSTIC_HEADROOM: usize = 32;
const GIT_REVISION_HEX_LENGTH: usize = 40;

type PublicSelectorKey = (String, String);
type IdentityBinding = (String, String);
type SealedSelectorMap = BTreeMap<PublicSelectorKey, IdentityBinding>;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DomainCatalogManifest {
    pub schema: String,
    pub name: String,
    pub shards: Vec<DomainShard>,
    pub variants: Vec<PackageVariant>,
    pub validation_roots: Vec<ValidationRoot>,
    pub limits: DomainCatalogLimits,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DomainCatalogLimits {
    pub max_shards: u32,
    pub max_packages: u32,
    pub max_aliases: u32,
    pub max_variants: u32,
    pub max_validation_roots: u32,
    pub max_artifacts: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DomainSourceLock {
    pub repository: String,
    pub reference: String,
    pub revision: String,
    pub source_digest_blake3: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DomainShard {
    pub schema: String,
    pub shard_identity_blake3: String,
    pub name: String,
    pub class: String,
    pub owner_label: String,
    pub source: DomainSourceLock,
    pub source_catalog_identity_blake3: String,
    pub packages: Vec<DomainPackage>,
    pub limits: DomainShardLimits,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DomainShardLimits {
    pub max_packages: u32,
    pub max_aliases: u32,
    pub max_artifacts: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DomainArtifact {
    pub role: String,
    pub digest_blake3: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DomainPackage {
    pub package_identity_blake3: String,
    pub name: String,
    pub public_selector: String,
    pub system: String,
    pub root_identity_blake3: String,
    pub policy_digest_blake3: String,
    pub buildable: bool,
    pub aliases: Vec<String>,
    pub artifacts: Vec<DomainArtifact>,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PackageVariant {
    pub variant_identity_blake3: String,
    pub public_selector: String,
    pub system: String,
    pub base_selector: String,
    pub base_package_identity_blake3: String,
    pub base_root_identity_blake3: String,
    pub variant_name: String,
    pub changed_policy_digest_blake3: String,
    pub root_identity_blake3: String,
    pub provenance_digest_blake3: String,
    pub aliases: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationRoot {
    pub validation_root_identity_blake3: String,
    pub name: String,
    pub system: String,
    pub package_selector: String,
    pub package_identity_blake3: String,
    pub validation_selector: String,
    pub validation_package_identity_blake3: String,
    pub sources: Vec<DomainArtifact>,
    pub tools: Vec<DomainArtifact>,
    pub dependencies: Vec<DomainArtifact>,
    pub policy_digest_blake3: String,
    pub expected_outcome: String,
    pub limits: ValidationLimits,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationLimits {
    pub max_sources: u32,
    pub max_tools: u32,
    pub max_dependencies: u32,
    pub max_output_bytes: u64,
    pub timeout_seconds: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PublicPackageRecord {
    pub public_selector: String,
    pub system: String,
    pub package_identity_blake3: String,
    pub root_identity_blake3: String,
    pub shard_identity_blake3: String,
    pub variant_base_identity_blake3: Option<String>,
    pub buildable: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DomainCatalog {
    pub schema: String,
    pub catalog_identity_blake3: String,
    pub name: String,
    pub shards: Vec<DomainShard>,
    pub packages: Vec<PublicPackageRecord>,
    pub variants: Vec<PackageVariant>,
    pub validation_roots: Vec<ValidationRoot>,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationRootPlan {
    pub validation_root_identity_blake3: String,
    pub domain_catalog_identity_blake3: String,
    pub package_selector: String,
    pub package_identity_blake3: String,
    pub package_root_identity_blake3: String,
    pub validation_selector: String,
    pub validation_package_identity_blake3: String,
    pub validation_root_package_identity_blake3: String,
    pub sources: Vec<DomainArtifact>,
    pub tools: Vec<DomainArtifact>,
    pub dependencies: Vec<DomainArtifact>,
    pub policy_digest_blake3: String,
    pub expected_outcome: String,
    pub max_output_bytes: u64,
    pub timeout_seconds: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationObservation {
    pub schema: String,
    pub validation_root_identity_blake3: String,
    pub outcome: String,
    pub realization_receipt_blake3: String,
    pub observed_output_bytes: u64,
    pub elapsed_milliseconds: u64,
    pub diagnostic_digest_blake3: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationReceipt {
    pub schema: String,
    pub receipt_identity_blake3: String,
    pub validation_root_identity_blake3: String,
    pub domain_catalog_identity_blake3: String,
    pub package_identity_blake3: String,
    pub package_root_identity_blake3: String,
    pub validation_package_identity_blake3: String,
    pub outcome: String,
    pub accepted: bool,
    pub realization_receipt_blake3: String,
    pub observed_output_bytes: u64,
    pub elapsed_milliseconds: u64,
    pub diagnostic_digest_blake3: Option<String>,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusArtifact {
    pub role: String,
    pub path: String,
    pub digest_blake3: String,
    pub bytes: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalCorpusEvidence {
    pub schema: String,
    pub evidence_identity_blake3: String,
    pub repository: String,
    pub source_reference: String,
    pub revision: String,
    pub source_digest_blake3: String,
    pub observed_license: String,
    pub license_path: String,
    pub license_digest_blake3: String,
    pub selected_packages: Vec<String>,
    pub producer_policy_digest_blake3: String,
    pub artifacts: Vec<CorpusArtifact>,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusArtifactObservation {
    pub path: String,
    pub digest_blake3: String,
    pub bytes: u64,
    pub catalog_packages: Vec<String>,
}

#[derive(Serialize)]
struct ShardIdentityPreimage<'a> {
    schema: &'a str,
    name: &'a str,
    class: &'a str,
    owner_label: &'a str,
    source: &'a DomainSourceLock,
    source_catalog_identity_blake3: &'a str,
    packages: &'a [DomainPackage],
    limits: &'a DomainShardLimits,
}

#[derive(Serialize)]
struct PackageIdentityPreimage<'a> {
    name: &'a str,
    public_selector: &'a str,
    system: &'a str,
    root_identity_blake3: &'a str,
    policy_digest_blake3: &'a str,
    buildable: bool,
    aliases: &'a [String],
    artifacts: &'a [DomainArtifact],
}

#[derive(Serialize)]
struct VariantIdentityPreimage<'a> {
    public_selector: &'a str,
    system: &'a str,
    base_selector: &'a str,
    base_package_identity_blake3: &'a str,
    base_root_identity_blake3: &'a str,
    variant_name: &'a str,
    changed_policy_digest_blake3: &'a str,
    root_identity_blake3: &'a str,
    provenance_digest_blake3: &'a str,
    aliases: &'a [String],
}

#[derive(Serialize)]
struct ValidationRootIdentityPreimage<'a> {
    name: &'a str,
    system: &'a str,
    package_selector: &'a str,
    package_identity_blake3: &'a str,
    validation_selector: &'a str,
    validation_package_identity_blake3: &'a str,
    sources: &'a [DomainArtifact],
    tools: &'a [DomainArtifact],
    dependencies: &'a [DomainArtifact],
    policy_digest_blake3: &'a str,
    expected_outcome: &'a str,
    limits: &'a ValidationLimits,
}

pub fn domain_package_identity_blake3(package: &DomainPackage) -> Result<String, CoreFailure> {
    let mut aliases = package.aliases.clone();
    let mut artifacts = package.artifacts.clone();
    aliases.sort();
    aliases.dedup();
    normalize_artifacts(&mut artifacts);
    debug_assert!(aliases.windows(2).all(|pair| pair[0] < pair[1]));
    debug_assert!(artifacts.windows(2).all(|pair| pair[0] < pair[1]));
    digest_serializable(
        DOMAIN_PACKAGE_IDENTITY_DOMAIN,
        &PackageIdentityPreimage {
            name: &package.name,
            public_selector: &package.public_selector,
            system: &package.system,
            root_identity_blake3: &package.root_identity_blake3,
            policy_digest_blake3: &package.policy_digest_blake3,
            buildable: package.buildable,
            aliases: &aliases,
            artifacts: &artifacts,
        },
        "package-identity-serialization-failed",
    )
}

pub fn domain_shard_identity_blake3(shard: &DomainShard) -> Result<String, CoreFailure> {
    let mut packages = shard.packages.clone();
    for package in &mut packages {
        package.aliases.sort();
        package.aliases.dedup();
        normalize_artifacts(&mut package.artifacts);
    }
    packages.sort();
    debug_assert_eq!(packages.len(), shard.packages.len());
    debug_assert!(packages.windows(2).all(|pair| pair[0] <= pair[1]));
    digest_serializable(
        DOMAIN_SHARD_IDENTITY_DOMAIN,
        &ShardIdentityPreimage {
            schema: &shard.schema,
            name: &shard.name,
            class: &shard.class,
            owner_label: &shard.owner_label,
            source: &shard.source,
            source_catalog_identity_blake3: &shard.source_catalog_identity_blake3,
            packages: &packages,
            limits: &shard.limits,
        },
        "shard-identity-serialization-failed",
    )
}

pub fn package_variant_identity_blake3(variant: &PackageVariant) -> Result<String, CoreFailure> {
    let mut aliases = variant.aliases.clone();
    aliases.sort();
    aliases.dedup();
    debug_assert!(aliases.len() <= variant.aliases.len());
    debug_assert!(aliases.windows(2).all(|pair| pair[0] < pair[1]));
    digest_serializable(
        DOMAIN_VARIANT_IDENTITY_DOMAIN,
        &VariantIdentityPreimage {
            public_selector: &variant.public_selector,
            system: &variant.system,
            base_selector: &variant.base_selector,
            base_package_identity_blake3: &variant.base_package_identity_blake3,
            base_root_identity_blake3: &variant.base_root_identity_blake3,
            variant_name: &variant.variant_name,
            changed_policy_digest_blake3: &variant.changed_policy_digest_blake3,
            root_identity_blake3: &variant.root_identity_blake3,
            provenance_digest_blake3: &variant.provenance_digest_blake3,
            aliases: &aliases,
        },
        "variant-identity-serialization-failed",
    )
}

pub fn validation_root_identity_blake3(root: &ValidationRoot) -> Result<String, CoreFailure> {
    let mut sources = root.sources.clone();
    let mut tools = root.tools.clone();
    let mut dependencies = root.dependencies.clone();
    normalize_artifacts(&mut sources);
    normalize_artifacts(&mut tools);
    normalize_artifacts(&mut dependencies);
    debug_assert!(sources.windows(2).all(|pair| pair[0] < pair[1]));
    debug_assert!(tools.windows(2).all(|pair| pair[0] < pair[1]));
    debug_assert!(dependencies.windows(2).all(|pair| pair[0] < pair[1]));
    digest_serializable(
        DOMAIN_VALIDATION_ROOT_IDENTITY_DOMAIN,
        &ValidationRootIdentityPreimage {
            name: &root.name,
            system: &root.system,
            package_selector: &root.package_selector,
            package_identity_blake3: &root.package_identity_blake3,
            validation_selector: &root.validation_selector,
            validation_package_identity_blake3: &root.validation_package_identity_blake3,
            sources: &sources,
            tools: &tools,
            dependencies: &dependencies,
            policy_digest_blake3: &root.policy_digest_blake3,
            expected_outcome: &root.expected_outcome,
            limits: &root.limits,
        },
        "validation-root-identity-serialization-failed",
    )
}

pub struct V1DomainShardInput<'a> {
    pub catalog: &'a MantlepkgsCatalog,
    pub name: &'a str,
    pub class: &'a str,
    pub owner_label: &'a str,
    pub source_repository: &'a str,
    pub limits: DomainShardLimits,
}

pub fn adapt_v1_catalog_to_domain_shard(input: V1DomainShardInput<'_>) -> Result<DomainShard, CoreFailure> {
    validate_catalog_identity(input.catalog)?;
    let mut packages = Vec::with_capacity(input.catalog.packages.len());
    for package in &input.catalog.packages {
        packages.push(adapt_v1_package(input.catalog, package)?);
    }
    packages.sort();
    let source = DomainSourceLock {
        repository: input.source_repository.into(),
        reference: input.catalog.source_lock.reference.clone(),
        revision: input.catalog.source_lock.revision.clone(),
        source_digest_blake3: input.catalog.source_lock.lock_digest_blake3.clone(),
    };
    let mut shard = DomainShard {
        schema: DOMAIN_SHARD_SCHEMA.into(),
        shard_identity_blake3: empty_digest(),
        name: input.name.into(),
        class: input.class.into(),
        owner_label: input.owner_label.into(),
        source,
        source_catalog_identity_blake3: input.catalog.catalog_identity_blake3.clone(),
        packages,
        limits: input.limits,
    };
    shard.shard_identity_blake3 = domain_shard_identity_blake3(&shard)?;
    validate_shard(&shard, 0)?;
    debug_assert_eq!(shard.packages.len(), input.catalog.packages.len());
    debug_assert_eq!(shard.source_catalog_identity_blake3, input.catalog.catalog_identity_blake3);
    Ok(shard)
}

pub fn seal_domain_manifest(manifest: &DomainCatalogManifest) -> Result<DomainCatalogManifest, CoreFailure> {
    let mut sealed = manifest.clone();
    seal_shards(&mut sealed.shards)?;
    let mut public_packages = collect_sealed_package_selectors(&sealed.shards)?;
    seal_variants(&mut sealed.variants, &mut public_packages)?;
    seal_validation_roots(&mut sealed.validation_roots, &public_packages)?;
    let normalized = normalize_domain_manifest(&sealed)?;
    debug_assert_eq!(normalized.shards.len(), manifest.shards.len());
    debug_assert_eq!(normalized.validation_roots.len(), manifest.validation_roots.len());
    Ok(normalized)
}

fn public_package_record_capacity(manifest: &DomainCatalogManifest) -> usize {
    let package_records = manifest
        .shards
        .iter()
        .flat_map(|shard| shard.packages.iter())
        .fold(0usize, |count, package| count.saturating_add(1).saturating_add(package.aliases.len()));
    manifest
        .variants
        .iter()
        .fold(package_records, |count, variant| count.saturating_add(1).saturating_add(variant.aliases.len()))
}

pub fn compose_domain_catalog(manifest: &DomainCatalogManifest) -> Result<DomainCatalog, CoreFailure> {
    let normalized = normalize_domain_manifest(manifest)?;
    #[allow(tigerstyle::numeric_units)] // Vec capacity is measured in package-record items.
    let package_capacity_items_count = public_package_record_capacity(&normalized);
    let mut diagnostics = Vec::with_capacity(VALIDATION_DIAGNOSTIC_HEADROOM);
    let mut public_selectors = BTreeMap::<(String, String), PublicPackageRecord>::new();
    let mut package_identities = BTreeSet::new();
    let mut packages = Vec::with_capacity(package_capacity_items_count);
    for shard in &normalized.shards {
        compose_shard(shard, &mut public_selectors, &mut package_identities, &mut packages, &mut diagnostics);
    }
    validate_variants(
        &normalized.variants,
        &mut public_selectors,
        &package_identities,
        &mut packages,
        &mut diagnostics,
    );
    validate_validation_roots(&normalized.validation_roots, &public_selectors, &mut diagnostics);
    diagnostics.sort();
    diagnostics.dedup();
    if !diagnostics.is_empty() {
        return Err(CoreFailure::from_diagnostics(diagnostics));
    }
    packages.sort_by(|left, right| {
        (&left.system, &left.public_selector, &left.package_identity_blake3).cmp(&(
            &right.system,
            &right.public_selector,
            &right.package_identity_blake3,
        ))
    });
    let non_claims = domain_non_claims();
    let identity = digest_serializable(
        DOMAIN_CATALOG_IDENTITY_DOMAIN,
        &(
            DOMAIN_CATALOG_SCHEMA,
            &normalized.name,
            &normalized.shards,
            &packages,
            &normalized.variants,
            &normalized.validation_roots,
            &non_claims,
        ),
        "domain-catalog-identity-serialization-failed",
    )?;
    debug_assert!(packages.len() <= package_capacity_items_count);
    debug_assert!(is_lower_hex_length(&identity, BLAKE3_HEX_LENGTH));
    Ok(DomainCatalog {
        schema: DOMAIN_CATALOG_SCHEMA.into(),
        catalog_identity_blake3: identity,
        name: normalized.name,
        shards: normalized.shards,
        packages,
        variants: normalized.variants,
        validation_roots: normalized.validation_roots,
        non_claims,
    })
}

pub fn validate_domain_catalog(catalog: &DomainCatalog) -> Result<(), CoreFailure> {
    require_equal(&catalog.schema, DOMAIN_CATALOG_SCHEMA, "catalog.schema", "unsupported-domain-catalog-schema")?;
    require_digest(&catalog.catalog_identity_blake3, "catalog.catalog_identity_blake3")?;
    let manifest = DomainCatalogManifest {
        schema: DOMAIN_MANIFEST_SCHEMA.into(),
        name: catalog.name.clone(),
        shards: catalog.shards.clone(),
        variants: catalog.variants.clone(),
        validation_roots: catalog.validation_roots.clone(),
        limits: inferred_catalog_limits(catalog),
    };
    let recomposed = compose_domain_catalog(&manifest)?;
    if recomposed.catalog_identity_blake3 != catalog.catalog_identity_blake3 || recomposed.packages != catalog.packages
    {
        return Err(CoreFailure::from_diagnostic(Diagnostic::new(
            "domain-catalog-identity-mismatch",
            "catalog.catalog_identity_blake3",
            "the domain catalog differs from its deterministic composition",
        )));
    }
    debug_assert_eq!(recomposed.catalog_identity_blake3, catalog.catalog_identity_blake3);
    debug_assert_eq!(recomposed.packages, catalog.packages);
    Ok(())
}

pub fn plan_validation_root(
    catalog: &DomainCatalog,
    validation_root_identity: &str,
) -> Result<ValidationRootPlan, CoreFailure> {
    validate_domain_catalog(catalog)?;
    let root = catalog
        .validation_roots
        .iter()
        .find(|root| root.validation_root_identity_blake3 == validation_root_identity)
        .ok_or_else(|| {
            CoreFailure::from_diagnostic(Diagnostic::new(
                "validation-root-not-found",
                "validation_root_identity_blake3",
                "the selected validation root is absent from the domain catalog",
            ))
        })?;
    let package = lookup_public_package(catalog, &root.system, &root.package_selector)?;
    let validation_package = lookup_public_package(catalog, &root.system, &root.validation_selector)?;
    if package.package_identity_blake3 != root.package_identity_blake3
        || validation_package.package_identity_blake3 != root.validation_package_identity_blake3
    {
        return Err(CoreFailure::from_diagnostic(Diagnostic::new(
            "validation-root-package-stale",
            "validation_root",
            "the validation root package identities differ from the composed catalog",
        )));
    }
    debug_assert_eq!(root.validation_root_identity_blake3, validation_root_identity);
    debug_assert!(package.buildable && validation_package.buildable);
    Ok(ValidationRootPlan {
        validation_root_identity_blake3: root.validation_root_identity_blake3.clone(),
        domain_catalog_identity_blake3: catalog.catalog_identity_blake3.clone(),
        package_selector: root.package_selector.clone(),
        package_identity_blake3: package.package_identity_blake3.clone(),
        package_root_identity_blake3: package.root_identity_blake3.clone(),
        validation_selector: root.validation_selector.clone(),
        validation_package_identity_blake3: validation_package.package_identity_blake3.clone(),
        validation_root_package_identity_blake3: validation_package.root_identity_blake3.clone(),
        sources: root.sources.clone(),
        tools: root.tools.clone(),
        dependencies: root.dependencies.clone(),
        policy_digest_blake3: root.policy_digest_blake3.clone(),
        expected_outcome: root.expected_outcome.clone(),
        max_output_bytes: root.limits.max_output_bytes,
        timeout_seconds: root.limits.timeout_seconds,
    })
}

pub fn record_validation_observation(
    plan: &ValidationRootPlan,
    observation: &ValidationObservation,
) -> Result<ValidationReceipt, CoreFailure> {
    debug_assert!(plan.max_output_bytes > 0);
    debug_assert!(plan.timeout_seconds > 0);
    require_equal(
        &observation.schema,
        VALIDATION_OBSERVATION_SCHEMA,
        "observation.schema",
        "unsupported-validation-observation-schema",
    )?;
    if observation.validation_root_identity_blake3 != plan.validation_root_identity_blake3 {
        return Err(CoreFailure::from_diagnostic(Diagnostic::new(
            "validation-observation-plan-mismatch",
            "observation.validation_root_identity_blake3",
            "the observation does not name the planned validation root",
        )));
    }
    validate_validation_outcome(&observation.outcome, "observation.outcome")?;
    require_digest(&observation.realization_receipt_blake3, "observation.realization_receipt_blake3")?;
    if let Some(diagnostic) = &observation.diagnostic_digest_blake3 {
        require_digest(diagnostic, "observation.diagnostic_digest_blake3")?;
    }
    if observation.observed_output_bytes > plan.max_output_bytes {
        return Err(CoreFailure::from_diagnostic(Diagnostic::new(
            "validation-output-limit-exceeded",
            "observation.observed_output_bytes",
            "the observed validation output exceeds the planned limit",
        )));
    }
    let timeout_ms = plan.timeout_seconds.checked_mul(MILLISECONDS_PER_SECOND).ok_or_else(|| {
        CoreFailure::from_diagnostic(Diagnostic::new(
            "validation-timeout-overflow",
            "plan.timeout_seconds",
            "the validation timeout does not fit milliseconds",
        ))
    })?;
    if observation.elapsed_milliseconds > timeout_ms && observation.outcome != OBSERVED_VALIDATION_TIMEOUT {
        return Err(CoreFailure::from_diagnostic(Diagnostic::new(
            "validation-timeout-misclassified",
            "observation.outcome",
            "an observation beyond the planned timeout must retain timeout status",
        )));
    }
    let is_accepted = observation.outcome == plan.expected_outcome;
    let non_claims = validation_non_claims();
    let identity = digest_serializable(
        VALIDATION_RECEIPT_IDENTITY_DOMAIN,
        &(VALIDATION_RECEIPT_SCHEMA, plan, observation, is_accepted, &non_claims),
        "validation-receipt-identity-serialization-failed",
    )?;
    Ok(ValidationReceipt {
        schema: VALIDATION_RECEIPT_SCHEMA.into(),
        receipt_identity_blake3: identity,
        validation_root_identity_blake3: plan.validation_root_identity_blake3.clone(),
        domain_catalog_identity_blake3: plan.domain_catalog_identity_blake3.clone(),
        package_identity_blake3: plan.package_identity_blake3.clone(),
        package_root_identity_blake3: plan.package_root_identity_blake3.clone(),
        validation_package_identity_blake3: plan.validation_package_identity_blake3.clone(),
        outcome: observation.outcome.clone(),
        accepted: is_accepted,
        realization_receipt_blake3: observation.realization_receipt_blake3.clone(),
        observed_output_bytes: observation.observed_output_bytes,
        elapsed_milliseconds: observation.elapsed_milliseconds,
        diagnostic_digest_blake3: observation.diagnostic_digest_blake3.clone(),
        non_claims,
    })
}

pub fn seal_external_corpus_evidence(evidence: &ExternalCorpusEvidence) -> Result<ExternalCorpusEvidence, CoreFailure> {
    let mut sealed = evidence.clone();
    sealed.selected_packages.sort();
    sealed.selected_packages.dedup();
    sealed.artifacts.sort();
    sealed.non_claims.sort();
    sealed.non_claims.dedup();
    let observed = external_corpus_evidence_identity_blake3(&sealed)?;
    seal_digest_field(&mut sealed.evidence_identity_blake3, &observed, "evidence.evidence_identity_blake3")?;
    Ok(sealed)
}

pub fn external_corpus_evidence_identity_blake3(evidence: &ExternalCorpusEvidence) -> Result<String, CoreFailure> {
    digest_serializable(
        CORPUS_EVIDENCE_IDENTITY_DOMAIN,
        &(
            &evidence.schema,
            &evidence.repository,
            &evidence.source_reference,
            &evidence.revision,
            &evidence.source_digest_blake3,
            &evidence.observed_license,
            &evidence.license_path,
            &evidence.license_digest_blake3,
            &evidence.selected_packages,
            &evidence.producer_policy_digest_blake3,
            &evidence.artifacts,
            &evidence.non_claims,
        ),
        "corpus-evidence-identity-serialization-failed",
    )
}

pub fn validate_external_corpus_evidence(
    evidence: &ExternalCorpusEvidence,
    observations: &[CorpusArtifactObservation],
) -> Result<(), CoreFailure> {
    let mut diagnostics = Vec::with_capacity(
        evidence
            .artifacts
            .len()
            .saturating_add(observations.len())
            .saturating_add(VALIDATION_DIAGNOSTIC_HEADROOM),
    );
    debug_assert!(diagnostics.capacity() >= evidence.artifacts.len());
    debug_assert!(diagnostics.capacity() >= observations.len());
    require_equal_diagnostic(
        &evidence.schema,
        CORPUS_EVIDENCE_SCHEMA,
        "evidence.schema",
        "unsupported-corpus-evidence-schema",
        &mut diagnostics,
    );
    validate_text(&evidence.repository, "evidence.repository", &mut diagnostics);
    validate_text(&evidence.source_reference, "evidence.source_reference", &mut diagnostics);
    validate_revision(&evidence.revision, "evidence.revision", &mut diagnostics);
    if !evidence.source_reference.contains(&evidence.revision) {
        diagnostics.push(Diagnostic::new(
            "floating-corpus-reference",
            "evidence.source_reference",
            "the corpus source reference does not contain the exact revision",
        ));
    }
    validate_digest(&evidence.source_digest_blake3, "evidence.source_digest_blake3", &mut diagnostics);
    require_equal_diagnostic(
        &evidence.observed_license,
        CORPUS_LICENSE_MIT,
        "evidence.observed_license",
        "unsupported-corpus-license-record",
        &mut diagnostics,
    );
    validate_relative_path(&evidence.license_path, "evidence.license_path", &mut diagnostics);
    validate_digest(&evidence.license_digest_blake3, "evidence.license_digest_blake3", &mut diagnostics);
    validate_digest(
        &evidence.producer_policy_digest_blake3,
        "evidence.producer_policy_digest_blake3",
        &mut diagnostics,
    );
    validate_string_set(&evidence.selected_packages, "evidence.selected_packages", &mut diagnostics);
    validate_corpus_artifacts(evidence, observations, &mut diagnostics);
    if evidence.non_claims.len() < REQUIRED_NON_CLAIM_COUNT {
        diagnostics.push(Diagnostic::new(
            "corpus-non-claims-incomplete",
            "evidence.non_claims",
            "the corpus evidence omits required bounded non-claims",
        ));
    }
    match external_corpus_evidence_identity_blake3(evidence) {
        Ok(identity) if identity == evidence.evidence_identity_blake3 => {}
        Ok(_) => diagnostics.push(Diagnostic::new(
            "corpus-evidence-identity-mismatch",
            "evidence.evidence_identity_blake3",
            "the corpus evidence identity differs from its canonical fields",
        )),
        Err(failure) => diagnostics.extend(failure.diagnostics),
    }
    diagnostics.sort();
    diagnostics.dedup();
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(CoreFailure::from_diagnostics(diagnostics))
    }
}

fn seal_shards(shards: &mut [DomainShard]) -> Result<(), CoreFailure> {
    for shard in shards {
        for package in &mut shard.packages {
            seal_package_identity(package)?;
        }
        shard.packages.sort();
        seal_shard_identity(shard)?;
    }
    Ok(())
}

fn collect_sealed_package_selectors(shards: &[DomainShard]) -> Result<SealedSelectorMap, CoreFailure> {
    let mut selectors = BTreeMap::new();
    for shard in shards {
        for package in &shard.packages {
            let value = (package.package_identity_blake3.clone(), package.root_identity_blake3.clone());
            register_seal_selector(
                &mut selectors,
                (package.system.clone(), package.public_selector.clone()),
                value.clone(),
            )?;
            for alias in &package.aliases {
                register_seal_selector(&mut selectors, (package.system.clone(), alias.clone()), value.clone())?;
            }
        }
    }
    let package_count = shards.iter().fold(0usize, |count, shard| count.saturating_add(shard.packages.len()));
    let expected_selector_count = shards
        .iter()
        .flat_map(|shard| shard.packages.iter())
        .fold(0usize, |count, package| count.saturating_add(1).saturating_add(package.aliases.len()));
    debug_assert_eq!(selectors.len(), expected_selector_count);
    debug_assert!(selectors.len() >= package_count);
    Ok(selectors)
}

fn seal_variants(variants: &mut [PackageVariant], selectors: &mut SealedSelectorMap) -> Result<(), CoreFailure> {
    let initial_selector_count = selectors.len();
    for variant in variants.iter_mut() {
        if let Some((package_identity, root_identity)) =
            selectors.get(&(variant.system.clone(), variant.base_selector.clone()))
        {
            seal_digest_field(
                &mut variant.base_package_identity_blake3,
                package_identity,
                "variant.base_package_identity_blake3",
            )?;
            seal_digest_field(
                &mut variant.base_root_identity_blake3,
                root_identity,
                "variant.base_root_identity_blake3",
            )?;
        }
        seal_variant_identity(variant)?;
        let value = (variant.variant_identity_blake3.clone(), variant.root_identity_blake3.clone());
        register_seal_selector(selectors, (variant.system.clone(), variant.public_selector.clone()), value.clone())?;
        for alias in &variant.aliases {
            register_seal_selector(selectors, (variant.system.clone(), alias.clone()), value.clone())?;
        }
    }
    debug_assert!(selectors.len() >= initial_selector_count);
    debug_assert!(selectors.len().saturating_sub(initial_selector_count) >= variants.len());
    Ok(())
}

fn seal_validation_roots(roots: &mut [ValidationRoot], selectors: &SealedSelectorMap) -> Result<(), CoreFailure> {
    let root_count = roots.len();
    for root in roots.iter_mut() {
        if let Some((package_identity, _)) = selectors.get(&(root.system.clone(), root.package_selector.clone())) {
            seal_digest_field(
                &mut root.package_identity_blake3,
                package_identity,
                "validation_root.package_identity_blake3",
            )?;
        }
        if let Some((validation_identity, _)) = selectors.get(&(root.system.clone(), root.validation_selector.clone()))
        {
            seal_digest_field(
                &mut root.validation_package_identity_blake3,
                validation_identity,
                "validation_root.validation_package_identity_blake3",
            )?;
        }
        seal_validation_root_identity(root)?;
    }
    debug_assert_eq!(roots.len(), root_count);
    debug_assert!(
        roots
            .iter()
            .all(|root| is_lower_hex_length(&root.validation_root_identity_blake3, BLAKE3_HEX_LENGTH))
    );
    Ok(())
}

fn register_seal_selector(
    selectors: &mut SealedSelectorMap,
    key: PublicSelectorKey,
    value: IdentityBinding,
) -> Result<(), CoreFailure> {
    if selectors.insert(key.clone(), value).is_some() {
        return Err(CoreFailure::from_diagnostic(Diagnostic::new(
            "ambiguous-public-selector-before-seal",
            &key.1,
            "more than one package or variant claims a selector needed for identity sealing",
        )));
    }
    Ok(())
}

fn seal_package_identity(package: &mut DomainPackage) -> Result<(), CoreFailure> {
    let observed = domain_package_identity_blake3(package)?;
    seal_digest_field(&mut package.package_identity_blake3, &observed, "package.package_identity_blake3")
}

fn seal_shard_identity(shard: &mut DomainShard) -> Result<(), CoreFailure> {
    let observed = domain_shard_identity_blake3(shard)?;
    seal_digest_field(&mut shard.shard_identity_blake3, &observed, "shard.shard_identity_blake3")
}

fn seal_variant_identity(variant: &mut PackageVariant) -> Result<(), CoreFailure> {
    let observed = package_variant_identity_blake3(variant)?;
    seal_digest_field(&mut variant.variant_identity_blake3, &observed, "variant.variant_identity_blake3")
}

fn seal_validation_root_identity(root: &mut ValidationRoot) -> Result<(), CoreFailure> {
    let observed = validation_root_identity_blake3(root)?;
    seal_digest_field(
        &mut root.validation_root_identity_blake3,
        &observed,
        "validation_root.validation_root_identity_blake3",
    )
}

#[allow(tigerstyle::ambiguous_params)] // Observed digest and diagnostic path are separate validated domains.
fn seal_digest_field(field: &mut String, observed: &str, path: &str) -> Result<(), CoreFailure> {
    if field == &empty_digest() {
        *field = observed.into();
        return Ok(());
    }
    if field == observed {
        return Ok(());
    }
    Err(CoreFailure::from_diagnostic(Diagnostic::new(
        "declared-identity-mismatch",
        path,
        "the declared identity is neither the zero placeholder nor the canonical identity",
    )))
}

fn normalize_domain_manifest(manifest: &DomainCatalogManifest) -> Result<DomainCatalogManifest, CoreFailure> {
    let mut normalized = manifest.clone();
    for shard in &mut normalized.shards {
        for package in &mut shard.packages {
            package.aliases.sort();
            package.aliases.dedup();
            package.artifacts.sort();
            package.artifacts.dedup();
        }
        shard.packages.sort();
    }
    for variant in &mut normalized.variants {
        variant.aliases.sort();
        variant.aliases.dedup();
    }
    for root in &mut normalized.validation_roots {
        normalize_artifacts(&mut root.sources);
        normalize_artifacts(&mut root.tools);
        normalize_artifacts(&mut root.dependencies);
    }
    normalized.shards.sort_by(|left, right| {
        (&left.name, &left.shard_identity_blake3).cmp(&(&right.name, &right.shard_identity_blake3))
    });
    normalized.variants.sort();
    normalized.validation_roots.sort();
    validate_domain_manifest(&normalized)?;
    debug_assert_eq!(normalized.shards.len(), manifest.shards.len());
    debug_assert_eq!(normalized.variants.len(), manifest.variants.len());
    Ok(normalized)
}

fn finish_diagnostics(mut diagnostics: Vec<Diagnostic>) -> Result<(), CoreFailure> {
    diagnostics.sort();
    diagnostics.dedup();
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(CoreFailure::from_diagnostics(diagnostics))
    }
}

#[derive(Default)]
struct DomainItemCounts {
    packages: usize,
    aliases: usize,
    artifacts: usize,
}

fn validate_domain_manifest(manifest: &DomainCatalogManifest) -> Result<(), CoreFailure> {
    #[allow(tigerstyle::numeric_units)] // Vec capacity is measured in diagnostic items.
    let diagnostic_capacity_items_count = manifest.shards.len().saturating_add(VALIDATION_DIAGNOSTIC_HEADROOM);
    let mut diagnostics = Vec::with_capacity(diagnostic_capacity_items_count);
    validate_manifest_header(manifest, &mut diagnostics);
    validate_manifest_collection_limits(manifest, &mut diagnostics);
    let counts = validate_manifest_shards(manifest, &mut diagnostics);
    validate_manifest_item_limits(manifest, &counts, &mut diagnostics);
    let expected_package_count =
        manifest.shards.iter().fold(0usize, |count, shard| count.saturating_add(shard.packages.len()));
    debug_assert!(diagnostics.capacity() >= diagnostic_capacity_items_count);
    debug_assert_eq!(counts.packages, expected_package_count);
    finish_diagnostics(diagnostics)
}

fn validate_manifest_header(manifest: &DomainCatalogManifest, diagnostics: &mut Vec<Diagnostic>) {
    require_equal_diagnostic(
        &manifest.schema,
        DOMAIN_MANIFEST_SCHEMA,
        "manifest.schema",
        "unsupported-domain-manifest-schema",
        diagnostics,
    );
    validate_text(&manifest.name, "manifest.name", diagnostics);
    validate_catalog_limits(&manifest.limits, diagnostics);
}

fn validate_manifest_collection_limits(manifest: &DomainCatalogManifest, diagnostics: &mut Vec<Diagnostic>) {
    let initial_diagnostic_count = diagnostics.len();
    validate_count(
        manifest.shards.len(),
        manifest.limits.max_shards,
        MAX_DOMAIN_SHARDS,
        "manifest.shards",
        diagnostics,
    );
    validate_count(
        manifest.variants.len(),
        manifest.limits.max_variants,
        MAX_DOMAIN_VARIANTS,
        "manifest.variants",
        diagnostics,
    );
    validate_count(
        manifest.validation_roots.len(),
        manifest.limits.max_validation_roots,
        MAX_DOMAIN_VALIDATION_ROOTS,
        "manifest.validation_roots",
        diagnostics,
    );
    debug_assert!(diagnostics.len() >= initial_diagnostic_count);
    debug_assert!(diagnostics.capacity() >= diagnostics.len());
}

#[derive(Default)]
struct ShardUniqueness {
    identities: BTreeSet<String>,
    names: BTreeSet<String>,
}

fn validate_manifest_shards(manifest: &DomainCatalogManifest, diagnostics: &mut Vec<Diagnostic>) -> DomainItemCounts {
    let mut uniqueness = ShardUniqueness::default();
    let mut counts = DomainItemCounts::default();
    for (index, shard) in manifest.shards.iter().enumerate() {
        if let Err(failure) = validate_shard(shard, index) {
            diagnostics.extend(failure.diagnostics);
        }
        validate_unique_shard(shard, index, &mut uniqueness, diagnostics);
        accumulate_shard_counts(shard, &mut counts);
    }
    debug_assert!(uniqueness.identities.len() <= manifest.shards.len());
    debug_assert!(uniqueness.names.len() <= manifest.shards.len());
    counts
}

fn validate_unique_shard(
    shard: &DomainShard,
    index: usize,
    uniqueness: &mut ShardUniqueness,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !uniqueness.identities.insert(shard.shard_identity_blake3.clone()) {
        diagnostics.push(Diagnostic::new(
            "duplicate-shard-identity",
            &format!("manifest.shards[{index}].shard_identity_blake3"),
            "the manifest repeats a shard identity",
        ));
    }
    if !uniqueness.names.insert(shard.name.clone()) {
        diagnostics.push(Diagnostic::new(
            "duplicate-shard-name",
            &format!("manifest.shards[{index}].name"),
            "the manifest repeats a shard name",
        ));
    }
}

fn accumulate_shard_counts(shard: &DomainShard, counts: &mut DomainItemCounts) {
    counts.packages = counts.packages.saturating_add(shard.packages.len());
    for package in &shard.packages {
        counts.aliases = counts.aliases.saturating_add(package.aliases.len());
        counts.artifacts = counts.artifacts.saturating_add(package.artifacts.len());
    }
}

fn validate_manifest_item_limits(
    manifest: &DomainCatalogManifest,
    counts: &DomainItemCounts,
    diagnostics: &mut Vec<Diagnostic>,
) {
    validate_count(
        counts.packages,
        manifest.limits.max_packages,
        MAX_DOMAIN_PACKAGES,
        "manifest.packages",
        diagnostics,
    );
    validate_count(counts.aliases, manifest.limits.max_aliases, MAX_DOMAIN_ALIASES, "manifest.aliases", diagnostics);
    let validation_artifact_count = manifest.validation_roots.iter().fold(0usize, |count, root| {
        count
            .saturating_add(root.sources.len())
            .saturating_add(root.tools.len())
            .saturating_add(root.dependencies.len())
    });
    let artifact_count = counts.artifacts.saturating_add(validation_artifact_count);
    validate_count(
        artifact_count,
        manifest.limits.max_artifacts,
        MAX_DOMAIN_ARTIFACTS,
        "manifest.artifacts",
        diagnostics,
    );
    debug_assert!(artifact_count >= counts.artifacts);
    debug_assert!(diagnostics.capacity() >= diagnostics.len());
}

fn validate_shard(shard: &DomainShard, index: usize) -> Result<(), CoreFailure> {
    let base = format!("manifest.shards[{index}]");
    #[allow(tigerstyle::numeric_units)] // Vec capacity is measured in diagnostic items.
    let diagnostic_capacity_items_count = shard.packages.len().saturating_add(VALIDATION_DIAGNOSTIC_HEADROOM);
    let mut diagnostics = Vec::with_capacity(diagnostic_capacity_items_count);
    validate_shard_header(shard, &base, &mut diagnostics);
    validate_shard_counts(shard, &base, &mut diagnostics);
    validate_shard_packages(shard, &base, &mut diagnostics);
    validate_shard_identity(shard, &base, &mut diagnostics);
    debug_assert!(diagnostics.capacity() >= diagnostic_capacity_items_count);
    debug_assert!(diagnostics.capacity() >= diagnostics.len());
    finish_diagnostics(diagnostics)
}

fn validate_shard_header(shard: &DomainShard, base: &str, diagnostics: &mut Vec<Diagnostic>) {
    require_equal_diagnostic(
        &shard.schema,
        DOMAIN_SHARD_SCHEMA,
        &format!("{base}.schema"),
        "unsupported-domain-shard-schema",
        diagnostics,
    );
    validate_text(&shard.name, &format!("{base}.name"), diagnostics);
    validate_domain_class(&shard.class, &format!("{base}.class"), diagnostics);
    validate_text(&shard.owner_label, &format!("{base}.owner_label"), diagnostics);
    validate_source_lock(&shard.source, &format!("{base}.source"), diagnostics);
    validate_digest(
        &shard.source_catalog_identity_blake3,
        &format!("{base}.source_catalog_identity_blake3"),
        diagnostics,
    );
}

fn validate_shard_counts(shard: &DomainShard, base: &str, diagnostics: &mut Vec<Diagnostic>) {
    validate_shard_limits(&shard.limits, &format!("{base}.limits"), diagnostics);
    validate_count(
        shard.packages.len(),
        shard.limits.max_packages,
        MAX_DOMAIN_PACKAGES,
        &format!("{base}.packages"),
        diagnostics,
    );
    let counts = shard.packages.iter().fold(DomainItemCounts::default(), |mut counts, package| {
        counts.aliases = counts.aliases.saturating_add(package.aliases.len());
        counts.artifacts = counts.artifacts.saturating_add(package.artifacts.len());
        counts
    });
    validate_count(
        counts.aliases,
        shard.limits.max_aliases,
        MAX_DOMAIN_ALIASES,
        &format!("{base}.aliases"),
        diagnostics,
    );
    validate_count(
        counts.artifacts,
        shard.limits.max_artifacts,
        MAX_DOMAIN_ARTIFACTS,
        &format!("{base}.artifacts"),
        diagnostics,
    );
    let expected_alias_count =
        shard.packages.iter().fold(0usize, |count, package| count.saturating_add(package.aliases.len()));
    debug_assert_eq!(counts.aliases, expected_alias_count);
    debug_assert!(diagnostics.capacity() >= diagnostics.len());
}

fn validate_shard_packages(shard: &DomainShard, base: &str, diagnostics: &mut Vec<Diagnostic>) {
    let mut package_ids = BTreeSet::new();
    for (package_index, package) in shard.packages.iter().enumerate() {
        validate_package(package, &format!("{base}.packages[{package_index}]"), diagnostics);
        if !package_ids.insert(&package.package_identity_blake3) {
            diagnostics.push(Diagnostic::new(
                "duplicate-package-identity",
                &format!("{base}.packages[{package_index}].package_identity_blake3"),
                "the shard repeats a package identity",
            ));
        }
    }
    debug_assert!(package_ids.len() <= shard.packages.len());
    debug_assert!(diagnostics.capacity() >= diagnostics.len());
}

fn validate_shard_identity(shard: &DomainShard, base: &str, diagnostics: &mut Vec<Diagnostic>) {
    match domain_shard_identity_blake3(shard) {
        Ok(identity) if identity == shard.shard_identity_blake3 => {}
        Ok(_) => diagnostics.push(Diagnostic::new(
            "stale-shard-identity",
            &format!("{base}.shard_identity_blake3"),
            "the shard identity differs from its canonical fields",
        )),
        Err(failure) => diagnostics.extend(failure.diagnostics),
    }
}

fn validate_package(package: &DomainPackage, path: &str, diagnostics: &mut Vec<Diagnostic>) {
    validate_text(&package.name, &format!("{path}.name"), diagnostics);
    validate_selector(&package.public_selector, &format!("{path}.public_selector"), diagnostics);
    validate_text(&package.system, &format!("{path}.system"), diagnostics);
    validate_digest(&package.root_identity_blake3, &format!("{path}.root_identity_blake3"), diagnostics);
    validate_digest(&package.policy_digest_blake3, &format!("{path}.policy_digest_blake3"), diagnostics);
    validate_optional_string_set(&package.aliases, &format!("{path}.aliases"), diagnostics);
    validate_artifacts(&package.artifacts, &format!("{path}.artifacts"), diagnostics);
    match domain_package_identity_blake3(package) {
        Ok(identity) if identity == package.package_identity_blake3 => {}
        Ok(_) => diagnostics.push(Diagnostic::new(
            "stale-package-identity",
            &format!("{path}.package_identity_blake3"),
            "the package identity differs from its canonical fields",
        )),
        Err(failure) => diagnostics.extend(failure.diagnostics),
    }
}

fn compose_shard(
    shard: &DomainShard,
    public_selectors: &mut BTreeMap<(String, String), PublicPackageRecord>,
    package_identities: &mut BTreeSet<String>,
    packages: &mut Vec<PublicPackageRecord>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for package in &shard.packages {
        if !package_identities.insert(package.package_identity_blake3.clone()) {
            diagnostics.push(Diagnostic::new(
                "conflicting-package-identity",
                &package.package_identity_blake3,
                "more than one shard contains the package identity",
            ));
        }
        let record = PublicPackageRecord {
            public_selector: package.public_selector.clone(),
            system: package.system.clone(),
            package_identity_blake3: package.package_identity_blake3.clone(),
            root_identity_blake3: package.root_identity_blake3.clone(),
            shard_identity_blake3: shard.shard_identity_blake3.clone(),
            variant_base_identity_blake3: None,
            buildable: package.buildable,
        };
        register_public_selector(public_selectors, &record, &package.public_selector, diagnostics);
        packages.push(record.clone());
        for alias in &package.aliases {
            let mut alias_record = record.clone();
            alias_record.public_selector = alias.clone();
            register_public_selector(public_selectors, &alias_record, alias, diagnostics);
            packages.push(alias_record);
        }
    }
}

struct VariantValidationContext<'a> {
    variant_selectors: &'a BTreeMap<(String, String), &'a PackageVariant>,
    public_selectors: &'a mut BTreeMap<(String, String), PublicPackageRecord>,
    package_identities: &'a BTreeSet<String>,
    packages: &'a mut Vec<PublicPackageRecord>,
    diagnostics: &'a mut Vec<Diagnostic>,
}

fn validate_variants(
    variants: &[PackageVariant],
    public_selectors: &mut BTreeMap<(String, String), PublicPackageRecord>,
    package_identities: &BTreeSet<String>,
    packages: &mut Vec<PublicPackageRecord>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let initial_package_count = packages.len();
    let initial_diagnostic_count = diagnostics.len();
    let expected_added_package_count = variants
        .iter()
        .fold(0usize, |count, variant| count.saturating_add(1).saturating_add(variant.aliases.len()));
    let variant_selectors = variants
        .iter()
        .map(|variant| ((variant.system.clone(), variant.public_selector.clone()), variant))
        .collect::<BTreeMap<_, _>>();
    let mut context = VariantValidationContext {
        variant_selectors: &variant_selectors,
        public_selectors,
        package_identities,
        packages,
        diagnostics,
    };
    for (index, variant) in variants.iter().enumerate() {
        validate_variant(index, variant, &mut context);
    }
    debug_assert_eq!(context.packages.len().saturating_sub(initial_package_count), expected_added_package_count);
    debug_assert!(context.diagnostics.len() >= initial_diagnostic_count);
}

fn validate_variant(index: usize, variant: &PackageVariant, context: &mut VariantValidationContext<'_>) {
    let path = format!("manifest.variants[{index}]");
    validate_variant_fields(variant, &path, context.diagnostics);
    if has_variant_cycle(variant, context.variant_selectors) {
        context.diagnostics.push(Diagnostic::new(
            "variant-cycle",
            &format!("{path}.base_selector"),
            "the variant base relationship contains a cycle",
        ));
    }
    let base_key = (variant.system.clone(), variant.base_selector.clone());
    let base = context.public_selectors.get(&base_key).cloned();
    let base_shard_identity =
        base.as_ref().map(|package| package.shard_identity_blake3.clone()).unwrap_or_else(empty_digest);
    let is_base_buildable = base.as_ref().map(|package| package.buildable).unwrap_or(false);
    match base {
        Some(base)
            if base.package_identity_blake3 == variant.base_package_identity_blake3
                && base.root_identity_blake3 == variant.base_root_identity_blake3 =>
        {
            if !context.package_identities.contains(&base.package_identity_blake3) {
                context.diagnostics.push(Diagnostic::new(
                    "variant-base-is-not-package",
                    &format!("{path}.base_selector"),
                    "the variant base is not an ordinary package in the composed generation",
                ));
            }
        }
        Some(_) => context.diagnostics.push(Diagnostic::new(
            "variant-base-stale",
            &format!("{path}.base_package_identity_blake3"),
            "the variant base identity or root differs from the composed package",
        )),
        None => context.diagnostics.push(Diagnostic::new(
            "variant-base-missing",
            &format!("{path}.base_selector"),
            "the variant base package is absent from the composed generation",
        )),
    }
    let record = PublicPackageRecord {
        public_selector: variant.public_selector.clone(),
        system: variant.system.clone(),
        package_identity_blake3: variant.variant_identity_blake3.clone(),
        root_identity_blake3: variant.root_identity_blake3.clone(),
        shard_identity_blake3: base_shard_identity,
        variant_base_identity_blake3: Some(variant.base_package_identity_blake3.clone()),
        buildable: is_base_buildable,
    };
    register_public_selector(context.public_selectors, &record, &variant.public_selector, context.diagnostics);
    context.packages.push(record.clone());
    for alias in &variant.aliases {
        let mut alias_record = record.clone();
        alias_record.public_selector = alias.clone();
        register_public_selector(context.public_selectors, &alias_record, alias, context.diagnostics);
        context.packages.push(alias_record);
    }
    debug_assert_eq!(context.packages.last().map(|package| &package.system), Some(&variant.system));
    debug_assert!(context.packages.len() >= variant.aliases.len().saturating_add(1));
}

fn validate_variant_fields(variant: &PackageVariant, path: &str, diagnostics: &mut Vec<Diagnostic>) {
    let initial_diagnostic_count = diagnostics.len();
    validate_selector(&variant.public_selector, &format!("{path}.public_selector"), diagnostics);
    validate_text(&variant.system, &format!("{path}.system"), diagnostics);
    validate_selector(&variant.base_selector, &format!("{path}.base_selector"), diagnostics);
    validate_digest(
        &variant.base_package_identity_blake3,
        &format!("{path}.base_package_identity_blake3"),
        diagnostics,
    );
    validate_digest(&variant.base_root_identity_blake3, &format!("{path}.base_root_identity_blake3"), diagnostics);
    validate_text(&variant.variant_name, &format!("{path}.variant_name"), diagnostics);
    validate_digest(
        &variant.changed_policy_digest_blake3,
        &format!("{path}.changed_policy_digest_blake3"),
        diagnostics,
    );
    validate_digest(&variant.root_identity_blake3, &format!("{path}.root_identity_blake3"), diagnostics);
    validate_digest(&variant.provenance_digest_blake3, &format!("{path}.provenance_digest_blake3"), diagnostics);
    validate_optional_string_set(&variant.aliases, &format!("{path}.aliases"), diagnostics);
    match package_variant_identity_blake3(variant) {
        Ok(identity) if identity == variant.variant_identity_blake3 => {}
        Ok(_) => diagnostics.push(Diagnostic::new(
            "stale-variant-identity",
            &format!("{path}.variant_identity_blake3"),
            "the variant identity differs from its canonical fields",
        )),
        Err(failure) => diagnostics.extend(failure.diagnostics),
    }
    debug_assert!(diagnostics.len() >= initial_diagnostic_count);
    debug_assert!(diagnostics.capacity() >= diagnostics.len());
}

fn validate_validation_roots(
    roots: &[ValidationRoot],
    public_selectors: &BTreeMap<(String, String), PublicPackageRecord>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut identities = BTreeSet::new();
    let mut names = BTreeSet::new();
    for (index, root) in roots.iter().enumerate() {
        let path = format!("manifest.validation_roots[{index}]");
        validate_validation_root_fields(root, &path, diagnostics);
        if !identities.insert(&root.validation_root_identity_blake3) {
            diagnostics.push(Diagnostic::new(
                "duplicate-validation-root-identity",
                &format!("{path}.validation_root_identity_blake3"),
                "the manifest repeats a validation root identity",
            ));
        }
        if !names.insert((&root.system, &root.name)) {
            diagnostics.push(Diagnostic::new(
                "duplicate-validation-root-name",
                &format!("{path}.name"),
                "the manifest repeats a validation root name for the system",
            ));
        }
        validate_validation_package_binding(root, public_selectors, &path, diagnostics);
    }
    debug_assert!(identities.len() <= roots.len());
    debug_assert!(names.len() <= roots.len());
}

fn validate_validation_root_fields(root: &ValidationRoot, path: &str, diagnostics: &mut Vec<Diagnostic>) {
    let initial_diagnostic_count = diagnostics.len();
    validate_text(&root.name, &format!("{path}.name"), diagnostics);
    validate_text(&root.system, &format!("{path}.system"), diagnostics);
    validate_selector(&root.package_selector, &format!("{path}.package_selector"), diagnostics);
    validate_digest(&root.package_identity_blake3, &format!("{path}.package_identity_blake3"), diagnostics);
    validate_selector(&root.validation_selector, &format!("{path}.validation_selector"), diagnostics);
    validate_digest(
        &root.validation_package_identity_blake3,
        &format!("{path}.validation_package_identity_blake3"),
        diagnostics,
    );
    validate_artifacts(&root.sources, &format!("{path}.sources"), diagnostics);
    validate_artifacts(&root.tools, &format!("{path}.tools"), diagnostics);
    validate_artifacts(&root.dependencies, &format!("{path}.dependencies"), diagnostics);
    validate_digest(&root.policy_digest_blake3, &format!("{path}.policy_digest_blake3"), diagnostics);
    if root.expected_outcome != EXPECTED_VALIDATION_PASS {
        diagnostics.push(Diagnostic::new(
            "unsupported-validation-expectation",
            &format!("{path}.expected_outcome"),
            "the validation root expectation is not supported",
        ));
    }
    validate_validation_limits(&root.limits, &format!("{path}.limits"), diagnostics);
    validate_count(
        root.sources.len(),
        root.limits.max_sources,
        MAX_DOMAIN_ARTIFACTS,
        &format!("{path}.sources"),
        diagnostics,
    );
    validate_count(
        root.tools.len(),
        root.limits.max_tools,
        MAX_DOMAIN_ARTIFACTS,
        &format!("{path}.tools"),
        diagnostics,
    );
    validate_count(
        root.dependencies.len(),
        root.limits.max_dependencies,
        MAX_DOMAIN_ARTIFACTS,
        &format!("{path}.dependencies"),
        diagnostics,
    );
    match validation_root_identity_blake3(root) {
        Ok(identity) if identity == root.validation_root_identity_blake3 => {}
        Ok(_) => diagnostics.push(Diagnostic::new(
            "stale-validation-root-identity",
            &format!("{path}.validation_root_identity_blake3"),
            "the validation root identity differs from its canonical fields",
        )),
        Err(failure) => diagnostics.extend(failure.diagnostics),
    }
    debug_assert!(diagnostics.len() >= initial_diagnostic_count);
    debug_assert!(diagnostics.capacity() >= diagnostics.len());
}

fn validate_validation_package_binding(
    root: &ValidationRoot,
    public_selectors: &BTreeMap<(String, String), PublicPackageRecord>,
    path: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let initial_diagnostic_count = diagnostics.len();
    let package_key = (root.system.clone(), root.package_selector.clone());
    match public_selectors.get(&package_key) {
        Some(package) if package.package_identity_blake3 == root.package_identity_blake3 && package.buildable => {}
        Some(package) if !package.buildable => diagnostics.push(Diagnostic::new(
            "validation-package-blocked",
            &format!("{path}.package_selector"),
            "the package under validation is not buildable",
        )),
        Some(_) => diagnostics.push(Diagnostic::new(
            "validation-package-stale",
            &format!("{path}.package_identity_blake3"),
            "the validation root package identity differs from the composed package",
        )),
        None => diagnostics.push(Diagnostic::new(
            "validation-package-missing",
            &format!("{path}.package_selector"),
            "the validation root package is absent from the composed generation",
        )),
    }
    let validation_key = (root.system.clone(), root.validation_selector.clone());
    match public_selectors.get(&validation_key) {
        Some(package)
            if package.package_identity_blake3 == root.validation_package_identity_blake3 && package.buildable => {}
        Some(package) if !package.buildable => diagnostics.push(Diagnostic::new(
            "validation-root-package-blocked",
            &format!("{path}.validation_selector"),
            "the executable validation package is not buildable",
        )),
        Some(_) => diagnostics.push(Diagnostic::new(
            "validation-root-package-stale",
            &format!("{path}.validation_package_identity_blake3"),
            "the executable validation package identity differs from the composed package",
        )),
        None => diagnostics.push(Diagnostic::new(
            "validation-root-package-missing",
            &format!("{path}.validation_selector"),
            "the executable validation package is absent from the composed generation",
        )),
    }
    debug_assert!(diagnostics.len() >= initial_diagnostic_count);
    debug_assert!(diagnostics.capacity() >= diagnostics.len());
}

fn validate_corpus_artifacts(
    evidence: &ExternalCorpusEvidence,
    observations: &[CorpusArtifactObservation],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let required_roles = BTreeSet::from([
        CORPUS_ROLE_GRAPH,
        CORPUS_ROLE_SOURCES,
        CORPUS_ROLE_CATALOG,
        CORPUS_ROLE_BLOCKERS,
        CORPUS_ROLE_PRODUCER,
    ]);
    let mut roles = BTreeSet::new();
    let mut paths = BTreeSet::new();
    let observed = observations
        .iter()
        .map(|observation| (observation.path.as_str(), observation))
        .collect::<BTreeMap<_, _>>();
    for (index, artifact) in evidence.artifacts.iter().enumerate() {
        let path = format!("evidence.artifacts[{index}]");
        if !roles.insert(artifact.role.as_str()) {
            diagnostics.push(Diagnostic::new(
                "duplicate-corpus-artifact-role",
                &format!("{path}.role"),
                "the corpus evidence repeats an artifact role",
            ));
        }
        if !paths.insert(artifact.path.as_str()) {
            diagnostics.push(Diagnostic::new(
                "duplicate-corpus-artifact-path",
                &format!("{path}.path"),
                "the corpus evidence repeats an artifact path",
            ));
        }
        validate_relative_path(&artifact.path, &format!("{path}.path"), diagnostics);
        validate_digest(&artifact.digest_blake3, &format!("{path}.digest_blake3"), diagnostics);
        match observed.get(artifact.path.as_str()) {
            Some(observation)
                if observation.digest_blake3 == artifact.digest_blake3 && observation.bytes == artifact.bytes => {}
            Some(_) => diagnostics.push(Diagnostic::new(
                "corpus-artifact-mismatch",
                &artifact.path,
                "the observed artifact digest or byte count differs from the evidence",
            )),
            None => diagnostics.push(Diagnostic::new(
                "corpus-artifact-missing",
                &artifact.path,
                "a bound corpus artifact is missing",
            )),
        }
    }
    validate_corpus_package_selection(evidence, observations, diagnostics);
    if roles != required_roles || evidence.artifacts.len() != REQUIRED_CORPUS_ROLE_COUNT {
        diagnostics.push(Diagnostic::new(
            "corpus-artifact-role-set-mismatch",
            "evidence.artifacts",
            "the corpus evidence does not contain the exact required artifact roles",
        ));
    }
    if observations.len() != evidence.artifacts.len() {
        diagnostics.push(Diagnostic::new(
            "corpus-artifact-observation-set-mismatch",
            "observations",
            "the observed corpus artifact set differs from the evidence set",
        ));
    }
    debug_assert!(roles.len() <= evidence.artifacts.len());
    debug_assert!(paths.len() <= evidence.artifacts.len());
}

fn validate_corpus_package_selection(
    evidence: &ExternalCorpusEvidence,
    observations: &[CorpusArtifactObservation],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let catalog_path = evidence
        .artifacts
        .iter()
        .find(|artifact| artifact.role == CORPUS_ROLE_CATALOG)
        .map(|artifact| artifact.path.as_str());
    let mut observed_packages = catalog_path
        .and_then(|path| observations.iter().find(|observation| observation.path == path))
        .map(|observation| observation.catalog_packages.clone())
        .unwrap_or_default();
    let mut selected_packages = evidence.selected_packages.clone();
    observed_packages.sort();
    selected_packages.sort();
    if observed_packages != selected_packages {
        diagnostics.push(Diagnostic::new(
            "corpus-package-selection-mismatch",
            "evidence.selected_packages",
            "the selected package set differs from the bound catalog artifact",
        ));
    }
    debug_assert!(observed_packages.windows(2).all(|pair| pair[0] <= pair[1]));
    debug_assert!(selected_packages.windows(2).all(|pair| pair[0] <= pair[1]));
}

fn adapt_v1_package(catalog: &MantlepkgsCatalog, package: &PlannedPackage) -> Result<DomainPackage, CoreFailure> {
    let is_buildable = matches!(package.disposition, PackageDisposition::Buildable { .. });
    let root_identity = digest_serializable(
        DOMAIN_PACKAGE_IDENTITY_DOMAIN,
        &("v1-root", &catalog.catalog_identity_blake3, package),
        "v1-package-root-identity-serialization-failed",
    )?;
    let artifacts = catalog
        .artifacts
        .iter()
        .map(|artifact| DomainArtifact {
            role: artifact.role.clone(),
            digest_blake3: artifact.digest_blake3.clone(),
        })
        .collect::<Vec<_>>();
    let mut adapted = DomainPackage {
        package_identity_blake3: empty_digest(),
        name: package.name.clone(),
        public_selector: package.name.clone(),
        system: package.system.clone(),
        root_identity_blake3: root_identity,
        policy_digest_blake3: catalog.manifest_digest_blake3.clone(),
        buildable: is_buildable,
        aliases: package.aliases.clone(),
        artifacts,
    };
    adapted.aliases.sort();
    adapted.aliases.dedup();
    adapted.artifacts.sort();
    adapted.artifacts.dedup();
    adapted.package_identity_blake3 = domain_package_identity_blake3(&adapted)?;
    debug_assert!(is_lower_hex_length(&adapted.package_identity_blake3, BLAKE3_HEX_LENGTH));
    debug_assert!(adapted.aliases.len() <= package.aliases.len());
    Ok(adapted)
}

fn register_public_selector(
    selectors: &mut BTreeMap<(String, String), PublicPackageRecord>,
    record: &PublicPackageRecord,
    selector: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let key = (record.system.clone(), selector.into());
    if let Some(previous) = selectors.insert(key, record.clone()) {
        let is_same_identity = previous.package_identity_blake3 == record.package_identity_blake3
            && previous.root_identity_blake3 == record.root_identity_blake3;
        let code = if is_same_identity {
            "duplicate-public-selector"
        } else {
            "conflicting-public-selector"
        };
        diagnostics.push(Diagnostic::new(
            code,
            selector,
            "more than one package or variant claims the public selector",
        ));
    }
}

fn has_variant_cycle(start: &PackageVariant, variants: &BTreeMap<(String, String), &PackageVariant>) -> bool {
    let mut seen = BTreeSet::new();
    let mut current = start;
    for _step in 0..=variants.len() {
        let key = (current.system.clone(), current.public_selector.clone());
        if !seen.insert(key) {
            return true;
        }
        let base_key = (current.system.clone(), current.base_selector.clone());
        match variants.get(&base_key) {
            Some(next) => current = next,
            None => return false,
        }
    }
    debug_assert!(seen.len() <= variants.len());
    true
}

#[allow(tigerstyle::ambiguous_params)] // System and selector are named parts of one catalog lookup key.
fn lookup_public_package<'a>(
    catalog: &'a DomainCatalog,
    system: &str,
    selector: &str,
) -> Result<&'a PublicPackageRecord, CoreFailure> {
    catalog
        .packages
        .iter()
        .find(|package| package.system == system && package.public_selector == selector)
        .ok_or_else(|| {
            CoreFailure::from_diagnostic(Diagnostic::new(
                "public-package-not-found",
                selector,
                "the public package selector is absent from the domain catalog",
            ))
        })
}

fn inferred_catalog_limits(catalog: &DomainCatalog) -> DomainCatalogLimits {
    let alias_count = catalog
        .shards
        .iter()
        .flat_map(|shard| shard.packages.iter())
        .fold(0usize, |count, package| count.saturating_add(package.aliases.len()));
    let package_artifact_count = catalog
        .shards
        .iter()
        .flat_map(|shard| shard.packages.iter())
        .fold(0usize, |count, package| count.saturating_add(package.artifacts.len()));
    let validation_artifact_count = catalog.validation_roots.iter().fold(0usize, |count, root| {
        count
            .saturating_add(root.sources.len())
            .saturating_add(root.tools.len())
            .saturating_add(root.dependencies.len())
    });
    let artifact_count = package_artifact_count.saturating_add(validation_artifact_count);
    #[allow(tigerstyle::numeric_units)] // This value is a typed set of limits, not one numeric quantity.
    let limits = DomainCatalogLimits {
        max_shards: bounded_inferred_limit(catalog.shards.len(), MAX_DOMAIN_SHARDS),
        max_packages: bounded_inferred_limit(catalog.packages.len(), MAX_DOMAIN_PACKAGES),
        max_aliases: bounded_inferred_limit(alias_count, MAX_DOMAIN_ALIASES),
        max_variants: bounded_inferred_limit(catalog.variants.len(), MAX_DOMAIN_VARIANTS),
        max_validation_roots: bounded_inferred_limit(catalog.validation_roots.len(), MAX_DOMAIN_VALIDATION_ROOTS),
        max_artifacts: bounded_inferred_limit(artifact_count, MAX_DOMAIN_ARTIFACTS),
    };
    debug_assert!(limits.max_shards >= MIN_ITEMS && limits.max_shards <= MAX_DOMAIN_SHARDS);
    debug_assert!(limits.max_artifacts >= MIN_ITEMS && limits.max_artifacts <= MAX_DOMAIN_ARTIFACTS);
    limits
}

fn bounded_inferred_limit(observed: usize, maximum: u32) -> u32 {
    u32::try_from(observed).unwrap_or(maximum).clamp(MIN_ITEMS, maximum)
}

fn validate_catalog_limits(limits: &DomainCatalogLimits, diagnostics: &mut Vec<Diagnostic>) {
    validate_named_limit(limits.max_shards, MAX_DOMAIN_SHARDS, "manifest.limits.max_shards", diagnostics);
    validate_named_limit(limits.max_packages, MAX_DOMAIN_PACKAGES, "manifest.limits.max_packages", diagnostics);
    validate_named_limit(limits.max_aliases, MAX_DOMAIN_ALIASES, "manifest.limits.max_aliases", diagnostics);
    validate_named_limit(limits.max_variants, MAX_DOMAIN_VARIANTS, "manifest.limits.max_variants", diagnostics);
    validate_named_limit(
        limits.max_validation_roots,
        MAX_DOMAIN_VALIDATION_ROOTS,
        "manifest.limits.max_validation_roots",
        diagnostics,
    );
    validate_named_limit(limits.max_artifacts, MAX_DOMAIN_ARTIFACTS, "manifest.limits.max_artifacts", diagnostics);
}

fn validate_shard_limits(limits: &DomainShardLimits, path: &str, diagnostics: &mut Vec<Diagnostic>) {
    validate_named_limit(limits.max_packages, MAX_DOMAIN_PACKAGES, &format!("{path}.max_packages"), diagnostics);
    validate_named_limit(limits.max_aliases, MAX_DOMAIN_ALIASES, &format!("{path}.max_aliases"), diagnostics);
    validate_named_limit(limits.max_artifacts, MAX_DOMAIN_ARTIFACTS, &format!("{path}.max_artifacts"), diagnostics);
}

fn validate_validation_limits(limits: &ValidationLimits, path: &str, diagnostics: &mut Vec<Diagnostic>) {
    let initial_diagnostic_count = diagnostics.len();
    validate_named_limit(limits.max_sources, MAX_DOMAIN_ARTIFACTS, &format!("{path}.max_sources"), diagnostics);
    validate_named_limit(limits.max_tools, MAX_DOMAIN_ARTIFACTS, &format!("{path}.max_tools"), diagnostics);
    validate_named_limit(
        limits.max_dependencies,
        MAX_DOMAIN_ARTIFACTS,
        &format!("{path}.max_dependencies"),
        diagnostics,
    );
    if limits.max_output_bytes == 0 {
        diagnostics.push(Diagnostic::new(
            "invalid-named-limit",
            &format!("{path}.max_output_bytes"),
            "the named limit must be greater than zero",
        ));
    }
    if limits.timeout_seconds == 0 {
        diagnostics.push(Diagnostic::new(
            "invalid-named-limit",
            &format!("{path}.timeout_seconds"),
            "the named limit must be greater than zero",
        ));
    }
    debug_assert!(diagnostics.len() >= initial_diagnostic_count);
    debug_assert!(diagnostics.capacity() >= diagnostics.len());
}

#[allow(tigerstyle::ambiguous_params)] // Internal validator names distinguish the field value from its diagnostic path.
fn validate_domain_class(class: &str, path: &str, diagnostics: &mut Vec<Diagnostic>) {
    if class != DOMAIN_CLASS_CORE && class != DOMAIN_CLASS_ECOSYSTEM {
        diagnostics.push(Diagnostic::new(
            "unknown-domain-class",
            path,
            "the domain class is neither core nor ecosystem",
        ));
    }
}

fn validate_source_lock(source: &DomainSourceLock, path: &str, diagnostics: &mut Vec<Diagnostic>) {
    validate_text(&source.repository, &format!("{path}.repository"), diagnostics);
    validate_text(&source.reference, &format!("{path}.reference"), diagnostics);
    validate_revision(&source.revision, &format!("{path}.revision"), diagnostics);
    validate_digest(&source.source_digest_blake3, &format!("{path}.source_digest_blake3"), diagnostics);
    if !source.reference.contains(&source.revision) {
        diagnostics.push(Diagnostic::new(
            "floating-domain-source",
            &format!("{path}.reference"),
            "the source reference does not contain the exact revision",
        ));
    }
}

#[allow(tigerstyle::ambiguous_params)] // Internal validator names distinguish the field value from its diagnostic path.
fn validate_revision(value: &str, path: &str, diagnostics: &mut Vec<Diagnostic>) {
    let is_git = is_lower_hex_length(value, GIT_REVISION_HEX_LENGTH);
    let is_blake3 = is_lower_hex_length(value, BLAKE3_HEX_LENGTH);
    if !is_git && !is_blake3 {
        diagnostics.push(Diagnostic::new(
            "invalid-source-revision",
            path,
            "the source revision must be one exact lowercase hexadecimal identity",
        ));
    }
}

fn validate_artifacts(artifacts: &[DomainArtifact], path: &str, diagnostics: &mut Vec<Diagnostic>) {
    let mut roles = BTreeSet::new();
    for (index, artifact) in artifacts.iter().enumerate() {
        validate_text(&artifact.role, &format!("{path}[{index}].role"), diagnostics);
        validate_digest(&artifact.digest_blake3, &format!("{path}[{index}].digest_blake3"), diagnostics);
        if !roles.insert(&artifact.role) {
            diagnostics.push(Diagnostic::new(
                "duplicate-artifact-role",
                &format!("{path}[{index}].role"),
                "the artifact role is repeated",
            ));
        }
    }
}

fn normalize_artifacts(artifacts: &mut Vec<DomainArtifact>) {
    artifacts.sort();
    artifacts.dedup();
}

fn validate_string_set(values: &[String], path: &str, diagnostics: &mut Vec<Diagnostic>) {
    if values.is_empty() {
        diagnostics.push(Diagnostic::new("empty-bounded-set", path, "the bounded set must contain at least one value"));
    }
    validate_optional_string_set(values, path, diagnostics);
}

fn validate_optional_string_set(values: &[String], path: &str, diagnostics: &mut Vec<Diagnostic>) {
    let mut seen = BTreeSet::new();
    for (index, value) in values.iter().enumerate() {
        validate_text(value, &format!("{path}[{index}]"), diagnostics);
        if !seen.insert(value) {
            diagnostics.push(Diagnostic::new(
                "duplicate-bounded-value",
                &format!("{path}[{index}]"),
                "the bounded set repeats a value",
            ));
        }
    }
}

#[allow(tigerstyle::ambiguous_params)] // Internal validator names distinguish the field value from its diagnostic path.
fn validate_selector(value: &str, path: &str, diagnostics: &mut Vec<Diagnostic>) {
    validate_text(value, path, diagnostics);
    let is_invalid = value.starts_with('.')
        || value.ends_with('.')
        || value.contains("..")
        || value.contains('/')
        || value.contains('\\');
    if is_invalid {
        diagnostics.push(Diagnostic::new(
            "unsafe-public-selector",
            path,
            "the public selector is not normalized and confined",
        ));
    }
}

#[allow(tigerstyle::ambiguous_params)] // Internal validator names distinguish the field value from its diagnostic path.
fn validate_relative_path(value: &str, path: &str, diagnostics: &mut Vec<Diagnostic>) {
    validate_text(value, path, diagnostics);
    let is_invalid = value.starts_with('/')
        || value.ends_with('/')
        || value.contains('\\')
        || value.split('/').any(|component| component.is_empty() || component == "." || component == "..");
    if is_invalid {
        diagnostics.push(Diagnostic::new(
            "unsafe-relative-path",
            path,
            "the path must be normalized, relative, and confined",
        ));
    }
}

#[allow(tigerstyle::ambiguous_params)] // Internal validator names distinguish the field value from its diagnostic path.
fn validate_text(value: &str, path: &str, diagnostics: &mut Vec<Diagnostic>) {
    if value.is_empty() || value.len() > MAX_TEXT_BYTES || value.chars().any(char::is_control) {
        diagnostics.push(Diagnostic::new(
            "invalid-text-field",
            path,
            "the text field is empty, too large, or contains a control character",
        ));
    }
}

#[allow(tigerstyle::ambiguous_params)] // Internal validator names distinguish the field value from its diagnostic path.
fn validate_digest(value: &str, path: &str, diagnostics: &mut Vec<Diagnostic>) {
    if !is_lower_hex_length(value, BLAKE3_HEX_LENGTH) {
        diagnostics.push(Diagnostic::new(
            "invalid-blake3-digest",
            path,
            "the digest must be one lowercase BLAKE3 hexadecimal value",
        ));
    }
}

#[allow(tigerstyle::ambiguous_params)] // Configured value and implementation maximum are distinct named bounds.
fn validate_named_limit(value: u32, maximum: u32, path: &str, diagnostics: &mut Vec<Diagnostic>) {
    if value < MIN_ITEMS || value > maximum {
        diagnostics.push(Diagnostic::new(
            "invalid-named-limit",
            path,
            "the named limit is zero or exceeds the implementation limit",
        ));
    }
}

#[allow(tigerstyle::ambiguous_params)] // Policy and implementation limits are distinct bounded counters.
fn validate_count(
    observed: usize,
    policy_limit: u32,
    implementation_limit: u32,
    path: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let initial_diagnostic_count = diagnostics.len();
    let observed_count = match u32::try_from(observed) {
        Ok(observed_count) => observed_count,
        Err(_) => {
            diagnostics.push(Diagnostic::new(
                "domain-count-overflow",
                path,
                "the observed item count does not fit the bounded counter",
            ));
            return;
        }
    };
    if observed_count > policy_limit || observed_count > implementation_limit {
        diagnostics.push(Diagnostic::new(
            "domain-count-limit-exceeded",
            path,
            "the observed item count exceeds a named limit",
        ));
    }
    debug_assert!(diagnostics.len() >= initial_diagnostic_count);
    debug_assert!(diagnostics.capacity() >= diagnostics.len());
}

#[allow(tigerstyle::ambiguous_params)] // Internal validator names distinguish the field value from its diagnostic path.
fn validate_validation_outcome(value: &str, path: &str) -> Result<(), CoreFailure> {
    let valid = [
        OBSERVED_VALIDATION_PASS,
        OBSERVED_VALIDATION_FAIL,
        OBSERVED_VALIDATION_TIMEOUT,
        OBSERVED_VALIDATION_MALFORMED,
    ];
    if valid.contains(&value) {
        Ok(())
    } else {
        Err(CoreFailure::from_diagnostic(Diagnostic::new(
            "unknown-validation-outcome",
            path,
            "the validation outcome is not supported",
        )))
    }
}

#[allow(tigerstyle::ambiguous_params)] // Contract value, expected value, path, and code have explicit semantic names.
fn require_equal(value: &str, expected: &str, path: &str, code: &str) -> Result<(), CoreFailure> {
    if value == expected {
        Ok(())
    } else {
        Err(CoreFailure::from_diagnostic(Diagnostic::new(
            code,
            path,
            "the value does not match the supported contract",
        )))
    }
}

#[allow(tigerstyle::ambiguous_params)] // Contract value, expected value, path, and code have explicit semantic names.
fn require_equal_diagnostic(value: &str, expected: &str, path: &str, code: &str, diagnostics: &mut Vec<Diagnostic>) {
    if value != expected {
        diagnostics.push(Diagnostic::new(code, path, "the value does not match the supported contract"));
    }
}

#[allow(tigerstyle::ambiguous_params)] // Digest value and diagnostic path are separate validated domains.
fn require_digest(value: &str, path: &str) -> Result<(), CoreFailure> {
    if is_lower_hex_length(value, BLAKE3_HEX_LENGTH) {
        Ok(())
    } else {
        Err(CoreFailure::from_diagnostic(Diagnostic::new(
            "invalid-blake3-digest",
            path,
            "the digest must be one lowercase BLAKE3 hexadecimal value",
        )))
    }
}

fn digest_serializable<T: Serialize + ?Sized>(domain: &[u8], value: &T, code: &str) -> Result<String, CoreFailure> {
    let bytes = serde_json::to_vec(value).map_err(|_| {
        CoreFailure::from_diagnostic(Diagnostic::new(code, "canonical", "the canonical value did not serialize"))
    })?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(&[DOMAIN_SEPARATOR]);
    hasher.update(&bytes);
    Ok(hasher.finalize().to_hex().as_str().into())
}

fn is_lower_hex_length(value: &str, length: usize) -> bool {
    value.len() == length && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn empty_digest() -> String {
    "0".repeat(BLAKE3_HEX_LENGTH)
}

fn domain_non_claims() -> Vec<String> {
    vec![
        "domain-class-is-not-package-trust".into(),
        "composition-is-not-package-correctness".into(),
        "validation-is-not-reproducibility".into(),
        "external-corpus-is-not-broad-compatibility".into(),
        "catalog-evidence-is-not-release-eligibility".into(),
    ]
}

fn validation_non_claims() -> Vec<String> {
    vec![
        "validation-success-is-not-package-correctness".into(),
        "validation-failure-does-not-change-package-bytes".into(),
        "validation-receipt-is-not-release-eligibility".into(),
        "realization-receipt-is-not-reproducibility".into(),
        "validation-policy-is-bounded-to-recorded-inputs".into(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const DIGEST_C: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
    const REVISION: &str = "a9a1af8abbf08b972dbce7bb9c2643c7d76d140d";
    const SYSTEM: &str = "x86_64-linux";
    const PACKAGE_LIMIT: u32 = 16;
    const ALIAS_LIMIT: u32 = 32;
    const ARTIFACT_LIMIT: u32 = 64;
    const VARIANT_LIMIT: u32 = 16;
    const VALIDATION_ROOT_LIMIT: u32 = 16;
    const OUTPUT_BYTE_LIMIT: u64 = 1_048_576;
    const TIMEOUT_SECONDS: u64 = 60;

    fn artifact(role: &str, digest: &str) -> DomainArtifact {
        DomainArtifact {
            role: role.into(),
            digest_blake3: digest.into(),
        }
    }

    fn package(name: &str, selector: &str, root: &str) -> DomainPackage {
        let mut package = DomainPackage {
            package_identity_blake3: empty_digest(),
            name: name.into(),
            public_selector: selector.into(),
            system: SYSTEM.into(),
            root_identity_blake3: root.into(),
            policy_digest_blake3: DIGEST_A.into(),
            buildable: true,
            aliases: vec![format!("{selector}-alias")],
            artifacts: vec![artifact("catalog", DIGEST_B)],
        };
        package.package_identity_blake3 = domain_package_identity_blake3(&package).expect("package identity");
        package
    }

    fn shard(name: &str, class: &str, packages: Vec<DomainPackage>) -> DomainShard {
        let mut shard = DomainShard {
            schema: DOMAIN_SHARD_SCHEMA.into(),
            shard_identity_blake3: empty_digest(),
            name: name.into(),
            class: class.into(),
            owner_label: format!("owner-{name}"),
            source: DomainSourceLock {
                repository: "https://github.com/ekala-project/corepkgs".into(),
                reference: format!("github:ekala-project/corepkgs/{REVISION}"),
                revision: REVISION.into(),
                source_digest_blake3: DIGEST_C.into(),
            },
            source_catalog_identity_blake3: DIGEST_A.into(),
            packages,
            limits: DomainShardLimits {
                max_packages: PACKAGE_LIMIT,
                max_aliases: ALIAS_LIMIT,
                max_artifacts: ARTIFACT_LIMIT,
            },
        };
        shard.packages.sort();
        shard.shard_identity_blake3 = domain_shard_identity_blake3(&shard).expect("shard identity");
        shard
    }

    fn variant(base: &DomainPackage) -> PackageVariant {
        let mut variant = PackageVariant {
            variant_identity_blake3: empty_digest(),
            public_selector: "hello-debug".into(),
            system: SYSTEM.into(),
            base_selector: base.public_selector.clone(),
            base_package_identity_blake3: base.package_identity_blake3.clone(),
            base_root_identity_blake3: base.root_identity_blake3.clone(),
            variant_name: "debug".into(),
            changed_policy_digest_blake3: DIGEST_B.into(),
            root_identity_blake3: DIGEST_C.into(),
            provenance_digest_blake3: DIGEST_A.into(),
            aliases: vec!["hello-with-debug".into()],
        };
        variant.variant_identity_blake3 = package_variant_identity_blake3(&variant).expect("variant identity");
        variant
    }

    fn validation_root(base: &DomainPackage, validation: &DomainPackage) -> ValidationRoot {
        let mut root = ValidationRoot {
            validation_root_identity_blake3: empty_digest(),
            name: "hello-check".into(),
            system: SYSTEM.into(),
            package_selector: base.public_selector.clone(),
            package_identity_blake3: base.package_identity_blake3.clone(),
            validation_selector: validation.public_selector.clone(),
            validation_package_identity_blake3: validation.package_identity_blake3.clone(),
            sources: vec![artifact("test-source", DIGEST_A)],
            tools: vec![artifact("test-tool", DIGEST_B)],
            dependencies: vec![artifact("test-dependency", DIGEST_C)],
            policy_digest_blake3: DIGEST_A.into(),
            expected_outcome: EXPECTED_VALIDATION_PASS.into(),
            limits: ValidationLimits {
                max_sources: PACKAGE_LIMIT,
                max_tools: PACKAGE_LIMIT,
                max_dependencies: PACKAGE_LIMIT,
                max_output_bytes: OUTPUT_BYTE_LIMIT,
                timeout_seconds: TIMEOUT_SECONDS,
            },
        };
        root.validation_root_identity_blake3 = validation_root_identity_blake3(&root).expect("validation identity");
        root
    }

    fn manifest() -> DomainCatalogManifest {
        let base = package("hello", "hello", DIGEST_A);
        let validation = package("hello-check", "hello-check", DIGEST_B);
        DomainCatalogManifest {
            schema: DOMAIN_MANIFEST_SCHEMA.into(),
            name: "default".into(),
            shards: vec![shard("core", DOMAIN_CLASS_CORE, vec![base.clone(), validation.clone()])],
            variants: vec![variant(&base)],
            validation_roots: vec![validation_root(&base, &validation)],
            limits: DomainCatalogLimits {
                max_shards: PACKAGE_LIMIT,
                max_packages: PACKAGE_LIMIT,
                max_aliases: ALIAS_LIMIT,
                max_variants: VARIANT_LIMIT,
                max_validation_roots: VALIDATION_ROOT_LIMIT,
                max_artifacts: ARTIFACT_LIMIT,
            },
        }
    }

    #[test]
    fn equivalent_domain_inputs_compose_to_one_identity() {
        let original = manifest();
        let mut reordered = original.clone();
        reordered.shards[0].packages.reverse();
        reordered.shards[0].shard_identity_blake3 =
            domain_shard_identity_blake3(&reordered.shards[0]).expect("identity for reordered canonical package set");

        let left = compose_domain_catalog(&original).expect("original catalog");
        let right = compose_domain_catalog(&reordered).expect("reordered catalog");

        assert_eq!(left.catalog_identity_blake3, right.catalog_identity_blake3);
        assert_eq!(left.packages, right.packages);
    }

    #[test]
    fn zero_identity_placeholders_seal_before_composition() {
        let mut input = manifest();
        input.shards[0].packages[0].package_identity_blake3 = empty_digest();
        input.shards[0].shard_identity_blake3 = empty_digest();
        input.variants[0].variant_identity_blake3 = empty_digest();
        input.variants[0].base_package_identity_blake3 = empty_digest();
        input.variants[0].base_root_identity_blake3 = empty_digest();
        input.validation_roots[0].validation_root_identity_blake3 = empty_digest();
        input.validation_roots[0].package_identity_blake3 = empty_digest();
        input.validation_roots[0].validation_package_identity_blake3 = empty_digest();

        let sealed = seal_domain_manifest(&input).expect("placeholder identities must seal");
        let catalog = compose_domain_catalog(&sealed).expect("sealed manifest must compose");

        assert_ne!(sealed.shards[0].shard_identity_blake3, empty_digest());
        assert_ne!(catalog.catalog_identity_blake3, empty_digest());
    }

    #[test]
    fn non_placeholder_identity_mismatch_is_rejected() {
        let mut input = manifest();
        input.shards[0].packages[0].name = "changed".into();

        let failure = seal_domain_manifest(&input).expect_err("stale declared identity must fail");

        assert!(failure.diagnostics.iter().any(|item| item.code == "declared-identity-mismatch"));
    }

    #[test]
    fn unknown_class_and_missing_owner_are_rejected() {
        let mut input = manifest();
        input.shards[0].class = "trusted".into();
        input.shards[0].owner_label.clear();
        input.shards[0].shard_identity_blake3 =
            domain_shard_identity_blake3(&input.shards[0]).expect("changed identity");

        let failure = compose_domain_catalog(&input).expect_err("invalid shard metadata must fail");

        assert!(failure.diagnostics.iter().any(|item| item.code == "unknown-domain-class"));
        assert!(failure.diagnostics.iter().any(|item| item.path.ends_with("owner_label")));
    }

    #[test]
    fn named_domain_limits_fail_closed() {
        let mut input = manifest();
        input.limits.max_packages = MIN_ITEMS;

        let failure = compose_domain_catalog(&input).expect_err("package count above policy must fail");

        assert!(failure.diagnostics.iter().any(|item| item.code == "domain-count-limit-exceeded"));
    }

    #[test]
    fn variant_cycles_are_reported() {
        let mut input = manifest();
        let first = input.variants[0].clone();
        let mut second = first.clone();
        input.variants[0].public_selector = "cycle-a".into();
        input.variants[0].base_selector = "cycle-b".into();
        input.variants[0].variant_name = "cycle-a".into();
        input.variants[0].variant_identity_blake3 =
            package_variant_identity_blake3(&input.variants[0]).expect("first cycle identity");
        second.public_selector = "cycle-b".into();
        second.base_selector = "cycle-a".into();
        second.variant_name = "cycle-b".into();
        second.variant_identity_blake3 = package_variant_identity_blake3(&second).expect("second cycle identity");
        input.variants.push(second);

        let failure = compose_domain_catalog(&input).expect_err("variant cycle must fail");

        assert!(failure.diagnostics.iter().any(|item| item.code == "variant-cycle"));
    }

    #[test]
    fn public_selector_conflicts_never_choose_a_winner() {
        let mut input = manifest();
        let conflicting = package("other", "hello", DIGEST_C);
        input.shards.push(shard("ecosystem", DOMAIN_CLASS_ECOSYSTEM, vec![conflicting]));

        let failure = compose_domain_catalog(&input).expect_err("conflicting selector must fail");

        assert!(failure.diagnostics.iter().any(|item| item.code == "conflicting-public-selector"));
    }

    #[test]
    fn stale_variant_base_is_rejected() {
        let mut input = manifest();
        input.variants[0].base_root_identity_blake3 = DIGEST_B.into();
        input.variants[0].variant_identity_blake3 =
            package_variant_identity_blake3(&input.variants[0]).expect("updated variant identity");

        let failure = compose_domain_catalog(&input).expect_err("stale variant base must fail");

        assert!(failure.diagnostics.iter().any(|item| item.code == "variant-base-stale"));
    }

    #[test]
    fn missing_variant_base_is_rejected() {
        let mut input = manifest();
        input.variants[0].base_selector = "absent".into();
        input.variants[0].variant_identity_blake3 =
            package_variant_identity_blake3(&input.variants[0]).expect("updated variant identity");

        let failure = compose_domain_catalog(&input).expect_err("missing variant base must fail");

        assert!(failure.diagnostics.iter().any(|item| item.code == "variant-base-missing"));
    }

    #[test]
    fn missing_validation_package_is_rejected() {
        let mut input = manifest();
        input.validation_roots[0].validation_selector = "absent-check".into();
        input.validation_roots[0].validation_package_identity_blake3 = DIGEST_C.into();
        input.validation_roots[0].validation_root_identity_blake3 =
            validation_root_identity_blake3(&input.validation_roots[0]).expect("updated validation identity");

        let failure = compose_domain_catalog(&input).expect_err("missing validation package must fail");

        assert!(failure.diagnostics.iter().any(|item| item.code == "validation-root-package-missing"));
    }

    #[test]
    fn test_only_change_preserves_package_identity() {
        let first = compose_domain_catalog(&manifest()).expect("first catalog");
        let first_root = first.validation_roots[0].clone();
        let first_package = first
            .packages
            .iter()
            .find(|package| package.public_selector == "hello")
            .expect("base package")
            .clone();
        let mut changed = manifest();
        changed.validation_roots[0].sources[0].digest_blake3 = DIGEST_C.into();
        changed.validation_roots[0].dependencies[0].digest_blake3 = DIGEST_A.into();
        changed.validation_roots[0].validation_root_identity_blake3 =
            validation_root_identity_blake3(&changed.validation_roots[0]).expect("changed validation identity");
        let second = compose_domain_catalog(&changed).expect("second catalog");
        let second_package =
            second.packages.iter().find(|package| package.public_selector == "hello").expect("base package");

        assert_eq!(first_package.package_identity_blake3, second_package.package_identity_blake3);
        assert_eq!(first_package.root_identity_blake3, second_package.root_identity_blake3);
        assert_ne!(
            first_root.validation_root_identity_blake3,
            second.validation_roots[0].validation_root_identity_blake3
        );
    }

    #[test]
    fn validation_plan_binds_separate_inputs_and_policy() {
        let catalog = compose_domain_catalog(&manifest()).expect("domain catalog");
        let root = &catalog.validation_roots[0];
        let plan = plan_validation_root(&catalog, &root.validation_root_identity_blake3).expect("validation plan");

        assert_eq!(plan.sources, root.sources);
        assert_eq!(plan.tools, root.tools);
        assert_eq!(plan.dependencies, root.dependencies);
        assert_eq!(plan.policy_digest_blake3, root.policy_digest_blake3);
        assert_eq!(plan.package_identity_blake3, root.package_identity_blake3);
    }

    #[test]
    fn failed_validation_keeps_package_identity() {
        let catalog = compose_domain_catalog(&manifest()).expect("domain catalog");
        let plan = plan_validation_root(&catalog, &catalog.validation_roots[0].validation_root_identity_blake3)
            .expect("validation plan");
        let observation = ValidationObservation {
            schema: VALIDATION_OBSERVATION_SCHEMA.into(),
            validation_root_identity_blake3: plan.validation_root_identity_blake3.clone(),
            outcome: OBSERVED_VALIDATION_FAIL.into(),
            realization_receipt_blake3: DIGEST_A.into(),
            observed_output_bytes: 0,
            elapsed_milliseconds: 1,
            diagnostic_digest_blake3: Some(DIGEST_B.into()),
        };

        let receipt = record_validation_observation(&plan, &observation).expect("failure receipt");

        assert!(!receipt.accepted);
        assert_eq!(receipt.package_identity_blake3, plan.package_identity_blake3);
        assert_eq!(receipt.package_root_identity_blake3, plan.package_root_identity_blake3);
    }

    #[test]
    fn blocked_package_cannot_back_validation_success() {
        let mut input = manifest();
        let base = input.shards[0]
            .packages
            .iter_mut()
            .find(|package| package.public_selector == "hello")
            .expect("base package");
        base.buildable = false;
        base.package_identity_blake3 = empty_digest();
        input.shards[0].shard_identity_blake3 = empty_digest();
        input.variants[0].variant_identity_blake3 = empty_digest();
        input.variants[0].base_package_identity_blake3 = empty_digest();
        input.variants[0].base_root_identity_blake3 = empty_digest();
        input.validation_roots[0].validation_root_identity_blake3 = empty_digest();
        input.validation_roots[0].package_identity_blake3 = empty_digest();
        input.validation_roots[0].validation_package_identity_blake3 = empty_digest();
        let sealed = seal_domain_manifest(&input).expect("blocked package facts still seal");

        let failure = compose_domain_catalog(&sealed).expect_err("blocked package validation must fail");

        assert!(failure.diagnostics.iter().any(|item| item.code == "validation-package-blocked"));
    }

    #[test]
    fn stale_validation_policy_observation_is_rejected() {
        let original_catalog = compose_domain_catalog(&manifest()).expect("original catalog");
        let original_plan = plan_validation_root(
            &original_catalog,
            &original_catalog.validation_roots[0].validation_root_identity_blake3,
        )
        .expect("original validation plan");
        let mut changed = manifest();
        changed.validation_roots[0].policy_digest_blake3 = DIGEST_B.into();
        changed.validation_roots[0].validation_root_identity_blake3 =
            validation_root_identity_blake3(&changed.validation_roots[0]).expect("updated policy identity");
        let changed_catalog = compose_domain_catalog(&changed).expect("changed catalog");
        let changed_plan = plan_validation_root(
            &changed_catalog,
            &changed_catalog.validation_roots[0].validation_root_identity_blake3,
        )
        .expect("changed validation plan");
        let stale_observation = ValidationObservation {
            schema: VALIDATION_OBSERVATION_SCHEMA.into(),
            validation_root_identity_blake3: original_plan.validation_root_identity_blake3.clone(),
            outcome: OBSERVED_VALIDATION_PASS.into(),
            realization_receipt_blake3: DIGEST_A.into(),
            observed_output_bytes: 0,
            elapsed_milliseconds: 1,
            diagnostic_digest_blake3: None,
        };

        let failure = record_validation_observation(&changed_plan, &stale_observation)
            .expect_err("stale policy observation must fail");

        assert_ne!(original_plan.policy_digest_blake3, changed_plan.policy_digest_blake3);
        assert!(failure.diagnostics.iter().any(|item| item.code == "validation-observation-plan-mismatch"));
    }

    #[test]
    fn malformed_validation_outcome_is_rejected() {
        let catalog = compose_domain_catalog(&manifest()).expect("domain catalog");
        let plan = plan_validation_root(&catalog, &catalog.validation_roots[0].validation_root_identity_blake3)
            .expect("validation plan");
        let observation = ValidationObservation {
            schema: VALIDATION_OBSERVATION_SCHEMA.into(),
            validation_root_identity_blake3: plan.validation_root_identity_blake3.clone(),
            outcome: "success".into(),
            realization_receipt_blake3: DIGEST_A.into(),
            observed_output_bytes: 0,
            elapsed_milliseconds: 1,
            diagnostic_digest_blake3: None,
        };

        let failure =
            record_validation_observation(&plan, &observation).expect_err("malformed validation result must fail");

        assert!(failure.diagnostics.iter().any(|item| item.code == "unknown-validation-outcome"));
    }

    #[test]
    fn malformed_validation_evidence_retains_package_identity() {
        let catalog = compose_domain_catalog(&manifest()).expect("domain catalog");
        let plan = plan_validation_root(&catalog, &catalog.validation_roots[0].validation_root_identity_blake3)
            .expect("validation plan");
        let observation = ValidationObservation {
            schema: VALIDATION_OBSERVATION_SCHEMA.into(),
            validation_root_identity_blake3: plan.validation_root_identity_blake3.clone(),
            outcome: OBSERVED_VALIDATION_MALFORMED.into(),
            realization_receipt_blake3: DIGEST_A.into(),
            observed_output_bytes: 0,
            elapsed_milliseconds: 1,
            diagnostic_digest_blake3: Some(DIGEST_B.into()),
        };

        let receipt = record_validation_observation(&plan, &observation)
            .expect("malformed validation evidence must remain recordable");

        assert!(!receipt.accepted);
        assert_eq!(receipt.package_identity_blake3, plan.package_identity_blake3);
        assert_eq!(receipt.package_root_identity_blake3, plan.package_root_identity_blake3);
    }

    #[test]
    fn validation_output_limit_is_enforced() {
        let catalog = compose_domain_catalog(&manifest()).expect("domain catalog");
        let plan = plan_validation_root(&catalog, &catalog.validation_roots[0].validation_root_identity_blake3)
            .expect("validation plan");
        let observation = ValidationObservation {
            schema: VALIDATION_OBSERVATION_SCHEMA.into(),
            validation_root_identity_blake3: plan.validation_root_identity_blake3.clone(),
            outcome: OBSERVED_VALIDATION_PASS.into(),
            realization_receipt_blake3: DIGEST_A.into(),
            observed_output_bytes: plan.max_output_bytes.saturating_add(1),
            elapsed_milliseconds: 1,
            diagnostic_digest_blake3: None,
        };

        let failure = record_validation_observation(&plan, &observation)
            .expect_err("validation output above the named limit must fail");

        assert!(failure.diagnostics.iter().any(|item| item.code == "validation-output-limit-exceeded"));
    }

    #[test]
    fn elapsed_validation_cannot_hide_timeout() {
        let catalog = compose_domain_catalog(&manifest()).expect("domain catalog");
        let plan = plan_validation_root(&catalog, &catalog.validation_roots[0].validation_root_identity_blake3)
            .expect("validation plan");
        let observation = ValidationObservation {
            schema: VALIDATION_OBSERVATION_SCHEMA.into(),
            validation_root_identity_blake3: plan.validation_root_identity_blake3.clone(),
            outcome: OBSERVED_VALIDATION_PASS.into(),
            realization_receipt_blake3: DIGEST_A.into(),
            observed_output_bytes: 0,
            elapsed_milliseconds: plan.timeout_seconds.saturating_mul(MILLISECONDS_PER_SECOND).saturating_add(1),
            diagnostic_digest_blake3: None,
        };

        let failure =
            record_validation_observation(&plan, &observation).expect_err("late validation must retain timeout status");

        assert!(failure.diagnostics.iter().any(|item| item.code == "validation-timeout-misclassified"));
    }

    #[test]
    fn corpus_revision_license_and_source_identity_fail_closed() {
        let valid = corpus_evidence();
        let observations = corpus_observations(&valid);
        let mut invalid = valid.clone();
        invalid.revision = "master".into();
        invalid.observed_license.clear();
        invalid.source_digest_blake3 = "wrong".into();
        invalid.evidence_identity_blake3 =
            external_corpus_evidence_identity_blake3(&invalid).expect("invalid facts still serialize");

        let failure =
            validate_external_corpus_evidence(&invalid, &observations).expect_err("invalid provenance must fail");

        assert!(failure.diagnostics.iter().any(|item| item.code == "invalid-source-revision"));
        assert!(failure.diagnostics.iter().any(|item| item.code == "unsupported-corpus-license-record"));
        assert!(failure.diagnostics.iter().any(|item| item.path == "evidence.source_digest_blake3"));
    }

    #[test]
    fn incomplete_corpus_evidence_is_rejected() {
        let mut evidence = corpus_evidence();
        let observations = corpus_observations(&evidence);
        evidence.artifacts.pop();
        evidence.evidence_identity_blake3 =
            external_corpus_evidence_identity_blake3(&evidence).expect("changed evidence identity");

        let failure =
            validate_external_corpus_evidence(&evidence, &observations).expect_err("missing corpus artifact must fail");

        assert!(failure.diagnostics.iter().any(|item| item.code == "corpus-artifact-role-set-mismatch"));
    }

    #[test]
    fn unsupported_corpus_package_is_rejected() {
        let mut evidence = corpus_evidence();
        let observations = corpus_observations(&evidence);
        evidence.selected_packages = vec!["unsupported-package".into()];
        evidence.evidence_identity_blake3 =
            external_corpus_evidence_identity_blake3(&evidence).expect("changed selection identity");

        let failure = validate_external_corpus_evidence(&evidence, &observations)
            .expect_err("unsupported package selection must fail");

        assert!(failure.diagnostics.iter().any(|item| item.code == "corpus-package-selection-mismatch"));
    }

    #[test]
    fn missing_corpus_source_artifact_is_rejected() {
        let mut evidence = corpus_evidence();
        evidence.artifacts.retain(|artifact| artifact.role != CORPUS_ROLE_SOURCES);
        evidence.evidence_identity_blake3 =
            external_corpus_evidence_identity_blake3(&evidence).expect("missing source identity");
        let observations = corpus_observations(&evidence);

        let failure =
            validate_external_corpus_evidence(&evidence, &observations).expect_err("missing source artifact must fail");

        assert!(failure.diagnostics.iter().any(|item| item.code == "corpus-artifact-role-set-mismatch"));
    }

    fn corpus_observations(evidence: &ExternalCorpusEvidence) -> Vec<CorpusArtifactObservation> {
        evidence
            .artifacts
            .iter()
            .map(|artifact| CorpusArtifactObservation {
                path: artifact.path.clone(),
                digest_blake3: artifact.digest_blake3.clone(),
                bytes: artifact.bytes,
                catalog_packages: if artifact.role == CORPUS_ROLE_CATALOG {
                    evidence.selected_packages.clone()
                } else {
                    Vec::new()
                },
            })
            .collect()
    }

    fn corpus_evidence() -> ExternalCorpusEvidence {
        let roles = [
            CORPUS_ROLE_GRAPH,
            CORPUS_ROLE_SOURCES,
            CORPUS_ROLE_CATALOG,
            CORPUS_ROLE_BLOCKERS,
            CORPUS_ROLE_PRODUCER,
        ];
        let artifacts = roles
            .into_iter()
            .map(|role| CorpusArtifact {
                role: role.into(),
                path: format!("evidence/{role}.json"),
                digest_blake3: DIGEST_A.into(),
                bytes: 1,
            })
            .collect::<Vec<_>>();
        let mut evidence = ExternalCorpusEvidence {
            schema: CORPUS_EVIDENCE_SCHEMA.into(),
            evidence_identity_blake3: empty_digest(),
            repository: "https://github.com/ekala-project/corepkgs".into(),
            source_reference: format!("github:ekala-project/corepkgs/{REVISION}"),
            revision: REVISION.into(),
            source_digest_blake3: DIGEST_B.into(),
            observed_license: CORPUS_LICENSE_MIT.into(),
            license_path: "LICENSE".into(),
            license_digest_blake3: DIGEST_C.into(),
            selected_packages: vec!["hello".into()],
            producer_policy_digest_blake3: DIGEST_A.into(),
            artifacts,
            non_claims: domain_non_claims(),
        };
        evidence.evidence_identity_blake3 =
            external_corpus_evidence_identity_blake3(&evidence).expect("corpus evidence identity");
        evidence
    }
}
