# Evaluator budget baseline

Run before evaluator-budget implementation on 2026-08-09.

## crunch-eval

```text

running 80 tests
................................................................................
test result: ok. 80 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.57s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

## benchmark harness

```text

running 23 tests
.......................
test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.28s

```

## smoke benchmark

```text
bundle=/tmp/mantle-eval-budget-baseline.json
workload=eval-fetch-git repeat_count=1 root_count=1
METRIC evaluation_wall_ns=125970865
METRIC total_wall_ns=125977448
```

BASELINE_STATUS=PASS
