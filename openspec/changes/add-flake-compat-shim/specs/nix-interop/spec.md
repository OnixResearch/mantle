## ADDED Requirements

### Requirement: Generated flake delegates to bundled crunch wrappers

The generated `flake.nix` MUST let Nix users invoke crunch workflows on
supported Linux Nix hosts without a host-installed crunch binary.

The shim MUST:
- import a configurable upstream crunch flake input
- expose generic wrapper apps for at least `crunch`, `build`, `run`, `shell`,
  `check`, and `refresh`
- expose `apps.default` as the same raw passthrough wrapper as `apps.crunch`
- bundle the crunch package plus runtime helpers needed by those wrappers,
  including `bubblewrap`, `git`, and an actual static busybox package
- export `SNIX_BUILD_SANDBOX_SHELL` to the bundled static busybox using an
  absolute store path before invoking crunch
- prepend bundled helper `bin/` directories to PATH before invoking crunch in
  stable order: bundled crunch, `bubblewrap`, `git`, static busybox, then the
  inherited PATH
- avoid substituting another shell implementation for the sandbox-shell export
  in this change

The shim MUST remain a delegation layer. It MUST NOT translate arbitrary crunch
project outputs into native Nix package derivations in this change.

Support scope in this change is `x86_64-linux` and `aarch64-linux` Nix hosts.
Unsupported systems MUST fail clearly instead of pretending the bundled runtime
is available.

Wrapper semantics MUST be:
- `apps.crunch` forwards raw user arguments to the bundled crunch binary
- `apps.build` forwards user arguments after a fixed `build` subcommand
- `apps.run` forwards user arguments after a fixed `run` subcommand
- `apps.shell` forwards user arguments after a fixed `shell` subcommand
- `apps.check` forwards user arguments after a fixed `check` subcommand
- `apps.refresh` forwards user arguments after a fixed `refresh` subcommand

#### Scenario: Nix run build delegates to bundled crunch

- GIVEN a project with a generated flake shim and no host-installed crunch
- WHEN `nix run .#build -- hello.ncl` runs
- THEN the wrapper invokes the bundled crunch binary
- AND crunch receives the `build hello.ncl` arguments

#### Scenario: Generic wrappers forward their fixed subcommand

- GIVEN a project with a generated flake shim
- WHEN the generated wrappers are inspected or executed
- THEN `apps.default` and `apps.crunch` forward raw user arguments to crunch
- AND `apps.build` forwards arguments after `build`
- AND `apps.run` forwards arguments after `run`
- AND `apps.shell` forwards arguments after `shell`
- AND `apps.check` forwards arguments after `check`
- AND `apps.refresh` forwards arguments after `refresh`

#### Scenario: Bundled sandbox shell is explicit

- GIVEN a generated flake shim
- WHEN a wrapper app is inspected or executed
- THEN it sets `SNIX_BUILD_SANDBOX_SHELL` to the bundled static busybox using
  an absolute store path
- AND it does not depend on an ambient host shell path for that variable
- AND it does not substitute another shell implementation for that export

#### Scenario: Scrubbed PATH still uses bundled helpers

- GIVEN a project with a generated flake shim and a host PATH without
  `crunch`, `git`, or `bubblewrap`
- WHEN `nix run .#build -- hello.ncl` runs on a supported Linux Nix host
- THEN the wrapper launches the bundled crunch binary
- AND it uses bundled `git` and `bubblewrap` from the generated flake runtime
- AND it does not depend on ambient PATH entries for those tools

#### Scenario: Scrubbed PATH preserves raw crunch passthrough

- GIVEN a project with a generated flake shim and a host PATH without
  `crunch`, `git`, or `bubblewrap`
- WHEN `nix run .#crunch -- --help` runs on a supported Linux Nix host
- THEN the wrapper launches the bundled crunch binary
- AND it forwards raw user arguments unchanged
- AND it does not depend on ambient PATH entries for helper tools

#### Scenario: Unsupported systems fail clearly

