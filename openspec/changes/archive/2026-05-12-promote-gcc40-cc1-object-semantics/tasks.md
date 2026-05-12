## Phase 1: GCC 4.0 cc1 object slice

- [x] [serial] Implement bounded installed-`cc1` compile-to-object behavior.
  - Evidence: `bootstrap/gcc-4.0.ncl` installs a `cc1` wrapper that parses bounded GCC-shaped frontend args, honors `-quiet`, `-dumpbase`, `-auxbase`, `-auxbase-strip`, `-o`, selects the first non-option input, and delegates to the validated TinyCC handoff with `-c`.
- [x] [depends:cc1] Add derivation-local and regression evidence for the cc1 object smoke.
  - Evidence: derivation-local smoke invokes `$out/libexec/gcc/x86_64-unknown-linux-musl/4.0.4/cc1 -quiet /tmp/cc1-smoke.c -o /tmp/cc1-smoke.o` and checks non-empty output.
  - Evidence: `bootstrap_parity::tests::gcc40_real_derivation_contains_cc1_object_smoke` covers the installed wrapper and smoke markers.
  - Evidence: built artifact `.crunch-drain/store/fg6rzprfyh3vv9ifjp9k3l55wpifb4zv-gcc-4.0.4` passed direct `cc1` bwrap smoke with the scratch store bound at logical `/crunch/store`.
- [x] [depends:evidence] Run eval/build/parity/OpenSpec checks.
  - Evidence completed 2026-05-12T22:21:13Z:
    - `cargo fmt --check`
    - `cargo test --bin crunch bootstrap_parity::tests -- --nocapture` — 39 passed
    - `./target/debug/crunch eval bootstrap/gcc-4.0.ncl` plus `/bin/sh -n /tmp/gcc40-cc1.sh`
    - `nix-shell -p bubblewrap --run './target/debug/crunch build bootstrap/gcc-4.0.ncl --store "$PWD/.crunch-drain/store" -j 4 --trust-unsigned'`
    - Direct logical `/crunch/store/.../cc1` bwrap object smoke
    - `openspec validate promote-gcc40-cc1-object-semantics --strict`
- [x] [depends:verify] Archive the change, commit, push, and report clean state.
  - Evidence: completed in final archive/commit step.
