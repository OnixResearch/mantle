## Contract and controls

- [x] [serial] Change only the interchange library's compatible hash dependency range and preserve owner store pins, features, selected lock, schemas, identities, and admission code. r[mantle.build_interchange.hash_dependency]
- [x] [serial] Add independently locked 1.8.2 and 1.8.7 consumers with original-fixture parity and changed-identity, builder, product, cache, and outcome rejection controls. r[mantle.build_interchange.hash_matrix]
- [x] [serial] Run owner tests, both consumer matrices, scoped Clippy, wasm, Nickel, fixture freshness, and Nix checks; retain exact commands and outputs. r[mantle.build_interchange.hash_matrix]
- [x] [serial] Publish immutable source and verify linkage with Neural Stream's admitted Animus and BLAKE3 1.8.7 without overrides or sibling product dependencies. r[mantle.build_interchange.linked_consumer]
- [x] [serial] Record source-bound verification evidence and review the lifecycle sync/archive plans. r[mantle.build_interchange.hash_dependency] r[mantle.build_interchange.linked_consumer]

## Evidence

`evidence/build-contract-hash-compatibility-2026-09-06.md` records published
source `5884354`, unchanged wire/store boundaries, seven owner tests, seven
controls in each independent hash lane, wasm, Clippy, fixture freshness,
Nickel, and the full pinned Tiger Style check. The exact committed-source
Nix matrix also passed. Neural source `4d0396c` links that published revision
with its unchanged Animus/BLAKE3 cohort, passes 54 core and 26 shell tests,
and passes the full gate after eleven campaign replays. No native executor
or production authority is inferred from those results. The reviewed sync plan
adds only the three hash-compatibility requirements. The initial archive plan
correctly blocked on this final review task. The plans must be refreshed after
recording its completion before lifecycle mutation.
