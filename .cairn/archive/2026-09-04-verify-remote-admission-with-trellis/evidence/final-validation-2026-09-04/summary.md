# Final Trellis remote-admission validation

Date: 2026-09-04

## Passing evidence

- `crunch-build` remote-attempt tests passed: 38 passed and 0 failed.
- `crunch-release-core` Trellis tests passed: 9 passed and 0 failed.
- Root remote-attempt tests passed: 14 passed and 0 failed.
- Root gateway tests passed: 20 passed and 0 failed.
- The broad root remote set passed serially: 301 passed and 0 failed.
- The evidence integration test passed: 2 passed and 0 failed.
- The evidence checker passed its positive and mutation self-tests.
- Machine contracts passed with 33 contracted and 67 classified surfaces.
- Nickel typecheck and deterministic JSON export comparison passed.
- `crunch-release-core` passed the `wasm32-unknown-unknown` check.
- Strict Clippy passed for `crunch-build` and `crunch-release-core`.
- The documented strict first-party root Clippy command passed.
- Formatting and `git diff --check --cached` passed.
- The pinned Tiger Style Nix gate passed in `../validation-2026-09-04/tigerstyle-pinned.attempt6.log`.
- The three focused Nix checks passed.
- `nix flake check --no-build -L --option secret-key-files ''` passed.
- Cairn validation found no issues or findings.
- Proposal, design, and tasks gates passed.
- Tracey reported 157 of 157 accepted requirements referenced before spec sync.
- The archived Trellis evidence records 23 focused Verus obligations, 17 focused tests, 10,992 full obligations, and 5,118 full tests.

The focused Nix checks were:

- `trellis-remote-admission-evidence`;
- `trellis-remote-admission`;
- `trellis-remote-admission-evidence-integration`.

## Bounded rerun

The first broad parallel root run had one lock-contention failure. It passed 300 tests before that failure. The failed test then passed in an exact isolated rerun. The complete 301-test set also passed with one test thread. The failed run remains in `root-remote-tests.log`.

## Strict gate repair

Tiger Style attempts 1 through 5 remain as failed evidence. They found real assertion, interface, sentinel, and naming defects. The final repair also removed latent gateway defects from commit `b6a22a2f`. Gateway behavior stayed covered by its positive and negative tests.

## Tool blockers

The installed Octet runs completed with warning-only status:

- `crunch-build`: 1,034 findings, 1,034 warnings, and 0 errors;
- `crunch-release-core`: 893 findings, 893 warnings, and 0 errors.

No baseline, warning budget, or disabled lint was added.

The immutable pinned Octet run could not start. Nix rejected this store import:

- specified: `sha256-q/gu/3mAuLgNfJlxV/Sw1jttbi4PIBjN+XH0bGmB5NQ=`;
- received: `sha256-WTRv7eyiu+VOfb8+90cALNJrUa3uLwRFIaeEr+tAIjQ=`.

`cargo deny check` could not run because `/home/brittonr/.cargo/bin/cargo-deny` does not exist.

## Bound identities

- Trellis revision: `8de4b24aa2d66cc2e6ec966d686df023492265d3`.
- Trellis source archive BLAKE3: `e13e9f71da4964ab4d4f04d9c29525b20f0e778997da5ada7b6caa0f1711f56c`.
- Oracle BLAKE3: `bd1e98e26dbd43f44189941c1eac22bd63139d031c2ea328f0c2d5ce8582aa32`.
- Oracle cases: 6,720 total, 5,882 supported, and 838 rejected.
- Kamacite canonical BLAKE3: `85316de03248e110837ad4c88c77177aa6cea554c8e8cfbb31cec3c4615406f3`.
- Kamacite projection BLAKE3: `c94a6ec0251e9b01a9e5c07765f89422cb755c593003782f5abce361ba0e0707`.
- Valence artifact BLAKE3: `744fdec501178acf4513386801469f54b2f425aa7c5965709ce54f90b4e78ca4`.
- Valence receipt BLAKE3: `3a0ebbeb89fdb5ad91516b6fa4d69d758d4b8811905f61340b34e02925ed411f`.

## Claim boundary

This evidence proves only the named abstract safety properties and recorded projection parity. It does not prove implementation equivalence, persistence, transport, cryptography, worker correctness, liveness, or release eligibility.
