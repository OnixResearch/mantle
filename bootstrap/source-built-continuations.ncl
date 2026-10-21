# Immutable resource reservations for source-owned cold continuations.
# Source-site/file counts are sizing hints, not proved command or macro counts.
# Exhaustion fails the attempt; it cannot omit work or authorize a larger graph.
# The existing recipe, action/event, time and disk ceilings remain authoritative.
{
  schema | String = "mantle-source-continuation-reservations-v1",
  macro_base_slots | Number = 8,
  macro_slots_per_source_site | Number = 4,
  native_base_slots | Number = 8,
  native_slots_per_source_file | Number = 4,
  # Exact phase-id overrides, reviewed as source policy rather than cached facts.
  overrides | { _: Number } = {},
}
