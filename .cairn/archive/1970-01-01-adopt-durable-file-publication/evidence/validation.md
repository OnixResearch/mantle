# Validation evidence

## Bounded subject

Mantle adopts `durable-file-publication` for immutable remote-attempt segment and anchor files only.

- Radicle RID: `rad:z3tAR4For7qw8ZirkJzoDw1VNDDLM`
- Reviewed revision: `951c27f59003cea9bfdb40ed4d89653d50fada1f`
- Producer source archive BLAKE3: `2c1d8b5adc8d7384f48a6f8336165e38c3eb196337ebbd66707e157a64b63210`
- Producer archive receipt: `3a11ed34c922a32678f4e5e72bd9ea48b3e3d0eba35edf0c821c27f5a7920fe4`
- Mantle baseline: `7875ec1c8b80662f183b76194ae4ef8e3cd52a28`
- Baseline focused tests: 8 passed
- Adoption evidence BLAKE3: `58346d94254de9e3d80e3974296804ec42b936b3db35c68c8f8f4e1e044ab5dc`

Cargo and Nix use the governed read-only HTTPS adapter at the exact revision. The Cargo and Nix source identities, package, RID, license, revision, and Nix `narHash` agree.

## Executed behavior

The shared production backend publishes immutable segment and anchor files relative to one no-follow opened parent. It uses no-replace, required durability, mode `0600`, bounded stage names, exact payload bounds, and post-rename parent synchronization.

Mantle maps every producer disposition. It keeps `CommittedDurabilityUnknown` separate from uncommitted failure and durable success. Existing destinations use Mantle's bounded no-follow exact-byte comparison.

The legacy backend remains available for an explicit code rollback. Production does not fall back automatically.

## Validation results

| Check | Result |
|---|---|
| Baseline focused remote-attempt tests | PASS, 8 tests |
| Post-change focused remote-attempt tests | PASS, 14 tests |
| Producer conformance corpus replay | PASS, 17 cases |
| Positive and negative Linux capability tests | PASS |
| `cargo fmt --check -p mantle` | PASS |
| Focused Clippy with `--no-deps` and `-D warnings` | PASS; a pre-existing vendored `snix-castore` dead-code warning remains outside the focused lint boundary |
| `scripts/check-first-party-tigerstyle.sh` | PASS |
| Nickel type checks and positive/negative validator tests | PASS |
| `checks.x86_64-linux.durable-file-publication-adoption` | PASS |
| Focused Tracey profile | PASS, 7 of 7 requirements; pre-sync receipt `e9fd7299d1aefecda9ce27a1780e1f2687d1805c1024e400f9c58371bb0a3904`; accepted-spec receipt `1e079e4436835cee4afdaca718b13d5a29bb5c04a04adc7ee11489b79fef177a` |
| Full Mantle binary test suite | PASS, 1,664 tests with one test thread; transient parallel process-fixture failures passed in isolation |
| Nix format, Clippy, Tiger Style, and adoption checks | PASS |
| `checks.x86_64-linux.crunch` | BLOCKED by the pre-existing source filter omission for tracked `fixtures/content-bound-requirements/mantle-registry.json` and `mantle-requirement-ref.json` |
| Broad `nix flake check -L` | BLOCKED by unrelated `checks.x86_64-linux.bootstrap-blocker-inventory`: 76 existing findings, 355 evidence-backed suppressions, and 0 promotion claims |
| Cairn validation and proposal/design/tasks gates | PASS |
| Cairn synchronization | PASS; receipt `9d353db84adc4c82a4fe03fc2e5ffd76852d541427c97c2da51bdcabf0e6f65e` |
| Cairn archive | PASS; receipt `2c1d05546a92f2db479d13d8d44d14d86b29f3c850b1a14494e56151d4efa2f6` |

## Current-main integration

Integration on 2026-08-02 retained the producer revision and adoption boundary. It refreshed the Cargo and Nix file identities after remote credential integration.

The current branch passed the eight-test baseline, the fourteen focused publication tests, remote credential migration tests, formatting, focused Clippy, Nickel positive and negative tests, the targeted Nix adoption check, and Cairn validation.

## Authority boundary

Mantle retains canonical JSON, object identity, limits, content equivalence, manifest replacement, chain validation, retention, deletion, receipts, diagnostics, retry policy, and release policy. The dependency does not receive build, cache, release, deployment, retention, garbage-collection, or deletion authority.

## Non-claims

This evidence does not prove producer source correctness, whole-Mantle correctness, remote-attempt semantic correctness, release eligibility, Android execution, other operating-system support, reboot recovery, retention, garbage collection, or deletion authority. It does not adopt the shared mechanism for replaceable manifests or release-bundle directories.
