# Tasks

## Spec

- [x] [serial] Add the Mantle build-tool boundary delta and validate the Cairn package. r[build_tool_boundary.mantle_not_module_layer] r[build_tool_boundary.onix_owns_module_lowering] r[build_tool_boundary.synthetic_system_eval_not_integration] Evidence: `evidence/scaffold-validation-transcript.md` records `cairn validate --root .` with `valid: true`.

## Implementation

- [ ] [serial] Document Mantle's stable integration surface as build-tool shaped: derivations, build plans, source inputs, store operations, build reports, and diagnostics. r[build_tool_boundary.mantle_not_module_layer]
- [ ] [serial] Remove, quarantine, or mark `mantle system eval` / `crates/crunch-system` as experimental/demo-only so it is not presented as an Onix or NixOS-style production module layer. r[build_tool_boundary.synthetic_system_eval_not_integration]
- [ ] [serial] Ensure Mantle-facing external frontend APIs do not introduce roles, tags, settings contracts, upstream exports, providers, packages, or artifacts as first-class Mantle core semantics. r[build_tool_boundary.mantle_not_module_layer]
- [ ] [serial] Define the handoff expected from an Onix-owned module layer to Mantle as concrete build inputs or opaque evaluated data, not normalized module invocation units. r[build_tool_boundary.onix_owns_module_lowering]
- [ ] [serial] Create or reference a separate Onix/onix-modules change for module ABI, real Nickel `impl` invocation, settings/defaults/contracts, upstream/provider flow, and lowering into Mantle build inputs. r[build_tool_boundary.onix_owns_module_lowering]

## Verification

- [ ] [serial] Add positive tests or fixtures showing an external frontend can hand Mantle concrete derivations/build-plan inputs without module-layer fields. r[build_tool_boundary.mantle_not_module_layer]
- [ ] [serial] Add negative tests or documentation checks proving raw Onix inventory/module-layer fields are not accepted as Mantle build-tool inputs. r[build_tool_boundary.mantle_not_module_layer]
- [ ] [serial] Add checks that documentation does not present `mantle system eval` synthetic fragments as production Onix integration evidence. r[build_tool_boundary.synthetic_system_eval_not_integration]
- [ ] [serial] Add Onix-side verification in the separate Onix change for real module semantics before lowering to Mantle. r[build_tool_boundary.onix_owns_module_lowering]
- [x] [serial] Run `cairn validate --root .`. r[build_tool_boundary.mantle_not_module_layer] Evidence: `evidence/scaffold-validation-transcript.md` records `valid: true`.
- [x] [serial] Run Cairn proposal, design, and tasks gates for this change before implementation. r[build_tool_boundary.mantle_not_module_layer] Evidence: `evidence/scaffold-validation-transcript.md` records proposal, design, and tasks gate `verdict: PASS`.
