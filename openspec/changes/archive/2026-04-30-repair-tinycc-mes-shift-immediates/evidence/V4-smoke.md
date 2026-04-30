Task-ID: V4
Covers: bootstrap.part.tinycc.0.9.26.shift-immediates

# TinyCC 0.9.27 smoke after predecessor shift repair

Result: PASS. The repaired TinyCC 0.9.27 reports the expected version, compiles a trivial object, compiles GNU make `getopt.c` into a non-empty object, and rejects malformed C without timeout or segfault.

Compiler:

```text
/home/brittonr/git/crunch/crunch/target/live-part-tinycc-0-9-27/run-current/store/44mb0vkwi7i3cs5r5dj8l92645a250g6-tinycc-0.9.27/bin/tcc
```

Smoke commands bind the physical output under its embedded logical `/crunch/store/...` path with `bwrap`, because this custom-store build records Mes runtime include/library paths relative to `/crunch/store`.

Observed output:

```text
tcc version 0.9.27 (x86_64 Linux)
hello_size=672
getopt_size=15516
malformed_rc=1
```

Malformed-input classification: PASS. Exit code `1` is nonzero and is neither timeout-derived `124` nor segfault-derived `139`; no malformed object was produced.

Full transcript: `evidence/V4-smoke-full.log`.
