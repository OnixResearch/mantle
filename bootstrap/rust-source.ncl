# Fail-closed source-built Rust compiler/sysroot provider recipe.
#
# This file intentionally does not import bootstrap/rust.ncl and does not wrap
# Nix, rustup, or the official prebuilt Rust standalone installer. It is the
# recipe anchor for the future real Rust-from-source provider. The selected
# mrustc-to-current-Rust source route is tracked separately in
# bootstrap/rust-source-plan.ncl. Until the full rustc/cargo/rustlib bootstrap
# chain exists, building this derivation aborts with the same deterministic
# blocker used by `mantle bootstrap rust-source-provider`.

let mantle = import "lib.ncl" in

{
  name = "rust-source-provider-blocked",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      echo "source-built Rust provider materialization is not implemented: Mantle has no receipt-bound Rust-from-source bootstrap that can build rustc/cargo/rustlib without prebuilt Rust" >&2
      echo "refusing to relabel Nix, rustup, or prebuilt Rust as source-built" >&2
      exit 1
    "%,
  ],
  inputs = [],
} | mantle.Derivation
