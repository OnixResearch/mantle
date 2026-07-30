# Protected transition v81: complete binutils boundary

## Result

The retained transition report has `status = complete`.
The plan contains 97 ordered stages.
The protected audit contains 76,569 allowed events and no denied event.
The transition and all reported stage inventories contain no fallback event.

This directory preserves the exact plan, report, audit, final component inventories, smoke observations, and 141 binutils command logs.
`blake3sums.txt` and `binutils-command-log-blake3sums.txt` bind the preserved files with BLAKE3.

## Bound transition identities

- lineage manifest: `ff13121154ebda99ec4b3887aff52f38584e1a91395a662f8ca3e704ae1bd80a`
- plan: `40cb3d15060a8d81c751fb0184751b8fa8ef186742802f1759affdfa3396f98f`
- source state: `18b54c3ac3fdd7931453a849f15c76ea642d248d2b597b1b8c7849ae97f4a772`
- transition report: `405f809ca3c7876b88a7e98f4746c1bbae518ffc8b1fefdafcef5bc383666a80`
- protected audit: `29ca382d6c8ca4364d30e1f0e92680bc49c743a45c85ec9b4e7bbbe3ab76bc91`
- audited and reproduced hex0: `cf21608d883b8bdcc1fa6438703630f2fa496cf74d483ce351f876c0656ecf80`
- source-built kaem: `edc664d028b349824cc8c66030a8883b81bdc03d07bc871e44e60ecb4311ab2a`

## Provider component identities

- self-hosted TinyCC: `8e6580da40c5892b941423ae108d6218b3636ebd3643bc3ba1e78e33d6c4898a`
- native-musl `libc.a`: `a9aa627c3a70fce68d928e0a7ccd423370472a87ae9fde795a85db4d0899366c`
- native-musl headers: `315e38bd3f318804adf63fadc9689a9b692ee69986e26c6fddd4670190498700`
- binutils inventory: `90a625d2066fb63f9b66073519343ee0562f3a0d8cf7b1a1bc55a42fc7e42afe`
- binutils source: `809e8c1d946b14650362cf2929520efe07623fe069ce8c16c5870dafe6d93603`

The binutils inventory records nine configure classes, eight components, 11 installed tools, four archives, and 4,891 sed calls.
Its accepted protected event bound is `[73,996, 74,057]`.

## Installed binutils identities

- `as`: `36bb17408403b4fd8283bf80f78410ae76eedb4e1565f0fc6db0f7a8c0a1eac4`
- `ld`: `e2939e05b0e115efa3530f66d60b63ef06627a1ba0c0c1a70ce71fabcf4ca158`
- `ar`: `c5836470e484f7b9137abc3bbdffbec7155a7700fdf094662b4253a576e76600`
- `ranlib`: `8c6d65ff0cc4b6936e6f93a016782890dd39e556ef2f99d1699d0071c6691533`
- `nm`: `bd92671b478f6f88d3aea88d910335cb024079bebcf4432c8abba77a51d1b00b`
- `objcopy`: `919f6ad3c798a023395ca4d1d4574f395c8316b5595b25fbdf3bdafc891552e3`
- `objdump`: `f48859e9dbfb3594a92cc52ba2281879c7f02855a5679a03ed5349489a019ca1`
- `readelf`: `9c13500d32b180628d9768d0c56c3eabdd242f35dee79119c60e7b52f4ad3e0d`
- `size`: `812bd48e35d078b759192073cb2de2accaf9620c075c2ae24df9329c5623a7dd`
- `strings`: `8ff50f5ca1c25ab91d259225eb8fc7ed85584e9071d732192a80d82f9c24866f`
- `strip`: `d5505e1aa9e5b016a0b976b067aa0c7d096c84d0e875a41ff456be612d8ccc07`

## Evidence limits

The original pueue command log is not present in the current pueue daemon.
The transition report, protected audit, inventories, smoke files, and component command logs remain available.

This evidence proves the recorded protected transition through authenticated binutils 2.30.
It supplies the components for an intermediate StageX provider.
It does not prove provider publication, provider admission, native TinyCC, GCC, compiler correctness, or Mantle self-build completion.
