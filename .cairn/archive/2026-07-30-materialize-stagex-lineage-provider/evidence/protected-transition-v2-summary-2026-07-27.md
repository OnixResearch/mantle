# Protected StageX transition v2 — 2026-07-27

## Result

Mantle completed the first real protected StageX transition slice.

The slice is not a normalized StageX provider. It does not complete or replace `bootstrap/evidence/stagex-lineage-provider-receipt.json`.

## Bound inputs

- Audited seed: `bootstrap/seeds/AMD64/hex0-seed`
- Seed bytes: 229
- Seed BLAKE3: `cf21608d883b8bdcc1fa6438703630f2fa496cf74d483ce351f876c0656ecf80`
- Annotated hex0 source BLAKE3: `0fb23576a10b41df29c165e39514f18c411da96ae71873a6a2e2f0b1d94de614`
- Minimal kaem source BLAKE3: `5a56b4164dca4d1e03ba35bc8ce4b418a0de2baf1cb9e6a0912607e26532d30d`
- Typed manifest: `bootstrap/stagex-transition-lineage.ncl`
- Generated runtime manifest: `bootstrap/stagex-transition-lineage.json`
- Runtime manifest BLAKE3: `6d47a1abe629cf7cd814518a3a85b621951585ad61a6ffc6801fc50445ceaa9d`

## Observed protected stages

1. The audited seed reproduced `hex0` from the annotated source.
2. Reproduced `hex0` built `kaem-0` from `kaem-minimal.hex0`.
3. Produced `kaem-0` executed an empty, source-state-bound smoke script.

Observed output identities:

- Reproduced hex0 BLAKE3: `cf21608d883b8bdcc1fa6438703630f2fa496cf74d483ce351f876c0656ecf80`
- Produced kaem-0 BLAKE3: `edc664d028b349824cc8c66030a8883b81bdc03d07bc871e44e60ecb4311ab2a`
- Smoke observation BLAKE3: `aedf8bf945950d23c46115e03fe101ad7d08108ef023916613b904e90290721d`
- Source-state BLAKE3: `cccf672912bb776e96883fdeafbdb229c771c5f3904aad99a71c32890bde248d`
- Plan BLAKE3: `5b271968bb586d9cde605547641b72f5f434127eac369823b84677fddc818bb8`

The seccomp audit contains exactly three allowed `execve` decisions. Each decision binds an absolute path, BLAKE3, and source-stage inventory ID.

The report contains `fallback_events: []`. The retained files contain no `/bin/sh`, BusyBox, bwrap, Nix-store, or rustup executable event.

## Limits

- Stage count: 3
- Jobs per stage: 1
- Timeout per executable: 30,000 ms
- Generated executable bound: 4 MiB
- Seed audit bound: 4 KiB
- Generated launch attempts: 16
- Retry delay: 20 ms

Generated executable retries accept only raw Linux `ETXTBSY`, or exit 126 with stderr containing `Text file busy`.

## Validation

- Pueue task `1397` ran the retained protected transition v2 test. Result: one passed, zero failed.
- Pueue task `1352` ran `crunch-bootstrap-core`. Result: 79 passed, zero failed.
- Pueue task `1353` ran the StageX seed-policy tests. Result: 3 passed, zero failed.
- Pueue task `1354` ran all StageX transition tests. Result: 4 passed, zero failed.
- The generated runtime manifest passed `validate_stagex_lineage_manifest` before seccomp installation.
- `protected-transition-v2-2026-07-27/blake3sums.txt` binds every retained transition artifact.
- The shell reads, validates, and hashes the runtime manifest itself.
- Promotion authority is derived from the validated runtime plan.

## Remaining blocker

The existing post-kaem derivations still use host `/bin/sh`, BusyBox, bwrap, or ambient store discovery.

Mantle must continue from the produced `kaem-0` through Stage0, Mes, TinyCC, and the normalized native provider. That continuation must keep the protected allowlist and observed per-stage reports.
