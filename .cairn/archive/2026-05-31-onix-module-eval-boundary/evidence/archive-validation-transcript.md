# Archive validation transcript

Task-ID: archive-validation
Covers: r[verification_evidence.proof_before_claim] r[verification_evidence.proof_before_claim.scenario.post-archive-validation]

## Archive note

`cairn archive onix-module-eval-boundary --root . --execute` created `cairn/archive/1970-01-01-onix-module-eval-boundary`. Per repo guidance, it was manually renamed to `cairn/archive/2026-05-31-onix-module-eval-boundary`. The archived build-tool-boundary delta was also synced into `cairn/specs/build-tool-boundary/spec.md` before validation.

## Post-archive validation

Command:

```sh
/nix/store/bs92xsdsf6a8bfkrlfc6ryisqh0vx8j8-cairn-0.1.0/bin/cairn validate --root .
```

Output:

```json
{
  "change_issues": [],
  "changes": 0,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 3,
  "valid": true
}
```

## Review-fix pre-commit validation

After resolving the final dirty-state checkpoint and updating the Onix archive reference, validation was rerun. The status output in this block is intentionally pre-commit and therefore shows the evidence files dirty.

Command:

```sh
/nix/store/bs92xsdsf6a8bfkrlfc6ryisqh0vx8j8-cairn-0.1.0/bin/cairn validate --root .
git diff --check
git status --short --branch
```

Output:

```text
{
  "change_issues": [],
  "changes": 0,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 3,
  "valid": true
}
## main...origin/main [ahead 3]
 M cairn/archive/2026-05-31-onix-module-eval-boundary/evidence/archive-validation-transcript.md
 M cairn/archive/2026-05-31-onix-module-eval-boundary/evidence/onix-lowering-validation.md
 M cairn/archive/2026-05-31-onix-module-eval-boundary/evidence/oracle-checkpoints.md
 M cairn/archive/2026-05-31-onix-module-eval-boundary/tasks.md
```

## Review-fix post-commit status

Command, run before this evidence-only correction:

```sh
git rev-parse --short HEAD
git status --short --branch
```

Output:

```text
b4c79963
## main...origin/main [ahead 4]
```
