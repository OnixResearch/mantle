# Kernel Bundles Specification

## Purpose

Defines the `kernel-bundles` capability.

## Requirements

### Requirement: Onix owns a typed kernel-bundle contract

r[kernel_bundles.contract] OnixOS MUST define a versioned typed Nickel `onix-kernel-bundle-v1` contract covering architecture, kernel release, boot format, kernel image, optional initrd/config/BTF/modules/firmware, selected ModulePacks and BPF Packs, provenance/evidence references, compatibility identities, named bounds, and explicit non-claims.

#### Scenario: Minimal bundle is valid

- GIVEN a manifest has the current schema, supported architecture and boot format, kernel release, required kernel image descriptor, valid identities, and no conflicting optional components
- WHEN pure bundle validation runs
- THEN it MUST return a canonical minimal bundle value
- AND omitted optional components MUST remain explicitly absent rather than inferred from host state.

#### Scenario: Manifest is incomplete or unbounded

- GIVEN a manifest omits a required field, uses an unknown schema/role/format, exceeds a named collection or size bound, duplicates a component, or overflows a range calculation
- WHEN validation runs
- THEN it MUST fail closed before build, export, selection, install, or deployment.

### Requirement: Kernel-bundle identity roles are distinct

r[kernel_bundles.identities] OnixOS MUST compute domain-separated BLAKE3 identities for exact components, kernel build, ModulePack, BPF Pack, complete resolved bundle composition, and canonical manifest bytes, and no identity role may substitute for another.

#### Scenario: Operational payload changes

- GIVEN two manifests share identical kernel image, config, BTF, architecture, and kernel release but differ in initrd, modules, firmware, or selected packs
- WHEN identities are computed
- THEN their kernel-build identities MAY match
- AND their complete bundle and manifest identities MUST differ.

#### Scenario: Pack identity is used as bundle identity

- GIVEN a consumer places a component, kernel-build, ModulePack, BPF Pack, or manifest digest in the complete bundle identity field
- WHEN role validation runs
- THEN it MUST reject the manifest even when the digest text is well formed.

### Requirement: KBI and OCI digests remain interoperability metadata

r[kernel_bundles.kbi_interop] OnixOS MUST preserve the exact KBI `kbi:sha256:` identity algorithm and OCI-required SHA-256 digests as typed external compatibility metadata while retaining BLAKE3 as the canonical identity for Onix-owned components, packs, bundles, manifests, receipts, and references.

#### Scenario: KBI-compatible identity verifies

- GIVEN `vmlinuz` and optional config/BTF bytes produce the declared KBI ID under the KBI algorithm
- WHEN compatibility validation runs
- THEN OnixOS MUST retain that KBI ID beside the Onix kernel-build identity
- AND it MUST state that the KBI ID excludes initrd, modules, firmware, and add-on packs.

#### Scenario: KBI ID is presented as full integrity

- GIVEN a manifest or report uses a KBI ID or OCI digest as the complete Onix bundle identity
- WHEN validation runs
- THEN it MUST reject the role confusion
- AND no full-bundle or release-integrity claim may be emitted.

### Requirement: Add-on packs bind to an exact kernel target

r[kernel_bundles.pack_binding] ModulePacks and BPF Packs MUST bind their canonical manifests and member identities to the target Onix kernel-build identity, architecture, and declared kernel release; BPF Packs MUST additionally declare object, section, attach, BTF, kfunc, and kernel type/field requirements.

#### Scenario: Compatible packs compose

- GIVEN a ModulePack and BPF Pack match the selected kernel target, every member exists once, module compatibility metadata agrees, required BTF is present, and every BPF manifest reference resolves
- WHEN pack admission runs
- THEN the packs MAY enter the resolved bundle composition
- AND their independent pack identities MUST be retained.

#### Scenario: Pack target or requirement mismatches

- GIVEN a pack names another kernel-build identity, architecture, or release, duplicates a member, has incompatible vermagic metadata, lacks a referenced BPF object, requires absent BTF, or contains an unresolved declared requirement
- WHEN pack admission runs
- THEN composition MUST fail with deterministic pack/member issue classes
- AND the pack MUST NOT be silently omitted or treated as runtime verified.

### Requirement: Onix and Mantle exchange a frontend-neutral projection

r[kernel_bundles.projection] OnixOS MUST lower admitted kernel-bundle intent into a frontend-neutral OCI projection request containing only component roles, safe object references, media types, canonical identities, required annotations, and round-trip expectations, and imported projections MUST regain Onix status only after full canonical identity reconstruction.

#### Scenario: Onix bundle round-trips through Mantle

- GIVEN Mantle exports and imports an admitted projection while verifying every OCI and Onix digest
- WHEN Onix reconstructs the canonical manifest
- THEN component, pack, kernel-build, bundle, and manifest identities MUST match the original
- AND registry tags or transport paths MUST NOT affect equality.

#### Scenario: External KBI image lacks Onix manifest

- GIVEN an imported image has a valid KBI ID and compatible media types but no valid full Onix manifest
- WHEN Onix admission runs
- THEN it MUST remain compatibility-only until explicit canonicalization reconstructs every full identity
- AND KBI validity alone MUST NOT satisfy Onix bundle admission.

### Requirement: Kernel-bundle receipts are safe and non-authoritative

r[kernel_bundles.receipts] OnixOS MUST emit read-only bundle validation and inspection receipts containing safe schema, role, identity, compatibility, issue, and non-claim fields with a domain-separated BLAKE3 receipt identity, and bundle validation MUST NOT authorize build, install, deploy, boot, or target mutation.

#### Scenario: Valid bundle is inspected

- GIVEN a canonical bundle and optional verified external digest metadata
- WHEN read-only inspection runs
- THEN it MUST report distinct identity roles, component/pack counts, compatibility status, and bounded diagnostics
- AND it MUST omit raw credentials, registry tokens, private keys, full host paths, component bytes, and unbounded tool output.

#### Scenario: Mutation is requested from validation result

- GIVEN a bundle validation receipt passes but no existing Onix authorization or lifecycle admission permits target mutation
- WHEN build, install, deploy, or boot is requested
- THEN the request MUST remain denied
- AND the validation receipt MUST NOT be interpreted as authority.

### Requirement: Kernel-bundle contracts have positive and negative evidence

r[kernel_bundles.verification] The kernel-bundle lane MUST include positive and negative Nickel, pure Rust, CLI, and Mantle round-trip fixtures for schema, canonicalization, identity roles, KBI compatibility, pack binding, imported states, bounds, receipt safety, and claim boundaries.

#### Scenario: Kernel-bundle change is ready to archive

- GIVEN maintainers intend to close the `onix-kernel-bundle-v1` change
- WHEN closeout validation runs
- THEN positive and negative fixtures, focused checks, documentation, Cairn validation, and proposal/design/tasks gates MUST pass
- AND any unavailable cross-repository or runtime rail MUST be recorded as a dependency or blocker rather than reported as proof.
