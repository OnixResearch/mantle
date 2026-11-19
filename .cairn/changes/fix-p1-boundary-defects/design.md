# Design: Fix the P1 boundary defects

## Goal and scope

Close the P1 review findings with minimal boundary changes: bounded reads,
typed limit errors, redacted secret diagnostics, explicit liveness outcomes,
and preservation of an operator's declared trust choice. No new scheduler,
store backend, parser, or trust model is introduced.

## Decisions

| Boundary | Decision | Rejected alternative |
| --- | --- | --- |
| Key files | One shared bounded, no-follow, private-mode admission helper; zeroizing buffers; public/secret agreement in the parser | Duplicated loader policies or rendering parser input |
| GC records | Preserve stored signatures verbatim; the signing invariant is scoped to build outputs | Signing unsigned imports or dropping live records |
| Worker liveness | Panic/unobserved errors complete the task; service retains quarantine | Permanent `pending()` future |
| Sandbox logs | Retain 480 KiB per stream, drain to EOF, append a truncation marker | Whole capture then filter, or closing the pipe |
| Acquisition | Separate compressed/decompressed/per-entry/total budgets with bounded spooling | One `take` around a decompressor |
| CI transport | Cargo `[patch]` from SSH to the identical HTTPS artifact revision | Git URL rewrites or an SSH agent |
| First-run key | Atomic no-replace temp file; losers load the published winner | `create_new` then write into a visible partial file |
| Tar modes | Map executable/non-executable to 0755/0644; never materialize special bits | Raw archive mode or silent chmod failure |
| Sandbox roots | `<state-dir>/build-work/<category>`, mode 0700, owner/type/mode admission | Fixed names under shared `/tmp` |

## Boundaries and ownership

- `src/signing_key.rs` and `src/shared_rust_signing_key.rs` own file
  admission; `vendor/nix-compat` owns keypair parsing.
- `crates/crunch-store/src/gc.rs` owns retained-record rewrites;
  `persist_and_export_signed_output` owns the build-output signing refusal.
- `crates/crunch-build/src/worker.rs` owns worker liveness;
  `vendor/snix-build` owns child/lease quarantine.
- `crates/crunch-build/src/fetcher.rs` owns acquisition and extraction budgets.
- `Cargo.toml` owns the transitive source redirect used by flake evaluation.

## Failure behavior

- Over-limit fetch/tar input returns `FetchError::LimitExceeded`, cleans the
  public wrapper output, and never admits a result.
- Invalid key files fail before reading contents; parse errors contain only a
  stable class.
- A panicking or unobserved build service yields a failed goal and lets watch
  shutdown drain; a service-held lease stays quarantined.
- Truncated sandbox logs are visibly marked.

## Verification

Focused unit and regression tests cover each boundary, followed by the
first-party quality gate for the touched packages. Evidence is captured under
`evidence/`.
