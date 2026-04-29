Task-ID: I2
Covers: bootstrap.part.grep.2.4

# grep 2.4 derivation audit

Checked file: `bootstrap/grep-2.4-musl.ncl`.

Result: PARTIAL. Source pin and intended output names are present, but compile robustness needs V2/V3 proof.

Matches upstream intent:

- Imports `tcc-musl-v2.ncl`, `musl-1.1.24-tcc-musl.ncl`, make, sed, and stage0.
- Fetches GNU grep 2.4 with fixed hash.
- Builds a static `grep` with tcc and musl headers.
- Installs `grep`, `egrep`, and `fgrep` names for downstream scripts.

Current Crunch deviations / risk:

- Uses a hand-written compile loop instead of upstream package build machinery.
- Suppresses individual source compile failures with `2>/dev/null || true`, so V3 must prove enough objects link into a working grep.
- Does not run `configure`; config defines are supplied manually.
