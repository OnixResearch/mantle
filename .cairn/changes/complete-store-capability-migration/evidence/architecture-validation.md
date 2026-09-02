# Store capability architecture validation

## `architecture-self-test`

Command:

```text
nix develop -c cargo -Zscript scripts/check-store-capability-architecture.rs --self-test
```

Output:

```text
store capability architecture positive and negative self-tests passed

```

## `architecture-check`

Command:

```text
nix develop -c cargo -Zscript scripts/check-store-capability-architecture.rs --root .
```

Output:

```text
store capability architecture verified: external runtime findings=0

```

## `nix-architecture`

Command:

```text
nix build path:/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-complete-store-capability-migration#checks.x86_64-linux.store-capability-architecture --no-link -L --builders ''
```

Output:

```text
this derivation will be built:
  /nix/store/ncz9mq0dm8pv0g4fx7l896rq4nxy6kdi-mantle-store-capability-architecture-check-0.1.0.drv
building '/nix/store/ncz9mq0dm8pv0g4fx7l896rq4nxy6kdi-mantle-store-capability-architecture-check-0.1.0.drv'...
mantle-store-capability-architecture-check> cargoArtifacts not set, will not reuse any cargo artifacts
mantle-store-capability-architecture-check> Running phase: unpackPhase
mantle-store-capability-architecture-check> unpacking source archive /nix/store/1p9pkf14g9612jhifhp411pl6gf9wzxn-source
mantle-store-capability-architecture-check> source root is source
mantle-store-capability-architecture-check> Running phase: patchPhase
mantle-store-capability-architecture-check> Executing configureCargoCommonVars
mantle-store-capability-architecture-check> Running phase: updateAutotoolsGnuConfigScriptsPhase
mantle-store-capability-architecture-check> Running phase: configurePhase
mantle-store-capability-architecture-check> will append /build/source/.cargo-home/config.toml with contents of /nix/store/dk6bnarpmw9b6cihqim3xsvphnc776qi-vendor-cargo-deps/config.toml
mantle-store-capability-architecture-check> default configurePhase, nothing to do
mantle-store-capability-architecture-check> Running phase: buildPhase
mantle-store-capability-architecture-check> +++ command cargo --version
mantle-store-capability-architecture-check> cargo 1.96.0-nightly (a357df4c2 2026-04-03)
mantle-store-capability-architecture-check> +++ command cargo -Zscript --offline scripts/check-store-capability-architecture.rs --self-test
mantle-store-capability-architecture-check>      Locking 9 packages to latest Rust 1.96.0-nightly compatible versions
mantle-store-capability-architecture-check>       Adding syn v2.0.117 (available: v3.0.3)
mantle-store-capability-architecture-check>    Compiling proc-macro2 v1.0.106
mantle-store-capability-architecture-check>    Compiling unicode-ident v1.0.24
mantle-store-capability-architecture-check>    Compiling quote v1.0.45
mantle-store-capability-architecture-check>    Compiling same-file v1.0.6
mantle-store-capability-architecture-check>    Compiling walkdir v2.5.0
mantle-store-capability-architecture-check>    Compiling syn v2.0.117
mantle-store-capability-architecture-check>    Compiling check-store-capability-architecture v0.0.0 (/build/source/scripts/check-store-capability-architecture.rs)
mantle-store-capability-architecture-check>     Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.69s
mantle-store-capability-architecture-check>      Running `.cargo-home/build/9c/ee76bed7267b58/target/debug/check-store-capability-architecture --self-test`
mantle-store-capability-architecture-check> store capability architecture positive and negative self-tests passed
mantle-store-capability-architecture-check> +++ command cargo -Zscript --offline scripts/check-store-capability-architecture.rs --root .
mantle-store-capability-architecture-check>     Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.09s
mantle-store-capability-architecture-check>      Running `.cargo-home/build/9c/ee76bed7267b58/target/debug/check-store-capability-architecture --root .`
mantle-store-capability-architecture-check> store capability architecture verified: external runtime findings=0
mantle-store-capability-architecture-check> Running phase: installPhase
mantle-store-capability-architecture-check> Running phase: fixupPhase
mantle-store-capability-architecture-check> shrinking RPATHs of ELF executables and libraries in /nix/store/97jyhgl8zyljg1dgr465sl1w2dz00ira-mantle-store-capability-architecture-check-0.1.0
mantle-store-capability-architecture-check> checking for references to /build/ in /nix/store/97jyhgl8zyljg1dgr465sl1w2dz00ira-mantle-store-capability-architecture-check-0.1.0...
mantle-store-capability-architecture-check> patching script interpreter paths in /nix/store/97jyhgl8zyljg1dgr465sl1w2dz00ira-mantle-store-capability-architecture-check-0.1.0

```

