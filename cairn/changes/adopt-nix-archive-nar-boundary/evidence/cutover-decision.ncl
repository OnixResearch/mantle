let MatrixCase = {
  id | String,
  disposition | String,
  checked_facts | Array String,
} in
let Cutover = {
  schema | String,
  adapter_version | String,
  upstream_version | String,
  upstream_commit | String,
  decision | String,
  required_cases | Array String,
  cases | Array MatrixCase,
  fallback | String,
  non_claim | String,
} in
{
  schema = "mantle-nar-cutover-v1",
  adapter_version = "mantle-filesystem-nar-v1",
  upstream_version = "0.1.0",
  upstream_commit = "14362ab589daa4869bda744d4fbe26a1914b5491",
  decision = "accepted",
  required_cases = [
    "byte-safe-tree",
    "upstream-goldens",
    "generated-trees",
    "failure-matrix",
    "large-stream",
  ],
  cases = [
    {
      id = "byte-safe-tree",
      disposition = "matched",
      checked_facts = ["bytes", "size", "all-supported-digests", "names", "modes", "symlinks", "order"],
    },
    {
      id = "upstream-goldens",
      disposition = "matched",
      checked_facts = ["regular", "executable", "symlink", "directory", "fixture-source-identity"],
    },
    {
      id = "generated-trees",
      disposition = "matched",
      checked_facts = ["bounded-generation", "deterministic-replay", "adapted-snix-parity"],
    },
    {
      id = "failure-matrix",
      disposition = "matched",
      checked_facts = ["missing", "read-failure", "case-collision", "root-substitution", "byte-limit"],
    },
    {
      id = "large-stream",
      disposition = "matched",
      checked_facts = ["chunked-reader", "castore-ingest", "no-production-full-buffer-decoder"],
    },
  ],
  fallback = "none",
  non_claim = "The accepted matrix proves only the selected Mantle boundary cases.",
} | Cutover
