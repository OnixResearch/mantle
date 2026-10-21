# ADR 0119: Declare native inputs for the offline Cargo oracle

- **Status:** Proposed
- **Date:** 2026-10-18

## Context

Issues [#1](https://github.com/OnixResearch/mantle/issues/1) and
[#2](https://github.com/OnixResearch/mantle/issues/2) require comparing Cargo unit
execution with the existing `offlineCargoPackage` lane using the same source,
lock, target, tools and native inputs. The native fixture's build script requires
an explicit compiler, archiver and Mantle-built native provider. The offline
helper previously declared no native roles or environment, so that comparison
could not use the same inputs without an ambient tool fallback.

The unit-plan lane already uses `NativeInputDeclaration` and `{{role:name}}`
templates. Its declarations are scoped to exact Cargo package identities. An
ordinary Cargo invocation has one inherited environment; it cannot claim that
package-level isolation.

## Decision drivers

- Preserve the ordinary offline Cargo lane as the comparison oracle.
- Bind every native tool and library root into the derivation's inputs.
- Reuse the existing `inputs`, `env`, `path` role-template vocabulary.
- Keep the helper's Cargo, Rust, shell and loader controls authoritative.
- Distinguish invocation-wide declaration from per-package unit admission.

## Decision

Add an optional `native_inputs = { inputs, env, path }` declaration to
`offlineCargoPackage`, with empty defaults. Roles identify whole store roots
under the declared logical prefix, not host paths or members disguised as
roots. Environment and PATH values resolve only explicitly declared role
markers, with the existing finite entry/value bounds and traversal rejection.
Native roots join the ordinary derivation input closure.

Pass resolved native environment values through the derivation's environment,
not interpolated shell commands. Quote declared PATH additions. Reject unknown
roles/fields, ambient paths, and overrides of the helper's reserved execution
controls. The invocation-wide environment has additional Cargo/Rust/shell/loader
restrictions; it does not weaken the unit-plan build-script environment policy.

Retain the resolved native declaration and the explicit
`scope = "cargo-invocation-wide"` in the existing v2 raw build sidecar. The generic
build report's bounded projection remains unchanged. Comparison evidence retains
the evaluated derivation, raw sidecar and exact native values for both lanes.

## Consequences

The current native build-script fixture can be supplied to both existing lanes
without host compiler discovery, prebuilt-only substitution or a second native
input protocol. This adds no resolver, scheduler, native authority or universal
build-script compatibility claim. Equal declared inputs do not themselves prove
output parity; the real cold, warm, edit and signed clean-client scenarios still
have to execute.

The Mantle stdlib/offline-Cargo maintainers own the helper and contract fixtures;
Rust unit-plan maintainers own package-scoped execution. Repeatability evidence
comes from the source-bound Cargo acceptance driver, not this ADR or an
unevaluated recipe. Acceptance remains pending execution and repository gates.
