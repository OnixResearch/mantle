## Why

`gcc.4.0` remains one of Mantle's central bootstrap trust blockers. The current evidence is deliberately partial: installed `cc1` has several bounded no-TinyCC-delegation semantic slices, but the native source-build path still falls back to the pass1 bridge and carries explicit frontier markers.

The next useful increment is not a broad GCC repair. It is a bounded native `cc1` source-frontier reduction that names one exact frontier, proves whether the frontier moves, and preserves the partial-only parity classification.

## What Changes

- Select one current GCC 4.0 native `cc1` source frontier from the checked boundary evidence.
- Add a focused diagnostic or derivation probe that attempts to move that frontier without broadening the pass1 bridge.
- Update the native-boundary receipt with the prior frontier, attempted probe, observed result, exact markers, and retirement condition.
- Keep `gcc.4.0` partial and blocked for live-bootstrap, Guix, and StageX until full native source-build/compiler-correctness evidence exists.

## Impact

- **Files**: likely `bootstrap/gcc-4.0.ncl`, `bootstrap/diag-gcc40-c-parse-boundary.ncl`, `bootstrap/evidence/gcc-4.0-native-cc1-build-frontier.json`, `bootstrap/evidence/gcc-4.0-native-boundary.json`, bootstrap parity checks, and lifecycle evidence.
- **Behavior**: only evidence and diagnostics for the selected frontier change; no full GCC correctness claim.
- **Testing**: focused GCC parity tests, source-frontier receipt checks, blocker inventory, and Cairn validation/gates.
