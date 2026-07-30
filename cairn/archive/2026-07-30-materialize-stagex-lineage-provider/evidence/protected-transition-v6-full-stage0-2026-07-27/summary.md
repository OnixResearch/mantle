# Protected full Stage0 success — 2026-07-27

## Result

The protected transition completed the audited seed through full Stage0.
It did not materialize Mes, TinyCC, or the normalized StageX provider.

- Test: `stagex_transition::tests::protected_transition_reproduces_seed_and_builds_kaem`
- Pueue task: `1786`
- Result: `1 passed; 0 failed`
- Lineage manifest BLAKE3: `9e9138e807dba3b348303ad333a74cbdb8094533bf7e3357ba0971781ac43d8a`
- Plan BLAKE3: `ee1b8c675e92d7a2d1de86bf4b5dceb97319e1aa75b2cf4dfe35540aa7163a56`
- Source-state BLAKE3: `05f921923c8717c98022a14c44f00ad2ed25e5f2f7a9ea5b74f4cb8b9f33d63f`
- Authenticated source-bundle BLAKE3: `7e93ccc7a29bacc1da6f75c10ae90655ee83afd0d95ca282c8d270c225372f45`
- Protected exec events: `132`
- Denied events: `0`
- Fallback events: `0`
- Full Stage0 executable outputs: `20`

The run used create-new scratch. It loaded and validated the source-bundle
manifest before protected execution. The source records bind seven direct Git
snapshot identities. The transition then executed only absolute paths with
exact BLAKE3 authorization.

The compact evidence keeps the plan, report, complete protected-exec audit,
source and executable inventories, generated absolute-path recipes, answers,
and bounded stderr logs. Authenticated source payloads are not duplicated in
this directory.

## Non-claim

This evidence proves the protected seed-to-full-Stage0 frontier only. It does
not prove the Mes or TinyCC transition, normalized-provider admission, StageX
receipt completion, or Mantle fixed-point parity.
