# Initial validation — source-built provider AWS-LC memcmp guard

Task-ID: V4
Covers: r[rust_package_planning.source_built_toolchain_closure.aws_lc_memcmp_guard]
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
nix run path:/home/brittonr/git/cairn#cairn -- gate proposal source-built-provider-aws-lc-memcmp-guard --root .
```

Exit status: `0`

stdout:

```text
{
  "change": "source-built-provider-aws-lc-memcmp-guard",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "100a07e452c20eee703fa0a5a2c69c70b999aa2d6529fa075bcb11e5529f7f9c",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "cacfa0676d274cc9494e06faa5f4747e3baadd77813507e26456a4ff6e25cda8",
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
nix run path:/home/brittonr/git/cairn#cairn -- gate design source-built-provider-aws-lc-memcmp-guard --root .
```

Exit status: `0`

stdout:

```text
{
  "change": "source-built-provider-aws-lc-memcmp-guard",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "a7f7f8c1b76ba2e23a6e16a5395403014e161bb2043202a8599fa66061812a21",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "cb4a150a9ebb6e6bcbb31f68315050942c6cdb8caada8fd485a65f6b2ceb9859",
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
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks source-built-provider-aws-lc-memcmp-guard --root .
```

Exit status: `0`

stdout:

```text
{
  "change": "source-built-provider-aws-lc-memcmp-guard",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "1f0bac234b6263141984d6b15c303464c347b8128fa54edc67bd970b6c46a87b",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "89de5afb4d62ff64260f8bbaccbd10e300501a9cb683b0103a4d21401199b845",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

```

stderr:

```text

```

