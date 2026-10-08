let mantle = import "lib.ncl" in
let plan_producer = import "../run-positive/producer.ncl" in
{
  undeclared = {
    name = "static-undeclared-plan", builder = "/bin/sh",
    addressing_mode = 'input-addressed,
    inputs = [{ name = "bad", producer = plan_producer, plan_output = "out", root = "unit.ia", unit_output = "out" }],
    env = { ROOT = "{{mantle-plan-output:bad}}" },
    args = ["-c", "/bin/cat \"$ROOT\" > \"$out\""],
  } | mantle.Derivation,
}
