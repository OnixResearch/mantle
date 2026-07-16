# C library and CLI project

This project keeps its C sources and Makefile as fixed local inputs, builds a static library plus CLI, runs unit tests, and exposes a downstream package check. Mantle uses its normalized bootstrap C toolchain rather than ambient host compilers.

Run from this directory:

```bash
mantle build .#source
mantle build
mantle build .#greet
mantle build .#checks.test-greet
```

The first build may fetch and reduce the pinned bootstrap toolchain, so this project is classified as heavyweight.

The package contains `bin/greet`, `lib/libgreet.a`, and `include/greet.h`. Both the C unit test and project check exercise positive and negative behavior.
