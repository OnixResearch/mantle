## MODIFIED Requirements

### Requirement: Enforced sandbox security (from Lix)

The build sandbox MUST enforce the strongest available isolation on each
platform for default and strict build modes. Mantle MUST NOT silently weaken the
sandbox for proof-eligible builds.

An explicit `--impure` mode MAY allow host-sensitive inputs for development and
compatibility, but it MUST be labeled as impure, MUST record typed audit facts,
MUST remain outside existing reproducibility/determinism proof classes, and
SHOULD retain security isolation controls such as no-new-privileges and
namespaces wherever compatible.

| Platform | Mechanism |
|---|---|
| Linux | seccomp-bpf syscall filtering + no-new-privileges + namespaces |
| macOS/Darwin | `sandbox-exec` profiles (or equivalent) |
| BSD | `pledge`/`unveil` (OpenBSD), `capsicum` (FreeBSD), or chroot fallback |
| Other | chroot + restricted PATH at minimum |

The security boundary is per-platform but the principle is uniform: proof-eligible
builds cannot escape the sandbox, escalate privileges, or access undeclared
inputs. Impure builds are explicitly outside proof eligibility unless future
policy defines a separate impure evidence class.

#### Scenario: Sandbox weakening is unavailable for proof-eligible builds

- GIVEN a user starts a default or strict build
- WHEN Mantle selects the platform sandbox
- THEN it enables the strongest available isolation for that platform
- AND no CLI option weakens that proof-eligible isolation boundary

#### Scenario: Impure mode is explicit and labeled

- GIVEN a user starts a build with `--impure`
- WHEN Mantle selects the execution mode
- THEN the run is labeled impure in reports and audit facts
- AND the run cannot satisfy existing reproducibility or deterministic proof
  classes

## MODIFIED Requirements

### Requirement: Independence from Nix

mantle MUST NOT depend on Nix experimental features being available in any Nix
installation. mantle operates independently. The features listed here are design
decisions for mantle, informed by but not dependent on Nix's experimental
feature process.

Mantle MAY provide an explicit `--impure` mode analogous to Nix's impure escape
hatch, but its semantics are Mantle-owned and MUST remain machine-readable,
labeled, and proof-blocking by default.

#### Scenario: Missing Nix experimental flags do not affect mantle

- GIVEN a host Nix installation has no experimental features enabled
- WHEN mantle evaluates or builds its own Nickel-based inputs
- THEN mantle behavior is unchanged because it does not depend on those flags

#### Scenario: Mantle impure mode does not depend on Nix impure derivations

- GIVEN Nix has or lacks `impure-derivations` support
- WHEN an operator runs `mantle build --impure`
- THEN Mantle applies its own impure-mode policy
- AND it does not require Nix feature flags
