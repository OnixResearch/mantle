# Mantle project-input manifest for refresh, stale detection, and lock generation.

{
  version = "1.0.0",
  inputs = [
    {
      name = "message-source",
      kind = { type = "file", url = "file://./sources/message.txt" },
      patches = ["message-fix"],
      mirrors = ["https://mirror.example.invalid/message.txt"],
      retention = { mode = "current" },
    },
  ],
  patches = [
    {
      name = "message-fix",
      source = { type = "local", path = "patches/message.patch" },
    },
  ],
}
