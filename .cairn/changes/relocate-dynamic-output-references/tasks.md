# Tasks: Relocate dynamic output references

Source implementation and fixture delivery are distinct from producer acceptance.
Checked core tasks do not accept the production loader, GCC family, launcher
cutover, closure parity, or second-prefix runtime gates.

## Phase 1: Baseline and contract

- [ ] [serial] T1.1 Create an isolated worktree from current `origin/main`. Record the current absolute-reference surfaces in the GCC shared-runtime family, the existing ELF rewriting modules, scanner behavior, and focused baseline test output. r[mantle.relocatable_outputs.bounded_fixup_admission]
- [ ] [serial] T1.2 Define the relativizable reference contract: entry forms, hash-literal invariant, link-time capacity reservation flags, and opt-in policy fields. r[mantle.relocatable_outputs.origin_relative_needed] r[mantle.relocatable_outputs.bounded_fixup_admission]
- [ ] [serial] T1.3 Record the hash-literal and no-interp-stub-first decisions in an ADR. r[mantle.relocatable_outputs.origin_relative_needed]

## Phase 2: Core fixup

- [x] [serial] T2.1 Implement pure fixup admission and planning over ELF images and a declared dependency map: capacity checks, symbol-tail overlap detection, typed refusals. r[mantle.relocatable_outputs.bounded_fixup_admission]
- [x] [parallel] T2.2 Add positive fixtures: padded RUNPATH rewrite, NEEDED relativization, byte-length invariance. r[mantle.relocatable_outputs.origin_relative_needed]
- [x] [parallel] T2.3 Add negative fixtures: symbol-tail overlap, missing capacity, wrong class or endianness, truncated image, dependency-map mismatch. r[mantle.relocatable_outputs.bounded_fixup_admission]

## Phase 3: Shell and adoption

- [ ] [serial] T3.1 Wire reserved-capacity link flags into the GCC shared-runtime link steps and run the fixup as a finish step for the adopted family. r[mantle.relocatable_outputs.origin_relative_needed]
- [ ] [serial] T3.2 Implement the launcher-record form for wrapper outputs with bounded admission and fail-closed records. r[mantle.relocatable_outputs.record_launchers]
- [ ] [serial] T3.3 Add scanner parity and closure parity fixtures for relativized outputs. r[mantle.relocatable_outputs.reference_scanning_parity]
- [ ] [serial] T3.4 Add the prefix-independence check: run the declared check from a copy under a second configured logical prefix. r[mantle.relocatable_outputs.prefix_independence]

## Phase 4: Verification and integration

- [ ] [parallel] T4.1 Add relocation and second-prefix end-to-end runs for the adopted family, including negative controls for absolute residue and wrong-hash entries. r[mantle.relocatable_outputs.prefix_independence] r[mantle.relocatable_outputs.reference_scanning_parity]
- [ ] [serial] T4.2 Run focused core and shell tests before and after changes, strict Clippy, and relevant Nix checks. Preserve exact blockers. r[mantle.relocatable_outputs.bounded_fixup_admission]
- [ ] [serial] T4.3 Sync accepted specs and archive through the isolated branch workflow with retained completion evidence. r[mantle.relocatable_outputs.reference_scanning_parity]

## Current source delivery and verification boundary (2026-10-23)

The bounded core and atomic publication shell exist in
`src/elf_dynamic_fixup_{core,shell}.rs`. Admission now also rejects member-relative
loader expansion tokens, NULs and RUNPATH colon/semicolon separators, so a
declared library member cannot inject another search directory. Dependency names
must match a full existing path component, not merely a member-name prefix.
Dynamic-string sharing is checked in both directions, including an unrewritten
SONAME sharing the exact target offset; symbol names beginning before a target
are protected as well as names beginning inside it. Native fixtures cover these
refusals and preservation of unpublished file bytes. No build or test was run
for this delivery, as requested; no new execution result is claimed.

Retained mechanism evidence and the actual adoption blockers are in
`evidence/native-elf-2026-10-01.md`: host ELF/scanner fixtures passed in isolation,
but musl 1.2.5 opens slash-bearing `$ORIGIN` NEEDED literally and the tested
executable exited 127. Basename NEEDED plus hash-bearing RUNPATH does not satisfy
the normative contract and was bypassed by hostile `LD_LIBRARY_PATH`. The
scratch-only loader patch is not production: immutable flat-store/closure-root
authority and writable replacement races remain unresolved. Lexically valid
hash-looking names do not authenticate a store root or symlink target.

The retained GCC rehearsal additionally failed before building at the missing
native StageX transition input. GCC link-owned capacity reservation, wrapper
launcher-record migration, adopted runtime checks under two configured logical
prefixes, and real PathInfo closure parity remain open. They must not be marked
accepted on the strength of host fixtures or a scratch loader. No accepted spec,
dependency pin, loader policy, or resource cap was weakened to bypass a check.

Parent integration commands (not yet executed here):

```sh
nix develop -c cargo test -p mantle --test elf_dynamic_relocation --test elf_dynamic_reference_scanner -- --nocapture
```

For an actual post-build CLI smoke, retain before/after bytes and `readelf -d`
from a native padded fixture, then run the CLI exercised in the retained proof:

```sh
target/debug/mantle bootstrap elf-relocate <elf-file> --depth 1 --needed <exact-needed> <hash-name> lib/libnative.so --runpath <exact-runpath> <hash-name> lib
```

Require two rewrites, unchanged byte length and literal dependency hash, execution
from the original and copied side-by-side closure, and typed refusal with no
publication for wrong member or unsafe member-relative path. The native fixture
suite authors these exact ELF inputs; do not substitute a musl loader or claim
production adoption from its host-interpreter result. Archive/sync and all
production acceptance work remain parent-owned and unexecuted here.
