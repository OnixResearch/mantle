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

# TODO: wire `seed-full.ncl` once Phase 7 lands the from-source chain.
# For now, unconditionally use the legacy seed.
legacy
