# Exact logical identity reserved for the freshly constructed StageX provider.
#
# The source-built fixed-point proof starts with an empty physical store,
# publishes the protected StageX provider at this path, validates it, and only
# then adopts it into fresh local state. This path does not authorize imported
# provider bytes or a cache hit.

let { StorePath, .. } = import "contracts.ncl" in

"/mantle/store/snzd91n8dv6l21xa89vml67229n9svkg-mantle-stagex-intermediate-provider" | StorePath
