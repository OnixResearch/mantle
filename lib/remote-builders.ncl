# Remote build farm configuration contract.
#
# Defines the typed, provider-neutral schema for remote builder pools,
# capability profiles, transfer limits, output trust roots, fallback
# policy, and publisher profiles.
#
# Runtime Rust consumes the evaluated JSON export of this contract.
# Provider-specific transport details remain inside adapters.
#
# r[impl remote_builds.production_operator_configuration]

let RemoteTransportMode = [|
  'stdio,
  'ssh-stdio,
  'p2p,
|] in

let RemoteSandboxMode = [|
  'practical,
  'strict,
|] in

let RemoteNetworkMode = [|
  'none,
  'local,
  'full,
|] in

let RemoteFallbackPolicy = [|
  'never,
  'always,
  'trusted-only,
|] in

let DefaultRemoteConcurrency = 1 in
let DefaultRemoteUploadBytes = 1073741824 in
let DefaultRemoteBuildTimeSecs = 3600 in
let DefaultRemoteRetryAttempts = 3 in
let DefaultRemoteRetryDelaySecs = 5 in
let DefaultRemoteAttemptTimeoutSecs = DefaultRemoteBuildTimeSecs in
let DefaultTransferChunkBytes = 65536 in
let DefaultTransferInFlightBytes = 1048576 in
let DefaultTransferInFlightChunks = 16 in
let DefaultTransferBufferedChunks = 16 in
let DefaultTransferArtifacts = 4096 in
let DefaultTransferChunks = 65536 in
let DefaultTransferTotalBytes = DefaultRemoteUploadBytes in
let DefaultTransferCheckpointBytes = 262144 in
let DefaultTransferIdleProgressSteps = 1024 in
let DefaultTransferReplayRounds = 4 in
let DefaultTransferControlBytes = 1048576 in
let DefaultTelemetryEventCapacity = 1024 in
let DefaultTelemetryBatchSize = 128 in
let DefaultOtlpTimeoutMs = 2000 in
let DefaultFailureDebugMetadataBytes = 1048576 in
let DefaultFailureDebugObjectBytes = 67108864 in
let DefaultFailureDebugRetentionSecs = 604800 in
let DefaultFailureCaptureFiles = 32 in
let DefaultFailureCaptureTotalBytes = 16777216 in
let DefaultFailureCaptureFileBytes = 8388608 in
let DefaultFailureCaptureDepth = 16 in

let GatewayBounds = {
  connections_max | Number | default = 1024,
  frame_bytes_max | Number | default = 1048576,
  message_bytes_max | Number | default = 4194304,
  concurrent_operations_max | Number | default = 1024,
  idle_secs_max | Number | default = 3600,
  cursor_bytes_max | Number | default = 512,
  log_window_bytes_max | Number | default = 1048576,
  event_page_items_max | Number | default = 1024,
  partial_transfer_bytes_max | Number | default = 1073741824,
  paths_max | Number | default = 4096,
} in

let _gateway_policy = {
  policy_identity_blake3
    | doc "BLAKE3 identity of the reviewed gateway policy"
    | String
    | default = "2ecf6b4eae0613dff46c65561b9d9921a26933b25cb2be42ec10a9a3ca9586bc",
  audience | String | default = "mantle-remote-gateway",
  api_version | Number | default = 1,
  nix_protocol_major | Number | default = 1,
  nix_protocol_minor_min | Number | default = 23,
  nix_protocol_minor_max | Number | default = 37,
  bounds | GatewayBounds | default = {},
  public_endpoint_enabled
    | doc "Public exposure is disabled unless the archived credential-hardening receipt is exact"
    | Bool
    | default = false,
  credential_hardening_evidence_blake3 | String | optional,
} in

let RemoteTelemetryPolicy = {
  event_capacity
    | doc "Maximum provider-neutral lifecycle events admitted per export cycle"
    | Number
    | default = DefaultTelemetryEventCapacity,
  batch_size
    | doc "Maximum events carried by one exporter batch"
    | Number
    | default = DefaultTelemetryBatchSize,
} in

