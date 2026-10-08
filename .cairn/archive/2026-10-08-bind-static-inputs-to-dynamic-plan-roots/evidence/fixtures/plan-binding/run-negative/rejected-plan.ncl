let mantle = import "lib.ncl" in
let plan_producer = {
  name = "static-rejected-producer", builder = "/bin/sh",
  addressing_mode = 'input-addressed,
  outputs = ["out", "plan"], dynamic_plan_outputs = ["plan"],
  args = ["-c", "echo not-json > \"$plan\"; echo ready > \"$out\""],
} | mantle.Derivation in
{
  rejected = {
    name = "static-rejected-consumer", builder = "/bin/sh",
    addressing_mode = 'input-addressed,
    inputs = [{ name = "bad", producer = plan_producer, plan_output = "plan", root = "unit.ia", unit_output = "out" }],
    env = { ROOT = "{{mantle-plan-output:bad}}" },
    args = ["-c", "/bin/cat \"$ROOT\" > \"$out\""],
  } | mantle.Derivation,
}
