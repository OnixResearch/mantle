# automate-cairn-lifecycle-evidence validation evidence

Captured: 2026-07-03T23:38:35Z

Task-ID: V1 V2 V3
Covers: verification_evidence.cairn_lifecycle_runner, verification_evidence.lifecycle_evidence_transcript, verification_evidence.lifecycle_runner_fail_closed

## Runner self-test positive and negative cases

```text
$ nix develop -c cargo -Zscript scripts/cairn-lifecycle-evidence.rs --self-test
cairn lifecycle evidence self-test passed
```

## Runner complete fixture dry-run

```text
$ nix develop -c cargo -Zscript scripts/cairn-lifecycle-evidence.rs --fixture tests/fixtures/cairn-lifecycle/complete --requirement-id verification_evidence.cairn_lifecycle_runner --requirement-id verification_evidence.lifecycle_evidence_transcript --requirement-id verification_evidence.lifecycle_runner_fail_closed --dry-run
cairn lifecycle evidence: ready
commands: 2
archive_path: cairn/archive/2026-07-03-fixture-change
diagnostics: none
```

## Format check

```text
$ nix develop -c cargo fmt -p mantle --check
```

## Whitespace diff check

```text
$ git diff --check
```

## Cairn validation

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 4,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 20,
  "valid": true
}
```

## Cairn proposal gate

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal automate-cairn-lifecycle-evidence --root .
{
  "change": "automate-cairn-lifecycle-evidence",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "114709cb9527c575707ca0982804e95d86121906bcaf26ced9cad6e3ca46ae37",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "821e6223bb5fc05df555c4c5b8ba5098f6de443dd003a69e3994caa57c886f6e",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
```

