# Trellis executable-model oracle

The oracle generator compiled `verified-logic` from the exact Trellis proof commit.

The generator evaluated these finite axes in stable order:

- Seven phases.
- Eight Mantle report variants.
- Five identity classes.
- Three event classes.
- Two history classes.
- Two worker scopes.
- Two output scopes.

The complete product contains 6,720 cases. Each fixed-width record stores the disposition, rejection class, next phase, and field changes.

The generated binary is 47,040 bytes. Its BLAKE3 is `bd1e98e26dbd43f44189941c1eac22bd63139d031c2ea328f0c2d5ce8582aa32`.

Pueue task 282 regenerated the binary and compared it byte-for-byte with `fixtures/trellis-remote-admission/oracle.v1.bin`.

The checked manifest records 5,882 supported Mantle projections. It records 838 unsupported semantic combinations with stable reasons.

The generator source and lock file in `generator/` are the exact files used for this run. They are evidence-only files.
