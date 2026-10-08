let mantle = import "lib.ncl" in
let plan_producer = import "../run-positive/producer.ncl" in
{
  missing_root = {
    name = "static-missing-root",
    builder = "/bin/sh",
    addressing_mode = 'input-addressed,
    inputs = [{ name = "wrong", producer = plan_producer, plan_output = "plan", root = "unit.absent", unit_output = "out" }],
    env = { ROOT = "{{mantle-plan-output:wrong}}" },
    args = ["-c", "/bin/cat \"$ROOT\" > \"$out\""],
  } | mantle.Derivation,
}
