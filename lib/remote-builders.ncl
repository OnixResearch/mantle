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

  pools
    | doc "List of configured remote builder pools."
    | Array RemoteBuilderPoolContract
    | default = [],
}
