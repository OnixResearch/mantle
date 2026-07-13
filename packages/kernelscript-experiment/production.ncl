# Disabled-by-default KernelScript production experiment.
# Resolve kernelscript-cohort.ncl from the pinned Nix cohort package:
#   mantle build -I <cohort>/share/mantle/kernelscript production.ncl
let mantle = import "lib.ncl" in
let cohort = import "kernelscript-cohort.ncl" in
{
  name = "kernelscript-production-evidence",
  builder = cohort.production_shell,
  inputs = cohort.closure_inputs,
  env = {
    EXPERIMENTAL_KERNELSCRIPT = "1",
  },
  addressing_mode = 'input-addressed,
} | mantle.Derivation
