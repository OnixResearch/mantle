# Baseline tests

These commands ran before composition-core implementation on 2026-08-09.

## nix develop -c cargo test -q -p snix-castore node --lib

```text

running 27 tests
...........................
test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 233 filtered out; finished in 0.00s


```

exit_status: 0

## nix develop -c cargo test -q -p snix-castore directory --lib

```text

running 94 tests
.......................................................i............................... 87/94
.......
test result: ok. 93 passed; 0 failed; 1 ignored; 0 measured; 166 filtered out; finished in 0.02s


```

exit_status: 0

## nix develop -c cargo test -q -p crunch-store handle::tests:: --lib

```text

running 71 tests
.......................................................................
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 282 filtered out; finished in 0.19s


```

exit_status: 0

## nix develop -c cargo test -q -p crunch-action-result-core

```text

running 12 tests
............
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


```

exit_status: 0

## nix develop -c cargo test -q -p crunch-build build_request::tests:: --lib

```text

running 53 tests
.....................................................
test result: ok. 53 passed; 0 failed; 0 ignored; 0 measured; 612 filtered out; finished in 0.00s


```

exit_status: 0

## nix develop -c cargo test -q -p crunch-build execution_profile --lib

```text

running 9 tests
.........
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 656 filtered out; finished in 0.00s


```

exit_status: 0

FINAL_STATUS=0
