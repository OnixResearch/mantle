# Construct a fixed fetcher from an explicit absolute file URL.
# The runbook materializes the checkout-specific URL in a scratch build root.
let mantle = import "../../../lib/lib.ncl" in

fun source_url => mantle.fetchurl {
  url = source_url,
  hash = "sha256-oi30yn6lrWqckP9gGVy/BNgDCI7nRC77ORXuqz3kR30=",
  name = "offline-source-bundle-payload.txt",
}
