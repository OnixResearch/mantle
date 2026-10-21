# Pinned pnpm authoring lowers to an ordinary Mantle derivation, never a new store.
let Derivation = import "derivation.ncl" in
let { Input, Name, .. } = import "contracts.ncl" in
let Role = { input | Input, path | String } in
let PlanRole = { input | Input, path | String, blake3 | String } in
let Workspace = {
  name | Name,
  adapter | Role,
  source | Role,
  mirror | Role,
  tools | Role,
  runtime | Array Role | default = [],
  # An admitted JSON file with the exact crunch-pnpm-core Request wire contract.
  # A separate file avoids argv/env size limits and source-digest self-reference.
  plan | PlanRole,
} in
{
  mkWorkspace = fun raw =>
    let package = raw | Workspace in
    {
      name = package.name,
      builder = package.adapter.path,
      args = [
        "build",
        package.plan.path,
        package.plan.blake3,
        package.source.path,
        package.mirror.path,
        package.tools.path,
      ],
      inputs = [package.adapter.input, package.plan.input, package.source.input, package.mirror.input, package.tools.input]
        @ std.array.map (fun role => role.input) package.runtime,
      addressing_mode = 'input-addressed,
      env = {},
    } | Derivation,
}
