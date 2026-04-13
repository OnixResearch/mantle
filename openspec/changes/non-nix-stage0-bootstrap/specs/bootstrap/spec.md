## ADDED Requirements

### Requirement: First bootstrap path is Nix-free by contract

The repo MUST define one first-bootstrap path on Linux that does not require
Nix to be installed and does not invoke Nix commands implicitly.

For that path:
- `crunch self-build` and checked-in helper scripts MUST NOT execute
  `nix-build`, `nix-store`, `nix-shell`, or `nix develop`, even as fallback
  behavior
- every external dependency used before crunch can build its own tools MUST be
  classified as an explicit host prerequisite or a pinned fetched artifact with
  a verified hash
- when a required stage0 artifact is missing, the command MUST fail fast with
  a clear missing-prerequisite error instead of trying an undeclared Nix
  recovery path

#### Scenario: Missing seed tool does not trigger hidden Nix fallback

- GIVEN a Linux host without `nix-build` installed
- AND a required static busybox or comparable stage0 seed artifact is missing
- WHEN the checked-in first-bootstrap helper starts
- THEN it fails before the long bootstrap begins
- AND the error names the missing prerequisite
- AND no Nix command is invoked as fallback behavior

#### Scenario: First bootstrap contract distinguishes host prerequisites from fetched seeds

- GIVEN a contributor reviews the first-bootstrap docs or helper output
- WHEN they inspect the stage0 dependency inventory
- THEN they can distinguish host prerequisites from pinned fetched artifacts
- AND they can see which later tools are crunch-built outputs
- AND there is no unclassified dependency that only appears through host-specific discovery

### Requirement: Non-Nix-host proof stays separate from self-hosting proof

The repo MUST distinguish the current self-hosting fixed-point proof from the
stronger claim that first bootstrap works on a host where Nix commands are not
available.

The stricter proof path MUST:
- run with `nix-build`, `nix-store`, `nix-shell`, and `nix develop` absent
  from `PATH`
- record the external stage0 prerequisites it used
- fail if the first-bootstrap path tries to cross the declared contract by
  invoking an unavailable Nix command

#### Scenario: Self-hosting proof is not overinterpreted as non-Nix-host proof

- GIVEN the checked-in stage1 -> stage2 self-hosting proof passes
- WHEN a contributor reads the bootstrap docs or proof summary
- THEN the repo states that this proves a self-hosting fixed point
- AND it does not claim that the stage0 bootstrap path is already proven on a host with no Nix commands

#### Scenario: Stronger proof detects hidden Nix dependency

- GIVEN a proof run where `nix-build`, `nix-store`, `nix-shell`, and
  `nix develop` are absent from `PATH`
- WHEN the first-bootstrap path attempts to invoke one of those commands
- THEN the proof fails
- AND the failure makes the hidden dependency visible to the reviewer
