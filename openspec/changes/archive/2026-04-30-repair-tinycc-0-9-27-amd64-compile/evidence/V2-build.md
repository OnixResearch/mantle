Task-ID: V2
Covers: bootstrap.part.tinycc.0.9.27.amd64.compile

# TinyCC 0.9.27 build

Result: PASS.

Command:

```sh
CRUNCH_NO_FUSE=1 \
SNIX_BUILD_SANDBOX_SHELL=/nix/store/4mdqc2snfiihr6r61ln1rqs4fis6br9b-busybox-static-x86_64-unknown-linux-musl-1.36.1/bin/busybox \
./target/debug/crunch build bootstrap/tinycc.ncl \
  --store target/live-part-tinycc-0-9-27/run-current/store \
  --state-dir target/live-part-tinycc-0-9-27/run-current/state \
  --no-substitute -j 1 --verbose --log-level info
```

Output path:

```text
/home/brittonr/git/crunch/crunch/target/live-part-tinycc-0-9-27/run-current/store/aqq1ifqcckz0nllskfqz3sz36mz23n8i-tinycc-0.9.27
```

Transcript summary:

- Build succeeded for `tinycc-0.9.27.drv`.
- Build log shows `tcc version 0.9.26 (x86_64 Linux)` compiling `tcc.c`.
- Produced compiler reports `tcc version 0.9.27 (x86_64 Linux)`.
- `hermeticity: practical (no degraded facts)`.

Full transcript: `evidence/V2-build-full.log`.
