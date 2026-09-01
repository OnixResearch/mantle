# V32 optional-dependency feature-edge repair

## Goal

Continue the preserved-provider Cargo-free diagnostic after the deterministic
manifest-path repair.

## Starting evidence

Remote pueue task `266` used source commit `0cd57953`, strict hermeticity, and
the preserved V30 provider closure. It produced 708 stage1 unit receipts. The
first failed unit was `num-bigint-dig@0.8.6`:

```text
error[E0432]: unresolved import `crate::bigrand`
  --> vendor-deps/num-bigint-dig/src/prime.rs:12:12
```

The native plan selected `prime` and the optional `rand` dependency artifact.
It did not emit `cfg(feature="rand")`. The crate gates `bigrand` behind that
feature.

The package feature definition is:

```toml
prime = ["rand/std_rng"]
```

## Root cause

The native feature resolver recorded non-weak dependency-feature edges and
forwarded the dependency feature. It did not activate an optional dependency's
implicit parent feature.

Cargo treats `dependency/feature` as an activating edge. It also selects the
implicit `dependency` feature unless a `dep:dependency` entry suppresses that
implicit feature. A weak `dependency?/feature` edge does not activate the
optional dependency.

## Repair

The native resolver now applies these three cases:

- non-weak optional edge: activate the dependency and implicit parent feature;
- weak optional edge: forward only when another feature activates the
  dependency;
- any `dep:dependency` entry: suppress the implicit parent feature while still
  allowing explicit activation.

This is general Cargo feature grammar. It is not a package-specific override.

## Validation

Pueue task `6502` recorded `local-validation.log`. Six native feature-resolver
tests passed. They cover defaults, explicit features, bare optional features,
non-weak edges, weak edges, `dep:` suppression, and malformed inputs. Changed-
file formatting, focused strict Clippy, and `git diff --check` also passed.

A corrected preserved-provider replay is still required.

## Non-claims

This diagnostic reuses provider outputs and cannot satisfy the promoted proof.
The repair does not enable an unrequested optional dependency, Cargo
invocation, network access, or fallback behavior.
