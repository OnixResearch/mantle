# Machine-local scheduling policy; never a derivation input or proof receipt.
let Bounded = fun max => std.contract.from_predicate (fun value =>
  std.is_number value && std.number.is_integer value && value > 0 && value <= max
) in
let Policy = {
  schema | std.contract.Equal "mantle-compile-slot-policy-v1",
  total_slots | Bounded 4096,
  per_build_max | Bounded 4096,
  total_queue | Bounded 65536,
  per_build_queue | Bounded 65536,
  wait_timeout_ms | Bounded 3600000,
  fairness | std.contract.Equal "round_robin",
} in
let Relations = std.contract.from_predicate (fun value =>
  value.per_build_max <= value.total_slots && value.per_build_queue <= value.total_queue
) in
{
  Policy = Policy,
  default = {
    schema = "mantle-compile-slot-policy-v1",
    total_slots = 8,
    per_build_max = 4,
    total_queue = 256,
    per_build_queue = 64,
    wait_timeout_ms = 300000,
    fairness = "round_robin",
  } | Policy | Relations,
}
