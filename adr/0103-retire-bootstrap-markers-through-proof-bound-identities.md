# ADR 0103: Retire bootstrap markers through proof-bound identities

**Status:** Accepted

## Context

The bootstrap blocker inventory scans literal marker text. The promoted V98
proof contains accepted StageX source and receipts whose stable names include
words such as `bridge`, `timeout`, `static link`, and `placeholder`. The same
scan also matches negative facts such as `configure_bridge_compiler_use =
false` and bounded timeout controls.

Removing these terms would hide useful evidence. Making the Nix check
report-only would weaken enforcement. A broad path or directory exemption would
also hide new blocker text.

## Decision

Mantle classifies a marker in one of two narrow ways:

1. A structural rule recognizes an explicit negative bridge-use fact, a bounded
   timeout control, a negative-test success statement, or a positive smoke
   name. Near-miss positive bridge use and observed timeout diagnostics remain
   actionable.
2. A V98 proof-bound rule accepts only a declared file and marker class. The
   rule activates only after the independent bundle manifest, verification
   receipt, parity receipt, and every declared file match their exact BLAKE3
   identities.

A missing `b3sum`, changed receipt, changed file, unknown path, or unknown marker
class disables the proof-bound classification. The normal clean-baseline and
promotion-drift enforcement remains unchanged.

## Consequences

- The inventory can become clean without deleting evidence or weakening the
  gate.
- Any byte change in a proof-bound file makes its marker text actionable until a
  new proof and review updates the identity.
- New marker paths and classes fail closed.
- The checker now has an explicit runtime dependency on `b3sum`, already present
  in the Nix check environment.
- This decision classifies the recorded V98 source. It does not prove compiler
  correctness or authorize future source changes.
