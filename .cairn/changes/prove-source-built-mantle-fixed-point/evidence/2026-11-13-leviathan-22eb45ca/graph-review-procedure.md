# Reproduce the independent read-only graph and guard inspection

Host `leviathan.cymric-daggertooth.ts.net`; frozen revision `22eb45ca5321416fce3278a228a43224dd0026c5`. Run on the local inspection workstation; `rsync` is for transfer only. Source profile validation and all build/provider commands are deliberately excluded. These sources are **specific inert recipe files**, not entire source manifests:

```
mkdir -p /tmp/mantle-source22-graph-review
rsync -az 'leviathan.cymric-daggertooth.ts.net:/home/brittonr/sbfp-20261113-berdKH/prepared-cohort-22eb45ca-resumed/inputs/mantle-source/bootstrap/source-built/mantle-stage*.json' /tmp/mantle-source22-graph-review/
rsync -az 'leviathan.cymric-daggertooth.ts.net:/home/brittonr/sbfp-20261113-berdKH/prepared-cohort-22eb45ca-resumed/program-sources/source-built-stagex-programs/stagex-*.json' /tmp/mantle-source22-graph-review/
rsync -az leviathan.cymric-daggertooth.ts.net:/home/brittonr/sbfp-20261113-berdKH/prepared-cohort-22eb45ca-resumed/program-sources/fixed-point-native-source-program/source-built/native/program.json /tmp/mantle-source22-graph-review/native-program.json
rsync -az leviathan.cymric-daggertooth.ts.net:/home/brittonr/sbfp-20261113-berdKH/prepared-cohort-22eb45ca-resumed/program-sources/fixed-point-rust-source-program/source-built/rust/program.json /tmp/mantle-source22-graph-review/rust-program.json
python3 local://source22-graph-review-script /tmp/mantle-source22-graph-review
python3 local://source22-graph-review-native-check /tmp/mantle-source22-graph-review
python3 local://source22-graph-review-guard-check /tmp/mantle-source22-graph-review
```

The first script checks the six captured sizes/BLAKE3 against independently measured remote originals. It parses one recipe at a time, inventories exact stage/producer/output identities, validates all Produced references in step inputs, nested controller declarations, roots, exports and semantic roots, enforces predecessor stage ordering, traverses backwards from exact roots (including ObservePhase edges), and checks graph cycles. The second checks the Rust provider canonical Tree export, Mantle semantic root sets and binary/`--help` smoke, all 12 authenticated PerlAsm products/consumers per stage, and local native-script state. Genuine external native-provider tool/runtime images are not misclassified as script state. The third is a **separate narrow guard-admission pass** matching frozen `src/source_built_recipe.rs:693-699,788-887,1823-1840` and `src/source_built_fixed_point_action_trust.rs:79-82`. It rejects guarded outputs used as roots/exports/semantic products, invalid source guard predicates, non-dominating or unguarded input edges (including ordinary outputs of OneOf/Normal Exec with exit-status), prior-stage guarded outputs, and malformed ObservePhase boundaries; it does not repeat the generic graph walk.

Full captured results: `/tmp/mantle-source22-graph-review/source22-graph-review.json` (updated with `guard_admission_supplement`) and `/tmp/mantle-source22-graph-review/source22-guard-review.json`. Summary: `local://source22-graph-review.json`. All three scripts are retained under `local://`. Counts for `cross_stage` are Produced **reference occurrences**, including controller declarations, not distinct edges. The source files and cohort are never edited. Static inspection does not discharge actual guarded statuses, dynamic request bytes/cwd/env, provider execution, compiler correctness, fixed-point equality, full-profile verification, or release eligibility.
