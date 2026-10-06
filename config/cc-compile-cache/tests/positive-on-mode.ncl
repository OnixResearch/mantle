# Policy-only synthetic executable inventory: not a build or proof result.
let Contracts = import "../contracts.ncl" in
let base = import "../default.ncl" in
{
  schema = base.schema,
  driver = {
    mode = "local-read-write",
    admitted_compiler_families = base.driver.admitted_compiler_families,
    driver_path = "/declared/cc-driver",
    compiler_path = "/declared/compiler",
    socket_path = "/private/cc.sock",
    receipt_path = "/private/receipt.json",
    driver_digest_blake3 = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
    compiler_digest_blake3 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    platform_digest = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    ambient_path_mutation = false,
    unclassified = base.driver.unclassified,
    daemon_unavailable = base.driver.daemon_unavailable,
  },
  identity = base.identity,
  limits = base.limits,
  daemon = base.daemon,
  receipts = base.receipts,
  probe = base.probe,
  proof = base.proof,
} | Contracts.Policy
