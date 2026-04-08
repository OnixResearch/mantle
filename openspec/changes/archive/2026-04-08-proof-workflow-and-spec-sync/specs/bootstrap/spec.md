## ADDED Requirements

### Requirement: Checked-in self-hosting proof entry point

The repo MUST provide one checked-in entry point for running the self-hosting
proof from the repo root.

That entry point MUST set the required build environment explicitly before it
invokes the proof. At minimum it MUST account for the Rust nightly toolchain,
C compiler availability, pkg-config / openssl lookup, and
`SNIX_BUILD_SANDBOX_SHELL`.

The entry point MUST reuse the existing ignored self-hosting proof path instead
of reimplementing the stage0 -> stage1 -> stage2 logic in a second place.

#### Scenario: Contributor runs the checked-in proof entry point

- GIVEN a contributor on Linux in the repo root
- AND the required host tools are installed
- WHEN they run the checked-in proof entry point
- THEN it prepares the required build environment
- AND it invokes the canonical ignored self-hosting proof
- AND the proof output still comes from the existing stage0 -> stage1 -> stage2 test path

#### Scenario: Missing prerequisite fails fast

- GIVEN a contributor is missing a required tool or environment input
- WHEN the checked-in proof entry point starts
- THEN it fails before the long proof build begins
- AND the error names the missing prerequisite

## MODIFIED Requirements

### Requirement: Documented self-hosting workflow

The repo MUST document the checked-in proof entry point as the default way to
run the self-hosting proof locally.

The docs MUST name the exact command, list the required host prerequisites, and
state what the proof demonstrates and what it does not.

#### Scenario: Docs and helper agree

- GIVEN the checked-in proof entry point exists
- WHEN a contributor follows the proof instructions in the repo docs
- THEN they run that same checked-in entry point
- AND they do not need to reconstruct the PATH / environment setup by hand
