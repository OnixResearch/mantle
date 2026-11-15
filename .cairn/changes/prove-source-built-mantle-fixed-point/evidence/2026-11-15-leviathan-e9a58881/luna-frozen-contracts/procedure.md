# Exact frozen consumer inspection procedure

Run host: `leviathan.cymric-daggertooth.ts.net`. All inspection commands below were read-only against the frozen source and published immutable cohort, except for retaining trusted inert scripts/results in the assigned local evidence folder and the named `/tmp` scratch. The pinned remote tools were Python `/nix/store/bya4ansy58az5133kryqp2cvrxbnz8qy-python3-3.14.10/bin/python3` and BLAKE3 `/nix/store/lp2fcmbwq2g4yxmb91kmsrb1a09s5rf5-b3sum-1.8.7/bin/b3sum`.

## Graph and supplement commands

Remote scratch path: `/tmp/luna-e9a58881-frozen-contracts`.

```sh
ssh leviathan.cymric-daggertooth.ts.net 'mkdir -p /tmp/luna-e9a58881-frozen-contracts'
rsync -a .cairn/changes/prove-source-built-mantle-fixed-point/evidence/2026-11-15-leviathan-e9a58881/luna-frozen-contracts/graph-current-inspect.py leviathan.cymric-daggertooth.ts.net:/tmp/luna-e9a58881-frozen-contracts/graph-current-inspect.py
ssh leviathan.cymric-daggertooth.ts.net "'/nix/store/bya4ansy58az5133kryqp2cvrxbnz8qy-python3-3.14.10/bin/python3' '/tmp/luna-e9a58881-frozen-contracts/graph-current-inspect.py' '/tmp/luna-e9a58881-frozen-contracts/current-graph.json'"
rsync -a leviathan.cymric-daggertooth.ts.net:/tmp/luna-e9a58881-frozen-contracts/current-graph.json .cairn/changes/prove-source-built-mantle-fixed-point/evidence/2026-11-15-leviathan-e9a58881/luna-frozen-contracts/current-graph.json
rsync -a .cairn/changes/prove-source-built-mantle-fixed-point/evidence/2026-11-15-leviathan-e9a58881/luna-frozen-contracts/graph-current-supplement.py leviathan.cymric-daggertooth.ts.net:/tmp/luna-e9a58881-frozen-contracts/graph-current-supplement.py
ssh leviathan.cymric-daggertooth.ts.net "'/nix/store/bya4ansy58az5133kryqp2cvrxbnz8qy-python3-3.14.10/bin/python3' '/tmp/luna-e9a58881-frozen-contracts/graph-current-supplement.py' '/tmp/luna-e9a58881-frozen-contracts/current-graph.json' '/tmp/luna-e9a58881-frozen-contracts/current-supplement.json'"
rsync -a leviathan.cymric-daggertooth.ts.net:/tmp/luna-e9a58881-frozen-contracts/current-supplement.json .cairn/changes/prove-source-built-mantle-fixed-point/evidence/2026-11-15-leviathan-e9a58881/luna-frozen-contracts/current-supplement.json
```

The graph inspector pins the six current recipe paths and B3-sums their physical bytes and the reviewed frozen source files. The supplement rechecks each physical recipe identity before the targeted current publication/native/Mantle guard and state scan. Both reports are retained next to these scripts. The first attempted generic `python3` command was not available on the remote host; the completed commands above use the pinned interpreter.

## Exact source-package query

The retained script `current-source-package-summary.py` performs only standard-library JSON reads of the two physical Mantle recipe files and prints each static `--source-package` call's step ID, original source reference, Tree output kind, and complete typed arguments.

