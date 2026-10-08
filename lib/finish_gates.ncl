# Closed, bounded contract for the shared post-install gate policy.
let NonEmpty = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0 && std.string.length value <= 256
) in
let Command = std.contract.from_predicate (fun value =>
  std.is_array value && std.array.length value > 0 && std.array.length value <= 16
    && std.array.all (fun arg => std.is_string arg && std.string.length arg > 0 && std.string.length arg <= 4096) value
) in
let Paths = std.contract.from_predicate (fun value =>
  std.is_array value && std.array.length value <= 128
    && std.array.all (fun path => std.is_string path && std.string.length path > 0 && std.string.length path <= 4096) value
) in
let Sonames = std.contract.from_predicate (fun value =>
  std.is_array value && std.array.length value <= 32
    && std.array.all (fun soname => std.is_string soname && std.string.length soname > 0 && std.string.length soname <= 256) value
) in
let Version = {
  enabled | Bool | default = true,
  command | Command | optional,
  expected | NonEmpty | optional,
  environment | { _ : String } | default = {},
} in
let Leak = {
  enabled | Bool | default = true,
  native | [| 'report, 'deny |] | default = 'report,
  build_platform_paths | Paths
    | doc "Absolute build-only paths denied on a match, including native builds."
    | default = [],
} in
let Relocation = {
  enabled | Bool | default = false,
} in
let Dlopen = {
  enabled | Bool | default = false,
  optional_sonames | Sonames | default = [],
} in
{
  Gate = {
    schema | std.contract.from_predicate (fun value => value == "mantle-derivation-finish-gates-v1")
      | default = "mantle-derivation-finish-gates-v1",
    version | Version | default = {},
    reference_leak | Leak | default = {},
    relocation | Relocation | default = {},
    dlopen | Dlopen | default = {},
  },
  Policy = {
    schema | std.contract.from_predicate (fun value => value == "mantle-derivation-finish-policy-v1"),
    max_command_args | std.contract.from_predicate (fun value => value == 16),
    max_command_arg_bytes | std.contract.from_predicate (fun value => value == 4096),
    max_scan_hits | std.contract.from_predicate (fun value => value == 128),
    max_excerpt_bytes | std.contract.from_predicate (fun value => value == 160),
    max_optional_sonames | std.contract.from_predicate (fun value => value == 32),
    gates | Gate,
  },
}
