# ADR 0088: Stage APK builds after verified tool extraction

## Status

Proposed

## Context

Android prebuilt SDK inputs are fixed-output *archives*. A caller-provided extracted directory is not evidence that the executable bytes came from the reviewed archive. Android APK stages also consume content-addressed outputs whose store paths cannot be guessed before realization. Re-extracting three multi-gigabyte archives in each of six tool steps needlessly multiplies work.

## Decision Drivers

- Bind every executable to an independently reviewed metadata record and archive SHA-256.
- Never execute a caller-selected substitute tree merely because it is a declared store path.
- Keep source extraction separate from five to six ordered APK tool stages.
- Do not introduce Gradle, ambient SDK discovery, or signing credentials.

## Decision

The pure no-std core validates and orders typed APK stages. The std adapter requires a reviewed toolchain cohort *separate from* the authored plan, checks the admitted archive metadata and bytes against that cohort before generating derivations, and generates exactly one deterministic extraction derivation for each of build-tools, JDK, and Android platform. Each extraction derivation rechecks its archive SHA-256 before unpacking. A trusted store capability must verify the realized extraction derivation, its archive input, and its authenticated output before exposing a tree for downstream tool stages. Steps declare only the verified extraction outputs they use, plus source and preceding realized output paths. Content-addressed stage outputs are handed to the next stage after realization; no pre-build output path is invented.

Signed builds require explicit keystore and password-file store inputs. The unsigned path ends at zipalign. Every stage pins epoch, locale, timezone, JVM properties, and sorted member ordering. Offline stub archives execute the generated extraction and tool-stage shell commands and prove the bounded ordering/wiring claim; only the separate real-build proof may claim official SDK execution.

## Consequences

The integration requires a store-backed extraction authority and staged realization rather than accepting an arbitrary extracted SDK directory or returning one lazily evaluated Nickel derivation graph. The capability's provenance check is security-critical; a test fixture implementing it is only an offline harness, not production trust. The extra three extraction derivations prevent repeated archive expansion while preserving source-archive identity. A real-SDK build and device runtime behavior remain outside this decision's proof boundary.
