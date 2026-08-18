# Current Blocker Evidence: Native Registry Transitive Producer Coverage

Task-ID: native-registry-transitive-producer-coverage.I1
Covers: r[rust_package_planning.native_registry_transitive_producer_coverage]
Date: 2026-05-27
Decision owner: coding agent

## Probe

Source: `target/mantle-self-rust-plan-probe-after-push-d09d699b/receipt.json`

```text
head: d09d699ba36c5371661f120abecdbbd4711c11de
probe_status=0
topology_execution=blocked
source_closure=true
native_registry_source_planning=true
native_git_source_planning=true
native_package_target_planning=true
native_unit_graph_planning=true
native_host_unit_graph_planning=true
unit_derivation_graph=true

blocker classes:
      1 missing-dependency-producer

topology blocker:
- missing-dependency-producer: no supported target producer lib unit for dependency package registry+https://github.com/rust-lang/crates.io-index#itertools@0.10.5
```

## Inspected facts

`native_package_target_planning` contains ready native package facts for `itertools@0.10.5`:

```text
registry+https://github.com/rust-lang/crates.io-index#itertools@0.10.5	itertools	0.10.5	/home/brittonr/git/mantle/vendor-deps/itertools-0.10.5/Cargo.toml	1	1	0	3
```

`unit_derivation_graph` contains producer units for `itertools@0.12.1` and `itertools@0.14.0`, but no producer for `itertools@0.10.5`:

```text
259:registry+https://github.com/rust-lang/crates.io-index#itertools@0.12.1:itertools:lib:build	registry+https://github.com/rust-lang/crates.io-index#itertools@0.12.1	itertools	lib	build	1	0
260:registry+https://github.com/rust-lang/crates.io-index#itertools@0.14.0:itertools:lib:build	registry+https://github.com/rust-lang/crates.io-index#itertools@0.14.0	itertools	lib	build	1	0
```

At least one consumer declares an artifact for `itertools@0.10.5`:

```text
artifact:registry+https://github.com/rust-lang/crates.io-index#itertools@0.10.5:itertools
```

## Decision

Create a Cairn change for native transitive producer coverage. The implementation should either add the supported `itertools@0.10.5` producer before topology execution or move unsupported coverage to an earlier deterministic graph-planning blocker.
