# Proposal

## Summary

Add a bounded native Rust registry host-artifact topology execution slice. Mantle already executes vendored registry `lib` dependencies through the unified native topology rail from ready `native_registry_source_planning` facts. The next unsupported package class is vendored registry packages that produce or consume supported host artifacts: `custom-build` build scripts and `proc-macro` crates.

This change makes registry-backed host units participate in host-artifact/unified topology execution only when both registry source facts and native host-unit graph facts are ready. It preserves the bounded claim: declared local/vendor source roots only, explicit unit derivation receipts only, no Cargo orchestration, no `$CARGO_HOME`, no registry cache fallback, no network, and no version solving.

## Motivation

Real registry packages frequently use build scripts or proc macros. Without a registry-host-artifact seam, Mantle can prove a simple vendored registry `lib` topology but still must block or fall back for common host-artifact shapes. This weakens the replacement-Cargo evidence boundary for Crunch/Mantle package execution.

## Scope

- Gate registry-backed host-artifact topology execution on ready `native_registry_source_planning` and `native_host_unit_graph_planning` evidence.
- Execute supported vendored registry `custom-build` and `proc-macro` host units before target consumers through existing explicit rustc/build-script metadata rails.
- Bind registry source identity, lockfile checksum, vendor root, source digest, host artifact digest, build-script metadata, and target consumer artifact digests into CLI JSON receipts.
- Add positive and negative CLI fixtures for supported vendored registry host-artifact topology and missing/stale/ambient-cache-dependent vendor material.

## Non-goals

- General Cargo registry compatibility.
- Cargo as build orchestrator.
- Network fetch, `$CARGO_HOME`, Cargo registry cache, git checkout, or target-dir fallback.
- General build.rs probing semantics beyond the already bounded metadata surfaces.
- Version solving or feature unification beyond existing supported planner facts.
- Broad native-link correctness claims outside declared and validated metadata.
