# Actual isolated frozen-source runner qualification

## Subject and result

Frozen code is `71c2bd4de8ec66e545b42314ba95bf050396a36d`. The complete source
identity remained
`b49c087adf2e6f7c15fd6e4fb80bc2c614546a4efba8a59df279c61d79059778`
before and after fresh source compilation and after isolated canonical gates.
No frozen source was retouched to manipulate Cargo freshness.

Source qualification **1596** and isolated canonical qualification **1597** both
completed Success. Their target directories were absent before start, and cache
wrapper variables were unset inside the checked-in development shell. The
source log records real `Compiling mantle` for the regular release runner, not
only a passing test executable or a warm-target completion status.

Captured source execution:

`test result: ok. 265 passed; 0 failed; 27 ignored; 0 measured; 2680 filtered out; finished in 6.09s`

Formatting, strict release Clippy and release build completed. The independent
fresh canonical target completed its CLI/contract and proposal/design/tasks
Cairn command sequence. Complete logs are `qualification-fresh-target.log` and
`canonical-fresh-target.log`; exact commands and task states are the retained
fresh-target scripts and `fresh-{qualification,canonical}-pueue*.json`.

## Actual regular executable binding

The newly installed runner is
`/home/brittonr/sbfp-20261113-berdKH/bin/mantle-71c2bd4d-fresh-target`, BLAKE3
`c980307680feebd9d8a5543d39635393dad6b39d729aad051e97e7dc75457971`.
It is not the older warmed image
`76654f00f2c2418e585a1f718da8c8d36ab0d110fbeddc4e69b6dd2dc8116504`.
The latter remains preserved at its historical paths and is not promoted.

The complete qualification transcript's received/local BLAKE3 equals the remote
value `5ceaef2087fcf775eabb9788930ae693a1e1d71c7f20c13b7fb8c52b19f1740c`.
Canonical source and complete-log equality are retained in
`fresh-canonical-source-parity.json`. `fresh-runner-qualified.json` binds the
actual source, commands, target preconditions, task results and executable.
A digest difference alone is not the source-binding argument; real isolated
compilation and retained exact inputs/outputs are.

## Source-only continuation and non-claims

After parent inspection, source-only preparation **1598** was released from its
stash behind successful 1596/1597. It uses the fresh runner and a new cohort
namespace; killed preparation 1595 and its partial files are not reused.
The exact selected command and runner bytes are in
`fresh-preparation-released.json`. Current preparation output is not yet
accepted by this checkpoint.

Maintenance owner: Mantle's operator proof workflow (ADR 0150). This is bounded
runner build and qualification evidence, not compiler/source correctness,
protected provider execution, a full-source fixed point or release eligibility.
A ready newly authored cohort, complete byte-transfer and physical/graph review,
protected stages and v2 reconciliation/receipt are still required. I3–I5 and
proof-dependent V2/V3 remain open.
