# Validation evidence

## Runtime and deterministic boundary

The generated GCC 4.0 wrapper now receives authority only from each explicit `libiberty`, `libcpp`, or `gcc` configure child. Its preprocessing branch rejects every request that lacks that authority, runs from another canonical directory, names a source other than `conftest.c`/`./conftest.c`, resolves outside the authority directory, requests `-o`, exceeds 65,536 source bytes, uses an unknown class, lacks audit/count state, or reaches 4,096 accepted invocations. Count admission occurs before arithmetic increment, preventing oversized persisted counts from wrapping into acceptance.

After admission and before the first compiler command, the wrapper records count, class, canonical directory, and canonical source. The parent requires a non-empty audit and validates its count and every class/directory/source tuple after configuration.

The archive-stable contract is `bootstrap/evidence/gcc-4.0-configure-preprocess-bridge.json`. It keeps `provider_eligible=false` and explicitly denies general preprocessing correctness, native GCC 4.0 correctness, seed trust removal, and independent reproducibility.

## Positive and negative checks

```text
$ nix develop -c ./scripts/check-gcc40-configure-bridge.rs --self-test
gcc40 configure preprocess bridge: PASS (classes=3, source_spellings=2, source_bytes_max=65536, invocation_count_max=4096)

$ nix develop -c cargo test -p crunch-pipeline derivation_file
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 18 filtered out; finished in 0.09s

$ nix develop -c cargo test -p mantle --test bootstrap_eval
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.46s
```

The checker includes one admitted fact set and rejects missing authority, directory mismatch, unknown class, non-conftest input, source escape, explicit output, oversized source, exhausted count, missing audit, and missing count. Separate drift fixtures remove the audit marker or flip provider eligibility and must fail. Its source-order check rejects any first compiler marker that precedes the durable audit marker.

## Broad-gate progression

The initial full `nix flake check -L` reached the Tiger Style consumer and failed on unchecked subtraction in `crates/crunch-build/src/fetcher.rs`. Replacing that subtraction with saturating arithmetic plus a compile-time bound assertion exposed seven already-landed pipeline findings. Those were repaired without changing lazy-resolution behavior: collection naming, assertion density, compound path validation, predicate names, request parameter aggregation, and function length. The focused eight-test lazy resolver rail passed after repair.

The dedicated Tiger Style Nix check then completed:

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 32.18s
```

The unavailable SSH builder remained a warning because Nix continued locally. Pueue task `180` then passed the bridge checker and the blocker checker self-tests; the enforced inventory reported `0 findings across 0 classes, 437 evidence-backed suppressions, 0 promotion claims, enforce=true`. Pueue task `181` built `checks.x86_64-linux.bootstrap-blocker-inventory` successfully in the Nix sandbox.

Pueue task `271` passed the machine-contract checker self-test and strict freshness check (`21 contracted, 50 classified`). Task `260` passed all four typed machine-contract tests. Task `266` validated the active Cairn tree and reported Tracey coverage `145/145 referenced (profile mantle-default)`. Final focused task `275` passed all 19 bootstrap evaluation tests.

Pueue task `190` ran the full build-mode `nix flake check -L`; its complete log is retained at `/tmp/mantle-full-flake-after-bootstrap.log`. The exact final output was:

```text
mantle-nextest>      Summary [  82.750s] 4048 tests run: 4048 passed, 8 skipped
mantle> test result: ok. 1596 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.81s
all checks passed!
warning: The check omitted these incompatible systems: aarch64-darwin, aarch64-linux, x86_64-darwin
```

The configured `ssh-ng://root@10.10.10.1` builder failed to connect once during the run, but Nix fell back locally and the complete host-compatible check set passed. The prior archive import mismatch did not recur in this flake rail; it was reproduced separately against the retained `bison-2.3-gcc-v6` state and remains a distinct PathInfo/NAR metadata issue outside this bridge requirement.

Pueue task `326` ran the exact `nix develop -c ./scripts/check-first-party-quality.sh` rail. The bridge checker and Rustfmt passed, strict first-party Clippy finished successfully, and serialized first-party workspace tests completed. The root package reported:

```text
[1/4] bounded GCC 4.0 configure bridge
gcc40 configure preprocess bridge: PASS (classes=3, source_spellings=2, source_bytes_max=65536, invocation_count_max=4096)
[2/4] rustfmt
[3/4] clippy
Finished `dev` profile [unoptimized + debuginfo] target(s) in 40.94s
[4/4] first-party workspace tests (serialized; vendored members excluded)
test result: ok. 1596 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.36s
```