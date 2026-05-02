# Tasks: Spike i386 live-bootstrap path

## Investigation

- [x] I1 Audit StageX and pinned upstream live-bootstrap ordering through `tcc-0.9.27 -> make-3.82 pass1`. [covers=bootstrap.i386-live-bootstrap-spike.reference-audit] Evidence: `evidence/I1-reference-audit.md`.
- [x] I2 Map Crunch amd64 assumptions that would need parameterization for an i386 proof target. [covers=bootstrap.i386-live-bootstrap-spike.crunch-assumptions] Evidence: `evidence/I2-crunch-assumption-map.md`.
- [x] I3 Decide the smallest Crunch-local prototype shape: native i386 userspace under amd64 kernel, qemu-user, or explicit non-pivot rejection. [covers=bootstrap.i386-live-bootstrap-spike.prototype-shape] Evidence: `evidence/I3-prototype-shape.md`.

## Validation

- [x] V1 Run the selected proof or record the blocking prerequisite with command transcript. [covers=bootstrap.i386-live-bootstrap-spike.runtime-proof] Evidence: `evidence/V1-proof-attempt.md`.
- [x] V2 Update the Make 3.82 runtime-validation decision: pivot to i386-first, continue amd64 repair, or keep both paths with explicit boundaries. [covers=bootstrap.i386-live-bootstrap-spike.decision] Evidence: `evidence/V2-decision.md`.
- [x] V3 Run OpenSpec validation/gates before archive. [covers=bootstrap.i386-live-bootstrap-spike.openspec] Evidence: `evidence/V3-openspec-verify.json`.
