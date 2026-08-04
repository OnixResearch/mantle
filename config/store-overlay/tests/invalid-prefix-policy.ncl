let Contracts = import "../contracts.ncl" in
{
  writable_overlay_count = 1,
  base_order = "declaration-order",
  base_capability = "read-only",
  require_same_prefix = false,
  reject_duplicate_bases = true,
  no_backfill = true,
} | Contracts.Composition
