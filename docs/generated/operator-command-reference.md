# Mantle command reference

Catalog BLAKE3: `49a0cf9c3653d3c714c10e535badf2ee6c3af1a6c90f0051e76f5577f49a7fce`

## Daily commands

### `mantle attest show`

Show an artifact attestation for a store path

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle build`

Evaluate and build derivation(s)

- Mutation: `store-state`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle check`

Validate project manifest, lockfile, and generated inputs

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle doctor`

Run no-mutate operator preflight checks for a workflow profile

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle refresh`

Refresh selected or all project inputs

- Mutation: `project-files`
- Network: `required`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle show`

Show resolved input state from the lockfile

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

## Advanced commands

### `mantle artifact`

Frontend-neutral admitted artifact commands

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle artifact export`

Validate and receipt a spec-admitted frontend artifact export

- Mutation: `store-state`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle artifact import`

Import local content into Mantle's frontend artifact store

- Mutation: `store-state`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle artifact oci-export`

Project admitted frontend objects into an atomic local OCI image layout

- Mutation: `store-state`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle artifact oci-import`

Verify and admit a local OCI image layout into the Mantle object store

- Mutation: `store-state`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle artifact oci-pull`

Pull immutable OCI image and Mantle metadata manifests, then admit the exact layout

- Mutation: `store-state`
- Network: `required`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle artifact oci-push`

Publish an admitted local OCI layout through a bounded registry transport

- Mutation: `store-state`
- Network: `required`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle attest`

Attestation inspection and verification commands

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle attest closure`

Assemble and print a runtime closure attestation for one or more roots

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle attest diff`

Diff two attestation documents or artifact selectors

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle attest key-show`

Show the trusted public key token for an existing signing keypair

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle attest policy-init`

Initialize verifier-local policy and revocation files in a verification directory

- Mutation: `project-files`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle attest project`

Render a project attestation from crunch-project.ncl, crunch.lock, and selected roots

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle attest release-show`

Show a signed release attestation from a verification directory

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle attest release-verify`

Verify a release attestation, witness set, and policy from a verification directory

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle attest verify`

Verify attestation bytes against canonical reconstruction

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle attest verify artifact`

Verify a persisted artifact attestation

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle attest verify closure`

Verify a persisted runtime closure attestation

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle attest verify project`

Verify a synthesized project attestation against a saved envelope/file or expected digest

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle attest witness-create`

Create and sign a witness attestation under a verification directory

- Mutation: `store-state`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle attest witness-import`

Import returned witness sidecars into a verification directory

- Mutation: `store-state`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle attest witness-show`

Show witness attestations from a verification directory

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle bootstrap`

Generate bootstrap seeds or validate bootstrap runtime evidence

- Mutation: `none`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle bootstrap capabilities`

Report executable and unsupported source-root operations from live host observations

- Mutation: `none`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle bootstrap full-source-provider-admit`

Bind a normalized full-source C/C++ provider to an expected BLAKE3 identity

- Mutation: `none`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle bootstrap full-source-rust-host-tools`

Materialize receipt-bound Make, CMake, Python, Perl, and BusyBox evidence

- Mutation: `none`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle bootstrap native-toolchain-closure`

Materialize a zero-seed source-built native toolchain closure manifest

- Mutation: `none`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle bootstrap parity-report`

Produce the deterministic whole-bootstrap parity gap report

- Mutation: `none`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle bootstrap rust-source-provider`

Materialize or import a source-built Rust provider after validation

- Mutation: `none`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle bootstrap validate`

Run build-profile preflight, build a bootstrap derivation, and save evidence

- Mutation: `none`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle bootstrap-pin`

Check or apply reviewable bootstrap source pin updates

- Mutation: `none`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle bootstrap-pin apply`

Verify and apply a reviewed plan to TOML pins and derived Nickel readers

- Mutation: `project-files`
- Network: `required`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-bootstrap-pin-apply-v1`

### `mantle bootstrap-pin check`

Poll all declared upstream releases and save a preimage-bound plan

- Mutation: `project-files`
- Network: `required`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-bootstrap-pin-plan-v1`

### `mantle dependents`

List graph dependents of an identity or alias

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle eval`

Evaluate a .ncl file and print the derivation JSON (no build)

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle export`

Export declared Nickel data to a deterministic external JSON payload and receipt

- Mutation: `store-state`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle filegen`

Plan or explicitly apply project-declared generated files

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle filegen apply`

