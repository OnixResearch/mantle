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
  max_concurrency
    | doc "Maximum parallel builds on this builder"
    | Number
    | default = DefaultRemoteConcurrency,
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

  pools
    | doc "List of configured remote builder pools."
    | Array RemoteBuilderPoolContract
    | default = [],

  telemetry
    | doc "Disabled-by-default bounded remote lifecycle telemetry exporters."
    | RemoteTelemetryExport
    | default = {},

  trace_context
    | doc "Disabled-by-default W3C diagnostic trace propagation; never authoritative."
    | RemoteTraceContext
    | default = {},
}
