# The full reviewed table changes independently of the source bundle. Project its
# selected rows through selected-table.ncl before passing them to mantle.ncl.
let mantle = import "../../lib/lib.ncl" in
fun input => mantle.fetchurl {
  name = "lock-vendor-reviewed-full-table",
  url = input.url,
  hash = input.sha256,
}
