# Protected transition v51: TinyCC musl-v2

The fresh create-new transition completed through TinyCC 0.9.27 rebuilt by the protected TinyCC 0.9.26 host. It also built the extended variadic runtime.

- Scratch: `/home/brittonr/.cargo-target/stagex-protected-transition-v51-tcc-musl-v2-20260728`
- Pueue task: `3224`
- Test result: `ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1783 filtered out; finished in 1159.51s`
- Protected stages: 70
- Protected events: 1396 allowed; zero denied
- Fallback events: zero
- Plan BLAKE3: `f272357b75f5e43a688ed955ba63badddaf158b63b84a114dccebbc6398eee3f`
- Manifest BLAKE3: `dba712b8c0cb82e7df6a1721e46af26cf5cbbfb1c899d7952bb24c62149b4652`
- Source-state BLAKE3: `fd8d6777751e1a0eaf86d17578f8b43c71c8059f03cdb07cb1933e486a96a855`
- Recipe BLAKE3: `c82e4b5c375d3f9ba767c916ad4dba18e995d239755ea492ae94a007d2f49190`
- Variadic runtime source BLAKE3: `39655914dd22719e8f64182e281622ccaa4104ab7d901f728db84338659cc942`
- Configured source BLAKE3: `b68ec9f7c46ca1cbec5508a98476f771e4bcc4d90b4e088a237065aac1c6061b`
- Compiler and alias BLAKE3: `e7c34884d2dd51b38db8a19034ce36cc79a66af9222e30fed1f37d7516351765`
- Runtime archive BLAKE3: `c201ffd35a5466b4171345fc924b64a75c650fabbe7d0c20d365cfe08faf0860`
- Normalized source tree BLAKE3: `f6d2aec4068377861ab1548e672389dd79aaa31412057e5b4d3187e59ec11e26`
- Positive variadic binary BLAKE3: `2a6733b887417ec6f848ece381d125e10d5d5544f0839dcfa896eacc88529fa6`

Four protected TinyCC 0.9.26 executions built the compiler and runtime. Four compiler executions handled version, object compilation, automatic static linking, and malformed-source rejection. One separately authorized execution ran the positive variadic binary successfully.

This evidence proves only the bounded v2 compiler handoff. It does not prove the later self-host, native runtime, GNU/GCC/binutils chain, or provider admission.
