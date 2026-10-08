# The invocation binds a content-addressed archive URL and reviewed recursive NAR hash.
# No host checkout, binary, or store path is baked into this recipe.
let mantle = import "../../lib/lib.ncl" in
fun input => mantle.fetchTarball {
  name = "lock-vendor-input-bundle",
  url = input.url,
  hash = input.nar_sha256,
}
