# Focused test results

Date: 2026-08-01

## Pure closure core

```text
$ SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -q -p crunch-store --lib http_closure::
running 13 tests
.............
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 251 filtered out; finished in 0.00s
```

## HTTP closure shell

```text
$ SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -q -p crunch-store --lib pull::tests::http_closure
running 11 tests
...........
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 253 filtered out; finished in 0.14s
```

## Full pull module regression

```text
$ SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -q -p crunch-store --lib pull
running 53 tests
.....................................................
test result: ok. 53 passed; 0 failed; 0 ignored; 0 measured; 211 filtered out; finished in 0.23s
```

## Store pull CLI

```text
$ SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -q -p mantle --test integration store_pull_
running 10 tests
..........
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 58 filtered out; finished in 0.09s
```

The dependency emitted its existing unused `Unimplemented` variant warning. No focused test failed.
