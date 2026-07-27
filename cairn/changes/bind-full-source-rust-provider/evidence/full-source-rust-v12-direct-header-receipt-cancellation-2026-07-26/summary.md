# Full-source Rust v12 cancellation

## Question

Could v12 provide durable completion evidence after the Linux-header support input was added?

## Inspected evidence

- v12 used the authenticated v5 host-tool manifest and generated explicit Linux-header include flags.
- Its first-stage plan bound the host-tool manifest digest.
- Its retained plan did not list the typed header support input.
- Its stage-construction identity did not contain a direct `linux-headers-blake3=` argument.
- The planned provider output did not exist when the run was canceled.
- The preserved driver, plan, generated script, partial build log, and `blake3sums.txt` record the canceled attempt.

## Decision

Cancel v12 before publication. A transitive host-manifest commitment is useful, but it is not the explicit per-stage Linux-header commitment required for durable review.

The replacement implementation retains support-input identities in each stage plan, copies them into the final binding receipt, and adds the exact Linux-header tree BLAKE3 to every stage-construction identity.

## Owner

Mantle full-source bootstrap implementation.

## Next action

Run focused positive and negative tests for the direct binding. Rebuild Mantle, remove the canceled scratch only after this record is durable, and start a fresh construction with new paths.
