# Negative fixture: generated files must stay inside the project root.
let project = import "../filegen-schema.ncl" in

{
  version = "1.0.0",
  files = [
    {
      name = "escaped-config",
      target = "../escaped-config.json",
      content = {
        type = "inline",
        text = "{}\n",
      },
      materialization = "copy",
    },
  ],
} | project.ProjectManifest
