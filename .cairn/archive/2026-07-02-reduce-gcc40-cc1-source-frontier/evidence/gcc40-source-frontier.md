# GCC 4.0 source-frontier reduction evidence

Task-ID: V3
Covers: r[gcc40_bridge.source_frontier_reduction]

## Selected frontier

Checked receipt: `bootstrap/evidence/gcc-4.0-native-cc1-build-frontier.json`.

Selected schema: `mantle-gcc40-native-cc1-source-frontier-reduction-v22`.

Prior frontier: v21 recovered the concrete include payload `config.h -> auto-host.h#define_ssize_t` plus `system.h -> stdio.h -> __NEED_ssize_t -> bits/alltypes.h:ssize_t`, while the real `c-parse.o` make target still failed with rc=2 and two truncated include-flood lines.

Attempted probe: comment out the generated `#define ssize_t` line in `gcc/auto-host.h`, run the real `make -C gcc c-parse.o` target, restore `auto-host.h`, and record whether the target advances.

Observed result: `narrowed-stable-blocker`. Disabling only the generated auto-host.h ssize_t definition did not advance the real make frontier. The receipt records `cparse_make_auto_host_ssize_cparse_o_rc=2`, compile command present, object absent, two include-flood lines, and two truncated include-flood lines.

Bounded non-claim: the evidence is diagnostic frontier evidence only and does not prove native GCC 4.0 compiler correctness.

## Positive and negative coverage

Pueue task 1757:

```text
running 4 tests
test bootstrap_parity::tests::gcc40_native_cc1_build_frontier_receipt_missing_fails_closed ... ok
test bootstrap_parity::tests::gcc40_native_cc1_build_frontier_parity_overclaim_fails_closed ... ok
test bootstrap_parity::tests::gcc40_native_cc1_build_frontier_missing_fields_fail_closed ... ok
test bootstrap_parity::tests::gcc40_native_cc1_build_frontier_marker_drift_fails_closed ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 1085 filtered out; finished in 0.00s
```

Pueue task 1748 also ran source-frontier and accepted-partial filters; the captured result lines include:

```text
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 1084 filtered out; finished in 0.00s

test bootstrap_parity::tests::gcc40_placeholder_inventory_matching_receipt_reports_partial ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1088 filtered out; finished in 0.00s
```

## Required checks

Pueue task 1762:

```text
report: /home/brittonr/git/mantle/target/bootstrap-parity-snapshot/latest/report.json
verdict: passed
report-blake3: 7f3f1b60f3c123e7ab7c294e2e039d0d134f0d2ac3860b9350dbb550b650c415
live-bootstrap: complete=false blockers=5
guix: complete=false blockers=6
stagex: complete=false blockers=2
[2/2] bootstrap parity snapshot receipt validation
running: cargo -Zscript scripts/check-bootstrap-parity-snapshot-receipt.rs target/bootstrap-parity-snapshot/latest/receipt.json
bootstrap parity snapshot receipt valid
  report: /home/brittonr/git/mantle/target/bootstrap-parity-snapshot/latest/report.json
  report_blake3: 7f3f1b60f3c123e7ab7c294e2e039d0d134f0d2ac3860b9350dbb550b650c415
  live-bootstrap: blockers=5
  guix: blockers=6
  stagex: blockers=2
bootstrap blocker inventory: 38 findings across 4 classes, 396 evidence-backed suppressions, 0 promotion claims, enforce=false
```

The same task ran `git diff --check` with no findings.

## Cairn lifecycle validation

Pueue task 1786 ran proposal/design/tasks gates plus validation. The tasks gate ended with:

```text
"stage": "tasks",
"valid": true,
"verdict": "PASS"
```

The same task's validation summary ended with:

```text
"change_issues": [],
"issues": [],
"spec_issues": [],
"valid": true
```

Pueue task 1792 dry-ran and executed `cairn sync reduce-gcc40-cc1-source-frontier --execute` with `"reasons": []`; follow-up validation reported `"valid": true` and `"specs_validated": 17`. The accepted requirement landed at `cairn/specs/gcc40-bridge/spec.md` with `r[gcc40_bridge.source_frontier_reduction]`.
