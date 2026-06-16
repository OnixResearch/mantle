# Typed bootstrap route for a future musl-host Rust-from-source provider.
#
# This is not a provider output and does not satisfy
# `mantle-rust-source-provider-v1` by itself. It mirrors the source-built
# route in `rust-source-plan.ncl`, but selects a musl host triple so a future
# materialized provider can pair with the source-root musl native closure.

let Source = {
  id | String,
  kind | String,
  name | String,
  version | String,
  url | String,
  sha256_hex | String,
} in
let Stage = {
  id | String,
  kind | String,
  source_ids | Array String,
  bootstrap_stage_id | String,
  rust_version | String,
  outputs | Array String,
  notes | Array String,
} in
let Output = {
  role | String,
  path | String,
} in
let Policy = {
  source_built | Bool,
  uses_prebuilt_rust | Bool,
  forbids_prebuilt_rust | Bool,
  reference | String,
} in
let Plan = {
  schema | String,
  provider_id | String,
  route | String,
  host_triple | String,
  target_triple | String,
  final_version | String,
  policy | Policy,
  sources | Array Source,
  stages | Array Stage,
  final_outputs | Array Output,
} in
{
  schema = "mantle-rust-source-provider-bootstrap-plan-v1",
  provider_id = "mantle-rust-source-provider",
  route = "mrustc-to-rust-current-source-route-musl-host",
  host_triple = "x86_64-unknown-linux-musl",
  target_triple = "x86_64-unknown-linux-musl",
  final_version = "1.94.0",
  policy = {
    source_built = true,
    uses_prebuilt_rust = false,
    forbids_prebuilt_rust = true,
    reference = "stagex core rust package route with musl host native closure",
  },
  sources = [
    {
      id = "mrustc-0.12.0",
      kind = "tarball",
      name = "mrustc-source",
      version = "0.12.0",
      url = "https://github.com/thepowersgang/mrustc/archive/refs/tags/v0.12.0.tar.gz",
      sha256_hex = "c1ba35f5fc5c4ca2952d9f5526e900dcb6632ea7fd4d71fa58029b3bb563ae56",
    },
    {
      id = "rust-1.90.0",
      kind = "tarball",
      name = "rust-compiler-source",
      version = "1.90.0",
      url = "https://static.rust-lang.org/dist/rustc-1.90.0-src.tar.gz",
      sha256_hex = "799a9f9cba4ed5351e071048bcf6b5560755d9009648def33a407dd4961f9b7e",
    },
    {
      id = "rust-1.91.1",
      kind = "tarball",
      name = "rust-compiler-source",
      version = "1.91.1",
      url = "https://static.rust-lang.org/dist/rustc-1.91.1-src.tar.gz",
      sha256_hex = "38dce205d39f61571261f0444237a1ce9efecb970e760d8ec4d957af5b445723",
    },
    {
      id = "rust-1.92.0",
      kind = "tarball",
      name = "rust-compiler-source",
      version = "1.92.0",
      url = "https://static.rust-lang.org/dist/rustc-1.92.0-src.tar.gz",
      sha256_hex = "9e0d2ca75c7e275fdc758255bf4b03afb3d65d1543602746907c933b6901c3b8",
    },
    {
      id = "rust-1.93.1",
      kind = "tarball",
      name = "rust-compiler-source",
      version = "1.93.1",
      url = "https://static.rust-lang.org/dist/rustc-1.93.1-src.tar.gz",
      sha256_hex = "4c230a44b3d9c9f3cef950943719f8380058d27c91fda5e36a9a947ef013e01f",
    },
    {
      id = "rust-1.94.0",
      kind = "tarball",
      name = "rust-compiler-source",
      version = "1.94.0",
      url = "https://static.rust-lang.org/dist/rustc-1.94.0-src.tar.gz",
      sha256_hex = "b83f921cd3f321ff614f9c06a8b870d89299fc02888b48a5549683a36823474c",
    },
  ],
  stages = [
    {
      id = "mrustc-to-rust-1.90.0",
      kind = "mrustc-seed",
      source_ids = ["mrustc-0.12.0", "rust-1.90.0"],
      bootstrap_stage_id = "",
      rust_version = "1.90.0",
      outputs = ["rustc", "cargo", "host-rustlib", "target-rustlib"],
      notes = [
        "build mrustc and minicargo from mrustc source first",
        "build the first Rust provider for a musl host triple",
      ],
    },
    {
      id = "rust-1.91.1-stage1",
      kind = "rustc-stage1",
      source_ids = ["rust-1.91.1"],
      bootstrap_stage_id = "mrustc-to-rust-1.90.0",
      rust_version = "1.91.1",
      outputs = ["rustc", "cargo", "host-rustlib", "target-rustlib"],
      notes = ["stage1 library plus cargo keeps the musl-host route moving forward"],
    },
    {
      id = "rust-1.92.0-stage1",
      kind = "rustc-stage1",
      source_ids = ["rust-1.92.0"],
      bootstrap_stage_id = "rust-1.91.1-stage1",
      rust_version = "1.92.0",
      outputs = ["rustc", "cargo", "host-rustlib", "target-rustlib"],
      notes = ["stage1 library plus cargo keeps the musl-host route moving forward"],
    },
    {
      id = "rust-1.93.1-stage1",
      kind = "rustc-stage1",
      source_ids = ["rust-1.93.1"],
      bootstrap_stage_id = "rust-1.92.0-stage1",
      rust_version = "1.93.1",
      outputs = ["rustc", "cargo", "host-rustlib", "target-rustlib"],
      notes = ["stage1 library plus cargo keeps the musl-host route moving forward"],
    },
    {
      id = "rust-1.94.0-final",
      kind = "rustc-final",
      source_ids = ["rust-1.94.0"],
      bootstrap_stage_id = "rust-1.93.1-stage1",
      rust_version = "1.94.0",
      outputs = ["rustc", "cargo", "rustdoc", "host-rustlib", "target-rustlib", "provider-receipt"],
      notes = ["install final musl-host toolchain plus provider metadata and receipts"],
    },
  ],
  final_outputs = [
    { role = "rustc", path = "bin/rustc" },
    { role = "cargo", path = "bin/cargo" },
    { role = "rustdoc", path = "bin/rustdoc" },
    { role = "host-rustlib", path = "lib/rustlib/x86_64-unknown-linux-musl/lib" },
    { role = "target-rustlib", path = "lib/rustlib/x86_64-unknown-linux-musl/lib" },
    { role = "provider-receipt", path = "share/mantle-rust-provider/receipts/build.json" },
  ],
} | Plan
