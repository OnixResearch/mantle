# document-foreign-import-trust-model validation evidence

Captured: 2026-07-03T23:03:30Z

Task-ID: V1 V2 V3
Covers: foreign_derivation_import.operator_trust_model_docs, foreign_derivation_import.receipt_non_claims_documentation, foreign_derivation_import.trust_model_guard

## V1 guard positive

```text
$ nix develop -c cargo -Zscript scripts/check-foreign-import-trust-model.rs
foreign import trust-model doc check passed
```

## V1 guard negative self-test

```text
$ nix develop -c cargo -Zscript scripts/check-foreign-import-trust-model.rs --self-test
foreign import trust-model checker self-test passed
```

## Operator proof guide guard

```text
$ nix develop -c cargo -Zscript scripts/check-operator-proof-guide.rs
operator proof guide drift check passed
```

## V2 trust-boundary inspection excerpts

```text
$ grep -nE 'What an import receipt does not claim|Graph provenance|Cache and substitution trust|Source verification|Sandbox capabilities|Realization and output verification|Guix-like|Nix-like|does not claim build success|Additional realization' docs/foreign-derivation-import-trust-model.md
33:## What an import receipt does not claim
35:A valid import receipt alone does not claim build success. It does not claim
40:Additional realization and verification evidence is required before claiming
51:### Graph provenance
53:Graph provenance answers where the lowered store graph came from. Guix-like
55:system. Nix-like producers may name `.drv` paths, derivation-JSON exports,
73:### Source verification
83:### Cache and substitution trust
95:### Sandbox capabilities
105:### Realization and output verification
115:## Guix-like hello import
117:A Guix-like `hello` import can be reviewable without Guix at consumption time:
128:> The Guix-like `hello` graph was admitted from explicit foreign import artifacts
129:> with receipt-bound policies. This does not claim build success, output trust,
134:## Nix-like hello import
136:A Nix-like `hello` import follows the same boundary:
148:> The Nix-like `hello` graph was admitted from concrete derivation artifacts and
149:> receipt-bound policy. This does not claim build success, output trust, package
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
  "changes": 7,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 23,
  "valid": true
}
```

