# I1 upstream ordering evidence

Task-ID: I1
Covers: bootstrap.part.tar.1.12

`bootstrap/tar-tcc.ncl` tracks GNU tar 1.12 after TinyCC and gzip 1.2.4 are available, replacing the stage0 untar utility with a fuller archive tool. The source pin now records live-bootstrap provenance and first-consumer notes beside the fixed-output fetch.
