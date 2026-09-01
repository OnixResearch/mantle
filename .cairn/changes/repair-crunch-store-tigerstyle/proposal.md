## Why

The strict Tiger Style gate reaches `crunch-store` and reports 183 focused
findings. The prior full-check closeout recorded 139 location-backed findings
because the flake stopped at a narrower target surface. The remaining findings
cover assertion density, function length, quantity names, collection bounds,
compound conditions, and explicit interfaces.

Mantle must repair this debt without lint allowances, scope reductions, warning
baselines, or store-semantic changes. The repair must preserve fail-closed
errors, bounded resource policy, functional-core ownership, shell authority,
and public compatibility.

## What Changes

- Add meaningful entry, exit, and transition assertions around established
  internal invariants. Keep malformed external input on typed error paths.
- Rename quantity-bearing locals with explicit units or counts.
- Reserve bounded collections before loops and keep existing admission limits.
- Decompose compound conditions into named decisions or explicit branches.
- Split long functions into narrow observation, validation, planning, and
  execution helpers without moving policy into the shell.
- Replace ambiguous internal parameter clusters with named input records while
  preserving public call surfaces where compatibility requires them.
- Run the focused `crunch-store` Tiger gate and the repository Tiger check with
  no allowances.

## Impact

- **Files:** `crates/crunch-store/src/` modules named by the baseline, focused
  tests, Tracey bindings, and lifecycle evidence.
- **Testing:** pre-change and post-change `crunch-store` tests, focused Tiger
  checks, repository Tiger check, strict Clippy, formatting, full Nix checks,
  Cairn validation, Tracey coverage, and lifecycle gates.
