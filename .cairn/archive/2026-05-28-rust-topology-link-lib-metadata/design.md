## Design

The change stays in the pure metadata parsing core. The imperative shell still only runs build scripts, captures stdout, and passes that stdout into the parser.

`cargo:rustc-link-lib` values are parsed as one of these bounded forms:

- `NAME`
- `KIND=NAME`, where `KIND` is one of Cargo/rustc's supported link kinds.
- `KIND[:MODIFIER...]=NAME`, where every modifier is a safe token segment and the final name is still a safe token.

The parser rejects empty values, whitespace, paths, empty kind/name/modifier segments, unsupported kinds, unsupported punctuation, and comma-separated rename forms until Mantle models those separately. The receipt stores the original directive value after validation so rustc argument replay remains faithful.

This is intentionally not a full native-link compatibility claim. It only moves the parser boundary past Cargo-accepted link-lib directives observed in the self-probe while keeping malformed metadata fail-closed.
