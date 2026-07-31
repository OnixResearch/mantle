# Exact logical identity reserved for the freshly executed StageX transition.
#
# The source-built fixed-point proof starts with an empty physical store,
# executes the protected transition, validates its report and audit, and only
# then adopts this transition tree into fresh local state. The path does not
# authorize imported transition bytes or a cache hit.

let { StorePath, .. } = import "contracts.ncl" in

"/mantle/store/ki5gkg5d6si77dl5k4mav4s6x9s8l25r-mantle-stagex-transition" | StorePath
