let Contracts = import "../contracts.ncl" in
{
  writable_overlay_count = 1,
  base_order = "declaration-order",
  base_capability = "read-only",
  require_same_prefix = true,
  reject_duplicate_bases = false,
  no_backfill = true,
} | Contracts.Composition
