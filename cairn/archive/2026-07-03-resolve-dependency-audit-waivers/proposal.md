## Why

The refreshed dependency audit now has a checked-in policy and current evidence, but some advisory waivers remain because they are upstream-blocked through transitive dependencies. Mantle should either retire those waivers with real dependency movement or record the exact upstream blockers with stronger regression guards.

## What Changes

- Investigate remaining waived advisories and their dependency paths.
- Attempt minimal safe upgrades or dependency feature changes that remove waivers.
- If a waiver remains blocked, document the precise upstream owner/version constraint and the next unblock condition.
- Strengthen audit regression tests so missing `deny.toml` or default-policy audit output cannot be mistaken for valid evidence.

## Impact

- **Files**: `Cargo.lock`, manifests, `deny.toml`, dependency-audit docs, evidence transcripts.
- **Testing**: `cargo-deny check --config deny.toml`, focused compile checks for affected crates, and negative missing-policy guard.

## Out of Scope

- Replacing major dependency stacks without a focused reason.
- Treating default cargo-deny output as accepted evidence.
- Removing a waiver without proving the advisory path is gone or safely blocked upstream.
