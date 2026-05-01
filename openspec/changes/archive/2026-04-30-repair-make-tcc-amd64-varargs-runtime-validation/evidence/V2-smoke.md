Task-ID: V2
Covers: bootstrap.part.make.3.82.amd64.runtime-validation
Status: deferred

The Make smoke tests require a produced `make-3.82-tcc` output path. V1 reached the Make derivation but did not produce an output because the Mes-linked TinyCC 0.9.27 compiler segfaults on static link, including a minimal `hello.o` link.

Deferred to OpenSpec change:

    repair-tinycc-0-9-27-amd64-link

After that repair passes its trivial static-link smoke, this runtime-validation successor should be resumed and the version, positive Makefile, and missing-target negative smokes should be run against the produced `bin/make`.
