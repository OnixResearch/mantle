# Frozen runner artifact binding hold

## Observed 71c2bd4d qualification scope

Tasks 1592, 1593 and 1594 completed successfully as commands. The frozen complete
source is `b49c087adf2e6f7c15fd6e4fb80bc2c614546a4efba8a59df279c61d79059778`.
The source unit leg executed `265 passed; 0 failed; 27 ignored`; formatting,
Clippy/build commands and the canonical CLI/contract/Cairn command sequence
completed. Complete local/remote transcript BLAKE3 values agree:

| Transcript | BLAKE3 |
| --- | --- |
| Assembly | `0c77553fd847beae666a92365ee382863467d58944d212c7e718b2064e0ae762` |
| Source qualification | `2b93ce10cace9eca8d5952f7846e63fc2e4be7f36eb3b4ac2ab38e0f11e89c36` |
| Canonical qualification | `600a92abe2364500b91a8e394be2bc754804fd794a7a2dd5c6335b0ee9123773` |

This is command/test evidence, **not accepted regular-runner source binding**.
The installed regular runner has BLAKE3
`76654f00f2c2418e585a1f718da8c8d36ab0d110fbeddc4e69b6dd2dc8116504`,
identical to the rejected e9 runner. Its shared target release artifact is
197,683,408 bytes and has timestamp `2026-11-15 01:38:53.461014985 -0500`, before
the new qualification installed that same-size output at
`2026-11-15 04:59:39.475804132 -0500`. The changed successor profile source has
an epoch timestamp and 5,317 bytes. The captured build log reports completion
without recompiling/linking that regular binary.

Those facts establish reuse of the older output. [INFERENCE] Normalized source
mtimes and a warmed target allowed the changed production author source to be
treated as fresh; this is not a general proof about Cargo's implementation.
Production reachability is present in
`source_built_fixed_point_authoring.rs::author`'s call to the Rust program author.
A passing new test binary does not prove that the installed regular runner was
relinked from those source bytes.

## Containment and exact replacement boundary

The parent stopped its own source-only preparation task **1595**, observed
`Killed`, and retained its log and partial state. No output or provider from it
is accepted and its namespace is not reused. This was not a cold proof launch.
The stopped task metadata is `interrupted-preparation-pueue1595.json`.

Fresh source qualification **1596** and canonical qualification **1597** use
separate target directories that must be absent before start. Compiler wrapper
variables are also unset inside the checked-in development shell. The frozen
source is not retouched; source/archive pins, compiler cohort, proof limits,
provider/lineage pins and all authority checks are unchanged. The actual scripts
and exact source/target paths are `qualify-fresh-target.sh`,
`canonical-fresh-target-{run,inside}.sh` and `fresh-runner-inputs.json`.

Replacement preparation **1598** is **stashed**, dependent on both fresh gates,
and requires parent inspection of the actual fresh build and installed byte
identity before enqueue. It selects a new runner and cohort path. An unchanged
digest is reconciled against actual evidence, not forbidden for every possible
source revision. Fresh results are pending at this checkpoint.

Maintenance owner: Mantle operator proof workflow. The durable correction is
ADR 0150's isolated target precondition plus reproducible retained commands;
ordinary developer cache behavior is unchanged. Neither a fresh target, test
success, source parity nor a transferred executable proves protected provider
execution, full-source bootstrap, fixed-point equality or release eligibility.
I3–I5 and proof-dependent V2/V3 remain open.
