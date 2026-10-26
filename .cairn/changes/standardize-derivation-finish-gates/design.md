# Design: Standardize derivation finish gates

## Goal and scope

One shared finish step ends every build with the same checks: version, prefix
leaks, opt-in relocation, opt-in dlopen audit. This proposal defines the
contract and the shell; adoption in existing bootstrap recipes is a separate
task inside this change.

Planning success means a native change package with requirements, ownership,
positive and negative tasks, and a recorded baseline. Gate success proves
package structure only.

## Current behavior

`builders/mk_derivation.ncl` produces string phases; smoke checks are written
per recipe in `bootstrap/*.ncl` (for example version prints, copied-tree
execution, malformed-input rejection). Nothing shared exists. Recorded
misses in the repository history: published wrappers retaining build-tree
paths, a compiler embedding logical store paths, and prefix-sensitive smokes
failing under content-addressed provisionals.

The reviewed external reference implements this as one `finish` step: version
check in an empty environment under an rtld-audit module, absolute-reference
leak warnings with a cross `--deny` list, and a relocation rerun. Its evidence
is in `evidence/repkgs-review.md`.

## Approach review

| Family | Mechanism | Disposition | Required check |
| --- | --- | --- | --- |
| Per-recipe smokes | Each recipe writes its own checks | Rejected: current state, uneven and optional | Recorded misses above |
| Shared finish gates | Builder-layer step with typed policy | Selected direction | Negative fixture per gate |
| Post-hoc verifier command | A separate `mantle verify` pass over outputs | Deferred: useful later, but builds must still fail at build time | Not blocking this change |

## Contract and component ownership

- Pure core: gate evaluation over a declared gate record and observed facts
  (command output, scan hits, audit events) in a new core module or the
  existing root-package core, with named bounds and typed denials.
- Shell: the finish step in the builder shell path runs the commands, the
  prefix scan, and the audit wiring; it records results into the existing
  build report (`crunch-build-report-v1` gains a finish-gates block, versioned).
- Policy: typed Nickel contract for gate fields, defaults, and opt-outs; the
  closed core derivation contract gains optional, documented fields.
- Reference scanning reuses the existing scanner semantics; the leak gate only
  reads it.

## Decisions

### Decision: Gates default on for new derivations only

**Choice:** Existing recipes are unchanged until they adopt the fields.

**Rationale:** Avoids a flag-day rebuild of the bootstrap chain; adoption
tasks move recipes family by family.

### Decision: Leak gate reports, never rewrites

**Choice:** The reference leak gate reports and denies only.

**Rationale:** Rewriting references is a separate mechanism owned by
`relocate-dynamic-output-references`; mixing them would hide which mechanism
changed an output.

## Risks / Trade-offs

- Empty-environment execution can break recipes that rely on ambient `HOME`;
  the contract allows a bounded allowlist of scalar environment values.
- The dlopen audit needs a loader-audit module per dynamic output family; the
  first target is the GCC shared-runtime milestone, not the static musl chain.
- Gate runtime cost is bounded by the version command and one optional copy.

## Nonzero version-command fixture and verification boundary

`tests/finish_gates.rs` reuses `tests/fixtures/finish_gates.ncl` and the existing
isolated real-CLI helper. The negative variant prints the same `demo-1.0` as the
positive fixture, then exits 7. Matching text is not sufficient: the build must
exit 1, report the version gate denied with expected version `1.0`, leave
reference-leak and relocation gates not-run, report zero built/cached and one
failed derivation, return no successful outcomes, and emit no action-result
publication report. Assertions use structured report fields, not diagnostic
wording. The existing positive fixture is the zero-exit control.

The fixture retains the existing bubblewrap capability gate and fresh temporary
store/state per invocation; a capability skip is not execution evidence. It does
not enable dlopen or claim a cross-toolchain build. The T3.2 coverage inventory
and observed verification are in `tasks.md`; pure-policy and host-loader checks
are distinguished there from sandboxed CLI fixtures. Six core and eight sandbox
fixtures passed. The retained v8 CLI control/denial pair is reused without
overwriting its evidence; the later duplicate-launch refusal remains a failed
launch, not a product pass. This fixture delivery changes neither historical
failures nor ADR/adoption/archive status.
