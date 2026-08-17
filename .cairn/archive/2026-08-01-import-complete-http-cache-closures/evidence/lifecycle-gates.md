# Lifecycle gate status

Date: 2026-08-01

## Policy selection

Mantle's checked-in generated policy predates Cairn's required `nominal_identity_policy` field. The active `extend-nominal-types-to-trust-boundaries` change owns that refresh.

This change used the current sibling Cairn policy explicitly:

```text
/home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json
policy_hash: 860fcf019180b030d855e90c3475a102dc6cdeb4aeb5214badeb84e0dd95145f
```

This is the same bounded workaround recorded by the nominal-types change. This change does not modify Cairn policy files.

## Read-only lifecycle gates

Pueue tasks `7153` through `7157` ran these commands with the explicit policy:

```text
cairn validate --root .
cairn gate proposal import-complete-http-cache-closures --root .
cairn gate design import-complete-http-cache-closures --root .
cairn gate tasks import-complete-http-cache-closures --root .
cairn tracey coverage --root .
```

Results:

```text
validate: valid=true
proposal: valid=true, verdict=PASS
design: valid=true, verdict=PASS
tasks: valid=true, verdict=PASS, done=18, todo=1
tracey before sync: valid=false; the active requirement ID was dangling
```

## Accepted-spec sync

Pueue task `7173` executed the reviewed sync plan. It added the requirement to `cairn/specs/cache-substitution/spec.md`.

```text
plan_hash: c671c63325085282ce5adeab6f41fe594f5e15d1378b60e3bd8388a786e47e64
receipt_hash: 7ac94838ad5c53f35eb018ed4b91e7f311f6898b08d1ebb1088cbcbd36cf1210
```

## Tracey after sync

Pueue task `7177` reran broad Tracey coverage after sync.

```text
requirements: 681
referenced: 254
valid: false
receipt_hash: 0a7d8877b2fa5220c19924d32e6ff245c3e3f49362c43a363901fe427df9bc58
```

The broad profile retains unrelated historical missing and dangling IDs. `cache_substitution.complete_http_closure_pull` is absent from both lists and increased the referenced count by one. This evidence does not claim global Tracey coverage is green.

## Final completed-task gates

Pueue tasks `7192` through `7195` reran validation and all three change gates after sync and task completion.

```text
validate: valid=true
proposal: valid=true, verdict=PASS
design: valid=true, verdict=PASS
tasks: valid=true, verdict=PASS, done=19, todo=0
```