- GIVEN a generated flake shim on a system other than `x86_64-linux` or
  `aarch64-linux`
- WHEN a generated app is launched or a generated develop entry point is
  entered
- THEN it fails with an explicit Linux-only error
- AND it does not pretend the bundled runtime is available

### Requirement: Generated flake provides two develop entry points

The generated flake MUST provide a default development entry point for the
project shell and a tooling-only entry point for raw crunch access.

Required behavior:
- `devShells.default` delegates to `crunch shell` using the bundled crunch
  runtime
- `devShells.tooling` provides a plain shell with crunch and the bundled runtime
  helpers on PATH
- both shell entry points expose the bundled crunch binary, `bubblewrap`,
  static busybox, and `git` on PATH, ahead of inherited host PATH entries
- both shell entry points set `SNIX_BUILD_SANDBOX_SHELL` to the bundled static
  busybox using an absolute store path

#### Scenario: Default develop enters project shell path

- GIVEN a project whose default dev shell is defined in `crunch.ncl`
- WHEN `nix develop` runs against the generated shim
- THEN the flake delegates to `crunch shell`
- AND the resulting shell behavior matches the project's crunch shell path

#### Scenario: Tooling shell exposes crunch directly

- GIVEN a project with a generated flake shim
- WHEN `nix develop .#tooling` runs
- THEN the shell has the bundled crunch binary on PATH ahead of inherited host
  PATH entries
- AND it has bundled helper tools on PATH without auto-entering the project
  shell
- AND it sets `SNIX_BUILD_SANDBOX_SHELL` to the bundled static busybox path

#### Scenario: Default develop prefers bundled helper tools

- GIVEN a generated flake shim and a host PATH that already contains `crunch`,
  `git`, and `bubblewrap`
- WHEN `nix develop` runs
- THEN the resulting shell finds the bundled `crunch`, `git`, and `bubblewrap`
  before the inherited host tools

#### Scenario: Default develop targets the flake project root from outside

- GIVEN a generated flake shim
- WHEN `nix develop <flake-uri>` is entered from outside the project root
- THEN the resulting default develop path still targets that flake's project
  root for delegated crunch shell behavior
- AND it does not require the caller's current working directory to contain
  `crunch.ncl`

#### Scenario: Refresh wrapper carries bundled git

- GIVEN a project with a generated flake shim and no host-installed `git`
- WHEN `nix run .#refresh -- --help` runs on a supported Linux Nix host
- THEN the wrapper launches the bundled crunch binary
- AND bundled `git` is present on PATH for delegated refresh flows
- AND it does not depend on ambient PATH entries for helper tools

### Requirement: Shim output stays deterministic and relocatable

The generated flake text MUST be deterministic for the same renderer inputs.

Renderer inputs include the selected crunch input reference, but MUST NOT
include delivery mode (`--stdout` vs write target), timestamps, hostnames,
ambient PATH contents, dynamic project-output enumeration, or absolute paths
from the generation machine checkout in this change.

Wrapper and develop entry points MUST target the flake's own project root at
runtime, so they work when launched through the flake from outside the project
checkout root.

#### Scenario: Repeated generation is byte-identical

- GIVEN the same project root and `--crunch-input` value
- WHEN `crunch flake-compat --stdout` runs twice
- THEN both outputs are byte-identical
- AND no timestamp, hostname, or other ambient machine state appears in the
  generated file

#### Scenario: Generated flake is checkout-relocatable

- GIVEN a generated flake shim checked into a project checkout
- WHEN the checkout is moved to a different absolute filesystem path
- THEN the generated `flake.nix` does not require regeneration solely because
  of that move
- AND it does not embed absolute paths from the generation machine checkout

#### Scenario: Wrappers target the flake project root at runtime

- GIVEN a generated flake shim
- WHEN `nix run <flake-uri>#build -- hello.ncl` is launched from outside the
  project root
- THEN the wrapper still targets that flake's project root for delegated crunch
  commands
- AND it does not require the caller's current working directory to contain
  `crunch.ncl`
