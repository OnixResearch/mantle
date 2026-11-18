# 0160. Classify type-only WASI imports outside deny-all virtualization

Status: Accepted

## Context

The wRPC telemetry emitter imports `wasi:otel/tracing@0.2.0-rc.2` for its
approved effect. It also imports `wasi:otel/types@0.2.0-rc.2` and
`wasi:clocks/wall-clock@0.2.3` only for type definitions: the printed clock
interface contains the `datetime` record and nothing else. The vendored
upstream dependency is `wasi:clocks@0.2.0`; `wasm-tools component wit` on the
compiled emitter shows the `0.2.3` import.

The component pipeline sent every component with a `wasi:` import through
WASI-Virt. The pinned WASI-Virt composes with `wasm-compose` 0.219, which found
no dependency to wire into a type-only instance import. The bundle therefore
stopped at `wasi-virt-failed` with `no dependencies of component ... were found`.
WAC could plug the deny-all adapter into the same import, but that would add
about 184 KB of adapter code that is never called. The 2026-10-30 integration
record attributed the failure to an emitter with no WASI imports; inspecting the
actual component showed the type-only clock import.

## Decision Drivers

- Keep deny-all virtualization for every WASI interface that can be called.
- Do not admit a capability by classification; only exact declared remaining
  imports may stay.
- Fail closed when the printed WIT cannot be attributed.
- Keep classification policy in the pure core and tool-output parsing in the
  shell.

## Decision

The shell parses one `wasm-tools component wit` inspection into
`ObservedImport { name, is_type_only }`. An import is type-only only when its
printed package interface declares no function, async function or resource.
An import without a printed package interface, or with any such item, is
callable. More than 1,024 printed interfaces fail the inspection.

`requires_wasi_virtualization` in `crunch-wasm-component-core` requires
WASI-Virt only for a callable import of a virtualized WASI package or a
Preview 1 namespace. A type-only WASI dependency, like any other import that
remains, must match the plan's `expected_remaining_imports` exactly. The
remaining-import check and every other stage are unchanged.

## Consequences

- A manifest must list type-only WASI dependencies such as
  `wasi:clocks/wall-clock@0.2.3` as remaining imports. Consumers resolve them as
  type-only instances and check that they declare no functions or resources.
- The decision admits no WASI function, resource, clock read or other effect,
  and does not change the WASI-Virt arguments for callable imports.
- Non-claims: classification is a static observation of the printed WIT, not
  proof of runtime behavior, sandbox effectiveness or Octet admission.
- The telemetry bundle still needs an Octet artifact profile that admits its
  imports; that policy belongs to Octet.
