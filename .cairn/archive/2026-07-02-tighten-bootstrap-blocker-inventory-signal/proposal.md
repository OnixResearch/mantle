## Why

Mantle's bootstrap blocker inventory already separates promotion claims from blocker markers, but its operator signal is still noisy. A report-only run can write useful JSON/Markdown while returning a failure status, and the unsuppressed findings include checked metadata records that make it harder to see the next real repair target.

Tightening this signal makes the blocker dashboard a reliable first step before choosing a bootstrap repair slice.

## What Changes

- Make report-only inventory generation succeed when it writes a valid report, even when blockers remain.
- Separate checked evidence metadata from actionable blocker findings in the JSON and Markdown outputs.
- Preserve enforcement behavior: clean-baseline mode still fails when unsuppressed blockers or promotion claims are present.
- Add focused positive and negative regression coverage for report-only, enforcement, metadata suppression, and promotion-overclaim paths.

## Impact

- **Files**: `scripts/check-bootstrap-blocker-inventory.{rs,sh}`, generated report shape under `target/bootstrap-blocker-inventory/`, and bootstrap blocker evidence docs if wording needs clarification.
- **Behavior**: operator-facing diagnostics become less ambiguous; no bootstrap blocker is promoted or hidden.
- **Testing**: inventory self-tests, report-only run, enforcement failure fixture, and Cairn validation/gates.
