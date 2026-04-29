Task-ID: V2
Covers: bootstrap.part.tinycc.0.9.27

# tinycc 0.9.27 build transcript

Result: PASS.

Command:

```sh
CRUNCH_NO_FUSE=1 \
SNIX_BUILD_SANDBOX_SHELL=/nix/store/4mdqc2snfiihr6r61ln1rqs4fis6br9b-busybox-static-x86_64-unknown-linux-musl-1.36.1/bin/busybox \
./target/debug/crunch build bootstrap/tinycc.ncl \
  --store "$PWD/target/live-part-tinycc-0-9-27/run-current/store" \
  --state-dir "$PWD/target/live-part-tinycc-0-9-27/run-current/state" \
  --no-substitute -j 1 --verbose --log-level info
```

Full transcript: `evidence/V2-build-full.log`.

Output path:

```text
/home/brittonr/git/crunch/crunch/target/live-part-tinycc-0-9-27/run-current/store/0vz94d750q8wbylhln3sza1zwcwsvb1p-tinycc-0.9.27
```

Elapsed time:

```text
real 0.58
user 0.46
sys 0.10
```

Relevant transcript excerpt:

```text
build succeeded drv=tinycc-0.9.27.drv outputs=["/home/brittonr/git/crunch/crunch/target/live-part-tinycc-0-9-27/run-current/store/0vz94d750q8wbylhln3sza1zwcwsvb1p-tinycc-0.9.27"]
worker streaming finished completed=1 succeeded=1 failed=0 roots=1
tcc version 0.9.26 (x86_64 Linux)
-> tcc.c
<- tcc-final
tcc version 0.9.27 (x86_64 Linux)
hermeticity: practical (no degraded facts)
```