Apply a reviewed generated-file plan after drift checks

- Mutation: `project-files`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle filegen plan`

Render a no-mutate generated-file plan

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle foreign-import`

Validate and plan from lowered foreign derivation import artifacts

- Mutation: `store-state`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle foreign-import audit`

Audit one realized foreign closure from signed PathInfo and castore facts

- Mutation: `store-state`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle foreign-import plan`

Emit a receipt-bound Mantle executable plan from lowered foreign import artifacts

- Mutation: `store-state`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle foreign-import prepare-sources`

Bind local source payloads into an admitted foreign source bundle

- Mutation: `store-state`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle foreign-import produce-aterm`

Lower explicit prefix-aware ATerm derivations into foreign import artifacts

- Mutation: `store-state`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle foreign-import produce-backend`

Run a registered Nix producer backend (fix or host-nix) and lower its `.drv` closure

- Mutation: `store-state`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle foreign-import produce-nix`

Lower concrete Nix derivation facts into foreign import artifacts

- Mutation: `store-state`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle foreign-import realize`

Realize an admitted executable plan through Mantle's scheduler and store

- Mutation: `store-state`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle foreign-import validate`

Validate lowered foreign derivation graph, package-index, policy, and optional receipt JSON

- Mutation: `store-state`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle graph`

Show a semantic build graph rooted at an output/proof/source identity or alias

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle import`

Import external project metadata into Mantle-owned build files

- Mutation: `store-state`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle import cargo`

Plan or apply a Cargo workspace scaffold for the offline Cargo build lane

- Mutation: `store-state`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle import pins`

Plan or apply external pin files into Mantle project files

- Mutation: `store-state`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle import pins apply`

Apply a blocker-free pin import plan immediately after recomputing it

- Mutation: `project-and-store`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle import pins plan`

Render a no-mutate pin import plan

- Mutation: `store-state`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle init`

Initialize a new Mantle project (manifest, lockfile, .mantle/)

- Mutation: `project-files`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle list-stale`

List inputs that would change on refresh (read-only)

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle log`

Show a stored build log

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle mantlepkgs`

Generate, verify, plan, and build locked Mantlepkgs catalogs

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle mantlepkgs build`

Rebuild one verified package with Nix-free catalog consumption and no substitution

- Mutation: `store-state`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle mantlepkgs corpus-verify`

Seal and verify one pinned external-corpus evidence record

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle mantlepkgs domain-adapt`

Adapt one verified v1 generation into one explicit domain shard

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle mantlepkgs domain-compose`

Seal typed domain identities and compose one deterministic public catalog

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle mantlepkgs generate`

Run the explicit locked Nixpkgs producer and publish one complete catalog generation

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle mantlepkgs impact`

Compare two contracted snapshots and write one deterministic impact report

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle mantlepkgs plan`

Compile one verified catalog package into the ordinary foreign executable plan

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle mantlepkgs prepare-sources`

Bind recorded foreign source paths into one explicit source bundle

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle mantlepkgs publish`

Publish previously recorded producer facts without running Nix

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle mantlepkgs update-advisory-observe`

Record one bounded OSV or Repology response or explicit failure

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle mantlepkgs update-execute`

Publish one immutable updated output tree after complete preimage validation

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle mantlepkgs update-plan`

Replay saved observations and write one deterministic dry-run update plan

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle mantlepkgs update-policy-seal`

Seal one typed update policy with its canonical BLAKE3 identity

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle mantlepkgs update-source-observe`

Record one bounded source response or explicit source failure

- Mutation: `none`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle mantlepkgs validate`

Evaluate and validate a typed Mantlepkgs Nickel manifest without running Nix

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle mantlepkgs validation-build`

Realize one separate validation root through the ordinary foreign build boundary

- Mutation: `store-state`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle mantlepkgs verify`

Verify one complete catalog generation and every bound artifact

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle mantlepkgs version`

Resolve historical package versions before ordinary Mantlepkgs production

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle mantlepkgs version index`

Observe one exact revision cohort and publish a compact per-system index

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle mantlepkgs version recheck`

Recheck selected revisions and emit existing Mantlepkgs manifests

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle mantlepkgs version resolve`

Replay a saved index and publish deterministic receipts and revision groups

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle nix-free-demo`

