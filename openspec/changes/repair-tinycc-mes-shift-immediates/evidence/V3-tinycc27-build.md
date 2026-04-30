Task-ID: V3
Covers: bootstrap.part.tinycc.0.9.26.shift-immediates

# TinyCC 0.9.27 build from repaired predecessor

Result: PASS. `bootstrap/tinycc.ncl` builds from the repaired TinyCC 0.9.26 predecessor.

Command:

```sh
export CRUNCH_NO_FUSE=1
export SNIX_BUILD_SANDBOX_SHELL=/nix/store/4mdqc2snfiihr6r61ln1rqs4fis6br9b-busybox-static-x86_64-unknown-linux-musl-1.36.1/bin/busybox
./target/debug/crunch build bootstrap/tinycc.ncl \
  --store "$PWD/target/live-part-tinycc-0-9-27/run-current/store" \
  --state-dir "$PWD/target/live-part-tinycc-0-9-27/run-current/state" \
  --no-substitute -j 1 --verbose --log-level info
```

Output:

```text
build succeeded drv=tinycc-0.9.27.drv outputs=["/home/brittonr/git/crunch/crunch/target/live-part-tinycc-0-9-27/run-current/store/44mb0vkwi7i3cs5r5dj8l92645a250g6-tinycc-0.9.27"]
hermeticity: practical (no degraded facts)
/home/brittonr/git/crunch/crunch/target/live-part-tinycc-0-9-27/run-current/store/44mb0vkwi7i3cs5r5dj8l92645a250g6-tinycc-0.9.27
```

Build log proof lines:

```text
tinycc-0.9.27: compile tcc-final with /crunch/store/idif5ffznlbrn0ji82q6fjix3pkw1n6s-tinycc-0.9.26/bin/tcc-0.9.26
tcc version 0.9.27 (x86_64 Linux)
```

Full transcript: `evidence/V3-tinycc27-build-full.log`.
