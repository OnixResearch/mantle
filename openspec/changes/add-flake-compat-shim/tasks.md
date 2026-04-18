# Tasks: flake-compat shim

## Phase 1: Spec and UX boundary

- [x] Add delta specs for CLI command surface and Nix interop behavior
  - Evidence: `specs/cli/spec.md` and `specs/nix-interop/spec.md` now exist
    under this change.
- [x] Pin the generated shim scope: delegation wrappers only, no native Nix
      translation of crunch package outputs
  - Evidence: proposal/design/specs all state delegation-not-translation as a
    hard boundary.
- [x] Pin the bundled runtime helper set and the generated shell/app entry
      points (`default`, `crunch`, `build`, `run`, `shell`, `check`,
      `refresh`, `devShells.default`, `devShells.tooling`)
  - Evidence: `specs/nix-interop/spec.md` and `design.md` now enumerate the
    helper set and entry points explicitly.

## Phase 2: Renderer and CLI plumbing

- [ ] Add a pure renderer that emits deterministic `flake.nix` text from a
      small structured config, embeds `github:brittonr/crunch` by default, and
      propagates `--crunch-input` overrides into the rendered input reference
- [ ] Implement generated wrapper apps with `apps.default` byte-/behavior-
      identical to `apps.crunch`, raw passthrough for that shared wrapper,
      fixed-subcommand forwarding for `build`, `run`, `shell`, `check`, and
      `refresh`, and runtime anchoring to the flake's own project root
- [ ] Implement bundled helper wiring: `bubblewrap`, `git`, a fixed static-
      busybox attr with deterministic PATH helper order, absolute-store-path
      `SNIX_BUILD_SANDBOX_SHELL`, no fallback to `bash-static` or another
      shell, and a clear supported-system failure if the fixed static-busybox
      attr is unavailable
- [ ] Implement `devShells.default` / `devShells.tooling` plus runtime
      anchoring to the flake's own project root, bundled helper PATH order, and
      absolute-store-path `SNIX_BUILD_SANDBOX_SHELL` for both entry points
- [ ] Implement the delegation-only output boundary and the explicit support
      gate for `x86_64-linux` / `aarch64-linux` only
- [ ] Add `crunch flake-compat` CLI plumbing that discovers the project root and
      writes `flake.nix` by default
- [ ] Add `--stdout`, `--output <path>`, `--crunch-input <flake-ref>`, and
      `--force` support with explicit existing-file, conflicting-flag,
      pre-render conflict rejection, parent-directory-creation,
      project-root-discovery, root-relative-output, and non-project errors

## Phase 3: Verification

- [ ] Add renderer tests proving deterministic output, the default crunch input
      reference, custom `--crunch-input` handling, stdout/file byte identity,
      that `--crunch-input` changes only the embedded flake reference, and
      correct wrapper env setup (`PATH` helper order plus an absolute-store-
      path `SNIX_BUILD_SANDBOX_SHELL` export that targets the fixed static-
      busybox package specifically)
- [ ] Add CLI tests covering write mode, stdout mode with no writes,
      alternate output path with parent-directory creation, proof that
      alternate output leaves the default project-root `flake.nix` untouched,
      existing-target refusal, `--force` overwrite, explicit
      `--stdout --output` and `--stdout --force` conflicts before rendering,
      nested-subdirectory root discovery, root-relative output resolution, and
      failure outside a crunch project
- [ ] Add PATH-scrubbed generated-shim integration coverage showing wrapper apps
      delegate to bundled crunch subcommands without requiring host-installed
      `crunch`, `git`, or `bubblewrap`, explicitly verify plain `nix run` uses
      a wrapper byte-/behavior-identical to `apps.crunch`, fixed-subcommand
      forwarding for `build`, `run`, `shell`, `check`, and `refresh`,
      deterministic helper PATH order, and wrappers targeting the flake's own
      project root at runtime
- [ ] Add supported-system coverage proving the generated flake shape and attr
      resolution work for both `x86_64-linux` and `aarch64-linux`
- [ ] Add unsupported-system verification proving systems outside
      `x86_64-linux` and `aarch64-linux` still evaluate generated outputs but
      fail only at launch/entry time with an explicit Linux-only error
- [ ] Add supported-system failure coverage proving a missing fixed static-
      busybox attr fails clearly with no fallback to `bash-static` or another
      shell
- [ ] Add integration coverage for `nix develop` default delegation to
      `crunch shell` from inside and outside the project root, and for
      `nix develop .#tooling`, with both entry points exposing bundled
      `crunch`, `bubblewrap`, static busybox, and `git` on PATH plus an
      absolute-store-path `SNIX_BUILD_SANDBOX_SHELL`, and with `tooling` not
      auto-entering the project shell
- [ ] Add a verification that generated output stays delegation-only and does
      not enumerate translated project package/check outputs beyond the generic
      wrappers and two dev shells
- [ ] Add a verification that generation does not create or mutate `flake.lock`
      and that generated output is checkout-relocatable
- [ ] Document the workflow in README and/or operator docs, including the
      generated `nix run` apps (with plain `nix run` using `apps.default`),
      both `nix develop` entry points, the Linux-only support boundary
      (`x86_64-linux`, `aarch64-linux`) plus unsupported-system behavior, and
      the explicit non-goal that this shim does not promise native Nix package
      derivations for crunch outputs

## Validation

- [ ] Run `openspec validate add-flake-compat-shim`
- [ ] Run proposal, design, and tasks gates for `add-flake-compat-shim`
