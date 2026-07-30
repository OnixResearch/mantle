# Binutils `ylwrap` protected-sed timeout

Date: 2026-07-30

## Result

The authenticated binutils parser-generation Make target timed out after 300 seconds while generating `binutils/arparse.c`.

The timeout is not in Bison or M4. The exact protected Bison 2.3 command completed successfully in a separate 30-second bound. It produced:

- `y.tab.c`: 50,168 bytes
- `y.tab.h`: 2,949 bytes

The authenticated `ylwrap` trace reached its rewrite of `y.tab.c`. The protected regular-file sed bridge did not complete this command within 30 seconds:

```text
sed -e '/^#/!b' \
    -e 's|<authenticated-source-prefix>/||' \
    -e 's|y\.tab\.c|arparse.c|g;s|y\.tab\.h|arparse.h|g;s|y\.output|arparse.output|g;' \
    -e 's|Y_TAB_C|ARPARSE_C|g;s|Y_TAB_H|ARPARSE_H|g;s|Y_OUTPUT|ARPARSE_OUTPUT|g;' \
    y.tab.c
```

The trace ended at this sed execution. Its signal trap removed the temporary `ylwrap` directory. Make then deleted the incomplete `arparse.c` target.

## Identities

- Authenticated `ylwrap`: `5512cfc54d2f148012cf53d6f60ced5a0bc0e67b16630bedc73a741ffd036756`
- Authenticated `arparse.y`: `9a0482cda2db564093b67bb772322732e4ef1231cc9170fff80d4ac7e65d0cbc`
- Direct Bison `y.tab.c`: `a47a58fed096cd8d4127245d3b004a1611bb7426510103033da929a003d2832f`
- Direct Bison `y.tab.h`: `fc99873f8f988dceabe51214349a6af00d4ddb29983dbeea73f90a082b93abec`
- Protected Bison: `3db09397aa2752eea9c438a4cb1c5ebd27605f58b56ec43781baf411c3f2b983`
- Protected M4: `3dfd2a1223ff6e0c2c540bc2507a097948ab24e2465f232e2363944e5b704741`
- Protected sed: `cc4bc898fbad65d9c0a07eab85c61c99abdb9d7d7d747948067bea7d9954618a`
- Full Bash: `3f166d5bb28aee29982ec38ba07f8b1dca3c488390737a0e78d35950876e4d4e`

## Boundary

This is a fail-closed performance blocker. No parser output was accepted.

A repair must preserve the authenticated `ylwrap` filename and header-guard transformations. It must use bounded regular-file input and output, exact invocation shapes, and a canonical producer identity. It must not authorize ambient sed, bypass `ylwrap` silently, or treat direct Bison output as the final accepted parser.

This evidence does not prove parser generation, scanner generation, full binutils compilation, binutils behavior, or provider admission.
