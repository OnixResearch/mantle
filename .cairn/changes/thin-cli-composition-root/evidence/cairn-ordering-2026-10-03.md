# Cairn ordering and V2 task-shape evidence — 2026-10-03

Candidate: private `fix/v2-cairn-order`, starting from published Mantle
`0feb347b8a0c6e4099be70635fe11baa5ea0385a`. From the candidate root,
using `/home/brittonr/git/OnixResearch/cairn/result-cairn/bin/cairn`, the
following commands exited 0; each linked file retains the **unfiltered raw
stdout and stderr** from that invocation:

| Command | Transcript | Observed result |
| --- | --- | --- |
| `cairn validate --root .` | [global validation](cairn-global-validate-2026-10-03.json) | `valid: true`, `issues: []`, 36 changes and 96 specs validated |
| `cairn gate tasks thin-cli-composition-root --root .` | [thin-cli task gate](cairn-thin-cli-tasks-2026-10-03.json) | `valid: true`, `verdict: PASS`, `issues: []`; edges I5→I7 and I6+I7→V1, `ready_tasks: [I6]`, V1 blocked by I6 |
| `cairn gate tasks add-dynamic-plan-source-slices --root .` | [V2 task gate](cairn-v2-tasks-2026-10-03.json) | `valid: true`, `verdict: PASS`, `issues: []`; 14 done, 2 todo |
| `nix develop --no-write-lock-file -c cargo -Zscript scripts/check-cli-architecture.rs --self-test` | [checker fixtures](cli-architecture-self-test-2026-10-03.txt) | `cli-architecture self-test: PASS` |
| `nix develop --no-write-lock-file -c cargo -Zscript scripts/check-cli-architecture.rs` | [checker repository scan](cli-architecture-scan-2026-10-03.txt) | `cli architecture: PASS` |

I7 is the already-completed preventive topology/dependency checker: its
`application_architecture.dependency_guard` scenarios and accepted/negative
checker fixtures can be verified after I5 typed error ownership without waiting
for every I6 effect-plan operation to migrate. I6 still requires coverage of
*each* application operation, including the outstanding 151-leaf inventory and
integration gates, and remains unchecked. V1's effect-failure and
wrong-observation verification requires **both** I6 and I7; the task receipt
shows V1 not ready while I6 is unfinished. V2–V5 are downstream of V1.

These are bounded Cairn structural gates and checker observations, not
completion of I6, V1–V5, V2 change T4.4 or T4.5, integrated quality, an archive,
accepted-spec sync, or a release. Neither sync nor archive was executed.
