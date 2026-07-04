# Remote build e2e operator rail validation

Task-ID: remote-build-e2e-operator-rail
Covers: r[remote_builds.operator_e2e_rail]
Date: 2026-07-04

## Baseline

`pueue task 55` ran before implementation:

```text
Command: nix develop -c cargo test -p mantle --bin mantle remote_build:: -- --nocapture

test result: ok. 86 passed; 0 failed; 0 ignored; 0 measured; 1100 filtered out; finished in 1.01s
```

## Positive and negative operator rail

`pueue task 59`:

```text
Command: nix develop -c cargo test -p mantle --bin mantle operator_remote_build_e2e_rail -- --nocapture

running 2 tests
test remote_build::tests::operator_remote_build_e2e_rail_rejects_cross_seam_failures ... ok
test remote_build::tests::operator_remote_build_e2e_rail_composes_route_stdio_input_admission_and_reports ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1186 filtered out; finished in 0.03s
```

The positive rail composes deterministic remote route planning, framed stdio response validation, input-sync summary, remote execution, signed output admission/import, bounded coordinator status, build observability, artifact-attestation path reporting, redaction, and explicit non-claims.

The negative rail covers missing output trust, remote route output-trust rejection, unframed stdout protocol pollution, stale imported source state, upload privacy rejection, and no-fallback policy for output-import failure.

## Broader focused remote-build regression suite

`pueue task 60`:

```text
Command: nix develop -c cargo test -p mantle --bin mantle remote_build:: -- --nocapture

test result: ok. 88 passed; 0 failed; 0 ignored; 0 measured; 1100 filtered out; finished in 1.01s
```

## Cairn validation and gates

`pueue task 65`:

```text
Command: nix run path:/home/brittonr/git/cairn#cairn -- validate --root .

{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 17,
  "valid": true
}
```

`pueue task 66` proposal gate excerpt:

```text
"stage": "proposal",
"valid": true,
"verdict": "PASS"
```

`pueue task 67` design gate excerpt:

```text
"stage": "design",
"valid": true,
"verdict": "PASS"
```

`pueue task 68` tasks gate excerpt:

```text
"stage": "tasks",
"valid": true,
"verdict": "PASS"
```

After marking V2 checked, `pueue task 70` reran validate plus the tasks gate. Final tasks gate excerpt:

```text
"stage": "tasks",
"valid": true,
"verdict": "PASS"
```
