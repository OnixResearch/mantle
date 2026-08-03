# Rust compiler cache daemon

Mantle can run eligible Rust compiler work through a user-owned daemon. The daemon uses a bounded Unix-socket protocol. It can reuse local and shared Rust unit results.

The wrapper is a Cargo `RUSTC_WRAPPER`. The driver must publish one sealed invocation manifest for each eligible compiler invocation before it starts Cargo.

## Components

- `mantle-rustc-manifest` seals driver input and publishes a manifest registry entry.
- `mantle-rustc-wrapper` validates the current Cargo invocation and sends a bounded request.
- `mantle rust-cache serve` admits the peer, rechecks the manifest, checks caches, and runs the compiler on a miss.
- `mantle-rust-cache-daemon` is the narrow standalone compatibility entry point for the same daemon shell.
- `crunch-rust-cache-core::wrapper` owns the pure protocol, identity, policy, and decision logic.
- `crunch-rustc-wrapper` owns file access, sockets, sandbox execution, cache access, and receipts.

The daemon verifies the peer UID with `SO_PEERCRED`. It accepts only UIDs in the policy. It also verifies the BLAKE3 identity of the configured Bubblewrap executable.

## 1. Export and review the policy

Start with `fixtures/policy.ncl`. Replace all example paths, the user ID, and `sandbox_program_blake3`.

```sh
nickel export rust-cache/daemon/fixtures/policy.ncl > policy.json
```

The policy selects these cache modes:

- `off`
- `local-read`
- `local-read-write`
- `shared-read`
- `shared-read-write`

The Boolean cache fields must match `cache_mode`. Shared reads require an explicit trust policy. Shared writes require a publication source, signer name, and signing-key path.

The policy contains no secret value. A signing-key path is a reference for the daemon shell. The wrapper and compiler do not receive signing-key bytes.

## 2. Publish manifests

The driver writes `mantle-rustc-wrapper-manifest-v1` input JSON. Each input binds:

- the Rust unit action;
- the exact compiler path and compiler BLAKE3 identity;
- the argument and admitted-environment identities;
- the working directory;
- readable and writable roots;
- compiler, source, sysroot, and dependency inputs;
- output paths and limits;
- filesystem, network, clock, and randomness policy.

Compute declared file or directory identities with bounded traversal:

```sh
mantle-rustc-manifest hash-path file "$SOURCE" 16777216
mantle-rustc-manifest hash-path directory "$SYSROOT" 8589934592
```

Publish each input to one private registry:

```sh
mantle-rustc-manifest publish unit-input.json "$MANIFEST_DIR"
```

The registry filename is the argument-list BLAKE3 identity. Publication does not replace an existing different manifest. A collision fails explicitly. The policy must include the registry in `readable_roots`.

## 3. Start the daemon

```sh
mantle \
  --state-dir "$STATE_DIR" \
  --store "$STORE_OUTPUT_DIR" \
  rust-cache serve \
  --policy policy.json \
  --receipt-dir "$RECEIPT_DIR"
```

The root command binds daemon state and store output to Mantle's global `--state-dir` and `--store` options. The standalone `mantle-rust-cache-daemon` binary accepts equivalent explicit paths.

Use `--once` for one bounded request in a test. `SIGINT` and `SIGTERM` stop admission. The daemon drains accepted worker requests before it exits.

The daemon uses a fixed worker count from `max_concurrency`. Frame, stream, artifact, request, response, source, candidate, and elapsed-time limits are explicit.

The focused socket test uses these conservative debug-build baselines:

- daemon startup plus one compile request: at most 5,000 ms;
- daemon startup plus one local cache hit: at most 2,000 ms.

The test includes daemon startup in both measurements. These limits detect large regressions. They are not production performance claims.

## 4. Configure Cargo

Set the wrapper and policy in the Cargo environment:

```sh
export RUSTC_WRAPPER="$(command -v mantle-rustc-wrapper)"
export MANTLE_RUST_CACHE_POLICY="$PWD/policy.json"
export MANTLE_RUSTC_MANIFEST_DIR="$MANIFEST_DIR"
cargo build
```

For one diagnostic invocation, `MANTLE_RUSTC_MANIFEST` can select an exact manifest file. Do not set both manifest variables.

Compiler queries, incremental compilation, response files, missing manifests, changed inputs, and unsupported outputs do not use strong cache eligibility. The policy selects fail-open bypass or fail-closed rejection.

## Output and receipt rules

The daemon compiles into a private sibling stage. It verifies declared files, directories, limits, and BLAKE3 identities before and after compiler execution. Escaping directory symlinks fail. Persistent input changes deny publication.

The V1 shell admits one declared output file for each invocation. It verifies the exact output set, type, mode, size, and BLAKE3 identity. One no-clobber rename publishes that file atomically. The daemon does not report `artifact_commit_complete` before the rename succeeds.

Each request produces a content-addressed wrapper receipt when the protocol can identify the request. Receipts state:

- local hit, shared hit, compiled, bypass, or rejected;
- whether the compiler ran;
- compiler status and bounded output;
- cache result identity when one exists;
- artifact BLAKE3 identities;
- whether artifact commit completed;
- stable reason codes.

Environment values are not present in receipts. The manifest and request bind an environment identity. The policy requires redaction.

## Shared results

Result sources are ordered by `priority`. A configured source ID must equal the endpoint-derived source ID from the shared Rust result adapter. HTTP adapters disable ambient proxies, reject URL credentials, reject redirects, and use explicit transfer limits.

Shared cache hits still pass the existing signed-envelope trust check. A configured source does not grant trust. Shared publication uses the existing signed result-envelope protocol.

## Non-claims

A wrapper receipt is bounded execution evidence. It is not proof of compiler correctness, source trust, sandbox correctness, release eligibility, deployment safety, or global cache availability.

The V1 production shell supports Bubblewrap as its sandbox provider. Mantle binds the executable path and BLAKE3 identity. It mounts only verified declared inputs as read-only paths and the private output stage as writable. Policy roots are admission boundaries; Mantle does not mount them in full. The shell clears the environment and unshares the network. Mantle does not infer isolation from process success.
