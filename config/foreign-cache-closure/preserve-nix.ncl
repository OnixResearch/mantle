let Contracts = import "contracts.ncl" in

{
  source_prefixes = ["/nix/store"],
  target_prefix = "/nix/store",
  rewrite_builder = true,
  rewrite_args = true,
  rewrite_env = true,
  rewrite_sources = true,
  rewrite_declared_references = true,
  allow_embedded_source_payload_rewrite = false,
  builtin_mappings = {
    "fixed-output-fetch" = "mantle.fetch",
    "nix.derivation" = "mantle.foreign.nix.derivation",
  },
  output_path_recompute_mode = "preserve-cache-paths-v1",
  trusted_cache_scopes = ["trusted-binary-cache"],
  allowed_sandbox_capabilities = [],
} | Contracts.PreservedPathTranslationPolicy
