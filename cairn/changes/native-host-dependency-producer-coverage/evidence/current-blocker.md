# Current Blocker Evidence: Native Host Dependency Producer Coverage

Task-ID: native-host-dependency-producer-coverage.I1
Covers: r[rust_package_planning.native_host_dependency_producer_coverage]
Date: 2026-05-27
Decision owner: coding agent

## Probe

Source: `target/mantle-self-rust-plan-probe-before-native-host-dependency-producer/receipt.json`

```text
head: ce51c175dc15631460220f221655745b9047043a
git_status_short_bytes=18
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
      1 missing-host-dependency-producer

topology blocker:
- missing-host-dependency-producer: no supported target producer lib unit for host dependency package registry+https://github.com/rust-lang/crates.io-index#rustversion@1.0.22
```

`git_status_short_bytes=18` is the active Cairn planning scaffold (`?? cairn/changes/`), not an implementation-code change.

## Inspected facts

Host units that consume `rustversion@1.0.22`:

```text
- 545:registry+https://github.com/rust-lang/crates.io-index#generator@0.8.8:build-script-build:custom-build:build
- 556:registry+https://github.com/rust-lang/crates.io-index#indoc@2.0.7:indoc:proc-macro:build
- 623:registry+https://github.com/rust-lang/crates.io-index#wasm-bindgen@0.2.117:build-script-build:custom-build:build
```

Available native producer for `rustversion@1.0.22`:

```text
- 602:registry+https://github.com/rust-lang/crates.io-index#rustversion@1.0.22:rustversion:proc-macro:build kind=proc-macro exec=host
```

## Decision

Create a Cairn change for selected host dependency producer coverage. The implementation should schedule supported proc-macro host producers before host consumers, while preserving deterministic blockers for unsupported host dependency shapes.
