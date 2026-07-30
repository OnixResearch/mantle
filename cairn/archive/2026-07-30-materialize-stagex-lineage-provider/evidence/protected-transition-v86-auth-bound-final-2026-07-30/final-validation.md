# Final StageX provider validation

## Passed checks

- Pueue task `4180`: committed-source protected transition. Result: 1 passed, 0 failed, in 1546.31 seconds.
- Pueue task `4277`: two provider publications and `diff -qr`. Result: passed with identical trees.
- Pueue task `4281`: existing-destination rejection. Result: expected exit status 3.
- Focused provider tests: 23 passed, 0 failed.
- StageX source-root tests: 4 passed, 0 failed.
- `nix develop -c cargo test -p mantle --test bootstrap_eval -- --nocapture`: 30 passed, 0 failed.
- `nix develop -c cargo test -p mantle --test bootstrap_parity_cli -- --nocapture`: 19 passed, 0 failed.
- Pueue task `4299`: `cargo fmt --check -p mantle -v`, strict first-party Clippy, serial binary tests, and `git diff --check`. The binary tests passed 1,886 tests with 0 failures and 61 ignored tests.
- Changed lineage source-pin audit: 1 file, 0 fetch blocks, 0 issues.
- Policy-compatible Cairn validation: `valid: true`.
- Cairn proposal gate: `PASS`, receipt `eab28671b3e59d0f0ad6c167e064fca0237806d540a6a647f0b27d4af8e5b30f`.
- Cairn design gate: `PASS`, receipt `0ea2a775b4e8049715628a50ec2870a966e8818da8dab95997ed198d9d73cbcf`.
- Cairn tasks gate: `PASS`, receipt `1ab6ce5a913cd7f60686b1af88289ff78bc00db31ae5223785b560532da3481c`.
- Cairn sync: executed, receipt `10bdd1ebc4b4bed65dd5499e9f94fa6728351b8bcb1053a6987113245ee23ea3`.
- Cairn archive: executed, receipt `c1407f8d065dcd552f147645c23d1ed830d601df622e3a84109e32b407c00114`.
- Post-archive Cairn validation: `valid: true`; exact output is in `post-archive-validation.txt`.

The compatible Cairn command source was:

```text
git+file:///home/brittonr/git/OnixResearch/cairn?rev=083e8d60f1be7122f57616447315c63705970d68#cairn
```

## Bounded unrelated blockers

The broad 179-file source-pin scan exits with status 1 for two existing factory expressions:

```text
FAIL  bootstrap/autoconf-gcc-factory.ncl:10: [bad-hash-format/fetchTarball] fetchTarball hash is not SRI format: 'spec.source_hash'
FAIL  bootstrap/automake-gcc-factory.ncl:7: [bad-hash-format/fetchTarball] fetchTarball hash is not SRI format: 'spec.source_hash'
source-pin audit: 179 files, 148 fetch blocks, 2 issues
```

Current Cairn exits before lifecycle validation because Mantle's generated policy predates a new required field:

```text
error: failed to parse policy cairn-policy/generated/cairn-policy.json: policy missing field nominal_identity_policy; diagnostic=policy-refresh-required; missing_field=nominal_identity_policy
```

The source-pin factory expressions and the Cairn policy-schema refresh are outside this StageX provider change. Their exact transcripts are preserved in this evidence directory.
