# 0162. Keep the vendored Snix data layer, with upstream terms recorded

Status: Proposed

## Context

`vendor/nix-compat`, `vendor/nix-compat-derive`, `vendor/snix-build`,
`vendor/snix-castore`, `vendor/snix-store` and `vendor/snix-tracing` are
workspace members carrying roughly 43K lines of Snix-derived Rust. They had no
manifest `license`, no license text, and no upstream record. The workspace is
`AGPL-3.0-or-later` and `deny.toml` skips private packages, so the license gate
never evaluated them.

Upstream states GPLv3 terms: snix.dev/about describes Snix as GPLv3-licensed,
and the Tvix README it descends from licenses all code under GPL-3.0 except the
protocol-buffer definitions (MIT). That applies to these files regardless of the
workspace-wide expression. `scripts/import-snix-build-fork.py` records that
`snix-build` came from a pinned Mantle tree with tracked patches, not from a
known upstream revision.

The six crates are the store-protocol data layer used by the build pipeline
(ADR 0001). Replacing them is a product change with its own compatibility and
evidence cost; consuming them unmodified is not currently possible because
Mantle patches BLAKE3 support in `nix-compat` and the bubblewrap service in
`snix-build`.

## Decision Drivers

- Distribute and modify upstream GPL-3.0 code only with its notices and
  modification record.
- Keep the license gate meaningful for every workspace member.
- Preserve an auditable base for upstream security fixes and local diffs.
- Keep the built binary's terms explicit per component.
- Avoid inventing provenance that cannot be verified here.

## Decision

The Snix data layer stays vendored and modified for now. Each of the six
directories carries:

- `LICENSE` — the GNU GPL version 3 text (as published by the FSF);
- `NOTICE` — the upstream repository, the upstream license expression used here
  (`GPL-3.0-only`), and a summary of Mantle's modifications;
- `upstream.json` — repository, base revision (or an explicit unresolved state),
  the method used, and residual uncertainty.

Each manifest sets `license = "GPL-3.0-only"`. `deny.toml` evaluates private
workspace members and explicitly admits `GPL-3.0-only` for these crates, in
addition to the existing `AGPL-3.0-or-later` entry for Mantle-owned code.
Mantle-authored code remains `AGPL-3.0-or-later`; the combined binary is offered
under AGPL-3.0-or-later with the per-component notices retained, and the README
states that structure.

The recorded upstream base revision is unresolved: `git.snix.dev` is unreachable
from the current build host, and the local history does not contain a
reconstructible import record. `upstream.json` states that explicitly rather
than asserting a revision. Identifying the base and producing the local patch
series is required before the next upstream sync and before any
redistribution-with-binary claim.

## Consequences

- Upstream terms are visible in each vendored directory and in the license gate,
  so relicensing them under the workspace expression can no longer happen by
  omission.
- The AGPL still covers Mantle-owned code; the optional commercial terms for the
  combined work cannot be applied to the Snix-derived parts.
- The unresolved base is a known compliance and maintenance gap: no diff base
  for upstream security fixes. A follow-up must clone the upstream repository
  from a host that can reach it, identify the nearest revision by content
  comparison, and record the patch series.
- `cargo deny check licenses` now evaluates these members; adding a new
  workspace member with an unlisted license fails the gate unless deliberately
  admitted.
- This record is engineering provenance, not legal advice; counsel review of the
  combined-work terms is required before any distribution claim.
