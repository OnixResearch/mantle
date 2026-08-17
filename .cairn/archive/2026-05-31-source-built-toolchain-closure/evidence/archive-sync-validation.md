# Archive and sync validation

Task-ID: archive
Covers: rust_package_planning.source_built_toolchain_closure

## Pre-archive validation

Command:

```sh
/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn validate --root .
/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn gate tasks source-built-toolchain-closure --root .
```

Output:

```json
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 4,
  "valid": true
}
{
  "change": "source-built-toolchain-closure",
  "input_hash": "3274182c002ce5a5fc3a6bb90be4afb62ce0b70cb5a58de0f917b38eeecb1dfb",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "88d395561e70f9f001c4f05a96aa705da6ee188a3e313f2fa04434857cc48b2d",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Archive dry-run blocker

Initial dry-run correctly blocked while this archive task was still unchecked:

```json
{
  "blocked": true,
  "reasons": [
    "source-built-toolchain-closure: tasks not archive-ready (todo: 1, in_progress: 0, unmarked: 0)"
  ]
}
```

The task was checked only after all implementation, proof, and validation evidence above passed.

## Ready gate

After checking the archive task, tasks gate passed:

```json
{
  "change": "source-built-toolchain-closure",
  "input_hash": "6f6c7b8f8e2b64b34ce0b559e250e556d337b07af420479003fc4e2e1549eb39",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "320cdd671d275fb4a13b505101188f055037185bcd60e0c3b8e8da9b338bb102",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Sync execution

Command:

```sh
/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn sync source-built-toolchain-closure --root . --execute
```

Output summary:

```json
{
  "actions": [
    {
      "description": "merge delta spec into main specs: ./cairn/changes/source-built-toolchain-closure/specs/rust-package-planning/spec.md",
      "kind": "sync_delta_spec",
      "path": "./cairn/changes/source-built-toolchain-closure/specs/rust-package-planning/spec.md"
    }
  ],
  "blocked": false,
  "dry_run": false,
  "mutated": true,
  "receipt_hash": "09247fd98408f7ae1262b5ac681562b393c6ec3d64ac0d390468af031669a7ac"
}
```

Canonical spec verification:

```text
cairn/specs/rust-package-planning/spec.md:2807:### Requirement: Source-built Rust toolchain closure proof
cairn/specs/rust-package-planning/spec.md:2809:r[rust_package_planning.source_built_toolchain_closure] Mantle MUST provide a separate audit-grade proof before claiming that a Cargo-free self-build or fixed-point run used a source-built compiler/toolchain closure.
```

## Archive execution

Command:

```sh
CAIRN_ARCHIVE_DATE=2026-05-31 /nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn archive source-built-toolchain-closure --root . --execute
```

Output summary:

```json
{
  "actions": [
    {
      "description": "move active change to archive: source-built-toolchain-closure",
      "kind": "archive_change",
      "path": "./cairn/archive/2026-05-31-source-built-toolchain-closure"
    }
  ],
  "blocked": false,
  "dry_run": false,
  "mutated": true,
  "receipt_hash": "86a39f5c4f77559c97879599a01ed17d0718515ad21174d6e2f4c1deb1c937b6"
}
```

Archive date was explicit and correct; no 1970 archive rename was needed.

## Post-archive validation

Command:

```sh
/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn validate --root .
find cairn/changes -mindepth 1 -maxdepth 1 -type d -printf '%f\n' | sort
rg -n 'Source-built Rust toolchain closure proof|source_built_toolchain_closure' cairn/specs/rust-package-planning/spec.md
rg -n '^- \[ \]' cairn/archive/2026-05-31-source-built-toolchain-closure/tasks.md || true
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

Additional checks:

```text
active changes: none
cairn/specs/rust-package-planning/spec.md:2807:### Requirement: Source-built Rust toolchain closure proof
cairn/specs/rust-package-planning/spec.md:2809:r[rust_package_planning.source_built_toolchain_closure] Mantle MUST provide a separate audit-grade proof before claiming that a Cargo-free self-build or fixed-point run used a source-built compiler/toolchain closure.
archived unchecked tasks: none
```
