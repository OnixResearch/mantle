# Protected StageX transition and intermediate provider v86

## Decision

Accepted for the bounded intermediate StageX provider claim.

This evidence does not admit final native GCC. It does not prove compiler correctness, successful behavior for every protected exec decision, complete musl or binutils behavior, Mantle self-build completion, release reproducibility, or release eligibility.

## Committed-source transition

Pueue task `4180` ran from commit `bfc4c14d265c3284bc77e5950e6d04979b0befbd` with the exact command in `full-transition-test.log`.

```text
test stagex_transition::tests::protected_transition_reproduces_seed_and_builds_kaem ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1946 filtered out; finished in 1546.31s
```

The retained transition has:

- status `complete`;
- 97 planned and completed stages;
- source-state BLAKE3 `18b54c3ac3fdd7931453a849f15c76ea642d248d2b597b1b8c7849ae97f4a772`;
- manifest BLAKE3 `4ad1f3b14dfa219fc64faf752dd0050e678f293af7828a65262f510cc4228e2a`;
- canonical plan BLAKE3 `a303fdfd6e5d86bdb701c806bd162502562b19e43984f6b223255a95f2413a31`;
- 76,576 allowed protected exec decisions;
- zero denied decisions;
- zero non-protected decisions;
- zero fallback events;
- no missing declared path-plus-digest authorization bindings;
- nine binutils configure classes;
- eight binutils components;
- four deterministic archives;
- 11 installed binutils tools;
- 4,891 protected sed invocations.

`authorization-binding-report.json` records the closed authorization review. Each complete stage authorization now has an allowed protected `execve` or `execveat` decision with the same absolute path and BLAKE3 digest identity. The 24 unused permissions found after v85 are absent from the v86 plan.

## Provider publication

Pueue task `4277` published from the same explicit transition root to two absent absolute destinations:

- `/home/brittonr/.cargo-target/stagex-intermediate-provider-v86-a-20260730`
- `/home/brittonr/.cargo-target/stagex-intermediate-provider-v86-b-20260730`

Both publications completed independent pre-rename and post-rename validation. `diff -qr` reported no differences. Both provider trees have NAR BLAKE3 `3c5a56caaab6c524117771caab609456751b1e3963698c7d1b3aa0bf7fa8c601`.

The complete receipt records:

- 97 complete stage reports;
- four exact provider roles;
- stage-graph BLAKE3 `71b288859a9b05d624f55ccd1015a51b3724c2aa200c2277766783cd2e5dc33f`;
- normalized provider BLAKE3 `37d33a89003cfd30d78052f20a1d3957ec8908038924ea27afe0ac79f698e615`;
- output BLAKE3 `6f6e9d6c12e2185a7742aa0741cd6ec886d4bed521541455e6002be661ccd1c9`;
- final-bundle BLAKE3 `f4e2613c993f99fa0a645c587cfca8fec4961ac48d4dc7c841ca68b59ac7dabf`;
- validation-audit BLAKE3 `b895024ed20acd76d7438ca9e2b7a99566e3c2140e040ca16fc77103d683123c`;
- full validation-report BLAKE3 `a42ae56c41f4d3edd64c24bcd40d136852665d3974f2b41188f0aabd7d444cb6`;
- receipt-payload BLAKE3 `21da57d48d7f5d889d9f9f701743411394899668c09e22ff0f3dfe48f12d658d`;
- zero fallback events.

The relocated provider validation has exactly seven allowed events in the required order. It includes promoted TinyCC and binutils smoke executables, positive status `42`, TinyCC negative rejection, and assembler negative rejection.

`bootstrap-parity-report.json` records `seed-full.stagex-lineage` as `complete` with provider kind `stagex-lineage`. Other StageX rows remain separately blocked.

Pueue task `4281` reused the first destination. It recorded CLI exit status `3` and rejected replacement:

```text
error: invalid StageX provider input: provider output must be an absent absolute path
```

## Evidence limitation

`full-transition-test.log` contains the exact command and the retained `pueue_log` result lines. The queue CLI could not export a longer transcript through its separate daemon context. The complete plan, report, 76,576-event audit, inventories, authorization review, publication logs, provider metadata, validation report, receipt, and BLAKE3 manifest remain present for independent inspection.

The v85 evidence remains a diagnostic record of the authorization-binding gap. It is not the final publication authority.

## Artifacts

- `transition-plan.json`
- `transition-report.json`
- `protected-exec-audit.json`
- `authorization-binding-report.json`
- `bootstrap-parity-report.json`
- `binutils-inventory.json`
- `final-validation.md`
- `source-pin-{lineage,full}.log`
- `cairn-validate.txt`
- `cairn-gate-{proposal,design,tasks}.txt`
- `cairn-current-validate.{stdout,stderr}.txt`
- `musl-native-inventory.json`
- `tcc-musl-selfhost-inventory.json`
- `provider.json`
- `stagex-provider-validation.json`
- `stagex-lineage-receipt.json`
- `blake3sums.txt`
- `full-transition-test.log`
- `publication-{a,b}.{stdout,stderr}.txt`
- `collision.{stdout,stderr}.txt`
