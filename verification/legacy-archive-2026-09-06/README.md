# Checked archive migration evidence

## Checked result

The unchanged desktop Mantle executable imported all eleven selected objects into fresh isolated state. It then reported `11 checked, 0 mismatches`, with `trusted_signatures=1/1` for each object.

- Original archive BLAKE3: `6b1d99205f585513974d15ec70b4ccb28de6b8f025361ad5cb3d99b7d009906b`.
- Migrated archive BLAKE3: `1c940c061ff70c2f15d9558e5890378272241513d1edf74a98c6e996a7b6dab1`.
- Receipt BLAKE3: `eebe0aa572f9fbbfcfcd21766bf560aed8a37591503f338b4da377d46465acbf`.
- Existing desktop Mantle BLAKE3: `3a8e1b46c634cb31f44b8f63ecd9d675dce2a1c236d051b91a938697e2f8f40c`, unchanged.
- Final local helper BLAKE3: `28f3f0b9448a266601df875db5e66c1ade2f4dd39c0a74d0654030b54f28b1be`.

The original source state and exported archive remain unchanged. No production signing key was read by the helper. A new Mantle destination state generated its own private key through normal initialization. That key is not retained here.

## Observations

All eleven objects matched the exact doubled-count reconstruction. Four matched marker-normalized CA: GMP (2 replacements), MPFR (2), MPC (1), and binutils 2.30 (1846). One matched final-NAR CA. Six already had absent CA. No CA field was removed.

Import reported `imported_count = 11`, `skipped_already_present_count = 0`, and `total_payload_bytes = 380573392`. `desktop-import.json` retains every path and final-NAR digest.

The final atomic-publication implementation produced byte-identical archive and receipt files to those imported. `final-replay.log` records the output identities after that replay. The committed source also includes the final Clippy-driven enum boxing change.

## Validation

- First helper baseline: 8 tests passed before metadata round-trip hardening.
- Final helper: 11 tests passed, including negative facts, malformed input, closure completeness, exact field preservation, and concurrent no-replace publication.
- Repair core library: 9 tests passed, including all seven existing repair tests.
- Focused Clippy: all helper targets passed with `-D warnings`.
- Stable rustfmt check: exit 0. It warned that repository nightly-only formatting settings were unavailable.
- Final migration replay: archive and receipt matched the imported versions byte for byte.
- Existing desktop import and independent physical verification: exit 0 with complete counts and signatures.

The helper tests compile through `cargo -Zscript test --offline --manifest-path scripts/migrate-legacy-archive.rs`. Clippy needs `cargo clippy -Zscript`, with the unstable flag after the subcommand. Core tests use `cargo test --offline --locked -p crunch-repair-core --lib`.

The local toolchain was the existing Nix-store Rust/Cargo 1.95.0 bundle. `RUSTC_BOOTSTRAP=1` enabled experimental Cargo-script/core attributes. It did not run a bootstrap graph. Isolated targets kept every compilation away from the installed Mantle binaries. Only the helper, its library dependencies, and library tests were compiled.

## Bounded review

Goal: prove both original identities before representation changes, preserve signed facts, and obtain a complete strict import without rebuilding Mantle or packages.

The correlated single-agent review compared three mechanisms:

| Mechanism | Decision |
|---|---|
| Replace CA, prune references, or bypass signatures | Rejected: changes identity or authority |
| Repair the live source store or rebuild the binary | Rejected: outside this operator boundary |
| Checked offline archive transformation | Accepted after original-node/CA proofs and complete ordinary import |

The audit added rejection of lossy non-node JSON conversion, moved closure decisions into the core, and replaced staged file links with atomic no-replace directory publication. Negative tests cover each boundary. This was not an independent agent review.

Initial estimate: 30–45 minutes. Worktree creation was at 01:10 UTC. Complete desktop verification finished at 01:36 UTC. Final hardening and evidence capture took approximately 50 minutes from worktree creation, plus initial source review. No package build filled that extra time.

The terminal result is validated for this explicit archive, not general package or release admission. Full workspace tests, the whole-repository quality gate, and reproducible helper packaging were not run or claimed.

## Retention and next boundary

Remote root:

`/datapool/mantle-store/proofs/onix-darkhttpd-prebuilt-20260905-1/checked-migration-1`

It contains the migrated `archive` with unchanged NAR payloads, `receipt.json`, `request.json`, fresh `state`, and materialized `store`. Local scratch is retained under `target/legacy-archive-work/` after verification. Earlier raw command paths used `.scratch/` before that relocation.

The helper source belongs to Mantle. Onix consumes this receipt and imported input state, not copied producer implementation. The next step is a new package-only Darkhttpd derivation using pre-existing store-path strings. The historical Stage0 graph remains prohibited and disabled.

No Darkhttpd compilation, compiler bootstrap, Mantle rebuild, VM run, ABI proof, package realization receipt, or release/physical promotion occurred here.
