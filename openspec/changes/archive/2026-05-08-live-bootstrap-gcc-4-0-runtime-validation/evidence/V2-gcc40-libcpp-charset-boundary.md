# V2 follow-up: GCC 4.0 libcpp charset boundary

Command:

```sh
timeout 520 nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/gcc40-final-store" \
  bootstrap validate bootstrap/gcc-4.0.ncl \
  --evidence-dir "$PWD/.crunch-drain/gcc40-final-evidence" \
  --resume
```

Result:

- `rc=1`
- validation summary: `status=build-failed`, `failure_class=build`, `build_exit_code=1`
- doctor passed before build attempt
- derivation log: `/home/brittonr/.local/state/crunch/logs/bsvcmxs049pqfvmdvc3hj0kz4bijp64g-gcc-4.0.4.drv.log`

Boundary:

- `bootstrap/gcc-4.0.ncl` now configures `libiberty`, `libcpp`, and `gcc`.
- TinyCC-hostile `libiberty` sources are replaced by bootstrap shims (`regex`, demangle, `md5`) and `libiberty.a` is created manually.
- Validation advances past the previous `libiberty/cplus-dem.c`, `libiberty/cp-demangle.c`, `libiberty/md5.c`, and full-`libiberty` `choose-temp.c` segmentation-fault boundaries.
- Current deterministic stop is the first `libcpp` compile:

```text
gcc40-cc ... -c /tmp/gcc-build/gcc-4.0.4/libcpp/charset.c
make: *** [charset.o] Segmentation fault (core dumped)
```

This is still boundary evidence, not full GCC 4.0 runtime validation. V3+ remain open.
