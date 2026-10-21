# Locked pnpm workspaces

`lib/pnpm.ncl` lowers a workspace to an ordinary input-addressed Mantle
`Derivation`. Its builder is the declared `crunch-pnpm` executable, not a new
scheduler, package resolver, store, credential agent, or signature authority.
Mantle's existing sandbox, source admission, PathInfo, signing, and trusted-cache
boundaries remain authoritative. Run product builds through `mantle build` with
`--strict-hermetic`; directly running the helper is **not** sandbox evidence.

## Exact supported fragment

- Linux x86-64, Node **22.14.0**, pnpm **9.15.9**, quoted lock schema **`9.0`**.
  Other cohorts fail before execution. The real proof driver uses the
  `/nix/store` logical prefix. Runtime roots must be exact `/nix/store` entries.
- A source tree containing `package.json`, `pnpm-workspace.yaml`, the complete
  `pnpm-lock.yaml`, all importer manifests, patches, build scripts, and sources.
  `packageManager` must be exactly `pnpm@9.15.9`. Workspace configuration admits
  only `packages`; root `pnpm` configuration admits only `patchedDependencies`.
  `.npmrc`, pnpmfile hooks, `.git`, existing `node_modules`, and user-store trees
  are rejected. All source/mirror/output symlinks and special files are rejected,
  even non-escaping symlinks; native pnpm-created installation links are checked
  separately against the copied workspace root.
- Registry packages have one **SHA-512 SRI** per package. Every `packages` entry,
  including platform-inapplicable optional entries, must have a supplied archive
  and matching BLAKE3. The mirror inventory is exact, not a best-effort cache.
  Archives must be gzip tarballs rooted at `package/`, containing a JSON manifest
  and only regular files/directories. Unknown resolution types fail closed.
- The supported Git source form is an **already prepared npm-package archive**,
  with integrity, BLAKE3, an immutable 40-hex revision, and an explicit HTTPS
  `resolution.tarball` URL whose path binds that exact revision. The archive is
  admitted using the generic `PackageMirror` source-observation profile. Its Git
  origin/revision is recorded as caller-supplied provenance, **not** an
  independently authenticated statement that archive bytes match a Git tree.
  Acquisition authority must establish that relationship before admission.
  Raw `git:`, `git+`, Git resolvers, branches, Git `prepare`, arbitrary codeload
  root layouts, credentials, query-bearing tarball URLs, and publishing are not
  supported. No Git executable or network checkout is invoked by this adapter.
- Workspace links stay inside the complete source tree and point to package
  manifests. pnpm owns link layout, the peer graph and platform-optional
  selection. Mantle does not synthesize a dependency graph, flatten peers,
  translate Cargo vendor decisions, or resolve omitted dependencies. Strict
  peer checks and frozen-lockfile checks use the pinned pnpm implementation.
- Dependency lifecycle/native builds are rejected (`requiresBuild` and lifecycle
  hooks), including hooks in packages that are optional on another platform.
  Supply an independently prepared, integrity-pinned package instead. Selected
  workspace scripts are an explicit ordered permission list, run **after** the
  frozen installation; an allowed `postinstall` is an explicit invocation, not
  pnpm's automatic install-phase traversal. Unlisted workspace install hooks are
  rejected. `enable-pre-post-scripts=false` prevents implicit pre/post expansion.
  `pnpm pack` runs with hooks disabled. A script may execute code from declared
  dependencies: granting a script is a code-execution capability, not static
  proof that its source is benign.
- The tool tree contains `bin/node`, **`bin/sh`**, and `pnpm.cjs` plus pnpm's
  complete distribution dependencies. Include only explicitly permitted native
  tools in `bin`. The entire tool tree and every runtime root are measured
  before any tool starts. Tool symlinks must resolve within those exact measured
  roots. Declare dynamic ELF loader/library closures as ordinary Mantle inputs;
  neither version strings nor a host PATH substitute for those identities.
- `PATH` is the declared tool tree. Child environments are cleared, HOME/XDG/
  cache/state/store directories are fresh and isolated, user/global npm config
  is disabled, and the script shell is the measured `bin/sh`. Package-manager
  auto-downloads and side-effect caches are disabled. Network denial is the
  enclosing native Mantle sandbox's responsibility, never an assertion made by
  an unisolated helper. Missing native tools and attempted downloads fail.