Validate or render the bounded Nix-free fixed-point demo bundle profile

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle nix-free-demo generate`

Generate a self-contained Nix-free demo bundle from explicit evidence inputs

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle nix-free-demo readme`

Render the operator README from a Nix-free demo bundle machine summary JSON file

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle nix-free-demo validate`

Validate a Nix-free demo bundle machine summary JSON file

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle receipt`

Export, list, verify, and import portable receipt bundles

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle receipt bundle`

Receipt bundle operations

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle receipt bundle export`

Export a portable receipt bundle from explicit evidence records and local output state

- Mutation: `store-state`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle receipt bundle import`

Import a verified receipt bundle into Mantle evidence state

- Mutation: `store-state`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle receipt bundle list`

List receipt bundle metadata

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle receipt bundle verify`

Verify receipt bundle structure, trust snapshot, completeness, and optional local output facts

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle refactor`

Plan, check, or apply structured refactor/migration sessions

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle refactor apply`

Explicitly apply a bounded structured refactor session

- Mutation: `project-files`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle refactor check`

Alias for plan; exits non-zero when conflicts exist

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle refactor list`

Show available structured refactor sessions

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle refactor plan`

Plan/check a structured refactor session without mutating files

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle release`

Create or verify a release evidence bundle

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle release attest`

Create and sign a release attestation for a verified release bundle

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle release create`

Create a release evidence bundle from local artifacts

- Mutation: `store-state`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle release function-address-bind`

Bind bundle-local function-address evidence into a Cairn-ready Mantle receipt

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle release gauntlet`

Validate, canonicalize, and aggregate reproducibility gauntlet evidence

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle release gauntlet canonicalize`

Validate and rewrite a gauntlet report as canonical compact JSON

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle release gauntlet continuous`

Aggregate current track evidence into a continuous gauntlet report

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle release gauntlet strict-hermeticity-regression`

Evaluate the strict hermeticity regression suite plan and fixture evidence

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle release global-reproducibility`

Evaluate a digest-bound universe before making any global reproducibility claim

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle release global-reproducibility-evidence`

Derive global reproducibility surface evidence from a verified release bundle

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle release nix-witness`

Compare release bundle artifacts against located Nix-built artifacts and emit a Nix witness receipt

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle release reproduce`

Rebuild and compare published release artifacts, then write a reproducibility report

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle release transport`

Pack, inspect, or unpack an opt-in chaptered release transport

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle release transport inspect`

Inspect and validate a chaptered release transport directory

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle release transport pack`

Pack a verified release directory into a chaptered transport directory

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle release transport unpack`

Unpack a validated chaptered transport into a verified release directory

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle release verify`

Verify a release evidence bundle using bundle-local contents only

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle release witness-export`

Export a portable witness-request directory from verified public artifacts

- Mutation: `store-state`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle release witness-rebuild`

Replay an exported witness request into witness sidecars and rebuild audit evidence

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle remote`

Remote builder access, ticket, and server commands

- Mutation: `none`
- Network: `required`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle remote debug`

Inspect, plan replay, execute replay, or retain remote failure debug bundles

- Mutation: `none`
- Network: `required`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle remote debug gc`

Delete only expired unleased remote failure debug roots

- Mutation: `store-state`
- Network: `required`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle remote debug inspect`

Validate and render one bounded redacted bundle summary without executing anything

- Mutation: `none`
- Network: `required`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle remote debug replay`

Execute the stored request as a new fenced attempt through ordinary admission

- Mutation: `none`
- Network: `required`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle remote debug replay-plan`

Validate and render a side-effect-free replay plan

- Mutation: `none`
- Network: `required`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle remote serve`

Print remote server protocol metadata or serve one framed stdio session

- Mutation: `store-state`
- Network: `required`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle remote status`

Print redacted remote queue, worker, and ticket status

- Mutation: `none`
- Network: `required`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle remote ticket`

Ticket management

- Mutation: `none`
- Network: `required`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle remote ticket create`

Create a redacted bearer ticket record

- Mutation: `store-state`
- Network: `required`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle remote ticket inspect`

Inspect one verifier-only ticket record

- Mutation: `none`
- Network: `required`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle remote ticket list`

List verifier-only ticket records

- Mutation: `none`
- Network: `required`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle remote ticket migrate-legacy`

Invalidate every plaintext-era ticket and upgrade state to verifier-only schema

