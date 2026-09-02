# Archive verification

- Archive path: `.cairn/archive/2026-09-02-bind-source-review-evidence-to-releases/`
- Cairn archive mutation manifest: `95aa24a464989e07e7377a6f86f48c40046ed9ae4f24a7f630de9ec2fc5c1dd9`
- Cairn archive receipt: `7d37ce9907ccfced107c0be7c26b46a95f00266dc5af1d3269f67e82d3562a2e`
- The active change path is absent.
- The accepted requirement is present in `.cairn/specs/verification-evidence/spec.md`.
- The canonical requirement block matches the archived delta block byte-for-byte.
- Post-archive Cairn validation reports `"valid": true`.
- Post-archive Tracey coverage is 155/155.
- Post-archive `nix flake check --no-build -L` passes.

The archive preserves the non-green ordinary full-check evidence. It does not convert either external blocker into a passing claim.
