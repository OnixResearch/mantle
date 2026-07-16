# Self-contained contracts keep scratch copies reviewable and type checked.
let GeneratedContent = {
  type | String,
  text | String,
} in
let GeneratedContract = {
  identity | String,
  required_json_fields | Array String,
} in
let GeneratedFile = {
  name | String,
  target | String,
  content | GeneratedContent,
  materialization | String,
  contract | GeneratedContract | optional,
} in

{
  ProjectManifest = {
    version | String,
    files | Array GeneratedFile,
  },
}