## Cairn design gate

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate design automate-cairn-lifecycle-evidence --root .
{
  "change": "automate-cairn-lifecycle-evidence",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "19786af78ecee3cf62a8a79452aa8c2345dd4206b0bbf7ee6deda8fb9f73021c",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "f4f16bbc44ffaf21164a7f9c33ce6e69522ac35ca7bdc76a047d9283770b125e",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
```

## Cairn tasks gate

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks automate-cairn-lifecycle-evidence --root .
{
  "change": "automate-cairn-lifecycle-evidence",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "86e66a0c900dfbff054932b8527be870b620a1bbb914089be61cfd93ef5f2914",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "6de253529d57a6ce311e9a345b985e3670af4afa51cb5ffb4ab2ad7a47fb45d1",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

# Accepted-spec sync evidence

Captured: 2026-07-03T23:39:42Z

Note: Cairn sync execute was run; accepted spec IDs were verified after manually merging the promoted requirement text because the full-spec-shaped delta did not insert new requirement IDs automatically.

## Cairn sync dry-run

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- sync automate-cairn-lifecycle-evidence --root .
{
  "actions": [
    {
      "description": "merge delta spec into main specs: ./cairn/changes/automate-cairn-lifecycle-evidence/specs/verification-evidence/spec.md",
      "kind": "sync_delta_spec",
      "path": "./cairn/changes/automate-cairn-lifecycle-evidence/specs/verification-evidence/spec.md"
    }
  ],
  "blocked": false,
  "change": "automate-cairn-lifecycle-evidence",
  "delta_specs": [
    "./cairn/changes/automate-cairn-lifecycle-evidence/specs/verification-evidence/spec.md"
  ],
  "dry_run": true,
  "input_hash": "9753490e37595e0eb98b9846bf88c3ed363982208f1cd6dd8c5ed0694b073a41",
  "layout": "cairn",
  "mutated": false,
  "mutation_manifest": null,
  "plan_hash": "0c9a646270e841c3450b1449ee87476db00f4560cacbf76a8b6a6aa8325d9841",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "reasons": [],
  "receipt_hash": "dfed6e5af00789b95368ef7b427c1d17ed095ac508544f47a1a725a0ec7442b7"
}
```

## Cairn sync execute

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- sync automate-cairn-lifecycle-evidence --root . --execute
{
  "actions": [
    {
      "description": "merge delta spec into main specs: ./cairn/changes/automate-cairn-lifecycle-evidence/specs/verification-evidence/spec.md",
      "kind": "sync_delta_spec",
      "path": "./cairn/changes/automate-cairn-lifecycle-evidence/specs/verification-evidence/spec.md"
    }
  ],
  "blocked": false,
  "change": "automate-cairn-lifecycle-evidence",
  "delta_specs": [
    "./cairn/changes/automate-cairn-lifecycle-evidence/specs/verification-evidence/spec.md"
  ],
  "dry_run": false,
  "input_hash": "9753490e37595e0eb98b9846bf88c3ed363982208f1cd6dd8c5ed0694b073a41",
  "layout": "cairn",
  "mutated": true,
  "mutation_manifest": {
    "after": {
      "entries": [
        {
          "content_hash": "35ae26e18972baf42d917f00f1b3c930418e7016b6eef5848533fbdbb68a2c46",
          "exists": true,
          "path": "./cairn/specs/verification-evidence/spec.md"
        }
      ],
      "manifest_hash": "914d7b901188a0a3d1a8c88bbc56d37b7e3f0fa7e2c11f41925bb1722efdffe0"
    },
    "before": {
      "entries": [
        {
          "content_hash": "35ae26e18972baf42d917f00f1b3c930418e7016b6eef5848533fbdbb68a2c46",
          "exists": true,
          "path": "./cairn/specs/verification-evidence/spec.md"
        }
      ],
      "manifest_hash": "914d7b901188a0a3d1a8c88bbc56d37b7e3f0fa7e2c11f41925bb1722efdffe0"
    },
    "kind": "sync",
    "manifest_hash": "932ee31cb63332e16830fe25b7d789a535865cd6a9e3ba41bd0085f5e8f32a1c"
  },
  "plan_hash": "00a4e58e2368b2fc4ba64f946d857fe539a64e42bbc4fc686076599af8d1e671",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "reasons": [],
  "receipt_hash": "a8bfcc7f17529ce33fed0e0a329de6c404e84cc76671468998ac021695e41234"
}
```

## Accepted spec ID presence

```text
$ grep -nE 'cairn_lifecycle_runner|lifecycle_evidence_transcript|lifecycle_runner_fail_closed' cairn/specs/verification-evidence/spec.md
43:r[verification_evidence.cairn_lifecycle_runner] Mantle SHOULD provide a repo-owned helper that runs an explicit Cairn change lifecycle command list and produces durable evidence for validation, gates, sync, archive, post-archive validation, and final status.
61:r[verification_evidence.lifecycle_evidence_transcript] Lifecycle automation MUST append or write durable transcripts that include command lines, relevant output summaries, receipt hashes when available, archive paths, post-archive validation output, and final status evidence.
79:r[verification_evidence.lifecycle_runner_fail_closed] Lifecycle automation MUST fail closed when required lifecycle evidence is absent, incomplete, contradictory, or disconnected from the tasks and requirement IDs it claims to prove.
```

## Post-sync Cairn validation

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 4,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 20,
  "valid": true
}
```

## Post-sync tasks gate

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks automate-cairn-lifecycle-evidence --root .
{
  "change": "automate-cairn-lifecycle-evidence",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "86e66a0c900dfbff054932b8527be870b620a1bbb914089be61cfd93ef5f2914",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "6de253529d57a6ce311e9a345b985e3670af4afa51cb5ffb4ab2ad7a47fb45d1",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

# Post-archive validation evidence

Captured: 2026-07-03T23:40:08Z

## Archive path

```text
cairn/archive/2026-07-03-automate-cairn-lifecycle-evidence
```

## Cairn validation after archive

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 3,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 19,
  "valid": true
}
```
