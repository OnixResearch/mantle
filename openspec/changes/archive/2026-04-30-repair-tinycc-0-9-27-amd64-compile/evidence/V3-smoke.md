Task-ID: V3
Covers: bootstrap.part.tinycc.0.9.27.amd64.compile

# TinyCC 0.9.27 smoke tests

Result: PASS.

Compiler:

```text
/home/brittonr/git/crunch/crunch/target/live-part-tinycc-0-9-27/run-current/store/aqq1ifqcckz0nllskfqz3sz36mz23n8i-tinycc-0.9.27/bin/tcc
```

Assertions:

- `bin/tcc -version` exited `0` and printed `tcc version 0.9.27 (x86_64 Linux)`.
- `timeout 5s bin/tcc -c hello.c -o hello.o` exited `0`.
- `hello.o` exists and is non-empty (`668` bytes).
- `timeout 5s bin/tcc -c malformed.c -o malformed.o` exited `1`.
- Malformed-input exit was not timeout-derived (`124`) and not signal/segfault-derived (`139`).
- `malformed.o` was not produced.

Full transcript: `evidence/V3-smoke-full.log`.
