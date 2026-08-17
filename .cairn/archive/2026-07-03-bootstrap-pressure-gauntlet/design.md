## Context

The self-hosting proof shows fixed points under bounded profiles, and provider fixed-point evidence strengthens release claims. The remaining gap is reducing the bootstrap seed root and proving no undeclared host tools enter protected phases.

## Decisions

### 1. Profiles form a closed ladder

**Choice:** Reports classify default, non-Nix-host, no-host-tools, source-built-provider, reduced-seed, and full-source-root attempts with stable verdicts.

**Rationale:** A closed ladder prevents an intermediate proof from being described as a stronger bootstrap claim.

### 2. Protected exec evidence is mandatory for no-host-tools profiles

**Choice:** No-host-tools cells must record protected-exec audit digests, declared seed inventory digests, observed execs, and forbidden-tool absence.

**Rationale:** The claim is about declared dependencies, so the audit must bind both allowed and denied execution surfaces.

### 3. Remaining seed trust is explicit

**Choice:** Reports carry a `remaining_trusted_root` section naming binary seeds, fetched artifacts, source-built stages, and open blockers.

**Rationale:** Source-built progress is valuable, but only a zero-blocker source-root report should support a full-source bootstrap claim.

## Risks / Trade-offs

- Full bootstrap pressure runs are long; separate smoke and release profiles.
- Kernel/seccomp/bwrap differences may make some hosts unsupported; unsupported host classes must be recorded.
