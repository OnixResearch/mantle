# Evidence: pre-change Snix baseline probes (2026-09-30)

Reported by the Mantle architecture worker. I recorded it here and did not
rerun it. This is partial input to T1.1 and does not complete it. T1.1 still
needs goldens for store paths, NAR SHA-256, signed PathInfo, GC plan
identities, store JSON reports, construction sites, and launchers.

Worktree: pristine detached HEAD `7ec5177718a6950297e04eb4eb957a10b02e23ce`
at `/tmp/mantle-snix-baseline-90404`. `git status --porcelain=v1` was empty.

```text
nix develop --offline --accept-flake-config --command cargo test --locked --offline -p crunch-store gc::tests::directory_outputs_survive_reopen_and_gc -- --exact --nocapture
  running 1 test
  test gc::tests::directory_outputs_survive_reopen_and_gc ... ok
  1 passed; 0 failed; 356 filtered out

nix develop --offline --accept-flake-config --command cargo test --locked --offline -p crunch-store overlay::tests::identity_record_rejects_prefix_drift -- --exact --nocapture
  running 1 test
  test overlay::tests::identity_record_rejects_prefix_drift ... ok
  1 passed; 0 failed; 356 filtered out
```

An earlier probe used a bare test name with `--exact` and ran zero tests. It
is not evidence.

Comparison rule for the post-change run: compare only consumer-visible
identity and GC facts, meaning identity-record acceptance or rejection,
retained outputs after reopen, and GC candidates. Do not compare incidental
wording.

Later work on 2026-10-04 recorded an actual signed historical fixture and
its exact prechange CLI stdout, PathInfo bytes, NAR hashes, GC identities,
fresh identity bytes, caller inventory, and launchers under
`prechange-snix-golden-2026-10-04.{md,json}`, `prechange-snix-capture.rs`,
and `finish-inventory-2026-10-04.md`. This note remains the scope statement
for the original *partial* 2026-09-30 probes; it is not the golden artifact.
