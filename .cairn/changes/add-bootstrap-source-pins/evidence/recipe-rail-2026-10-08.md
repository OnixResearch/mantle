# Reader and recipe-binding rail (2026-10-08 UTC)

Guarded Leviathan run `/home/brittonr/mantle-cairn-finish/logs/add-bootstrap-source-pins/readers-20261008T005259Z` (mcf-guard.sh; full log retained on Leviathan).
Job script: `/home/brittonr/mantle-cairn-finish/bin/jobs/bsp-readers.sh`.

## guard.log

```text
phase=preflight start=2026-10-08T00:52:59Z free=1134184968192 floor=214748364800 peak_reserve=429496729600 dev=66305
cwd=/home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins head=e24bbbc2f803f59872e2e59370bd8ce0829c918e CARGO_TARGET_DIR=/home/brittonr/mantle-cairn-finish/targets/add-bootstrap-source-pins TMPDIR=/home/brittonr/mantle-cairn-finish/tmp/add-bootstrap-source-pins CARGO_BUILD_JOBS=32
command=nix develop /home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins --command /home/brittonr/mantle-cairn-finish/bin/jobs/bsp-readers.sh
phase=running launcher=1077016 pgid=1077016
phase=child_exited exit=0 elapsed_s=75 free=1119137099776
phase=final exit=0 group_survivors=0 end=2026-10-08T00:54:14Z elapsed_s=75 free=1119135510528
```

## exit-code

```text
0
```

## output.log (cargo Compiling/Checking progress lines elided)

```text
warning: Git tree '/home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins' is dirty
head=e24bbbc2f803f59872e2e59370bd8ce0829c918e
+ nickel --version
nickel-lang-cli nickel 1.17.0 (rev 1320a98)
nickel_version_exit=0 nickel_version_wall_s=0
+ py --version
warning: Git tree '/home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins' is dirty
Python 3.13.12
python_version_exit=0 python_version_wall_s=0
+ py bootstrap/pins/generate_readers.py --check
warning: Git tree '/home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins' is dirty
generated/cmake.json
generated/picolibc.json
readers_check_exit=0 readers_check_wall_s=2
+ py bootstrap/pins/generate_readers.py --check-recipes
warning: Git tree '/home/brittonr/mantle-cairn-finish/wt/add-bootstrap-source-pins' is dirty
generated/cmake.json
evaluated cmake-3.31.8-gcc10.ncl
generated/picolibc.json
evaluated picolibc-1.8.12-src.ncl
evaluated picolibc-1.8.12-diagnostic.ncl
readers_check_recipes_exit=0 readers_check_recipes_wall_s=1
```
