Task-ID: I1
Covers: bootstrap.part.tinycc.0.9.27.amd64.compile

# TinyCC 0.9.27 amd64 compile hang reproduction

Result: PASS. The compile failure is reproduced with bounded probes.

Compiler under test:

```text
target/live-part-tinycc-0-9-27/run-current/store/0vz94d750q8wbylhln3sza1zwcwsvb1p-tinycc-0.9.27/bin/tcc
```

Commands and observed results:

```sh
bin/tcc -version
# rc=0, prints: tcc version 0.9.27 (x86_64 Linux)

timeout 5s bin/tcc -c hello.c -o hello.o
# rc=124, no hello.o written

timeout 5s bin/tcc -vv -c hello.c -o hello-vv.o
# rc=139, prints Mes-libc-corrupted format text before segfault

timeout 5s bin/tcc -c malformed.c -o malformed.o
# rc=139, prints `%s:%d: error: %s expected` before segfault

strace -ff bin/tcc -c hello.c -o hello-strace.o
# reads hello.c, then repeats brk() growth until timeout kills it
```

Localization summary:

- Version output alone is not meaningful compile evidence.
- The positive compile path opens and reads `hello.c`, then enters heap growth without writing output.
- Verbose and malformed-input paths hit Mes-libc/varargs-corrupted diagnostics and segfault.
- The failure is before downstream GNU make: `bootstrap/tinycc.ncl` itself is not an amd64 object-compile boundary.

Full transcript: `evidence/I1-reproduction-full.log`.
