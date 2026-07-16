# Negative fixture: the declared JSON contract requires a mode field.
let project = import "../filegen-schema.ncl" in

{
  version = "1.0.0",
  files = [
    {
      name = "incomplete-config",
      target = "generated/incomplete.json",
      content = {
        type = "inline",
        text = "{\"schema_version\":1,\"name\":\"incomplete\"}\n",
      },
      materialization = "copy",
      contract = {
        identity = "mantle-filegen-demo-v1",
        required_json_fields = ["schema_version", "name", "mode"],
      },
    },
  ],
} | project.ProjectManifest
