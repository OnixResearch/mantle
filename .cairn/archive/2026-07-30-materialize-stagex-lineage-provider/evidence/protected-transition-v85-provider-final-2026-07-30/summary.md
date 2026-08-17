# Protected StageX transition and intermediate provider v85

## Decision

Diagnostic implementation evidence only after the authorization-binding hardening below.

This evidence does not admit final native GCC. It does not prove compiler correctness, complete musl or binutils behavior, Mantle self-build completion, release reproducibility, or release eligibility.

## Transition evidence

Pueue task `3986` ran the exact protected transition command recorded in `full-transition-test.log`.

```text
test stagex_transition::tests::protected_transition_reproduces_seed_and_builds_kaem ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1944 filtered out; finished in 1547.74s
```

The retained transition has:

- status `complete`;
- 97 planned and completed stages;
- source-state BLAKE3 `18b54c3ac3fdd7931453a849f15c76ea642d248d2b597b1b8c7849ae97f4a772`;
- manifest BLAKE3 `4ad1f3b14dfa219fc64faf752dd0050e678f293af7828a65262f510cc4228e2a`;
- canonical plan BLAKE3 `a9f7e3e4d3bca6dcfc3fcdc78a030c36b7ab0f260346e92ac558f19d621e92d8`;
- 76,555 protected execution events;
- zero denied events;
- zero fallback events;
- a 74,002-event binutils suffix within the accepted `[73,991, 74,057]` closure;
- nine binutils configure classes;
- eight binutils components;
- four deterministic archives;
- 11 installed binutils tools;
- 4,891 protected sed invocations.

The canonical Flex executable BLAKE3 is `502324a00e1b35d6fc18a6cf6c3a257d3578b7d5fd8bffc27ce41b9da3c1ebbf`. It closes the process-state section-name drift diagnosed by v83 and the read-mode ordering issue diagnosed by v84.

## Provider publication

Pueue tasks `4062` and `4076` published from the same explicit transition root to two absent absolute destinations:

- `/home/brittonr/.cargo-target/stagex-intermediate-provider-v85-c-20260730`
- `/home/brittonr/.cargo-target/stagex-intermediate-provider-v85-d-20260730`

Both publications completed independent pre-rename and post-rename validation. `diff -qr` and `cmp` reported no differences. After lint-only source cleanup, pueue task `4197` published a third provider with the current tree and `diff -qr` again reported no differences. All provider trees have NAR BLAKE3 `893769d7e234c50d492ceefa4603bb39ee32995eb78a3add8ddb1db725bb3af8`.

The complete receipt records:

- 97 complete stage reports;
- four exact provider roles;
- normalized provider BLAKE3 `37d33a89003cfd30d78052f20a1d3957ec8908038924ea27afe0ac79f698e615`;
- output BLAKE3 `ce00c92751f01d43f2e3396b4e69f0d4db8cd884995e5871d38ce6067a722873`;
- final-bundle BLAKE3 `29c802239208b30a308473d02bcc0ad0b7d79e4279f28d02e9a138a354767fd8`;
- validation-audit BLAKE3 `b895024ed20acd76d7438ca9e2b7a99566e3c2140e040ca16fc77103d683123c`;
- full validation-report BLAKE3 `a42ae56c41f4d3edd64c24bcd40d136852665d3974f2b41188f0aabd7d444cb6`;
- receipt-payload BLAKE3 `c2296197f15d4d988dc073c939583f6acecf48e8f9e03fae92d4330eaeca4fa0`;
- zero fallback events.

The relocated provider validation has exactly seven allowed events in the required order. It includes promoted TinyCC and binutils smoke executables, positive status `42`, TinyCC negative rejection, and assembler negative rejection.

Pueue task `4046` reused an existing destination and recorded CLI exit status `3` with:

```text
error: invalid StageX provider input: provider output must be an absent absolute path
```

## Post-run authorization-binding finding

An adversarial receipt review after publication rejected the weaker declared-set/raw-audit split. The strict check found 24 declared binutils authorization IDs whose exact path-plus-digest identities never appeared in the raw audit. They were seven unused coreutils tools repeated across three binutils stages and three installed binutils tools that the smoke checked by identity but did not execute. `authorization-binding-gap.json` preserves the exact set.

The plan now omits those unused executable authorizations, and complete stage reports require every remaining declared path-plus-digest identity to appear in the protected audit. The v85 transition and publication remain diagnostic implementation evidence. The committed-source v86 transition replaced this checked receipt as the final provider authority.

## Evidence limitation

The original v81 pueue command log was unavailable. Its retained plan, report, audit, inventories, and artifacts remain historical evidence. The v85 transition has a current pueue transcript in this directory.

## Artifacts

- `transition-plan.json`
- `transition-report.json`
- `protected-exec-audit.json`
- `binutils-inventory.json`
- `musl-native-inventory.json`
- `tcc-musl-selfhost-inventory.json`
- `provider.json`
- `stagex-provider-validation.json`
- `stagex-lineage-receipt.json`
- `blake3sums.txt`
- `full-transition-test.log`
- `publication-{c,d,e}.{stdout,stderr}.txt`
- `authorization-binding-gap.json`
