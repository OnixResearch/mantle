let Contracts = import "contracts.ncl" in
{
  schema = "mantle-authority-caveats-v1",
  caveat_domain = "mantle",
  caveat_type = "filter-v1",
  accepted_kinds = ["rewrite", "reject", "alternatives", "unknown-deny-all"],
  limits = {
    max_chain = 8,
    max_alternatives = 8,
    max_pattern_bytes = 256,
    max_caveat_bytes = 8192,
    max_value_bytes = 512,
    max_segments = 16,
    max_declared_jobs = 32,
  },
  order = "newest-first-right-to-left",
  rejection = "silent-discard",
  recheck_original_and_parent_prefixes = true,
  non_claims = [
    "UCAN authenticates signed grant and proof; this policy does not",
    "Basalt generic authority consumes verified facts; this policy does not verify them",
    "site policy and effect ordering stay with the local receiver",
  ],
} | Contracts.Policy
