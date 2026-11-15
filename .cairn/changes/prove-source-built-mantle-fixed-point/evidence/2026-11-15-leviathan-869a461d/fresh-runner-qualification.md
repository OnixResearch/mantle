# Corrected frozen runner qualification

## Accepted subject

Product code `869a461d6170404e752865a2029092391240b741` was qualified on
Leviathan from the assembled source snapshot
`f1c0468c177aa6ccc0cfed3ea950726123f9528cce0b9e6b7c18229e607fa911`.
Both qualification wrappers verified that source identity before and after
their commands. Each used a separately extracted source and a previously
absent Cargo target, with compiler-wrapper and cached-artifact environment
variables cleared.

| Qualification | Pueue task | Actual result |
|---|---:|---|
| Frozen runner, release-mode source-built tests, formatting, strict Clippy, release build | 1619 | Success; exact source-built result below |
| Independent canonical CLI, receipt, closure, machine-contract, release-core and Cairn gates | 1620 | Success; individual results in `canonical-completed.json` |

The captured source-built result is:

```text
test result: ok. 268 passed; 0 failed; 27 ignored; 0 measured; 2680 filtered out; finished in 5.16s
```

The installed runner and the binary in the fresh release target were
independently measured equal at BLAKE3
`ef890d46746969a4010632ea28ad138fa916e6f832007b468e12530007cd7f8b`.
The installed path is
`/home/brittonr/sbfp-20261113-berdKH/bin/mantle-869a461d-request-workspace`.

## Evidence and next gate

`fresh-runner-qualified.json` binds actual task results, complete-source
metadata, source-before/after values, installed/fresh-target hashes, and
transferred log checksums. `runner-qualification.log` and
`canonical-qualification.log` retain actual command output;
`canonical-completed.json` retains every selected `test result:` line and the
final successful source binding.

After inspecting those results, the parent released only the pre-existing
source-only preparation task 1621. `preparation-released.json` records its
exact command and state transition. Its output still requires actual complete
preparation results, both source-verifier results, profile transport parity and
new-subject graph/physical reviews under `review-contract.md`.

## Claim boundary and repeatability

This qualifies a runner built by the declared development toolchain and proves
execution of the recorded focused/canonical gates. It does not establish a
source-built compiler/provider, protected execution, fixed-point equality,
reproducibility, release eligibility or signed native-row acceptance. Ignored
tests are not executed proof evidence. The rejected older cohort remains
unlaunched; no earlier counts, runner, manifest or review verdict are adopted.

Mantle's repository-owned qualification/preparation scripts own this repeatable
source/runner gate. The current consumer is the source-built fixed-point proof;
its durable capability is an explicit, reproducible absent-target qualification
boundary before source preparation, separate from protected-proof authority.
