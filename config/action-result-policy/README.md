# Shared action-result policy

`default.ncl` is the typed source of truth for shared action-result source classes, trust requirements, bounds, offline behavior, publication, claim strength, and GC retention.

Regenerate the checked-in runtime JSON after policy changes:

```sh
nix run nixpkgs#nickel -- export --format json \
  config/action-result-policy/default.ncl \
  > config/action-result-policy/generated/action-result-policy.json
```

Rust loads the generated JSON and asserts that its bounds match the pure action-result core. Candidate/index presence and CA mappings remain advisory. GC retains action-result metadata only while its output paths are independently live; candidate metadata never roots output objects.
