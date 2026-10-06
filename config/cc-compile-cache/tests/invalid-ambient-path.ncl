let Contracts = import "../contracts.ncl" in
{
  mode = "local-read-write",
  admitted_compiler_families = ["x86_64-linux-musl-*", "stage-tinycc", "stage-gcc"],
  driver_path = "/declared/cc-driver",
  compiler_path = "/declared/compiler",
  socket_path = "/private/cc.sock",
  receipt_path = "/private/receipt.json",
  driver_digest_blake3 = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
  compiler_digest_blake3 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  platform_digest = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  ambient_path_mutation = true,
  unclassified = "forward-exact-arguments",
  daemon_unavailable = "forward-exact-arguments",
} | Contracts.DriverShape | Contracts.DriverPolicy
