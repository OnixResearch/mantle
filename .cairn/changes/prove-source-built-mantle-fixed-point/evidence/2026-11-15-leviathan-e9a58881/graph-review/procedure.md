# Exact inspection procedure (e9 frozen consumer slice)

Host: `leviathan.cymric-daggertooth.ts.net`.
Frozen source root: `/home/brittonr/sbfp-20261113-berdKH/runner-source-e9a58881-typed-index`.
Current cohort root: `/home/brittonr/sbfp-20261113-berdKH/prepared-cohort-e9a58881-typed-index`.
Remote inert scratch: `/tmp/luna-e9a58881-expansion-edges-20261115/`.
Pinned Python: `/nix/store/bya4ansy58az5133kryqp2cvrxbnz8qy-python3-3.14.10/bin/python3`.
Pinned BLAKE3: `/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum`.

Exact static commands used:

* `ssh leviathan.cymric-daggertooth.ts.net '/nix/store/bya4ansy58az5133kryqp2cvrxbnz8qy-python3-3.14.10/bin/python3 /tmp/luna-e9a58881-expansion-edges-20261115/expansion-edge-audit-exact.py'` — reads the current cold route, StageX lineage and both StageX indexes/recipes, native expansion index/recipe, Rust route/expansion index/recipe; writes `expansion-edge-audit.json`.
* `ssh leviathan.cymric-daggertooth.ts.net '/nix/store/bya4ansy58az5133kryqp2cvrxbnz8qy-python3-3.14.10/bin/python3 /tmp/luna-e9a58881-expansion-edges-20261115/dynamic-request-trace.py'` — reads the already saved parent trace `/tmp/luna-e9a58881-expansion-edges-20261115/encoded-trace.json` and the current routed Rust recipe; writes the exact Produced-request/consumer trace `dynamic-request-trace.json`.
* `ssh leviathan.cymric-daggertooth.ts.net "cd /home/brittonr/sbfp-20261113-berdKH/runner-source-e9a58881-typed-index && /nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum --no-names src/source_built_fixed_point_stagex_actions.rs src/source_built_fixed_point_rust_actions.rs src/source_built_recipe.rs src/source_built_recipe_exec.rs src/source_built_mantle_programs_graph.rs src/source_built_mantle_programs_metadata_helper.rs src/source_built_fixed_point_cold.rs src/source_built_fixed_point_mantle_actions.rs src/source_built_fixed_point_native_actions.rs src/source_built_rust_programs_translation.rs src/source_built_rust_programs_minicargo.rs src/source_built_rust_programs_successors.rs src/source_built_rust_programs_successors_profile.rs src/source_built_rust_programs_successors_invocation.rs src/source_built_rust_resolve.rs src/source_built_rust_programs_minicargo_resolve.rs"` — output is `current-consumer-hashes.txt`.
* Native edge detail: same SSH/Python invocation with `/tmp/luna-e9a58881-expansion-edges-20261115/native-edge-detail.py`; output is `native-edge-detail.json`.

The saved scripts and JSON results are in this directory. Source reads were via `functions.read` against the pinned frozen tree. Relevant ranges: StageX `source_built_fixed_point_stagex_actions.rs:1-150,150-420,460-715,720-915`; recipe `source_built_recipe.rs:552-575,1095-1135,1354-1580,1806-1985,2238-2450`; exec consumer `source_built_recipe_exec.rs:240-310,880-935,1153-1215`; Rust translator `source_built_rust_programs_translation.rs:100-180,258-345,700-790`; minicargo feature template `source_built_rust_programs_minicargo.rs:48-50,738-740`; successor invocation `source_built_rust_programs_successors.rs:560-675`; successor profile template `source_built_rust_programs_successors_profile.rs:1-125`; embedded source helper `source_built_rust_programs_successors_invocation.rs:1-80`; path normalizer `source_built_rust_resolve.rs:180-220`.

Parent-supplied isolated baseline reproduction was read, not rerun: `parent-word-inspection/reproduce-profile-admission-before.log` and `parent-word-inspection/reproduction-pueue26.json` (Pueue 26, failed exit 101). It is not a product/proof run and not a result of this reviewer's execution.
