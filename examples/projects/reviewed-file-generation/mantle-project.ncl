# Reviewed generated-file declarations. `mantle filegen plan` is non-mutating;
# `mantle filegen apply` accepts only the still-current reviewed plan.
let project = import "filegen-schema.ncl" in

{
  version = "1.0.0",
  files = [
    {
      name = "application-config",
      target = "generated/app-config.json",
      content = {
        type = "inline",
        text = m%"
          {"schema_version":1,"name":"mantle-filegen-demo","mode":"reviewed"}
        "%,
      },
      materialization = "copy",
      contract = {
        identity = "mantle-filegen-demo-v1",
        required_json_fields = ["schema_version", "name", "mode"],
      },
    },
    {
      name = "operator-note",
      target = "generated/README.txt",
      content = {
        type = "inline",
        text = "generated only after a reviewed Mantle plan\n",
      },
      materialization = "copy",
    },
  ],
} | project.ProjectManifest
