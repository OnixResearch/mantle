# Graph-review handoff to Luna reviewer — 2026-11-14

The previously started **trusted read-only JSON inspection commands have completed**. No further inspection was started after the model-routing correction. Actual configured route for this pass was reported by Main as `underclass/gpt-6-sol` (slow), **not Luna**; the Luna reviewer owns the final six-recipe launch verdict. Treat the existing `ACCEPT` strings in the reports/summary as **advisory outputs of these bounded static inspection scripts**, not a final independent Luna decision or proof launch.

## Retained files and physical scope

All files are under this `graph-review/` directory: `graph-current-inspect.py` (first inspector), `graph-current-result.json` (its completed output), `graph-current-supplement.py` (second inspector), `graph-current-supplement.json` (its completed output), `graph-current-procedure.md` (exact reproducible commands, frozen consumer fingerprints, scope), `graph-current-review.md` (advisory human summary). The only remote writes were trusted-inspector scripts/results under `/tmp/mantle-8fa84260-graph-review-20261114-luna/`; the cohort/frozen source were read-only. Remote host: `leviathan.cymric-daggertooth.ts.net`; cohort `/home/brittonr/sbfp-20261113-berdKH/prepared-cohort-8fa84260-aws`; frozen source `/home/brittonr/sbfp-20261113-berdKH/runner-source-8fa84260-aws`. Inspection interpreter `/nix/store/bya4ansy58az5133kryqp2cvrxbnz8qy-python3-3.14.10/bin/python3`; physical-byte hasher `/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum`. `graph-current-procedure.md` has the commands used to transport/invoke the scripts and retain results.

Measured directly over six current physical recipe files (bytes; BLAKE3):

- `program-sources/source-built-stagex-programs/stagex-transition.json`: 31,730,468; `8340c69c2e898193ba7a95d861718c1681e5d446c7e5cf7210be784eb18d9e34`.
- `program-sources/source-built-stagex-programs/stagex-provider-publication.json`: 52,226; `95b34b8881195c596046834e806f0324150387b2fc4a91c09ef05f499d4eccd8`.
- `program-sources/fixed-point-native-source-program/source-built/native/program.json`: 18,268,649; `aaf5403428b921f312b8c74336a8dba56b230fd657b88b4b5fa9d28b0fdd413c`.
- `program-sources/fixed-point-rust-source-program/source-built/rust/program.json`: 259,029,252; `2401c71c1efe2053251f9e65021b4a9ec16fcc98359f5b4536359ca75c01a407`.
- `inputs/mantle-source/bootstrap/source-built/mantle-stage1.json`: 172,677,322; `f0cd61f74c0346dffdbccd75dd4e5a5c14ce2aa494692d9c299ebc898b558c4d`.
- `inputs/mantle-source/bootstrap/source-built/mantle-stage2.json`: 172,677,322; `a35d5167194df66c913adf5e7300b5763b52f60526479d225619c193e4d85442`.

## Completed static observations (not final launch verdict)

The first inspector reported six per-recipe advisory `accept` outcomes with `issues: {}`: 125,355/125,355 steps reachable, zero cycle candidates, 1,683,067 Produced reference occurrences including 254,845 nested-controller occurrences, and 389,739 cross-stage reference occurrences (occurrences, not distinct edges). Stage step/reachability counts respectively: 8,131/8,131; 18/18; 5,122/5,122; 63,882/63,882; 24,101/24,101; 24,101/24,101. Its checks included exact local and predecessor stage/action/output references, local ordering, roots/exports/semantic roots, embedded controller catalogs, guard predicates and dominance, canonical Rust Tree export, Mantle semantic-root union, exact executable binary/`--help` smoke, and local static native-script state. It found 80 guarded commands, 80 explicit phase observations, zero guarded roots/exports/semantic products, 64 optional-failure direct input occurrences, and 14 allowed StageX optional-failure ordinary roots. Each Mantle stage had 1,045 Rust-unit + 98 build-script semantic keys, 1,242 roots, and 12 reachable PerlAsm authenticators whose outputs were consumed.

The completed supplement reported `issues: {}` and advisory `accept`: current sibling StageX publication expansion selector has `binutils_admitted: false`; its canonical tree transitively references the exact transition Tree and three selected executable tool exports, and its four selected validation commands have static Exec/exit-policy contracts. It checked native provider's three canonical Tree roles, provided concrete guard/phase/authenticator/optional-root reference samples, and checked static native cwd/request/environment/history owner bindings. Full details and exact references appear in the two JSON reports; the scripts were retained verbatim.

## Unfinished / excluded checks for final reviewer

- **Luna has not reviewed** these reports/scripts/current physical recipes independently or issued the final launch accept/reject decision. Existing advisory `accept` outputs must not be relabelled final.
- No complete StageX transition expansion-index/lineage edge inventory or full native/Rust source-route/closure correctness was independently established by these six-recipe graph scripts. The supplement checked the *publication* selector and canonical dependency only.
- No dynamic controller-selected turns, generated request bytes, actual cwd/env/exit outcomes, provider validation, compiler correctness, reconciliation, fixed-point equality, reproducibility, proof execution or release eligibility was established. No product, generator, helper, provider, recipe, build, lint, formatter or product test ran. There is no proof launch or remote action dispatch.