Bounds are 4,096 package records, 128 selected targets, 16 script permissions per
workspace, 32 ordinary output declarations per workspace, 128 runtime roots,
2 MiB per structured document, 100,000 entries/64 levels per tree or archive,
512 MiB per input tree, and 512 MiB total expanded package payload. Ordinary
Mantle build/resource limits additionally apply; this adapter does not create a
second process supervisor.

## Acquisition and execution

Acquisition is a separate, existing Mantle source-admission operation. Supply
independently verified package archives and exact admitted tool/runtime roots.
A mirror name is observational provenance, not permission to fetch it. The
builder has **no acquisition fallback**.

For every invocation the builder verifies role identities and the complete
archive inventory, creates an empty temporary pnpm store, and seeds pnpm's
native integrity-keyed store using `pnpm store add --offline <local-archive>`.
It then executes:

```text
pnpm install --offline --frozen-lockfile --ignore-scripts --strict-peer-dependencies
pnpm --dir <selected-workspace> run <explicit-script>  # in declared order
pnpm --dir <selected-workspace> pack --pack-destination <declared-package-output>
```

The wanted lock is captured before store assembly and must remain byte-identical
through install/build/pack. Builds cannot consult an ambient pnpm store. A warm
Mantle hit bypasses execution only through the existing trusted signed-output
lookup; the helper itself has no cache and refuses an already-existing output.

## Authoring contract

`crunch-pnpm-core::Request` is the closed JSON wire contract, read from a separate
admitted JSON plan file. `mkWorkspace` declares that plan file as an ordinary
input with an independently pinned raw-byte BLAKE3, checked before parsing,
avoiding argv/environment size limits and source-digest self-reference.
Every role has an ordinary admitted `input` and exact sandbox
`path`; a path alone grants no input authority. The parent derivation binds the
builder and all input store roots. The semantic plan binds source/tool tree
identities, runtime content, archive bytes, Node/pnpm/platform cohort, target
order, script permissions and every declared output.

The Rust core consumes `Request` through `validate(Request)` and returns
`Result<AdmittedRequest, String>`. The admitted result retains the original
`request`, its domain-separated `identity`, and ordered `sources` observations.
The std executor consumes that request once and reuses those observations in
provenance rather than recomputing admission after execution. `member(String)`
returns the admitted member string; borrowed path checks remain private helpers.
These ownership changes do not alter the JSON wire contract or identity domain.

```nickel
let pnpm = import "pnpm.ncl" in
pnpm.mkWorkspace {
  name = "web-workspace",
  adapter = { input = adapter_root, path = adapter_executable },
  source = { input = source_root, path = source_root },
  mirror = { input = mirror_root, path = mirror_root },
  tools = { input = tools_root, path = tools_root },
  runtime = runtime_roles,
  plan = { input = plan_root, path = plan_json_file, blake3 = plan_blob_blake3 },
}
```

A target record is:

```json
{"workspace":"packages/lib","scripts":["build"],"outputs":["dist"],"package_output":"package"}
```

`package_output: null` omits packing. Non-null destinations contain **real pnpm
pack tarballs**, not invented package receipts. The final output preserves
`<workspace>/<declared-member>` and writes `provenance.json` containing the
versioned input identity, complete semantic plan, generic source observations,
and each selected artifact's BLAKE3. Mantle separately signs the final output
and publishes its normal build/output evidence. Provenance JSON alone does not
establish sandbox execution or trust.

Use `crunch-pnpm digest PATH` for the versioned canonical tree digest (sorted
UTF-8 paths, entry type, executable bits, exact bytes or symlink target).
`crunch-pnpm blob-digest FILE` is raw-byte BLAKE3 for package archives; it is
**not** the tree digest. `crunch-pnpm identity PLAN_FILE` validates and identifies
the semantic plan in an admitted JSON file. These read-only commands do not build
or admit sources.

## Executable real-tool proof

`scripts/prove-pnpm-workspace.py` supplies a real two-package workspace, prepared
local package tarballs, a patched package with a peer dependency, a workspace
link, a Darwin-only optional dependency, and a prepared immutable Git-origin
package archive. Package contents are synthetic; **Node, pnpm, package
installation, patch application, peer/link/platform behavior, pnpm pack, Mantle
sandboxes, signing and cache reuse are real**, not mocked adapters.

The driver deliberately does not fetch a Node/pnpm release or discover host
stores. Before running it, the operator must provide:

