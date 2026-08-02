# Portfolio search and adversarial audit

## Success contract

The goal was a bounded foreign provenance audit from signed PathInfo and castore
facts. Completion required deterministic positive and negative tests, a receipt,
CLI evidence, documentation, and passing lifecycle gates.

False completion included host-path scanning, caller-supplied observations,
unbounded decompression, silent unknown executables, stale receipts, or a new
package-correctness claim.

The search budget allowed three mechanism families, two audit rounds, repository
sources as authority, one advisory model review, and deterministic Rust, Nickel,
and Cairn checks.

## Approach registry

| Family | Mechanism | Result |
|---|---|---|
| castore-observation | Preflight signed PathInfo, retain checked values, then walk content-addressed services | validated candidate |
| build-correctness-only | Accept caller observations and run the existing validator | rejected because no production observation source exists |
| exported-filesystem | Scan realized host paths after build | rejected because host paths are replaceable and incomplete |

The selected mechanism converts scanner references into existing
build-correctness reports. It adds typed payload observations only where the
existing model has no payload class.

## Adversarial audit

The audit checked mutable-service ordering, custom store-prefix signatures,
missing content, foreign paths, unknown targets, links, containers, unsupported
executables, limits, receipt tamper, and overwrite behavior.

One actionable issue was found. A relative link inside an archive could use the
outer output root instead of the container root. The resolver now receives an
explicit link root. A negative tar fixture checks this escape.

The advisory model also proposed expected failures as possible defects. Unknown
executables, unsupported compressed containers, and depth exhaustion are
intentional fail-closed outcomes. Deterministic tests remain authoritative.

## Terminal state

The surviving castore-observation family reached validation. Residual uncertainty
is limited to static analysis: the audit does not prove dynamic loading, generated
code behavior, package correctness, or runtime safety.
