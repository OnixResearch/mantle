# Tasks: Relocate dynamic output references

All implementation and acceptance tasks remain open. Proposal creation is not
producer acceptance.

## Phase 1: Baseline and contract

- [ ] [serial] T1.1 Create an isolated worktree from current `origin/main`. Record the current absolute-reference surfaces in the GCC shared-runtime family, the existing ELF rewriting modules, scanner behavior, and focused baseline test output. r[mantle.relocatable_outputs.bounded_fixup_admission]
- [ ] [serial] T1.2 Define the relativizable reference contract: entry forms, hash-literal invariant, link-time capacity reservation flags, and opt-in policy fields. r[mantle.relocatable_outputs.origin_relative_needed] r[mantle.relocatable_outputs.bounded_fixup_admission]
- [ ] [serial] T1.3 Record the hash-literal and no-interp-stub-first decisions in an ADR. r[mantle.relocatable_outputs.origin_relative_needed]

## Phase 2: Core fixup

- [ ] [serial] T2.1 Implement pure fixup admission and planning over ELF images and a declared dependency map: capacity checks, symbol-tail overlap detection, typed refusals. r[mantle.relocatable_outputs.bounded_fixup_admission]
- [ ] [parallel] T2.2 Add positive fixtures: padded RUNPATH rewrite, NEEDED relativization, byte-length invariance. r[mantle.relocatable_outputs.origin_relative_needed]
- [ ] [parallel] T2.3 Add negative fixtures: symbol-tail overlap, missing capacity, wrong class or endianness, truncated image, dependency-map mismatch. r[mantle.relocatable_outputs.bounded_fixup_admission]

## Phase 3: Shell and adoption

- [ ] [serial] T3.1 Wire reserved-capacity link flags into the GCC shared-runtime link steps and run the fixup as a finish step for the adopted family. r[mantle.relocatable_outputs.origin_relative_needed]
- [ ] [serial] T3.2 Implement the launcher-record form for wrapper outputs with bounded admission and fail-closed records. r[mantle.relocatable_outputs.record_launchers]
- [ ] [serial] T3.3 Add scanner parity and closure parity fixtures for relativized outputs. r[mantle.relocatable_outputs.reference_scanning_parity]
- [ ] [serial] T3.4 Add the prefix-independence check: run the declared check from a copy under a second configured logical prefix. r[mantle.relocatable_outputs.prefix_independence]

## Phase 4: Verification and integration

- [ ] [parallel] T4.1 Add relocation and second-prefix end-to-end runs for the adopted family, including negative controls for absolute residue and wrong-hash entries. r[mantle.relocatable_outputs.prefix_independence] r[mantle.relocatable_outputs.reference_scanning_parity]
- [ ] [serial] T4.2 Run focused core and shell tests before and after changes, strict Clippy, and relevant Nix checks. Preserve exact blockers. r[mantle.relocatable_outputs.bounded_fixup_admission]
- [ ] [serial] T4.3 Sync accepted specs and archive through the isolated branch workflow with retained completion evidence. r[mantle.relocatable_outputs.reference_scanning_parity]
