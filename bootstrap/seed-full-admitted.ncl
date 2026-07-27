# Exact logical identity of the independently admitted full-source provider.
#
# Update this input only with a fresh `mantle-full-source-provider-admission-v2`
# report whose observed and expected provider-tree BLAKE3 values agree.

let { StorePath, .. } = import "contracts.ncl" in

"/mantle/store/q366f9q6n1silgmv8w0c2ypxxw4p4yw3-full-source-seed-toolchain" | StorePath