## Cairn proposal gate

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal document-foreign-import-trust-model --root .
{
  "change": "document-foreign-import-trust-model",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "900b4fedbaaaab49dce65ed5fa471f4122487dd020d40cfedfa3352c5b4181fd",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "a6ad563cdcd51c74af059459617c30577995838c1d456b825521824878ad4c7a",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
```

## Cairn design gate

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate design document-foreign-import-trust-model --root .
{
  "change": "document-foreign-import-trust-model",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "ad5e3a3ca7fed2643c091ebc8940c265ead1ea3e5dd0da385619316c24833f1f",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "e94d3af878a5aeb03ae9eff2c27fbfa02937eefb71376d3423e4e55845281368",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
```

## Cairn tasks gate

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks document-foreign-import-trust-model --root .
{
  "change": "document-foreign-import-trust-model",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "6c1decc2bcf822879acf1212ddc1a66efaae965534192562bfcc13a23a906861",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "706a2a268f49e1c309c335425a0da76496c8a3f53e6420d7f33efb1e00572249",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

# Accepted-spec sync evidence

Captured: 2026-07-03T23:06:03Z

Note: Cairn sync execute was run; accepted spec IDs were verified after manually merging the promoted requirement text because the full-spec-shaped delta did not insert new requirement IDs automatically.

## Cairn sync dry-run

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- sync document-foreign-import-trust-model --root .
{
  "actions": [
    {
      "description": "merge delta spec into main specs: ./cairn/changes/document-foreign-import-trust-model/specs/foreign-derivation-import/spec.md",
      "kind": "sync_delta_spec",
      "path": "./cairn/changes/document-foreign-import-trust-model/specs/foreign-derivation-import/spec.md"
    }
  ],
  "blocked": false,
  "change": "document-foreign-import-trust-model",
  "delta_specs": [
    "./cairn/changes/document-foreign-import-trust-model/specs/foreign-derivation-import/spec.md"
  ],
  "dry_run": true,
  "input_hash": "297be1aa8deda2a2b0551fb4fda679d5f8c70dbedb69e5da69768df519796897",
  "layout": "cairn",
  "mutated": false,
  "mutation_manifest": null,
  "plan_hash": "da4eaf9840f7213c3afa274aaa25853aa6693f72ebc9f29684d16e1ac3ab359b",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "reasons": [],
  "receipt_hash": "eb592464e8addfdfae27bf6c75f11c4f14fcce4124012b49cc67942bc1468bfb"
}
```

## Cairn sync execute

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- sync document-foreign-import-trust-model --root . --execute
{
  "actions": [
    {
      "description": "merge delta spec into main specs: ./cairn/changes/document-foreign-import-trust-model/specs/foreign-derivation-import/spec.md",
      "kind": "sync_delta_spec",
      "path": "./cairn/changes/document-foreign-import-trust-model/specs/foreign-derivation-import/spec.md"
    }
  ],
  "blocked": false,
  "change": "document-foreign-import-trust-model",
  "delta_specs": [
    "./cairn/changes/document-foreign-import-trust-model/specs/foreign-derivation-import/spec.md"
  ],
  "dry_run": false,
  "input_hash": "297be1aa8deda2a2b0551fb4fda679d5f8c70dbedb69e5da69768df519796897",
  "layout": "cairn",
  "mutated": true,
  "mutation_manifest": {
    "after": {
      "entries": [
        {
          "content_hash": "d7773c13ab3b28c1ac75ce1486eb37a8dfc0a0921548a1c3695786e166e65e86",
          "exists": true,
          "path": "./cairn/specs/foreign-derivation-import/spec.md"
        }
      ],
      "manifest_hash": "01612eade54852778921b22b98bf4c37046f710a6d7a68d773bb48cff64e5863"
    },
    "before": {
      "entries": [
        {
          "content_hash": "d7773c13ab3b28c1ac75ce1486eb37a8dfc0a0921548a1c3695786e166e65e86",
          "exists": true,
          "path": "./cairn/specs/foreign-derivation-import/spec.md"
        }
      ],
      "manifest_hash": "01612eade54852778921b22b98bf4c37046f710a6d7a68d773bb48cff64e5863"
    },
    "kind": "sync",
    "manifest_hash": "5e3b7d5c483921e9f29342542bc5888360ecf35da6408d99aabc3f3dccb0e094"
  },
  "plan_hash": "3012412669e90d19f73fdd2ad6786caa7bb324e994457d4eec274e8b4eae14c5",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "reasons": [],
  "receipt_hash": "7cbd9e0335ae0be157c8b102009304efc7105afd4f7026d1cf917229aee7d6ab"
}
```

## Accepted spec ID presence

```text
$ grep -nE 'operator_trust_model_docs|receipt_non_claims_documentation|trust_model_guard' cairn/specs/foreign-derivation-import/spec.md
105:r[foreign_derivation_import.operator_trust_model_docs] Mantle documentation MUST explain the trust boundaries for foreign derivation import receipts, including graph provenance, policy digests, source verification, cache/substitution trust, sandbox capabilities, realization, and output verification.
123:r[foreign_derivation_import.receipt_non_claims_documentation] Foreign import documentation MUST state that an import receipt alone does not claim build success, package correctness, bootstrap parity, output trust, reproducibility, or foreign-frontend availability.
141:r[foreign_derivation_import.trust_model_guard] Mantle SHOULD include a lightweight guard that fails when required foreign import trust-model headings or non-claim language disappear from operator documentation.
```

## Post-sync Cairn validation

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 7,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 23,
  "valid": true
}
```

# Post-archive validation evidence

Captured: 2026-07-03T23:06:42Z

## Archive path

```text
cairn/archive/2026-07-03-document-foreign-import-trust-model
```

## Cairn validation after archive

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 6,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 22,
  "valid": true
}
```
