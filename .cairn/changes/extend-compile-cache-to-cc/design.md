# Design: Extend the compile cache to C and C++ builders

## Goal and scope

The Rust unit cache seam now includes a narrowly classified GNU C/C++ driver,
versioned action and wire contracts, and private local compile-object storage.
Production StageX/GCC inventory admission, sandbox endpoint mapping, and C
proof-lane exclusion are not yet wired; `config/cc-compile-cache/default.ncl`
therefore remains `off`. Local GCC/G++ byte-parity and probe fixtures do not
replace a timed representative bootstrap baseline, representative-chain
cache-on/off comparison, or cache-off fixed-point proof. ADR 0087 remains proposed.

## Local execution evidence (2026-10-06; not StageX proof)

The final isolated driver (BLAKE3
`b28c2cd5efd88e236de5233b82acf80ae4d3292eefc08b50c6a780ad99598f80`)
ran against the standalone Rust cache daemon on private Unix sockets. For both
real GCC and G++, uncached, cold published, same-root hit, and
rebuilt-identical relocated hit objects and depfiles matched the corresponding
direct compiler outputs byte for byte. Relocation kept each compiler's action
key and object bytes while reconstructing the direct compiler's relocated
depfile bytes. A changed C header produced a new published key and object; a
z-before-a include fixture preserved GCC's encountered dependency order and
72-column wrapping on both cold and hit paths.

A classified GCC configure failure published once and replayed on the next
request with the same native exit status, both diagnostic streams, and
depfile, including a hit after daemon restart. Changed probe script, header,
platform, flags, and GCC-to-G++ toolchain each produced a different published
failure key. Missing-header compiles with no complete manifest, and failures
whose diagnostics exceeded 262,144 bytes, ran the compiler on both requests
without failure replay; the oversized case still streamed the full native
diagnostic and reproduced its depfile. A classified successful configure
compile published then hit with native object and depfile bytes. Omitting the
required deterministic diagnostic flags on an initial success smoke correctly
forwarded unchanged as `fallback/unclassified-probe-environment`; adding the
required flags admitted the successful probe. None of these local checks
establish a representative StageX chain baseline, cache-on/off chain parity,
protected inventory integration, or fixed-point proof.

## Current behavior

Existing Rust unit result payloads in `crunch-rust-cache` are castore-backed.
The new C objects are private content-addressed local files in the same
per-user `rust-unit-cache` state directory, not castore blobs.
`crunch-rustc-wrapper` plus `mantle-rust-cache-daemon` serve Rust builds
through `rust_plan` with
`local_rust_cache`/`shared_rust_cache` CLI modes. Before this change C/C++
compiles were uncached: StageX and GCC chain sessions recompiled every object
on recipe change, and proof runs measured hours. The accepted
`rustc-cache-adapter` spec already demands strict evidence lanes exclude
ambient wrapper caching; the present change extends the same discipline.

The external reference implements the C-side shape: a driver that is `cc` on
PATH, content-masked keys, depfile-learned manifests, cached failures and
configure probes, and a bitcask daemon with machine-wide build slots
(`evidence/repkgs-review.md`). Its trust model is explicit: whoever can write
the cache can inject object code; CA outputs make tampering detectable, not
impossible.

## Approach review

| Family | Mechanism | Disposition | Required check |
| --- | --- | --- | --- |
| sccache-style wrapper | Environment wrapper around each compiler | Rejected for protected lanes: ambient PATH mutation outside inventory | Supervisor audit parity |
| Declared driver seam | Policy-admitted driver in the tool inventory, forwarding what it cannot classify | Selected direction | Forward-unchanged and inventory fixtures |
| Wrapper inside sandbox with cache bind | Bind cache dir into sandbox as input | Rejected: makes the cache a derivation input | Byte-identity with cache on/off |
| Host daemon via mapped socket | Execution-time endpoint mapping, not a derivation input | Selected storage framing | Identical outputs with and without |

## Contract and component ownership

- Pure core: `crunch-rust-cache-core::cc` owns versioned action keys, ordered
  manifests, bounds, wire envelopes, and typed probe failure markers.
- Shell: the standalone driver forwards unclassified compiler arguments;
  the existing daemon serves C requests and writes private objects and
  receipts. Protected build-shell endpoint mapping is still open.
- Policy: typed Nickel export admitting the driver per toolchain family with
  explicit non-claims.
- Storage: the existing local cache store; no remote sharing in this change.

## Decisions

### Decision: Reuse the existing daemon and store

**Choice:** Extend `mantle-rust-cache-daemon` and the local store instead of a
new daemon.

**Rationale:** The Rust cache daemon and Rust strict lanes already exist;
Rust lanes have default-off/scrub discipline. C-specific strict receipt
exclusion and proof integration remain open, but a second C daemon would
duplicate the local transport and state.

### Decision: Depfile-learned manifests, not argument-only keys

**Choice:** A C key binds tool, source, explicit platform digest, normalized
arguments without store paths, and whole-tree digests of source/include roots.
Depfiles learn root-relative dependency labels and content digests in compiler
encounter order. A classified GNU hit reconstructs the target and prerequisites
with GCC's 72-column make-rule wrapping; exotic make syntax or relative output
paths forward to the real compiler rather than risk altered output bytes.

**Rationale:** Argument-only keys miss header edits — precisely the rebuild
case the chain needs to catch. Unknown or incomplete manifests must miss.
An explicit canonical `--probe-script` and stable `LC_ALL=C` diagnostics select
the narrowly classified configure-compile path. Success objects and bounded
failure markers use distinct keys binding script, compiler, source, platform,
semantic flags, and complete roots; failure replay also checks the current
ordered depfile inputs. Incomplete manifests or oversized diagnostics cannot
be published or replayed, and per-read dispositions are recorded. General
configure commands and arbitrary compile failures remain uncached.

## Risks / Trade-offs

- Trust: cache writes can inject object code; mitigation is per-machine
  per-user scope, receipts, and proof-lane exclusion, matching the reference's
  own framing and Mantle's evidence rules.
- Driver classification must track compiler flag churn; unclassifiable
  invocations forward unchanged and are logged, never guessed.
- Probe-cache keys over toolchain identity need the identity set the protected
  inventory already records.
