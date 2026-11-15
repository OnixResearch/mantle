# Reproduce the 2026-11-14 six-recipe read-only graph inspection

Target `leviathan.cymric-daggertooth.ts.net`; cohort `/home/brittonr/sbfp-20261113-berdKH/prepared-cohort-8fa84260-aws`; frozen source `/home/brittonr/sbfp-20261113-berdKH/runner-source-8fa84260-aws`. Run from this `graph-review/` evidence directory on the inspection workstation. `ssh` and `rsync` are only transport and remote trusted-inspector invocation; **never execute the recipes or product/provider/authoring/helper binaries**.

```
HOST=leviathan.cymric-daggertooth.ts.net
SCRATCH=/tmp/mantle-8fa84260-graph-review-20261114-luna
PYTHON=/nix/store/bya4ansy58az5133kryqp2cvrxbnz8qy-python3-3.14.10/bin/python3
B3=/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum
ssh "$HOST" "mkdir -p '$SCRATCH'"
rsync -a graph-current-inspect.py graph-current-supplement.py "$HOST:$SCRATCH/"
ssh "$HOST" "'$PYTHON' '$SCRATCH/graph-current-inspect.py' '$SCRATCH/graph-current-result.json'"
ssh "$HOST" "'$PYTHON' '$SCRATCH/graph-current-supplement.py' '$SCRATCH/graph-current-result.json' '$SCRATCH/graph-current-supplement.json'"
rsync -a "$HOST:$SCRATCH/graph-current-result.json" ./graph-current-result.json
rsync -a "$HOST:$SCRATCH/graph-current-supplement.json" ./graph-current-supplement.json
```

`graph-current-inspect.py` pins all six *current physical* full recipe paths and measures each with `Path.stat()` and the pinned remote `b3sum --no-names`; both inspectors assert the same physical identity before parsing. The first script measures the five frozen production-consumer file hashes, checks exact recipe stages/variants, enumerates every physical step/output and step input + embedded-controller/roots/exports/semantic Produced reference, checks stage/producer/output and predecessor ordinal, computes complete local backward closure (including `observe-phase`) and a whole-six-stage cycle check; it also checks guard predicate/dominance and canonical Rust/Mantle outputs and local native-script state. The supplement reads the current sibling publication expansion selector for its `binutils_admitted` inventory; checks production-style actual-operand dependency traversal, validation command identities and export kinds, canonical native provider, detailed guarded/observed/PerlAsm examples, native cwd/request/environment/history ownership, and counts the StageX optional-failure ordinary roots without incorrectly rejecting them.

The exact commands executed for this report used the paths and pinned interpreter/b3sum shown above. On the first transfer, the local scratch source scripts were at `/home/brittonr/.omp/agent/sessions/-git-OnixResearch-mantle/2026-11-13T16-09-53-880Z_01a1d5ac-2fd8-765c-9a54-013d42028dd4/local/`; the retained scripts here are the same logic, and the final results were read from `$SCRATCH` then copied to this evidence directory. Old procedures were reviewed as reference algorithms only: **do not** use their earlier cohort paths or expected physical identities. The scripts never import product code or process external input as executable instructions; JSON parsing and BLAKE3/size measurement are the complete remote workload.

Evidence: `graph-current-result.json` (complete per-recipe identity/counts, cross-stage by owner, exact canonical references, issues and decision), `graph-current-supplement.json` (concrete guard/phase/authenticator/optional-root samples and publication/native decisions), `graph-current-review.md` (human-readable acceptance and claim limits), and these two retained trusted inspection scripts. Any later physical change requires fresh inspection; this is not a proof-launch receipt or compiler validation.
