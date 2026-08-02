let Contracts = import "contracts.ncl" in

{
  schema = 'mantle-foreign-cache-closure-policy-v1,
  max_paths = 4096,
  max_references = 65536,
  max_nar_bytes = 8589934592,
  max_depth = 128,
} | Contracts.CacheClosurePolicy