let RemotePrometheusTelemetry = {
  enabled
    | doc "Opt in to atomic Prometheus node-textfile snapshots"
    | Bool
    | default = false,
  textfile_path
    | doc "Existing non-symlinked node-textfile collector directory and output path"
    | String
    | default = "",
} in

let RemoteOtlpTelemetry = {
  enabled
    | doc "Opt in to OTLP/HTTP metrics delivery; plaintext endpoints must be literal loopback addresses"
    | Bool
    | default = false,
  endpoint
    | doc "Loopback OTLP/HTTP metrics endpoint without credentials, query, or fragment"
    | String
    | default = "",
  timeout_ms
    | doc "Bounded connect, write, and response timeout"
    | Number
    | default = DefaultOtlpTimeoutMs,
} in

let RemoteTelemetryExport = {
  telemetry | RemoteTelemetryPolicy | default = {},
  prometheus | RemotePrometheusTelemetry | default = {},
  otlp | RemoteOtlpTelemetry | default = {},
} in

let RemoteTraceContext = {
  enabled
    | doc "Opt in to bounded W3C TRACEPARENT/TRACESTATE propagation for diagnostics only"
    | Bool
    | default = false,
} in

let RemoteFailureCapturePolicy = {
  enabled
    | doc "Opt in to allowlisted failed-sandbox regular-file capture before cleanup"
    | Bool
    | default = false,
  allowed_relative_paths
    | doc "Exact normalized relative sandbox paths allowed for capture"
    | Array String
    | default = [],
  sensitivity
    | [| 'public-diagnostic, 'restricted-diagnostic |]
    | default = 'restricted-diagnostic,
  file_count_max | Number | default = DefaultFailureCaptureFiles,
  total_bytes_max | Number | default = DefaultFailureCaptureTotalBytes,
  file_bytes_max | Number | default = DefaultFailureCaptureFileBytes,
  depth_max | Number | default = DefaultFailureCaptureDepth,
  failure_mode
    | [| 'diagnostic-only |]
    | default = 'diagnostic-only,
} in

let RemoteFailureDebugPolicy = {
  policy_name | String | default = "mantle-remote-failure-debug-default-v1",
  metadata_bytes_max | Number | default = DefaultFailureDebugMetadataBytes,
  object_bytes_max | Number | default = DefaultFailureDebugObjectBytes,
  retention_secs | Number | default = DefaultFailureDebugRetentionSecs,
  replay_enabled
    | doc "Allow explicit replay planning and execution; replay always receives new authority"
    | Bool
    | default = false,
  capture | RemoteFailureCapturePolicy | default = {},
} in

let RemoteTransferPolicy = {
  chunk_bytes_max | Number | default = DefaultTransferChunkBytes,
  in_flight_bytes_max | Number | default = DefaultTransferInFlightBytes,
  in_flight_chunks_max | Number | default = DefaultTransferInFlightChunks,
  buffered_chunks_max | Number | default = DefaultTransferBufferedChunks,
  artifact_count_max | Number | default = DefaultTransferArtifacts,
  chunk_count_max | Number | default = DefaultTransferChunks,
  total_bytes_max | Number | default = DefaultTransferTotalBytes,
  checkpoint_bytes_max | Number | default = DefaultTransferCheckpointBytes,
  idle_progress_steps_max | Number | default = DefaultTransferIdleProgressSteps,
  replay_rounds_max | Number | default = DefaultTransferReplayRounds,
  control_bytes_max | Number | default = DefaultTransferControlBytes,
} in

let NamedResourceQuantity = {
  name | String,
  quantity
    | doc "Positive bounded quantity validated by the Rust admission core"
    | Number,
} in

let RemoteResourceVector = {
  cpu_units | Number,
  memory_bytes | Number,
  scratch_bytes | Number,
  accelerators | Array NamedResourceQuantity | default = [],
  named_tokens
    | doc "Opaque scarce-token quantities; never credentials"
    | Array NamedResourceQuantity
    | default = [],
} in

let RemoteResourceInventory = {
  total | RemoteResourceVector,
} in

let RemoteResourceRequirements = {
  quantities | RemoteResourceVector,
  semantic_accelerator_classes
    | doc "Semantic classes affect action identity; quantities remain scheduling-only"
    | Array String
    | default = [],
} in

let RemoteLocalityPolicy = {
  basis | [| 'receiver-verified |] | default = 'receiver-verified,
  stale | [| 'reject |] | default = 'reject,
  unverified | [| 'no-verified-content |] | default = 'no-verified-content,
} in

let RemoteRetryPolicy = {
  max_attempts
    | doc "Total bounded attempts, including the initial assignment"
    | Number
    | default = DefaultRemoteRetryAttempts,
  retry_delay_secs
    | doc "Provider-neutral delay before a retry becomes eligible"
    | Number
    | default = DefaultRemoteRetryDelaySecs,
  attempt_timeout_secs
    | doc "Per-attempt timeout evaluated against coordinator-supplied time facts"
    | Number
    | default = DefaultRemoteAttemptTimeoutSecs,
} in

let RemoteWorkspacePolicy = {
  modes
    | doc "Workspace modes this endpoint admits; mutable-session must be explicitly listed"
    | Array [| 'none, 'immutable-snapshot, 'mutable-session |]
    | default = ['none],
  authority_class
    | doc "Authority partition used in workspace compatibility and lease checks"
    | String
    | default = "local-default",
  guest_paths
    | doc "Stable guest mount paths admitted by this endpoint"
    | Array String
    | default = ["/build/.mantle-workspace"],
  workspace_count_max | Number | default = 64,
  bytes_max | Number | default = 4294967296,
  files_max | Number | default = 100000,
  snapshots_max | Number | default = 8,
  idle_generations_max | Number | default = 16,
  age_generations_max | Number | default = 256,
  quarantine_count_max | Number | default = 16,
} in

let RemoteCapabilityProfile = {
  system
    | doc "Target system triple"
    | String
    | default = "x86_64-linux",
  sandbox_mode
    | RemoteSandboxMode
    | default = 'practical,
  network_mode
    | RemoteNetworkMode
    | default = 'none,
  features
    | Array String
    | default = [],
  worker_generation
    | doc "Positive monotonic worker/content generation; advance before restart or destructive cache changes"
    | Number
    | default = 1,
  max_concurrency
    | doc "Maximum parallel builds on this builder"
    | Number
    | default = DefaultRemoteConcurrency,
  resource_inventory
    | doc "Explicit quantified capacity; absence is never inferred from concurrency"
    | RemoteResourceInventory
    | optional,
  max_upload_bytes
    | doc "Maximum upload size per request"
    | Number
    | default = DefaultRemoteUploadBytes,
  max_build_time_secs
    | doc "Maximum build duration"
    | Number
    | default = DefaultRemoteBuildTimeSecs,
  retry_policy
    | RemoteRetryPolicy
    | default = {},
  transfer_policy
    | doc "Bounded resumable transfer, receiver credit, checkpoint, and replay policy"
    | RemoteTransferPolicy
    | default = {},
  workspace_policy
    | doc "Explicit workspace registration, compatibility, quota, and retention policy"
    | RemoteWorkspacePolicy
    | default = {},
} in

let RemoteEndpoint = {
  endpoint_id
    | doc "Unique identifier for this remote builder endpoint"
    | String,
  transport
    | RemoteTransportMode
    | default = 'stdio,
  profile
    | RemoteCapabilityProfile
    | default = {},
} in

let TrustRoot = {
  public_key_name
    | doc "Verifier name (e.g. cache.example.com-1)"
    | String,
  public_key_base64
    | doc "Ed25519 public key in base64"
    | String,
} in

let PublisherProfile = {
  mode
    | doc "Publication target type"
    | [| 'nix-cache, 'local-archive, 's3 |]
    | default = 'nix-cache,
  target_url
    | doc "Publication endpoint URL"
    | String
    | default = "",
  trusted_public_keys
    | Array TrustRoot
    | default = [],
} in

let BatchDispatcherCommand = {
  program
    | doc "Absolute executable path; the adapter verifies its BLAKE3 digest before every operation"
    | String,
  expected_digest_blake3 | String,
  args
    | doc "Bounded static arguments; credentials and policy secrets are forbidden"
    | Array String
    | default = [],
  timeout_secs | Number,
  stdout_limit_bytes | Number,
  stderr_limit_bytes | Number,
} in

let BatchEnvironmentHandle = {
  name
    | doc "Allowlisted shell-owned environment handle name; values never enter canonical records"
    | String,
} in

let BatchDispatcherProfile = {
  instance_id | String,
  generation | Number,
  adapter | [| 'direct-process-v1, 'slurm-cli-v1 |],
  protocol_schema | String | default = "mantle-batch-dispatch-adapter-v1",
  allowed_operations
    | Array [| 'submit, 'observe, 'cancel, 'reconcile |]
    | default = ['submit, 'observe, 'cancel, 'reconcile],
  provider_class | String,
  submit | BatchDispatcherCommand,
  observe | BatchDispatcherCommand,
  cancel | BatchDispatcherCommand,
  reconcile | BatchDispatcherCommand,
  worker_program | String,
  worker_program_digest_blake3 | String,
  worker_args | Array String | default = [],
  environment_handles | Array BatchEnvironmentHandle | default = [],
  bootstrap_policy
    | [| 'direct-worker-exec-v1, 'fixed-template-v1 |]
    | default = 'direct-worker-exec-v1,
  redact_provider_output | Bool | default = true,
  startup_timeout_secs | Number,
  terminal_timeout_secs | Number,
  max_reconcile_attempts | Number,
} in

let RemoteBuilderPoolContract = {
  pool_id
    | doc "Unique pool identifier"
    | String,
  endpoints
    | Array RemoteEndpoint,
  fallback_policy
    | RemoteFallbackPolicy
    | default = 'trusted-only,
  output_trust_roots
    | Array TrustRoot
    | default = [],
  publishers
    | Array PublisherProfile
    | default = [],
} in

{
  RemoteBuilderPool
    | doc "A configured remote builder pool with typed capability and trust facts."
    = RemoteBuilderPoolContract,

  RemoteTelemetry
    | doc "Bounded disabled-by-default Prometheus and OTLP exporter configuration."
    = RemoteTelemetryExport,

  GatewayPolicy
    | doc "Private-by-default Nix and asynchronous Build API gateway policy."
    = _gateway_policy,

  ResourceInventory = RemoteResourceInventory,
  ResourceRequirements = RemoteResourceRequirements,
  LocalityPolicy = RemoteLocalityPolicy,

  RemoteFailureDebug
    | doc "Bounded disabled-by-default remote failure capture and replay configuration."
    = RemoteFailureDebugPolicy,

  BatchDispatcher
    | doc "Typed confined external allocation dispatcher profile."
    = BatchDispatcherProfile,

  pools
    | doc "List of configured remote builder pools."
    | Array RemoteBuilderPoolContract
    | default = [],

  dispatchers
    | doc "Bounded external allocation dispatchers; empty disables external batch scheduling."
    | Array BatchDispatcherProfile
    | default = [],

  telemetry
    | doc "Disabled-by-default bounded remote lifecycle telemetry exporters."
    | RemoteTelemetryExport
    | default = {},

  gateway
    | doc "Private-by-default gateway limits and public-exposure evidence gate."
    | _gateway_policy
    | default = {},

  trace_context
    | doc "Disabled-by-default W3C diagnostic trace propagation; never authoritative."
    | RemoteTraceContext
    | default = {},

  failure_debug
    | doc "Bounded remote-failure bundle, capture, retention, and replay policy; payload capture and replay are disabled by default."
    | RemoteFailureDebugPolicy
    | default = {},
}
