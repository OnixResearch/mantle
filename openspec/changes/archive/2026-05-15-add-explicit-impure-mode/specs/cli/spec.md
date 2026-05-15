## ADDED Requirements

### Requirement: Build-entry commands expose explicit impure mode

The CLI MUST expose `--impure` on build-entry commands that can execute builders
against host-sensitive inputs. At minimum `mantle build` and `mantle self-build`
MUST accept `--impure` and pass hermeticity mode `impure` unchanged into the
pipeline or self-build orchestration.

`--impure` MUST be mutually exclusive with `--strict-hermetic`. If both are
provided, the command MUST fail before evaluation or build execution with a clear
operator diagnostic.

#### Scenario: Build selects impure mode

- GIVEN a Nickel file that can be built normally
- WHEN `mantle build --impure hello.ncl` runs
- THEN the pipeline executes with hermeticity mode `impure`
- AND JSON/human output identifies the run as impure

#### Scenario: Strict and impure flags conflict

- GIVEN an operator invokes `mantle build --strict-hermetic --impure hello.ncl`
- WHEN CLI parsing completes
- THEN the command exits non-zero before running the build
- AND the diagnostic says strict hermetic and impure modes are mutually exclusive

#### Scenario: Self-build selects impure mode

- GIVEN a self-build invocation for development compatibility
- WHEN `mantle self-build --impure --store /tmp/store` runs
- THEN the self-build flow records hermeticity mode `impure`
- AND the final report labels the result as impure
