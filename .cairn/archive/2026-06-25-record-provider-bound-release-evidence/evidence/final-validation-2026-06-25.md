# Final validation — provider-bound release evidence

Task-ID: V2
Covers: r[verification_evidence.provider_bound_release_evidence_transcripts]
Date: 2026-06-25

## diff-check

Command:

```sh
git diff --check
```

Exit status: `0`

stdout:

```text

```

stderr:

```text

```

## cairn-validate

Command:

```sh
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
```

Exit status: `0`

stdout:

```text
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 7,
  "valid": true
}

```

stderr:

```text

```

## cairn-gate-proposal

Command:

```sh
nix run path:/home/brittonr/git/cairn#cairn -- gate proposal record-provider-bound-release-evidence --root .
```

Exit status: `0`

stdout:

```text
{
  "change": "record-provider-bound-release-evidence",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "100c5150bd6b96b3c5ab1a4dcd25281833b0eb4ae9d380b731cc9f882f5f837e",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "698ccdfd286ba1618940b569147075a52fbc7b41c381a957e0faf6ec39fc2df7",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}

```

stderr:

```text

```

## cairn-gate-design

Command:

```sh
nix run path:/home/brittonr/git/cairn#cairn -- gate design record-provider-bound-release-evidence --root .
```

Exit status: `0`

stdout:

```text
{
  "change": "record-provider-bound-release-evidence",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "23c1392678f316777fa27bcdb04840a5aa7c0331dc688138d657763f501596d6",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "71b6d031992dd2aff604f5f6d49d97d39b5f24d67d2781e5bc4b512e6e258708",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}

```

stderr:

```text

```

## cairn-gate-tasks

Command:

```sh
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks record-provider-bound-release-evidence --root .
```

Exit status: `0`

stdout:

```text
{
  "change": "record-provider-bound-release-evidence",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "31e11cbf920b8ec1fbcf5b97b8738f82a4d40e041ba53cea5928aac3a0c3d4cc",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "c561bb83a22548d1f5d2e3026c5937e497d938b98a1b46c3233784ceb127fb9b",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

```

stderr:

```text

```

## cairn-gate-tasks-after-completion

Command:

```sh
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks record-provider-bound-release-evidence --root .
```

Exit status: `0`

stdout:

```text
{
  "change": "record-provider-bound-release-evidence",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "dadef73a8f0329e914f4609bb06de99e280b0417f506c0fa1908d90c73c4ecb3",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "e34c283d94397cebde4140f1160cd58fb130b57b4d6f984afc0f500fd2ef45ea",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

```

stderr:

```text

```

## cairn-validate-after-completion

Command:

```sh
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
```

Exit status: `0`

stdout:

```text
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 7,
  "valid": true
}

```

stderr:

```text

```

## post-archive-rename-and-validate

Command:

```sh
nix run path:/home/brittonr/git/cairn#cairn -- archive record-provider-bound-release-evidence --execute --root .
find cairn/archive -maxdepth 1 -type d -name '*record-provider-bound-release-evidence' -print | sort
```

Result: Cairn created `cairn/archive/1970-01-01-record-provider-bound-release-evidence`; this was manually renamed to `cairn/archive/2026-06-25-record-provider-bound-release-evidence` before post-archive validation.

Archive output excerpt:

```text
"kind": "archive"
"receipt_hash": "6dba17f432423541f72700cb27436b7b5d23f559e2c8c4282095d0cc7c717bd4"
cairn/archive/1970-01-01-record-provider-bound-release-evidence
```

Post-archive command:

```sh
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
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
  "specs_validated": 6,
  "valid": true
}
```

## post-archive-manual-spec-sync-validate

Manual sync: copied r[verification_evidence.provider_bound_release_evidence_transcripts] from the archived change delta into `cairn/specs/verification-evidence/spec.md` because archive only moved the change directory.

Command:

```sh
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
```

Exit status: `0`

stdout:

```text
{
  "change_issues": [],
  "changes": 0,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 6,
  "valid": true
}

```

stderr:

```text

```