1. A built `mantle` and `crunch-pnpm` from the integrated immutable source.
2. A working strict native sandbox (bwrap/user namespaces/static sandbox shell)
   and an existing Mantle state/store containing the admitted adapter, pinned
   Node/pnpm distribution, explicit script shell, and complete runtime closure.
3. An explicit signing key and corresponding trusted public-key token.
4. An `inputs.json` with exact admitted roles and measured content identities:

```json
{
  "adapter": {"input":"/nix/store/ADMITTED-adapter","path":"/nix/store/ADMITTED-adapter/bin/crunch-pnpm"},
  "tools": {"input":"/nix/store/ADMITTED-tools","path":"/nix/store/ADMITTED-tools","blake3":"EXACT_TREE_DIGEST"},
  "runtime": [{"input":"/nix/store/ADMITTED-runtime","path":"/nix/store/ADMITTED-runtime","blake3":"EXACT_TREE_DIGEST"}]
}
```

The capitalized values describe operator-provided real inputs, not checked-in
fake pins. An empty `runtime` list is valid only for genuinely self-contained
static tools. No script downloads or invents those prerequisites.

`tests/fixtures/pnpm/tool-sources.json` records concrete upstream acquisition
pins: the official Node 22.14.0 Linux-x64 tarball SHA-256 from
[Node's checksum manifest](https://nodejs.org/dist/v22.14.0/SHASUMS256.txt), and
pnpm 9.15.9's SHA-512 SRI from its
[registry metadata](https://registry.npmjs.org/pnpm/9.15.9). These are acquisition
inputs, not substitutes for the prepared tool/runtime tree BLAKE3 identities.
The official Node executable's ELF interpreter may need relocation in a declared
preparation derivation; merely adding a glibc directory to `inputs` does not
create an undeclared `/lib64` mount. Preserve pnpm's complete `dist` tree beside
`pnpm.cjs`, rather than copying only its JavaScript entry point.

```bash
python3 scripts/prove-pnpm-workspace.py \
  --mantle /absolute/path/to/mantle \
  --adapter /absolute/path/to/crunch-pnpm \
  --inputs-json /absolute/path/to/inputs.json \
  --store /absolute/path/to/admitted-store \
  --state-dir /absolute/path/to/admitted-state \
  --store-prefix /nix/store \
  --signing-key /absolute/path/to/signing-key \
  --trusted-public-key 'operator-key:BASE64_PUBLIC_KEY' \
  --work-dir /absolute/path/to/new-proof-directory \
  --lib "$PWD/lib"
```

The proof materializes fixture sources/mirrors through ordinary signed Mantle
producer derivations. Product execution uses `mkWorkspace` and `mantle build
--strict-hermetic --no-substitute`. It requires: the exact app HTML and packed
library content; an unchanged signed warm hit; a distinct uncached derivation
with byte-identical output; source-change identity/output invalidation; and
failed stale-lock, archive-integrity, escaping-symlink, missing-package,
prohibited-script, undeclared-tool and network scenarios. A network timeout is
**not** accepted as proof of denial: the negative requires `ENETUNREACH` from the
sandbox. It also verifies a matched trusted-signature row for the cold output.
Successful completion writes `evidence.json` and per-command transcripts. Merely
running `--help`, building the Rust crates, or discovering tests is not this
proof. No accepted real-tool transcript is bundled by this implementation.

Focused permanent contract tests are `cargo test -p crunch-pnpm-core --test
contract`. They cover consumer-visible identity invalidation and fail-closed
path/tool/runtime boundaries, not mocked pnpm command echoes. Run them as part
of the integrated source-bound gate, not while sibling changes are in flight.

## Reproducibility limits

Input-addressed identity invalidates when declared source, lock, patches,
manifest, tool/runtime content, options, permissions, target order or output
selection changes. Output reuse requires Mantle's trusted signatures. A clean
comparison compares a separately executed derivation with a fresh empty pnpm
store; it is not a second hit to an ambient package cache. The fixture checks
actual content and package bytes. This does not prove arbitrary JavaScript
scripts deterministic, Node/compiler correctness, general Git equivalence,
all platforms, all pnpm versions, public-registry behavior, or all native
packages. pnpm pack normalizes archive timestamps in the pinned implementation,
but arbitrary selected scripts may still encode clocks, paths or randomness.
