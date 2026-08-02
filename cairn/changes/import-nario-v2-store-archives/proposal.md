# Import Nario v2 store archives

## Why

Determinate Nix can export store paths through the streaming Nario v2 format. Mantle currently supports only its own archive format.

Nario v2 input can reduce producer-side glue for offline Nixpkgs conversion. It can carry exact source-store payloads, signatures, references, and content-addressed metadata.

Nario does not contain package recipes or Nixpkgs selection meaning. Mantle still needs the foreign derivation graph and package index.

## What Changes

- Pin one exact Determinate Nix Nario v2 format source and producer revision for compatibility evidence.
- Add bounded Nario v2 list and import support through the existing store archive command.
- Keep Nario export unsupported in this change.
- Preserve exact Nix store identity during direct store import and apply Mantle trust policy before admission.
- Allow foreign source preparation to satisfy exact plan source requirements from matching Nario records.
- Re-materialize projected sources under target Mantle identities instead of treating original Nix signatures as target signatures.
- Retain positive and negative producer fixtures from the pinned Nario implementation.

## Impact

- **Planned files**: a pure Nario framing core, store archive adapters, foreign source preparation, CLI format selection, policy, fixtures, and operator documentation.
- **Testing**: pinned compatibility fixtures, bounded streaming tests, signature and CA tests, source projection, malformed input, truncation, duplicate records, limits, and wrong-prefix failures.
- **Compatibility**: supported direction is Nario v2 list and import only. Mantle-native archives remain the default format.
- **Current effect**: lifecycle planning only. Mantle does not yet claim Nario compatibility.
