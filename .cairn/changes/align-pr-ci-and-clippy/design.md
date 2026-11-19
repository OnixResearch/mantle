# Design: Run PR CI and align Clippy target coverage

## Decision

| Boundary | Decision | Rejected alternative |
| --- | --- | --- |
| Trigger | `push`, `pull_request`, `workflow_dispatch` | Push-only |
| Permissions | `contents: read` for all events | Secrets or write authority |
| Clippy targets | `--workspace --all-targets --no-deps` | `--workspace --lib` |
| Vendored exclusions | Keep the canonical list | Linting vendored members or hiding failures |
| Toolchain evidence | `rustc -Vv` step | Ambiguous nightly label |

Fork PRs run the same read-only workflow. The workflow uses only public inputs
and the already-pinned credential-free dependency sources.

## Verification

Evaluate `checks.x86_64-linux.clippy.drvPath`; a deliberate binary-only lint
regression must fail the check rather than escape a library-only leg.