- Mutation: `none`
- Network: `required`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle remote ticket revoke`

Revoke one ticket

- Mutation: `store-state`
- Network: `required`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle remote ticket rotate-keys`

Rotate service keys and invalidate tickets tied to older verifier key ids

- Mutation: `store-state`
- Network: `required`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle run`

Build and run an executable from a project package or .ncl file

- Mutation: `store-state`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle rust-cache`

Operate the daemon-backed Rust compiler cache

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle rust-cache serve`

Serve bounded rustc wrapper requests from one user-owned cache daemon

- Mutation: `store-state`
- Network: `required`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle rust-plan`

Capture Cargo oracle metadata and unit graph as a normalized Rust package plan receipt

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle self-build`

Build Mantle from its own source (self-hosting)

- Mutation: `store-state`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle shell`

Enter a development shell from the compatibility-named crunch.ncl devShells

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle source`

Plan, export, import, list, and verify source/input bundles

- Mutation: `none`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle source bundle`

Source bundle operations

- Mutation: `none`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle source bundle bootstrap-profile`

Build a named bootstrap source-bundle profile from local inputs

- Mutation: `none`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle source bundle export`

Export a source bundle JSON file

- Mutation: `store-state`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle source bundle hydrate-self-build`

Hydrate a fresh checkout's explicit self-build inputs from a verified bundle

- Mutation: `store-state`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle source bundle import`

Import a source bundle into Mantle source state

- Mutation: `store-state`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle source bundle list`

List a source bundle without mutating state

- Mutation: `none`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle source bundle plan`

Plan source records without mutating source state

- Mutation: `none`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle source bundle preflight`

Compare build-root source requirements with imported source state before building

- Mutation: `none`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle source bundle refresh-mantle-source`

Replace only the Mantle source record in a verified source-built profile

- Mutation: `project-files`
- Network: `required`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle source bundle verify`

Verify a bundle and optionally imported source state

- Mutation: `none`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle stage0-inventory`

Generate a host-tool-free stage0 inventory from explicit seed paths

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle store`

Inspect the store's PathInfo database

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle store archive`

Export, import, or list a Mantle-native single-file store archive

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle store archive export`

Export selected store paths and recursive closure to an archive

- Mutation: `store-state`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle store archive import`

Import a Mantle-native or supported compatibility archive

- Mutation: `store-state`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle store archive list`

List archive contents without importing

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle store composition`

Plan or realize an experimental frontend-neutral castore composition root

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle store composition plan`

Validate a bounded generic projection and report canonical identities

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-composition-plan-v1`

### `mantle store composition realize`

Realize a validated composition from complete local castore roots

- Mutation: `store-state`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-composition-receipt-v1`

### `mantle store gc`

Plan garbage collection, or execute one accepted unchanged plan

- Mutation: `store-state`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle store info`

Show detailed PathInfo for a store path

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle store list`

List all known store paths

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle store pin`

Pin a logical store path as a retained GC root

- Mutation: `store-state`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle store pull`

Pull (import) store paths from a binary cache directory or HTTP cache URL

- Mutation: `store-state`
- Network: `required`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle store push`

Push store paths to a binary cache directory

- Mutation: `store-state`
- Network: `required`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle store repair-final-nar`

Repair stale signed final-NAR metadata for one exact local path

- Mutation: `store-state`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle store roots`

List retained GC roots with provenance

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle store sign`

Sign PathInfo entries with an ed25519 key

- Mutation: `store-state`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle store unpin`

Remove a retained GC root

- Mutation: `store-state`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle store usage`

Report bounded retained, reclaimable, shared, and unknown store usage

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle store verify`

Verify NAR hash and trusted signatures of stored paths

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle transcript`

Execute Markdown Mantle transcripts with isolated store/state defaults

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle transcript run`

Run a Markdown executable transcript

- Mutation: `store-state`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle upgrade`

Migrate project files to the current schema version

- Mutation: `project-files`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle wasm-component`

Build a portable WebAssembly component through the pinned production cohort

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle wasm-component build`

Evaluate a typed Nickel request and execute every available component stage

- Mutation: `store-state`
- Network: `optional`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

### `mantle why`

Explain why an output/proof identity exists from semantic graph records

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`

## Compatibility commands

### `mantle develop`

Alias for `shell` (deprecated, use `crunch shell`)

- Mutation: `none`
- Network: `none`
- Exit classes: `policy-rejection, success, usage`
- JSON schema: `mantle-command-json-v1`