```sh
rsync -a .cairn/changes/prove-source-built-mantle-fixed-point/evidence/2026-11-15-leviathan-e9a58881/luna-frozen-contracts/current-source-package-summary.py leviathan.cymric-daggertooth.ts.net:/tmp/luna-e9a58881-frozen-contracts/current-source-package-summary.py
ssh leviathan.cymric-daggertooth.ts.net "'/nix/store/bya4ansy58az5133kryqp2cvrxbnz8qy-python3-3.14.10/bin/python3' '/tmp/luna-e9a58881-frozen-contracts/current-source-package-summary.py'"
```

The stdout was retained as `current-source-package-summary.json`.

## Size-bound command

```sh
ssh leviathan.cymric-daggertooth.ts.net 'stat -c "%s %n" /home/brittonr/sbfp-20261113-berdKH/prepared-cohort-e9a58881-typed-index/inputs/rust-source-archives/source-built/rust/program.json.expansion.json /home/brittonr/sbfp-20261113-berdKH/prepared-cohort-e9a58881-typed-index/inputs/rust-source-archives/source-built/rust/program.json /home/brittonr/sbfp-20261113-berdKH/prepared-cohort-e9a58881-typed-index/program-sources/source-built-stagex-programs/stagex-transition.json.expansion.json /home/brittonr/sbfp-20261113-berdKH/prepared-cohort-e9a58881-typed-index/program-sources/source-built-stagex-programs/stagex-provider-publication.json.expansion.json'
```

Observed size/path pairs, in command order: `4827279` Rust index bytes; `259029252` Rust recipe bytes; `367497` StageX transition index bytes; `325` StageX publication index bytes. This command did not parse those indexes or validate route/lineage contents.

## Source read ranges

The following `functions.read` ranges were read from the exact frozen source tree:

- `ssh://leviathan.cymric-daggertooth.ts.net/home/brittonr/sbfp-20261113-berdKH/runner-source-e9a58881-typed-index/src/source_built_fixed_point_stagex_actions.rs:25-105`
- `.../src/source_built_fixed_point_stagex_actions.rs:100-280`
- `.../src/source_built_fixed_point_stagex_actions.rs:281-530`
- `.../src/source_built_fixed_point_stagex_actions.rs:530-810`
- `.../src/source_built_fixed_point_stagex_actions.rs:780-940`
- `.../src/source_built_fixed_point_rust_actions.rs:1-175`
- `.../src/source_built_fixed_point_rust_actions.rs:175-335`
- `.../src/source_built_recipe.rs:550-815`
- `.../src/source_built_recipe.rs:805-860`
- `.../src/source_built_recipe.rs:900-1045`
- `.../src/source_built_recipe.rs:1097-1219`
- `.../src/source_built_recipe.rs:1220-1385`
- `.../src/source_built_recipe.rs:1380-1580`
- `.../src/source_built_recipe.rs:1680-1750`
- `.../src/source_built_recipe.rs:1810-1888`
- `.../src/source_built_recipe.rs:1935-1980`
- `.../src/source_built_recipe.rs:2080-2220`
- `.../src/source_built_recipe_exec.rs:468-560`
- `.../src/source_built_recipe_exec.rs:790-935`
- `.../src/source_built_recipe_exec.rs:1240-1305`
- `.../src/source_built_fixed_point_action_trust.rs:620-644`
- `.../src/source_built_fixed_point_action_trust.rs:730-915`
- `.../src/source_built_fixed_point_action_trust.rs:1015-1055`
- `.../src/source_built_fixed_point_action_trust.rs:1150-1165`
- `.../src/source_built_fixed_point_cold.rs:120-315`
- `.../src/source_built_mantle_programs_graph.rs:370-570`
- `.../src/source_built_mantle_programs_graph.rs:618-710`
- `.../src/source_built_mantle_programs_metadata_helper.rs:530-570`
- `.../src/source_built_mantle_programs_metadata_helper.rs:895-960`
- `.../src/source_toolchain_closure.rs:1183-1307`

The graph report contains the BLAKE3 for each of these frozen consumer files. No Rust/Python product implementation or helper binary was executed; inspection scripts used only filesystem metadata, BLAKE3, and standard-library JSON parsing.
