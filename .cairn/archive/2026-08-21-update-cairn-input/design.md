# Design: Update cairn input

## Context

Mantle never declared a `cairn` input. The binary on PATH came from the user profile, and its policy parser predates the committed generated policy.

## Decision

Declare one pinned root input (`695124d459574ba7aeba6097310d237f393c243c`) and place its package on the dev-shell PATH ahead of any ambient install. The octet input keeps its own internal cairn node; nothing follows anything.

## Verification

- `readlink -f $(which cairn)` inside `nix develop` resolves to the pinned build, not the profile path.
- `cairn validate --root .` passes.
- The Nix check failure set is unchanged from base.

## Claim Boundary

This proves tool/schema agreement for lifecycle commands. It does not prove bootstrap inventory cleanliness or release eligibility.
