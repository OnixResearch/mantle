# Recipe-binding rail negative controls (2026-10-08 UTC)

Guarded Leviathan run `/home/brittonr/mantle-cairn-finish/logs/add-bootstrap-source-pins/rail-negatives-20261008T005414Z` (mcf-guard.sh; full log retained on Leviathan).
Job script: `/home/brittonr/mantle-cairn-finish/bin/jobs/bsp-rail-negatives.sh`.

## guard.log

```text
phase=preflight start=2026-10-08T00:54:14Z free=1119132880896 floor=214748364800 peak_reserve=429496729600 dev=66305
cwd=/home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins head=e24bbbc2f803f59872e2e59370bd8ce0829c918e CARGO_TARGET_DIR=/home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins TMPDIR=/home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins CARGO_BUILD_JOBS=32
command=nix develop /home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins --command /home/brittonr/mantle-cairn-finish/bin/jobs/bsp-rail-negatives.sh
phase=running launcher=1101527 pgid=1101527
phase=child_exited exit=0 elapsed_s=62 free=1100538777600
phase=final exit=0 group_survivors=0 end=2026-10-08T00:55:16Z elapsed_s=62 free=1100523368448
```

## exit-code

```text
0
```

## output.log (cargo Compiling/Checking progress lines elided)

```text
warning: Git tree '/home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins' is dirty
head=e24bbbc2f803f59872e2e59370bd8ce0829c918e work=/home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.Kz069K/rail-neg-20261008T005503Z
--- control mutation
+ python3 /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.Kz069K/rail-neg-20261008T005503Z/control/bootstrap/pins/generate_readers.py --check-recipes
control_exit=0
generated/picolibc.json
evaluated picolibc-1.8.12-src.ncl
evaluated picolibc-1.8.12-diagnostic.ncl
--- hardcoded_url mutation
14c14
<   url = pin.artifacts.source.url,
---
>   url = "https://example.org/hardcoded-source.tar.gz",
+ python3 /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.Kz069K/rail-neg-20261008T005503Z/hardcoded_url/bootstrap/pins/generate_readers.py --check-recipes
hardcoded_url_exit=1
  File "/home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.Kz069K/rail-neg-20261008T005503Z/hardcoded_url/bootstrap/pins/generate_readers.py", line 76, in check_recipe
    raise ValueError(f"{recipe}: evaluated {role} fetch URL, sha256 hash, or flat/tree mode differs from pin")
ValueError: cmake-3.31.8-gcc10.ncl: evaluated source fetch URL, sha256 hash, or flat/tree mode differs from pin
--- old_url_after_bump mutation
14c14
<   url = pin.artifacts.source.url,
---
>   url = "https://github.com/Kitware/CMake/releases/download/v3.31.7/cmake-3.31.7.tar.gz",
+ python3 /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.Kz069K/rail-neg-20261008T005503Z/old_url_after_bump/bootstrap/pins/generate_readers.py --check-recipes
old_url_after_bump_exit=1
  File "/home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.Kz069K/rail-neg-20261008T005503Z/old_url_after_bump/bootstrap/pins/generate_readers.py", line 76, in check_recipe
    raise ValueError(f"{recipe}: evaluated {role} fetch URL, sha256 hash, or flat/tree mode differs from pin")
ValueError: cmake-3.31.8-gcc10.ncl: evaluated source fetch URL, sha256 hash, or flat/tree mode differs from pin
--- wrong_hash mutation
15c15
<   hash = pin.artifacts.source.hash,
---
>   hash = "sha256-Gl7VQbtxjjyFy8rCJelqLhdl4TbO3L0FPumoBQb2+28=",
+ python3 /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.Kz069K/rail-neg-20261008T005503Z/wrong_hash/bootstrap/pins/generate_readers.py --check-recipes
wrong_hash_exit=1
  File "/home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.Kz069K/rail-neg-20261008T005503Z/wrong_hash/bootstrap/pins/generate_readers.py", line 76, in check_recipe
    raise ValueError(f"{recipe}: evaluated {role} fetch URL, sha256 hash, or flat/tree mode differs from pin")
ValueError: cmake-3.31.8-gcc10.ncl: evaluated source fetch URL, sha256 hash, or flat/tree mode differs from pin
--- flat_mode mutation
13c13
< let cmake_src = crunch.fetchTarball {
---
> let cmake_src = crunch.fetchurl {
+ python3 /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.Kz069K/rail-neg-20261008T005503Z/flat_mode/bootstrap/pins/generate_readers.py --check-recipes
flat_mode_exit=1
  File "/home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.Kz069K/rail-neg-20261008T005503Z/flat_mode/bootstrap/pins/generate_readers.py", line 76, in check_recipe
    raise ValueError(f"{recipe}: evaluated {role} fetch URL, sha256 hash, or flat/tree mode differs from pin")
ValueError: cmake-3.31.8-gcc10.ncl: evaluated source fetch URL, sha256 hash, or flat/tree mode differs from pin
=== stale derived reader
stale_reader_exit=1
    raise ValueError(f"stale derived pin reader: {output}")
ValueError: stale derived pin reader: /home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins/nix-shell.Kz069K/rail-neg-20261008T005503Z/stale_reader/bootstrap/pins/generated/cmake.json
=== unknown TOML field
unknown_field_exit=1
    raise ValueError(f"pin fields must be exactly {PIN_FIELDS}; got {sorted(record)}")
ValueError: pin fields must be exactly ('schema', 'source', 'package_url', 'version', 'release_date', 'artifacts', 'resolve', 'recipes'); got ['artifacts', 'mystery', 'package_url', 'recipes', 'release_date', 'resolve', 'schema', 'source', 'version']
rail_negatives_done
```
