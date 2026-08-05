## Phase 1: Source and diagnostic construction

- [x] [serial] I1 Pin one Picolibc release and its complete source authority. r[bootstrap_inventory.picolibc_stagex_comparison]
  - Evidence: `evidence/pin-and-diagnostic-build.md`; `bootstrap/picolibc-1.8.12-src.ncl` pins release 1.8.12 with a `--fix`-resolved hash; licenses classified; README reference added.
  - Record the release, URLs, fixed-output hash, source BLAKE3, selected Linux profile, and configuration identity.
  - Classify library, test, and helper licenses. Preserve the required notices.
  - Add the Picolibc repository to the README reference list after source use begins.
- [x] [depends:I1] I2 Add a research-only x86_64 Linux static diagnostic under `bootstrap/`. r[bootstrap_inventory.picolibc_stagex_comparison]
  - Evidence: `bootstrap/picolibc-1.8.12-diagnostic.ncl` builds in the sandbox with explicit meson/ninja/gcc/binutils inputs; `evidence/pin-and-diagnostic-build.md` records the output and sandbox findings.
  - Use explicit Meson, Ninja, compiler, linker, archiver, and source inputs.
  - Deny network access and reject host-libc or ambient startup-library dependence.
  - Emit exact compiled-source, generated-file, tool, configuration, output, and license facts.
- [ ] [depends:I1] I3 Add the Rust comparison core and its thin report shell. r[bootstrap_inventory.picolibc_stagex_comparison]
  - Parse bounded in-memory reports and normalize path-independent comparison facts.
  - Select only `candidate`, `rejected`, or `blocked` through pure deterministic functions.
  - Use named limits, checked arithmetic, explicit assertions, and stable BLAKE3 identities.

## Phase 2: Behavior and decision evidence

- [ ] [depends:I2] I4 Run the shared libc behavior matrix and two isolated diagnostic builds. r[bootstrap_inventory.picolibc_stagex_comparison]
  - Include positive runtime behavior and malformed-source rejection.
  - Include source tamper, tool omission, host-libc dependence, and output-digest divergence cases.
  - Compare compiled units, changed source files, rewrite operations, tool roles, output bytes, behavior, and BLAKE3 identities.
- [ ] [depends:I3] I5 Emit the final comparison report and record the decision. r[bootstrap_inventory.picolibc_stagex_comparison]
  - Bind the exact native-musl baseline commit and evidence identities.
  - Write an oracle checkpoint with question, inspected evidence, decision, owner, and next action.
  - Write an ADR that preserves the outcome and all non-claims.
  - Do not modify provider selection, StageX lineage, bootstrap parity, or release status.

## Phase 3: Verification

- [ ] [serial] V1 Run positive and negative self-tests for the comparison core and report shell. r[bootstrap_inventory.picolibc_stagex_comparison]
  - Prove all three outcomes and reject malformed, oversized, incomplete, path-dependent, and contradictory reports.
- [ ] [serial] V2 Run both isolated Picolibc builds and preserve exact diagnostic transcripts. r[bootstrap_inventory.picolibc_stagex_comparison]
  - Preserve build reports, behavior output, source and tool manifests, artifact BLAKE3 values, and the final comparison report.
  - Record failed construction as `blocked` or `rejected`. Do not mark the task complete from compile output alone.
- [ ] [serial] V3 Run repository and lifecycle gates before archive. r[bootstrap_inventory.picolibc_stagex_comparison]
  - Run the source-pin checker, focused comparison self-test, `git diff --check`, and relevant format checks.
  - Run `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .`.
  - Run all proposal, design, and tasks gates for `evaluate-picolibc-stagex-runtime`.
  - Run Cairn traceability coverage and the smallest relevant Nix check.
