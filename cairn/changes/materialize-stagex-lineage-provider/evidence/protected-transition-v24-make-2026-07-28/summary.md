# Protected StageX transition through GNU Make 3.82

## Result

A fresh protected transition completed from the audited 229-byte hex0 seed through source-built GNU Make 3.82.

- Scratch: `/home/brittonr/.cargo-target/stagex-protected-transition-v24-make-20260728`
- Test task: pueue `2214`
- Test result: `ok. 1 passed; 0 failed; 0 ignored`
- Runtime: 1401.13 seconds
- Report status: `complete`
- Plan BLAKE3: `24f0847cc377e33df98597b6bc419dd223119fea60e1cab7bc143b9c483fe376`
- Manifest BLAKE3: `b240a5e1e348e22dc2cbdfebc72e0832348901d478ed5872234d33568d527f99`
- Source-bundle manifest BLAKE3: `7e93ccc7a29bacc1da6f75c10ae90655ee83afd0d95ca282c8d270c225372f45`
- Source-state BLAKE3: `05f921923c8717c98022a14c44f00ad2ed25e5f2f7a9ea5b74f4cb8b9f33d63f`
- Planned stages: 38
- Allowed protected execution events: 531
- Denied protected execution events: 0
- Fallback events: 0

## GNU Make boundary

The transition authenticated GNU Make 3.82 source identity `fixed-url-ae11d5ec6f5d6b01fdeeb081181a8a51ad8208ab2d8b651cd2d8383f8fcda3f0` with content BLAKE3 `b768ff74b8f16f6c41fb869833582c7cf614e98a8728a1c74616a6b6157ece77`.

The bounded Rust shell applied the checked compatibility patch, compiled 27 source files with the protected TinyCC 0.9.27 executable, linked GNU Make, and built a source-bound recipe runner. The protected smoke boundary checked version output, dispatched a real recipe, and rejected malformed Makefile input.

- Patched source BLAKE3: `32f6a43a61aeb01f8f5c95739f1d67eae26c90ebf1fdf658ce7ab52bf1d147e9`
- `make-3.82`: `8a5aa115eab6069a8f69cf1b2c63c69c478d64b271bc13fda7ea4e3531b841d1`
- `make-recipe-runner`: `028ececbb67a3394595027c9273a44ed2ba5b465c209767589b5a989032b93d1`
- Recipe smoke output: `8dd632d8df367d6751cc2e2497e8d9c38e0c5a1e58b100d59e1c0fdac235e3a2`

## Non-claim and next blocker

This evidence proves the bounded protected lineage through GNU Make 3.82 only. It does not prove a general shell, the later GNU and musl toolchain, normalized StageX provider admission, fixed-point reconstruction, or parity promotion.

The current source bundle does not contain an authenticated GNU patch 2.5.9 record. The next exact step is to export and bind that fixed source, then materialize GNU patch under the same closed protected authority.
