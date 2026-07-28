# Reproducible release bundle

This project assembles a small payload, normalizes file modes and timestamps, creates the release archive in two independently named derivations, and compares the resulting bytes. A second check proves that an appended-byte tamper is detected.

```sh
cd examples/projects/reproducible-release
mantle build
mantle build .#release-a
mantle build .#release-b
mantle build .#checks.reproducible
mantle build .#checks.tamper-detection
```

Each release output contains `release-demo.tar` and `release-demo.tar.blake3`.

The integration test checks the expected BLAKE3 sidecar against the archive bytes.

This proves reproducibility only for the selected payload, BusyBox tar implementation, and declared normalization procedure. It does not prove a release signature or cross-platform reproducibility.
