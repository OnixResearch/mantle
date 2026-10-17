let mantle = import "lib.ncl" in
let plan_producer = import "../run-positive/producer.ncl" in
{
  missing_output = {
    name = "static-missing-output",
    builder = "/bin/sh",
    addressing_mode = 'input-addressed,
    inputs = [{ name = "wrong", producer = plan_producer, plan_output = "plan", root = "unit.ia", unit_output = "dev" }],
    env = { ROOT = "{{mantle-plan-output:wrong}}" },
    args = ["-c", "/bin/cat \"$ROOT\" > \"$out\""],
  } | mantle.Derivation,
}
