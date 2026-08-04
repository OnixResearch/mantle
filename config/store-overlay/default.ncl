let Contracts = import "contracts.ncl" in
let MaxBaseLayers = 8 in
let MaxGenerationMembers = 1000000 in
let MaxGenerationBytes = 1099511627776 in
let MaxDescriptorBytes = 65536 in

{
  schema = "mantle-store-overlay-policy-v1",
  policy_name = "mantle-store-overlay-default-v1",
  hash_algorithm = "BLAKE3",

  limits = {
    max_base_layers = MaxBaseLayers,
    max_generation_members = MaxGenerationMembers,
    max_generation_bytes = MaxGenerationBytes,
    max_descriptor_bytes = MaxDescriptorBytes,
  },

  composition = {
    writable_overlay_count = 1,
    base_order = "declaration-order",
    base_capability = "read-only",
    require_same_prefix = true,
    reject_duplicate_bases = true,
    no_backfill = true,
  },

  trust = {
    policy_id = "mantle-store-overlay-trust-v2",
    verification = "ed25519-pathinfo",
    unknown_policy = "reject",
    shadow_failure = "block",
    skip_failure_classes = [],
  },

  generation = {
    allowed_state_schemas = ["mantle-store-state-v1"],
    bind_root_inventory = true,
    bind_pathinfo = true,
    bind_directories = true,
    bind_blobs = true,
    revalidate_before_execution = true,
    revalidate_before_output_admission = true,
  },

  failures = {
    prefix_mismatch = "block",
    duplicate_base = "block",
    writable_base = "block",
    unknown_trust = "block",
    corrupt_higher_layer = "block",
    incomplete_higher_layer = "block",
    generation_drift = "block",
    base_to_overlay_reference = "block",
  },

  non_claims = [
    "overlay-composition-does-not-prove-base-immutability",
    "base-generation-does-not-prove-whole-database-atomicity",
    "layer-trust-does-not-prove-content-correctness",
    "read-through-does-not-authorize-base-mutation",
  ],
} | Contracts.Policy
