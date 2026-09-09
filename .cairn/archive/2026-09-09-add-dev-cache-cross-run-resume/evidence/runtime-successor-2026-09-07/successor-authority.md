# Runtime successor authority

## Goal

Run a new V2 cold-to-cached-to-adopt cycle with source that stages `fixtures/`.

## Predecessor

`dev-cached-8e941df2` ended with exit code `1`.

Its stage-1 Rust plan could not read the required compile-time fixture. The bounded terminal receipt is in `../runtime-acceptance-2026-09-05/cached-terminal-8e941df2-attempt4/`.

## Decision

Commit `a7841aea055bf643cc473164b98a2442014f080b` stages `fixtures/` in self-build sources. The supported `refresh-mantle-source` operation created a new profile.

The new profile semantic digest is `d507f1dd17104c43789b9a29df6656249027904ab3f161c3fee96870e4d10461`. Its file BLAKE3 is `6b643b28be9de586cc5cfd8aa2dccfce7238f72c3873545cf76fb03f2d111803`.

The old provider cache cannot authorize a new profile. Cache acceptance requires an exact source-authority and plan digest. The successor starts with a new empty dev cache.

## Allowed next action

Start `dev-cold-a7841aea` only after the profile validation and free-space checks pass. A later cached or adopted run requires the prior report review.

## Non-claims

The profile refresh proves record preservation and source/vendor identity. It does not prove a build, a V2 cycle, a promoted proof, or release eligibility.
