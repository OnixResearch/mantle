let Contracts = import "../contracts.ncl" in
let good = (import "positive-on-mode.ncl").driver in
{
  mode = good.mode,
  admitted_compiler_families = good.admitted_compiler_families,
  driver_path = "",
  compiler_path = good.compiler_path,
  socket_path = good.socket_path,
  receipt_path = good.receipt_path,
  driver_digest_blake3 = good.driver_digest_blake3,
  compiler_digest_blake3 = good.compiler_digest_blake3,
  platform_digest = good.platform_digest,
  ambient_path_mutation = good.ambient_path_mutation,
  unclassified = good.unclassified,
  daemon_unavailable = good.daemon_unavailable,
} | Contracts.DriverShape | Contracts.DriverPolicy
