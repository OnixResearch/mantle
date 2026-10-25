# Opt-in binary-cache substitution

`mantle build --substituters` now names no binary cache by default; it used to
default to `https://cache.nixos.org`. Remote entry points resolve through
operator endpoint bindings (ADR 0128), and a store that names an HTTP substituter
refuses to open without one, so the old default made every plain `mantle build`
without `--remote-binding-config` fail. An empty list builds without
substitution, like `--no-substitute`.

To substitute, name each cache with `--substituters` and bind it in the endpoint
binding table that `--remote-binding-config` admits. `mantle store pull --from
<http URL>` has the same requirement: an unbound source is refused with
`store-source-admission-required` before anything is imported.

Signature, trust and content checks on substituted and pulled paths are
unchanged.
