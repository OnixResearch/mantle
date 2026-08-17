let Contracts = import "contracts.ncl" in
let PolicyId = "cairn.nominal-evidence-identities.v1" in
let RuntimeSchema = "cairn.nominal-evidence-identity-runtime.v1" in
let OctetSchema = "octet-nominal-domain-policy/v1" in
let DomainIdMaxBytes = 128 in
let ActorIdMaxBytes = 256 in
let Blake3RawHexBytes = 64 in
let Blake3LongestPrefixBytes = 7 in
let Blake3DisplayMaxBytes = Blake3RawHexBytes + Blake3LongestPrefixBytes in
let GitOidSha1HexBytes = 40 in
let Sha256HexBytes = 64 in
let CoreScope = "crates/cairn-core/src/verified" in
let WireScope = "crates/cairn-core/src/verified" in
let MantleIdentifierMaxBytes = 4096 in
let MantleRemoteIdentifierMaxBytes = 256 in
let BootstrapCoreScope = "crates/crunch-bootstrap-core/src" in
let BootstrapWireScope = "crates/crunch-bootstrap-core/src/stagex.rs" in
let RemoteCoreScope = "src/remote_nominal.rs" in
let RemoteWireScope = "src/remote_build.rs" in
let ReleaseCoreScope = "crates/crunch-release-core/src/content_bound_nominal.rs" in
let ReleaseWireScope = "crates/crunch-release-core/src/content_bound_requirements.rs" in
let FrontendCoreScope = "src/frontend_artifact_spec.rs" in
let BuildCoreScope = "crates/crunch-build/src/trust_boundary_nominal.rs" in
let BuildWireScope = "crates/crunch-build/src/fetcher.rs" in
let NonClaim = "nominal identities prove configured local syntax and role separation only; they do not prove evidence truth, authority, policy correctness, verifier soundness, mutation safety outside checked plans, or release eligibility" in
let domain = fun id kind hash_algorithm prefix byte_limit symbols => {
  domain_id = id,
  category = kind,
  algorithm = hash_algorithm,
  required_prefix = prefix,
  max_bytes = byte_limit,
  rust_symbols = symbols,
  core_scopes = [CoreScope],
  wire_compatibility_scopes = [WireScope],
  raw_representation = "string",
  wrapper_kind = "marker-generic",
  constructor_policy = "marker-only",
} in
let refined_domain = fun id kind hash_algorithm byte_limit symbols checked_scopes compatible_scopes => {
  domain_id = id,
  category = kind,
  algorithm = hash_algorithm,
  required_prefix = "",
  max_bytes = byte_limit,
  rust_symbols = symbols,
  core_scopes = checked_scopes,
  wire_compatibility_scopes = compatible_scopes,
  raw_representation = "string",
  wrapper_kind = "refined-newtype",
  constructor_policy = "private-fallible",
} in
let RuntimeDomains = [
  domain "actor-id" "domain-ref" "none" "actor:" ActorIdMaxBytes ["ActorId"],
  domain "archive-digest" "blake3-digest" "BLAKE3" "" Blake3DisplayMaxBytes ["ArchiveDigest"],
  domain "archive-id" "domain-id" "none" "" DomainIdMaxBytes ["ArchiveId"],
  domain "artifact-digest" "blake3-digest" "BLAKE3" "" Blake3DisplayMaxBytes ["ArtifactDigest"],
  domain "binary-digest" "blake3-digest" "BLAKE3" "" Blake3DisplayMaxBytes ["BinaryDigest"],
  domain "chain-digest" "blake3-digest" "BLAKE3" "" Blake3DisplayMaxBytes ["ChainDigest"],
  domain "change-id" "domain-id" "none" "" DomainIdMaxBytes ["ChangeId"],
  domain "content-root-digest" "blake3-digest" "BLAKE3" "" Blake3DisplayMaxBytes ["ContentRootDigest"],
  domain "evidence-kind" "domain-id" "none" "" DomainIdMaxBytes ["EvidenceKind"],
  domain "git-oid-sha1" "imported-digest" "Git-OID-SHA-1" "" GitOidSha1HexBytes ["GitOidSha1Digest"],
  domain "graph-digest" "blake3-digest" "BLAKE3" "" Blake3DisplayMaxBytes ["GraphDigest"],
  domain "input-digest" "blake3-digest" "BLAKE3" "" Blake3DisplayMaxBytes ["InputDigest"],
  domain "ledger-digest" "blake3-digest" "BLAKE3" "" Blake3DisplayMaxBytes ["LedgerDigest"],
  refined_domain "mantle-ca-plan-id" "domain-id" "none" MantleIdentifierMaxBytes ["DerivationName", "OutputName", "LogicalStorePrefix", "ProvisionalStorePath", "FinalStorePath", "DerivationKey"] [BuildCoreScope] ["crates/crunch-build/src/ca_plan.rs", "crates/crunch-pipeline/src/lib.rs"],
  refined_domain "mantle-content-bound-digest" "blake3-digest" "BLAKE3" Blake3RawHexBytes ["CheckedBlake3Hex", "CheckedRequirementDigest", "CheckedContentBoundDigest"] [ReleaseCoreScope] [ReleaseWireScope],
  refined_domain "mantle-content-bound-id" "domain-id" "none" MantleIdentifierMaxBytes ["ContentBoundRepositoryId", "ContentBoundRequirementId", "ContentBoundReleaseId"] [ReleaseCoreScope] [ReleaseWireScope],
  domain "mantle-digest" "blake3-digest" "BLAKE3" "" Blake3DisplayMaxBytes ["MantleDigest"],
  refined_domain "mantle-fetch-id" "domain-ref" "none" MantleIdentifierMaxBytes ["FetchUrl", "GitRevision", "ArchiveInputPath", "FetchOutputDirectory"] [BuildCoreScope] [BuildWireScope],
  refined_domain "mantle-frontend-id" "domain-id" "none" MantleIdentifierMaxBytes ["FrontendSpecId", "FrontendValidatorKind", "FrontendArtifactKind", "FrontendArtifactRef", "FrontendBuildRoot"] [FrontendCoreScope] [FrontendCoreScope],
  refined_domain "mantle-frontend-spec-hash" "blake3-digest" "BLAKE3" Blake3RawHexBytes ["FrontendSpecHash"] [FrontendCoreScope] [FrontendCoreScope],
  refined_domain "mantle-lineage-id" "domain-id" "none" MantleIdentifierMaxBytes ["LineageSourceArtifactId", "LineageGeneratedArtifactId", "LineageTransitionToolId", "LineagePatchId", "LineageEnvironmentAssumptionId"] [BootstrapCoreScope] ["crates/crunch-bootstrap-core/src/lineage.rs"],
  refined_domain "mantle-remote-id" "domain-id" "none" MantleRemoteIdentifierMaxBytes ["RemoteRequestId", "RemoteProtocolSessionId", "RemoteEndpointId", "RemoteOutputId"] [RemoteCoreScope] [RemoteWireScope],
  refined_domain "mantle-stagex-digest" "blake3-digest" "BLAKE3" Blake3RawHexBytes ["StagexLineageManifestDigest", "StagexSourceStateDigest", "StagexExecutableDigest"] [BootstrapCoreScope] [BootstrapWireScope],
  refined_domain "mantle-stagex-id" "domain-id" "none" MantleIdentifierMaxBytes ["StagexStageId", "StagexArtifactId", "StagexExecutableAuthorizationId", "StagexEnvironmentAssumptionId"] [BootstrapCoreScope] [BootstrapWireScope],
  domain "operation-id" "domain-id" "none" "operation:" DomainIdMaxBytes ["OperationId"],
  domain "pack-digest" "blake3-digest" "BLAKE3" "" Blake3DisplayMaxBytes ["PackDigest"],
  domain "plan-digest" "blake3-digest" "BLAKE3" "" Blake3DisplayMaxBytes ["PlanDigest"],
  domain "policy-digest" "blake3-digest" "BLAKE3" "" Blake3DisplayMaxBytes ["PolicyDigest"],
  domain "policy-kind" "domain-id" "none" "" DomainIdMaxBytes ["PolicyKind"],
  domain "projection-role" "domain-id" "none" "" DomainIdMaxBytes ["ProjectionRole"],
  domain "receipt-digest" "blake3-digest" "BLAKE3" "" Blake3DisplayMaxBytes ["ReceiptDigest"],
  domain "sha256-digest" "imported-digest" "SHA-256" "" Sha256HexBytes ["Sha256Digest"],
  domain "sidecar-digest" "blake3-digest" "BLAKE3" "" Blake3DisplayMaxBytes ["SidecarDigest"],
  domain "source-digest" "blake3-digest" "BLAKE3" "" Blake3DisplayMaxBytes ["SourceDigest"],
  domain "state-digest" "blake3-digest" "BLAKE3" "" Blake3DisplayMaxBytes ["StateDigest"],
  domain "valence-digest" "blake3-digest" "BLAKE3" "" Blake3DisplayMaxBytes ["ValenceDigest"],
  domain "workspace-id" "domain-id" "none" "workspace:" DomainIdMaxBytes ["WorkspaceId"],
] in
let octet_domain = fun entry => {
  domain_id = entry.domain_id,
  symbols = entry.rust_symbols,
  raw_representation = entry.raw_representation,
  core_scopes = entry.core_scopes,
  compatibility_scopes = entry.wire_compatibility_scopes,
  compatibility_required = true,
  wrapper_kind = entry.wrapper_kind,
  constructor_policy = entry.constructor_policy,
} in
{
  policy_id = PolicyId,
  runtime = {
    schema_version = RuntimeSchema,
    policy_id = PolicyId,
    domains = RuntimeDomains,
  },
  octet = {
    schema_version = OctetSchema,
    policy_id = PolicyId,
    default_mode = "deny",
    heuristic_suffixes = ["Id", "Digest", "Hash", "Root"],
    domains = std.array.map octet_domain RuntimeDomains,
    non_claims = [NonClaim],
  },
  non_claims = [NonClaim],
} | Contracts.NominalIdentityPolicy
