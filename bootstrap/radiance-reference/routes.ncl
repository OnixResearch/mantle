# Typed two-route stage graph mirrored by crunch-radiance-reference-core.
let stage_one = 1 in
let stage_two = stage_one + 1 in
let stage_three = stage_two + 1 in
let PositiveStage = std.contract.from_predicate (fun value =>
  std.is_number value && value >= stage_one && value <= stage_three
) in
let Route = std.contract.from_predicate (fun value =>
  value == "seed" || value == "c99"
) in
let Launch = std.contract.from_predicate (fun value =>
  value == "native-bootstrap" || value == "emulated-compiler"
) in
let Stage = {
  route | Route,
  stage | PositiveStage,
  launch_kind | Launch,
  predecessor_role | String,
  source_projection | std.contract.Equal ".",
  output_role | String,
} in
let Graph = {
  schema | std.contract.Equal "mantle-radiance-reference-route-graph-v1",
  stages | Array Stage,
  non_claim | String,
} in

{
  schema = "mantle-radiance-reference-route-graph-v1",
  stages = [
    { route = "seed", stage = stage_one, launch_kind = "emulated-compiler", predecessor_role = "radiance-seed", source_projection = ".", output_role = "seed-stage-1" },
    { route = "seed", stage = stage_two, launch_kind = "emulated-compiler", predecessor_role = "seed-stage-1", source_projection = ".", output_role = "seed-stage-2" },
    { route = "seed", stage = stage_three, launch_kind = "emulated-compiler", predecessor_role = "seed-stage-2", source_projection = ".", output_role = "seed-stage-3" },
    { route = "c99", stage = stage_one, launch_kind = "native-bootstrap", predecessor_role = "radiance-s0", source_projection = ".", output_role = "c99-stage-1" },
    { route = "c99", stage = stage_two, launch_kind = "emulated-compiler", predecessor_role = "c99-stage-1", source_projection = ".", output_role = "c99-stage-2" },
    { route = "c99", stage = stage_three, launch_kind = "emulated-compiler", predecessor_role = "c99-stage-2", source_projection = ".", output_role = "c99-stage-3" },
  ],
  non_claim = "route equality is convergence evidence only",
} | Graph
