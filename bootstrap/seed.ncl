# Bootstrap seed selector.
#
# Delegates to either the full-source live-bootstrap chain or the legacy
# musl.cc-fetched seed depending on `CRUNCH_LEGACY_SEED`.
#
# Once the full-source chain is validated (selftest + integration-test +
# self-build all pass), the legacy path can be removed.
#
# The public contract is unchanged: callers get { name, target,
# dynamic_linker, toolchain, provider }.

let legacy = import "seed-legacy.ncl" in

# The full-source chain is not yet functional (placeholder derivations
# still exit with errors). Default to legacy until the chain is complete.
#
# TODO: once seed-full.ncl builds successfully, switch the default:
#   let full = import "seed-full.ncl" in
#   full
legacy
