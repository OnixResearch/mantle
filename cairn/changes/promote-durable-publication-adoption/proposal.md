# Promote durable publication adoption

## Why

Mantle accepted the shared durable publisher at `d1f3d6d96e2b0d9cd8497cd89b8e5a93d4a7dfaf`. The published feature branch is one commit ahead of `origin/main`, and `origin/main` at `7875ec1c8b80662f183b76194ae4ef8e3cd52a28` is its ancestor.

## What Changes

- Require the accepted Mantle commit as a promotion ancestor.
- Confirm that Onix Core canonical `main` contains its accepted Radicle admission first.
- Re-run focused adoption, package, test, formatting, Clippy, Tiger Style, Nickel, and Cairn checks.
- Advance Mantle `main` only through an authorized normal fast-forward push.
- Verify the remote result. A separate dependent Cairn owns cleanup after archive publication.

## Impact

This change moves accepted work onto the canonical branch. It does not alter publication mechanics, rollback policy, source pins, product manifests, or release authority.

## Non-claims

Canonical branch placement does not prove whole-Mantle correctness, release eligibility, future producer availability, or reboot recovery.
