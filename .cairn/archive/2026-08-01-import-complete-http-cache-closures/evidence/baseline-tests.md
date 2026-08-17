# Baseline tests

Date: 2026-08-01

The commands ran before implementation in the isolated worktree.

## Closure tests

```text
$ SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -q -p crunch-store --lib closure

running 21 tests
.....................
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 219 filtered out; finished in 0.07s
```

## Pull tests

```text
$ SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p crunch-store --lib pull

test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 198 filtered out; finished in 0.23s
```

The baseline proves only the existing focused tests. It does not prove recursive HTTP closure import.
