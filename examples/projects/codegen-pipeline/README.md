# Code-generation pipeline project

This project builds a real dependency pipeline:

```text
model -> generated shell source -> runnable application -> package check
```

Run from this directory:

```bash
mantle build
mantle build .#generated-source
mantle build .#app
mantle build .#checks.app
```

The check covers both the successful invocation and rejection of an unexpected argument.
