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
      'i386-linux,
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
      a lazy derivation-file record `{ derivation_file }`, an output selection
      `{ drv, output }`, or a late-bound dynamic-plan root
      `{ name, producer, plan_output, root, unit_output }`.
      Use `{{mantle-plan-output:<name>}}` in consumer args or env values;
      the producer must declare plan_output in dynamic_plan_outputs.
    "%
    = std.contract.from_validator (fun value =>
      if std.is_string value then
        if std.string.is_match store_path_regex value
        then 'Ok
        else 'Error { message = "input string must be a valid absolute store path, got: %{value}" }
      else if std.is_record value then
        if std.record.has_field "producer" value
          || std.record.has_field "plan_output" value
          || std.record.has_field "root" value
          || std.record.has_field "unit_output" value then
          if std.array.length (std.record.fields value) == 5
            && std.record.has_field "name" value
            && std.record.has_field "producer" value
            && std.record.has_field "plan_output" value
            && std.record.has_field "root" value
            && std.record.has_field "unit_output" value
            && std.is_string value.name
            && std.string.length value.name <= 64
            && std.string.is_match "^[a-z][a-z0-9_.-]*$" value.name
            && std.is_string value.root
            && std.string.length value.root <= 128
            && std.string.is_match "^[a-z][a-z0-9_.-]*$" value.root
            && std.is_string value.plan_output
            && std.string.length value.plan_output <= 64
            && std.string.is_match "^[a-z][a-z0-9_+-]*$" value.plan_output
            && std.is_string value.unit_output
            && std.string.length value.unit_output <= 64
            && std.string.is_match "^[a-z][a-z0-9_+-]*$" value.unit_output
            && std.is_record value.producer
            && std.record.has_field "name" value.producer
            && std.record.has_field "builder" value.producer
            && std.record.has_field "outputs" value.producer
            && std.record.has_field "dynamic_plan_outputs" value.producer
            && std.is_array value.producer.outputs
            && std.is_array value.producer.dynamic_plan_outputs
            && std.array.elem value.plan_output value.producer.outputs
            && std.array.elem value.plan_output value.producer.dynamic_plan_outputs
          then 'Ok
          else 'Error { message = "plan-output input requires exactly name, producer, plan_output, root, unit_output; valid names and a declared producer dynamic-plan output" }
        else
        if std.record.has_field "derivation_file" value then
          if std.is_string value.derivation_file
            && (!std.record.has_field "output" value || std.is_string value.output)
            && std.array.all
              (fun field => field == "derivation_file" || field == "output")
              (std.record.fields value)
          then
            'Ok
          else
            'Error { message = "derivation-file input must contain a string 'derivation_file' and optional string 'output'" }
        else if std.record.has_field "drv" value && std.record.has_field "output" value then
          # Output selection: { drv = <Derivation>, output = "dev" }
          if std.is_record value.drv && std.is_string value.output then
            'Ok
          else
            'Error { message = "output selection: 'drv' must be a record, 'output' must be a string" }
        else
          'Ok  # Bare derivation record
      else
        'Error { message = "input must be a Derivation record, derivation-file record, output selection, plan-output reference, or StorePath string" }
    ),
}
