# I2 derivation audit evidence

Task-ID: I2
Covers: bootstrap.part.diffutils.2.7

- Audited `bootstrap/diffutils-2.7-musl.ncl` source pin, declared predecessor imports, installed wrapper entrypoints, and output/smoke checks.
- Intentional Crunch deviation: this early bridge currently installs BusyBox-backed `diff`/`cmp` wrappers while preserving the GNU diffutils source pin/order for the live-bootstrap step.
- Required failure semantics: installed wrapper entrypoints and both equal/different-file smoke paths must pass before the output contract is satisfied.
