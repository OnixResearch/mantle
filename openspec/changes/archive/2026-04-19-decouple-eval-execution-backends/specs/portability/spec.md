## ADDED Requirements

### Requirement: Eval execution strategy stays outside the portable core

The portable evaluation core MUST NOT require OS process management, daemon
lifecycle, or mandatory global thread-pool state in order to evaluate Nickel or
force roots.

`crunch-eval` MUST remain usable with its required serial inline backend alone.
Threaded and subprocess execution strategies MAY be shipped as host-specific
optimizations, but they MUST stay optional layers above the portable forcing
core.

If a subprocess backend is shipped, it MUST NOT require a resident service and
MUST NOT make library callers or non-process targets spawn another `crunch`
binary in order to use the eval core.

#### Scenario: Portable eval core works without host process helpers

- GIVEN a host or embedding target that does not want subprocess spawning or a
  resident worker service
- WHEN it uses the `crunch-eval` lazy forcing APIs through the required inline
  backend
- THEN evaluation and root forcing still work
- AND the portable core does not require daemon lifecycle management

#### Scenario: Subprocess backend remains optional host policy

- GIVEN a host runtime that adds a local subprocess backend for eval work
- WHEN that backend is available
- THEN it is selected as an optional host policy choice
- AND the portable eval core still remains usable without subprocess support
- AND the architecture does not require a resident daemon or cross-command
  worker service
