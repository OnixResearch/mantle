# V3 bash tcc link boundary

Direct focused validation of `bootstrap/bash-2.05b-tcc.ncl` now advances past the prior `no input files`/empty-object fallback boundary. The derivation compiles the upstream core source set plus generated bootstrap headers/stubs far enough to reach a full TinyCC static link. The current concrete blocker is missing Bash/runtime support symbols at link time (for example `expand_word_unsplit`, `xrealloc`, `tcgetattr`, `setifs`, and related expansion/options helpers).

Evidence prefix: `V3-bash-tcc-link-boundary-*`.

This does not complete V3; it narrows the active `binutils-2.30-tcc -> bash-2.05b-tcc` blocker from object creation/fallback plumbing to the remaining Bash link closure.
