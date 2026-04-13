# crunch stdlib: contracts and validators.
#
# Validators for store paths and derivation names.
# Enum types for categorical fields.

let store_path_regex = "^(/[A-Za-z0-9._+-]+)+/[a-z0-9]{32}-.+$" in

{
  StorePath
    | doc "Validates an absolute store path string: <store-prefix>/<32 chars>-<name>"
    = std.contract.from_validator (fun value =>
      if std.is_string value
        && std.string.is_match store_path_regex value
      then 'Ok
      else 'Error { message = "expected an absolute <store-prefix>/<hash>-<name> path, got: %{std.serialize 'Json value}" }
    ),

  Name
    | doc "Validates a derivation name (alphanumeric, plus . _ ? = -)"
    = std.contract.from_validator (fun value =>
      if std.is_string value
        && std.string.length value > 0
        && std.string.is_match "^[a-zA-Z0-9+._?=-][a-zA-Z0-9+._?=-]*$" value
      then 'Ok
      else 'Error { message = "invalid derivation name: %{std.serialize 'Json value}" }
    ),

  System
    | doc "Target platform enum."
    = [|
      'x86_64-linux,
      'aarch64-linux,
      'x86_64-darwin,
      'aarch64-darwin,
    |],

  HashAlgo
    | doc "Hash algorithm for fixed-output derivations."
    = [| 'md5, 'sha1, 'sha256, 'sha512, 'blake3 |],

  HashMode
    | doc "Hashing mode: flat (single file) or recursive (NAR serialization)."
    = [| 'flat, 'recursive |],

  Sandbox
    | doc "Sandbox backend for build execution."
    = [| 'native, 'oci, 'wasm |],

  Input
    | doc m%"
      A build input: a StorePath string, a Derivation record (all outputs),
      or an output selection record `{ drv, output }` (single output).
    "%
    = std.contract.from_validator (fun value =>
      if std.is_string value then
        if std.string.is_match store_path_regex value
        then 'Ok
        else 'Error { message = "input string must be a valid absolute store path, got: %{value}" }
      else if std.is_record value then
        if std.record.has_field "drv" value && std.record.has_field "output" value then
          # Output selection: { drv = <Derivation>, output = "dev" }
          if std.is_record value.drv && std.is_string value.output then
            'Ok
          else
            'Error { message = "output selection: 'drv' must be a record, 'output' must be a string" }
        else
          'Ok  # Bare derivation record
      else
        'Error { message = "input must be a Derivation record, output selection { drv, output }, or a StorePath string" }
    ),
}
