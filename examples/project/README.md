# Mantle project workflow example

`crunch.ncl` is the compatibility file name for project outputs. It keeps the
same frontend-neutral boundary as other Mantle examples: the file already
contains concrete derivations and checks; it is not a module-layer inventory.

## Prerequisite

Generate host seed paths before running the seed-dependent project example:

```bash
mantle bootstrap -o examples/project/seed.ncl
```

## Commands and expected outputs

Run commands from this directory:

```bash
cd examples/project
```

| Command | What it selects | Expected output shape |
|---|---|---|
| `mantle build` | default package (`hello`) | store path containing `bin/hello` |
| `mantle build .#hello` | named `hello` package | store path containing `bin/hello` |
| `mantle run .#hello` | named `hello` executable | stdout contains `Hello from mantle project!` |
| `mantle build .#goodbye` | named `goodbye` package | store path containing `bin/goodbye` |
| `mantle build .#checks.test-hello` | named check | store path containing `result` with text `ok` |

The fast validation suite uses a generated local project fixture with the same
selector shapes so these commands stay documented without requiring generated
seed material on every test run.
