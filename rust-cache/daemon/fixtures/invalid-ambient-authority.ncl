let Contracts = import "../contracts.ncl" in
let policy = import "policy.ncl" in
(policy & {
  cache_mode | force = "shared-read",
  shared_reads_enabled | force = true,
  result_sources | force = [{
    source_id = "remote-v1",
    kind = "http",
    endpoint = "https://token@example.invalid/cache",
    priority = 0,
  }],
}) | Contracts.Policy
