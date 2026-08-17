# Full self-hosting fixed-point proof with CRUNCH_NO_FUSE after manifest-dir leak fix

Date: 2026-06-29T17:10:52Z

## Command

~~~text
CRUNCH_NO_FUSE=1 ./scripts/prove-self-hosting.sh
~~~

## Transcript

~~~text
proof mode: fixed-point
selected provider kind: legacy-fetch
proof scratch root: /home/brittonr/git/mantle/target/self-hosting-proof/work
proof scratch source: default repo-local policy
proof TMPDIR: /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp
proof CARGO_TARGET_DIR: /home/brittonr/git/mantle/target/self-hosting-proof/work/cargo-target
proof scratch free: 2425658 MiB
proof bundle dir: /home/brittonr/git/mantle/target/self-hosting-proof/run-20260629T131052Z-2021688
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.52s
     Running tests/self_hosting.rs (target/self-hosting-proof/work/cargo-target/debug/deps/self_hosting-d660496efc741690)

running 1 test
proof dir: /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr
proof mode: FixedPoint
later-stage hermeticity: strict

=== PROOF: Stage 0 (checkout -> stage1) ===

stage0 store: /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store
stage0 state: /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/state0
stage0 stdout: /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/stage0-stdout.txt
stage0 stderr: /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/stage0-stderr.txt
=== crunch self-build ===
  invoking binary: /home/brittonr/.cargo-target/repo-targets/mantle/self-hosting-proof/work/cargo-target/debug/crunch
  WARNING: no mantle-built bwrap in /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store; using external bwrap at /nix/store/gr9l6ql3wg70idpqlqhnfdx81hak22c8-bubblewrap-0.11.0/bin/bwrap
source: /home/brittonr/git/mantle

[1/4] Staging source...
  copying selected source tree...
  checking staged vendored cargo inputs...
  source tree: 856 MiB
  staged: /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/zi7ps2paplpbljym0i0z8jkdq0ymibxq-mantle-src
Generated signing key: crunch-britton-desktop-1 (/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/state0/signing-key)

[2/4] Building bootstrap tools...
self-build-proof: progress=bootstrap-tool-start:bwrap.ncl
  building bwrap.ncl...
[2m2026-06-29T17:11:15.384414Z[0m [32m INFO[0m [2mcrunch_store::handle[0m[2m:[0m blob service opened [3mpath[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/state0/blobs
[2m2026-06-29T17:11:15.385446Z[0m [32m INFO[0m [2mredb::db[0m[2m:[0m Opening database "FileBackend { lock_supported: true, file: File { fd: 10, path: \"/home/brittonr/.cargo-target/repo-targets/mantle/self-hosting-proof/work/tmp/.tmp9xxeXr/state0/directories.redb\", read: true, write: true } }"
[2m2026-06-29T17:11:15.436089Z[0m [33m WARN[0m [2mredb::db[0m[2m:[0m Database "FileBackend { lock_supported: true, file: File { fd: 10, path: \"/home/brittonr/.cargo-target/repo-targets/mantle/self-hosting-proof/work/tmp/.tmp9xxeXr/state0/directories.redb\", read: true, write: true } }" not shutdown cleanly. Repairing
[2m2026-06-29T17:11:15.442035Z[0m [32m INFO[0m [2mredb::db[0m[2m:[0m Opening database "FileBackend { lock_supported: true, file: File { fd: 11, path: \"/home/brittonr/.cargo-target/repo-targets/mantle/self-hosting-proof/work/tmp/.tmp9xxeXr/state0/pathinfo.redb\", read: true, write: true } }"
[2m2026-06-29T17:11:15.443758Z[0m [33m WARN[0m [2mredb::db[0m[2m:[0m Database "FileBackend { lock_supported: true, file: File { fd: 11, path: \"/home/brittonr/.cargo-target/repo-targets/mantle/self-hosting-proof/work/tmp/.tmp9xxeXr/state0/pathinfo.redb\", read: true, write: true } }" not shutdown cleanly. Repairing
[2m2026-06-29T17:11:15.448697Z[0m [32m INFO[0m [2mcrunch_store::handle[0m[2m:[0m PathInfo database opened [3mpath[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/state0/pathinfo.redb
[2m2026-06-29T17:11:15.449227Z[0m [32m INFO[0m [2mcrunch_build::worker[0m[2m:[0m worker streaming started [3mjobs[0m[2m=[0m4
[2m2026-06-29T17:11:15.584700Z[0m [32m INFO[0m [2mcrunch_pipeline[0m[2m:[0m converted, sending to worker [3mdrv[0m[2m=[0mdy6gw4nnjh1al57dxpkpz81grssadh9i-bwrap.drv [3mlabel[0m[2m=[0mbwrap [3mentries[0m[2m=[0m17
[2m2026-06-29T17:11:15.586340Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m building [3mdrv[0m[2m=[0mbwrap-src.drv
[2m2026-06-29T17:11:15.586797Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m building [3mdrv[0m[2m=[0mmpfr-src.drv
[2m2026-06-29T17:11:15.587094Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m building [3mdrv[0m[2m=[0mmpc-src.drv
[2m2026-06-29T17:11:15.587414Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m building [3mdrv[0m[2m=[0mgmp-src.drv
[2m2026-06-29T17:11:15.587538Z[0m [32m INFO[0m [2mcrunch_build::fetch_build_service[0m[2m:[0m fetching tarball [3murl[0m[2m=[0mhttps://github.com/containers/bubblewrap/releases/download/v0.11.0/bubblewrap-0.11.0.tar.xz
[2m2026-06-29T17:11:15.587747Z[0m [32m INFO[0m [2mcrunch_build::fetch_build_service[0m[2m:[0m fetching tarball [3murl[0m[2m=[0mhttps://mirrors.kernel.org/gnu/mpfr/mpfr-4.1.0.tar.bz2
[2m2026-06-29T17:11:15.587760Z[0m [32m INFO[0m [2mcrunch_build::fetch_build_service[0m[2m:[0m fetching tarball [3murl[0m[2m=[0mhttps://mirrors.kernel.org/gnu/mpc/mpc-1.2.1.tar.gz
[2m2026-06-29T17:11:15.587760Z[0m [32m INFO[0m [2mcrunch_build::fetch_build_service[0m[2m:[0m fetching tarball [3murl[0m[2m=[0mhttps://mirrors.kernel.org/gnu/gmp/gmp-6.2.1.tar.bz2
[2m2026-06-29T17:11:15.587882Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m building [3mdrv[0m[2m=[0mgcc-src.drv
[2m2026-06-29T17:11:15.588143Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m building [3mdrv[0m[2m=[0mmusl-gcc-raw.drv
[2m2026-06-29T17:11:15.588305Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m building [3mdrv[0m[2m=[0mbinutils-src.drv
[2m2026-06-29T17:11:15.588433Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m building [3mdrv[0m[2m=[0mmusl-src.drv
[2m2026-06-29T17:11:15.588555Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m building [3mdrv[0m[2m=[0mdash-src.drv
[2m2026-06-29T17:11:15.588654Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m building [3mdrv[0m[2m=[0mmake-src.drv
[2m2026-06-29T17:11:15.884775Z[0m [32m INFO[0m [2mcrunch_build::fetch_build_service[0m[2m:[0m fetching tarball [3murl[0m[2m=[0mhttps://mirrors.kernel.org/gnu/gcc/gcc-13.3.0/gcc-13.3.0.tar.gz
[2m2026-06-29T17:11:15.900048Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m build succeeded [3mdrv[0m[2m=[0mbwrap-src.drv [3moutputs[0m[2m=[0m["/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/yni075arfsnj807a151aaj9z3q7pnljg-bwrap-src"]
[2m2026-06-29T17:11:16.223845Z[0m [32m INFO[0m [2mcrunch_build::fetch_build_service[0m[2m:[0m fetching tarball [3murl[0m[2m=[0mhttps://musl.cc/x86_64-linux-musl-native.tgz
[2m2026-06-29T17:11:16.338573Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m build succeeded [3mdrv[0m[2m=[0mmpc-src.drv [3moutputs[0m[2m=[0m["/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/p9dxx2mn8fkfllpw2j7dsza5ws54ghwr-mpc-src"]
[2m2026-06-29T17:11:17.099624Z[0m [32m INFO[0m [2mcrunch_build::fetch_build_service[0m[2m:[0m fetching tarball [3murl[0m[2m=[0mhttps://mirrors.kernel.org/gnu/binutils/binutils-2.42.tar.gz
[2m2026-06-29T17:11:17.522165Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m build succeeded [3mdrv[0m[2m=[0mmpfr-src.drv [3moutputs[0m[2m=[0m["/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/66yb9y96cjyqkr3hyzgdrhzk9vh48ijg-mpfr-src"]
[2m2026-06-29T17:11:18.478573Z[0m [32m INFO[0m [2mcrunch_build::fetch_build_service[0m[2m:[0m fetching tarball [3murl[0m[2m=[0mhttps://musl.libc.org/releases/musl-1.2.5.tar.gz
[2m2026-06-29T17:11:19.241918Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m build succeeded [3mdrv[0m[2m=[0mgmp-src.drv [3moutputs[0m[2m=[0m["/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/lrijcfa79mzlayc975ahqfh05lrpxdaz-gmp-src"]
[2m2026-06-29T17:11:20.303559Z[0m [32m INFO[0m [2mcrunch_build::fetch_build_service[0m[2m:[0m fetching tarball [3murl[0m[2m=[0mhttp://gondor.apana.org.au/~herbert/dash/files/dash-0.5.12.tar.gz
[2m2026-06-29T17:11:20.811060Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m build succeeded [3mdrv[0m[2m=[0mmusl-src.drv [3moutputs[0m[2m=[0m["/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/p36m6lwn63zllahxrm756vd760ss7i7c-musl-src"]
[2m2026-06-29T17:11:21.252769Z[0m [32m INFO[0m [2mcrunch_build::fetch_build_service[0m[2m:[0m fetching tarball [3murl[0m[2m=[0mhttps://mirrors.kernel.org/gnu/make/make-4.4.1.tar.gz
[2m2026-06-29T17:11:21.293433Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m build succeeded [3mdrv[0m[2m=[0mdash-src.drv [3moutputs[0m[2m=[0m["/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/zb9gm8bmwzrsxshr1qb85ppf2m6gp1cf-dash-src"]
[2m2026-06-29T17:11:22.485543Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m build succeeded [3mdrv[0m[2m=[0mmake-src.drv [3moutputs[0m[2m=[0m["/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/sadgqs1ykvs2m6zs35qn8q4qg5s38rr3-make-src"]
[2m2026-06-29T17:11:36.413475Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m build succeeded [3mdrv[0m[2m=[0mmusl-gcc-raw.drv [3moutputs[0m[2m=[0m["/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/a0qkrlw0f9jm2l716xl67cb16fc3ggn0-musl-gcc-raw"]
[2m2026-06-29T17:11:36.414386Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m building [3mdrv[0m[2m=[0mmusl-seed-toolchain.drv
[2m2026-06-29T17:11:36.415865Z[0m [32m INFO[0m [1mdo_build[0m[2m:[0m [2msnix_build::buildservice::bwrap[0m[2m:[0m Starting bwrap build [3mbuild_name[0m[2m=[0m3cbf80cc-f50f-4ee4-bb5e-9d4c0e51e78a [3msandbox_path[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/crunch-builds/3cbf80cc-f50f-4ee4-bb5e-9d4c0e51e78a-4Rv4vP
[2m2026-06-29T17:11:36.538077Z[0m [32m INFO[0m [1mdo_build[0m[2m:[0m [2msnix_build::buildservice::bwrap[0m[2m:[0m CRUNCH_NO_FUSE set, materializing inputs to disk [3mpath[0m[2m=[0m"/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/crunch-builds/3cbf80cc-f50f-4ee4-bb5e-9d4c0e51e78a-4Rv4vP/host_inputs_dir"
[2m2026-06-29T17:11:49.383860Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m build succeeded [3mdrv[0m[2m=[0mbinutils-src.drv [3moutputs[0m[2m=[0m["/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/2mgxyaw8zwivswh3kgak7nvbkxrprih7-binutils-src"]
test self_hosting_stage0_stage1_stage2 has been running for over 60 seconds
[2m2026-06-29T17:12:02.916894Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m CA output path resolved [3mdrv[0m[2m=[0mmusl-seed-toolchain.drv [3moutput[0m[2m=[0mout [3mca_path[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain
[2m2026-06-29T17:12:02.945642Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m build succeeded [3mdrv[0m[2m=[0mmusl-seed-toolchain.drv [3moutputs[0m[2m=[0m["/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain"]
[2m2026-06-29T17:12:02.946079Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m building [3mdrv[0m[2m=[0mgnumake.drv
[2m2026-06-29T17:12:02.946340Z[0m [32m INFO[0m [1mdo_build[0m[2m:[0m [2msnix_build::buildservice::bwrap[0m[2m:[0m Starting bwrap build [3mbuild_name[0m[2m=[0mec5f042e-4316-4372-a20c-2143f0c8a686 [3msandbox_path[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/crunch-builds/ec5f042e-4316-4372-a20c-2143f0c8a686-tUnsAh
[2m2026-06-29T17:12:03.017694Z[0m [32m INFO[0m [1mdo_build[0m[2m:[0m [2msnix_build::buildservice::bwrap[0m[2m:[0m CRUNCH_NO_FUSE set, materializing inputs to disk [3mpath[0m[2m=[0m"/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/crunch-builds/ec5f042e-4316-4372-a20c-2143f0c8a686-tUnsAh/host_inputs_dir"
[2m2026-06-29T17:12:10.008250Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m CA output path resolved [3mdrv[0m[2m=[0mgnumake.drv [3moutput[0m[2m=[0mout [3mca_path[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/096b231pbs9cqw67p32sdk011lpm60xl-gnumake
[2m2026-06-29T17:12:10.016726Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m build succeeded [3mdrv[0m[2m=[0mgnumake.drv [3moutputs[0m[2m=[0m["/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/096b231pbs9cqw67p32sdk011lpm60xl-gnumake"]
[2m2026-06-29T17:12:10.017359Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m building [3mdrv[0m[2m=[0mdash.drv
[2m2026-06-29T17:12:10.017739Z[0m [32m INFO[0m [1mdo_build[0m[2m:[0m [2msnix_build::buildservice::bwrap[0m[2m:[0m Starting bwrap build [3mbuild_name[0m[2m=[0m31981ed2-a3d9-4552-9ff3-ba3b790f46d5 [3msandbox_path[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/crunch-builds/31981ed2-a3d9-4552-9ff3-ba3b790f46d5-PBID2a
[2m2026-06-29T17:12:10.087569Z[0m [32m INFO[0m [1mdo_build[0m[2m:[0m [2msnix_build::buildservice::bwrap[0m[2m:[0m CRUNCH_NO_FUSE set, materializing inputs to disk [3mpath[0m[2m=[0m"/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/crunch-builds/31981ed2-a3d9-4552-9ff3-ba3b790f46d5-PBID2a/host_inputs_dir"
[2m2026-06-29T17:12:15.295787Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m CA output path resolved [3mdrv[0m[2m=[0mdash.drv [3moutput[0m[2m=[0mout [3mca_path[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/hvmnd5pnb2zg3l92ycff9p8ms7msigmm-dash
[2m2026-06-29T17:12:15.313555Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m build succeeded [3mdrv[0m[2m=[0mdash.drv [3moutputs[0m[2m=[0m["/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/hvmnd5pnb2zg3l92ycff9p8ms7msigmm-dash"]
[2m2026-06-29T17:12:15.314106Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m building [3mdrv[0m[2m=[0mbinutils.drv
[2m2026-06-29T17:12:15.314240Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m building [3mdrv[0m[2m=[0mmusl.drv
[2m2026-06-29T17:12:15.314331Z[0m [32m INFO[0m [1mdo_build[0m[2m:[0m [2msnix_build::buildservice::bwrap[0m[2m:[0m Starting bwrap build [3mbuild_name[0m[2m=[0mc346e590-c221-49e1-b69d-c8dfad93ff39 [3msandbox_path[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/crunch-builds/c346e590-c221-49e1-b69d-c8dfad93ff39-WOk5A7
[2m2026-06-29T17:12:15.314349Z[0m [32m INFO[0m [1mdo_build[0m[2m:[0m [2msnix_build::buildservice::bwrap[0m[2m:[0m Starting bwrap build [3mbuild_name[0m[2m=[0ma7e84a4d-639a-4b3d-8f76-d2722a5362bd [3msandbox_path[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/crunch-builds/a7e84a4d-639a-4b3d-8f76-d2722a5362bd-VUdHjM
[2m2026-06-29T17:12:15.392891Z[0m [32m INFO[0m [1mdo_build[0m[2m:[0m [2msnix_build::buildservice::bwrap[0m[2m:[0m CRUNCH_NO_FUSE set, materializing inputs to disk [3mpath[0m[2m=[0m"/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/crunch-builds/c346e590-c221-49e1-b69d-c8dfad93ff39-WOk5A7/host_inputs_dir"
[2m2026-06-29T17:12:15.392891Z[0m [32m INFO[0m [1mdo_build[0m[2m:[0m [2msnix_build::buildservice::bwrap[0m[2m:[0m CRUNCH_NO_FUSE set, materializing inputs to disk [3mpath[0m[2m=[0m"/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/crunch-builds/a7e84a4d-639a-4b3d-8f76-d2722a5362bd-VUdHjM/host_inputs_dir"
[2m2026-06-29T17:12:27.863999Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m CA output path resolved [3mdrv[0m[2m=[0mmusl.drv [3moutput[0m[2m=[0mout [3mca_path[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/lpr77qmlvx3yq3dxgavrlgpc64m7p0b2-musl
[2m2026-06-29T17:12:27.871582Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m build succeeded [3mdrv[0m[2m=[0mmusl.drv [3moutputs[0m[2m=[0m["/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/lpr77qmlvx3yq3dxgavrlgpc64m7p0b2-musl"]
[2m2026-06-29T17:13:18.444823Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m build succeeded [3mdrv[0m[2m=[0mgcc-src.drv [3moutputs[0m[2m=[0m["/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/kqv0h44wl9in55dlgwd9l9amlgbp0qp8-gcc-src"]
[2m2026-06-29T17:13:24.802052Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m CA output path resolved [3mdrv[0m[2m=[0mbinutils.drv [3moutput[0m[2m=[0mout [3mca_path[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/qvc3wivg193x36v92pcdf1xq51b4jjky-binutils
[2m2026-06-29T17:13:24.804836Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m build succeeded [3mdrv[0m[2m=[0mbinutils.drv [3moutputs[0m[2m=[0m["/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/qvc3wivg193x36v92pcdf1xq51b4jjky-binutils"]
[2m2026-06-29T17:13:24.805240Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m building [3mdrv[0m[2m=[0mgcc.drv
[2m2026-06-29T17:13:24.805553Z[0m [32m INFO[0m [1mdo_build[0m[2m:[0m [2msnix_build::buildservice::bwrap[0m[2m:[0m Starting bwrap build [3mbuild_name[0m[2m=[0ma3181016-b15f-460e-80d6-7ec022f42bb1 [3msandbox_path[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/crunch-builds/a3181016-b15f-460e-80d6-7ec022f42bb1-MZuy9h
[2m2026-06-29T17:13:24.870757Z[0m [32m INFO[0m [1mdo_build[0m[2m:[0m [2msnix_build::buildservice::bwrap[0m[2m:[0m CRUNCH_NO_FUSE set, materializing inputs to disk [3mpath[0m[2m=[0m"/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/crunch-builds/a3181016-b15f-460e-80d6-7ec022f42bb1-MZuy9h/host_inputs_dir"
[2m2026-06-29T17:19:25.558313Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m CA output path resolved [3mdrv[0m[2m=[0mgcc.drv [3moutput[0m[2m=[0mout [3mca_path[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/rcpdzyz11r40n4wh2pq6q26hxs279kfy-gcc
[2m2026-06-29T17:19:25.560789Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m build succeeded [3mdrv[0m[2m=[0mgcc.drv [3moutputs[0m[2m=[0m["/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/rcpdzyz11r40n4wh2pq6q26hxs279kfy-gcc"]
[2m2026-06-29T17:19:25.561234Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m building [3mdrv[0m[2m=[0mbwrap.drv
[2m2026-06-29T17:19:25.561621Z[0m [32m INFO[0m [1mdo_build[0m[2m:[0m [2msnix_build::buildservice::bwrap[0m[2m:[0m Starting bwrap build [3mbuild_name[0m[2m=[0mc5551495-74ba-42e6-8029-1a59d1822116 [3msandbox_path[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/crunch-builds/c5551495-74ba-42e6-8029-1a59d1822116-P1J8on
[2m2026-06-29T17:19:25.627229Z[0m [32m INFO[0m [1mdo_build[0m[2m:[0m [2msnix_build::buildservice::bwrap[0m[2m:[0m CRUNCH_NO_FUSE set, materializing inputs to disk [3mpath[0m[2m=[0m"/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/crunch-builds/c5551495-74ba-42e6-8029-1a59d1822116-P1J8on/host_inputs_dir"
[2m2026-06-29T17:19:28.760307Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m CA output path resolved [3mdrv[0m[2m=[0mbwrap.drv [3moutput[0m[2m=[0mout [3mca_path[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/pys6ig7mb98pf53s7cq35iafz2wll35f-bwrap
[2m2026-06-29T17:19:28.776632Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m build succeeded [3mdrv[0m[2m=[0mbwrap.drv [3moutputs[0m[2m=[0m["/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/pys6ig7mb98pf53s7cq35iafz2wll35f-bwrap"]
[2m2026-06-29T17:19:28.776778Z[0m [32m INFO[0m [2mcrunch_build::worker[0m[2m:[0m worker streaming finished [3mcompleted[0m[2m=[0m17 [3msucceeded[0m[2m=[0m1 [3mfailed[0m[2m=[0m0 [3mroots[0m[2m=[0m1
--- build log: bwrap ---
Compiling bubblewrap...
Verifying bwrap...
bubblewrap 0.11.0
Bwrap build complete: /nix/store/qgpv3ggfamjm7bxm5pf8mhvzdykbm94s-bwrap/bin/bwrap

In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from bubblewrap.c:33:
./sys/capability.h:11: warning: "cap_valid" redefined
   11 | #define cap_valid(x) ((x) >= 0 && (x) <= __CAP_BITS)
      | 
In file included from ./sys/capability.h:3:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/capability.h:420: note: this is the location of the previous definition
  420 | #define cap_valid(x) ((x) >= 0 && (x) <= CAP_LAST_CAP)
      | 
bubblewrap.c: In function 'set_required_caps':
bubblewrap.c:698:7: warning: implicit declaration of function 'capset' [-Wimplicit-function-declaration]
  698 |   if (capset (&hdr, data) < 0)
      |       ^~~~~~
bubblewrap.c: In function 'has_caps':
bubblewrap.c:748:7: warning: implicit declaration of function 'capget' [-Wimplicit-function-declaration]
  748 |   if (capget (&hdr, data)  < 0)
      |       ^~~~~~
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition

--- end log ---
hermeticity: practical (no degraded facts)
/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/pys6ig7mb98pf53s7cq35iafz2wll35f-bwrap
self-build-proof: progress=bootstrap-tool-done:bwrap.ncl
self-build-proof: progress=bootstrap-tool-start:busybox.ncl
  building busybox.ncl...
[2m2026-06-29T17:19:28.868497Z[0m [32m INFO[0m [2mcrunch_store::handle[0m[2m:[0m blob service opened [3mpath[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/state0/blobs
[2m2026-06-29T17:19:28.868814Z[0m [32m INFO[0m [2mredb::db[0m[2m:[0m Opening database "FileBackend { lock_supported: true, file: File { fd: 10, path: \"/home/brittonr/.cargo-target/repo-targets/mantle/self-hosting-proof/work/tmp/.tmp9xxeXr/state0/directories.redb\", read: true, write: true } }"
[2m2026-06-29T17:19:28.869130Z[0m [32m INFO[0m [2mredb::db[0m[2m:[0m Found valid allocator state, full repair not needed
[2m2026-06-29T17:19:28.889345Z[0m [32m INFO[0m [2mredb::db[0m[2m:[0m Opening database "FileBackend { lock_supported: true, file: File { fd: 11, path: \"/home/brittonr/.cargo-target/repo-targets/mantle/self-hosting-proof/work/tmp/.tmp9xxeXr/state0/pathinfo.redb\", read: true, write: true } }"
[2m2026-06-29T17:19:28.889526Z[0m [32m INFO[0m [2mredb::db[0m[2m:[0m Found valid allocator state, full repair not needed
[2m2026-06-29T17:19:28.891834Z[0m [32m INFO[0m [2mcrunch_store::handle[0m[2m:[0m PathInfo database opened [3mpath[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/state0/pathinfo.redb
[2m2026-06-29T17:19:28.892279Z[0m [32m INFO[0m [2mcrunch_build::worker[0m[2m:[0m worker streaming started [3mjobs[0m[2m=[0m4
[2m2026-06-29T17:19:29.029391Z[0m [32m INFO[0m [2mcrunch_pipeline[0m[2m:[0m converted, sending to worker [3mdrv[0m[2m=[0mb128giswclx382g61b2pqw477z3dc6f6-busybox.drv [3mlabel[0m[2m=[0mbusybox [3mentries[0m[2m=[0m17
[2m2026-06-29T17:19:29.030675Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m building [3mdrv[0m[2m=[0mbusybox-src.drv
[2m2026-06-29T17:19:29.041014Z[0m [32m INFO[0m [2mcrunch_build::fetch_build_service[0m[2m:[0m fetching tarball [3murl[0m[2m=[0mhttps://busybox.net/downloads/busybox-1.37.0.tar.bz2
[2m2026-06-29T17:19:29.044153Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m all outputs cached, skipping build [3mdrv[0m[2m=[0mmpfr-src.drv
[2m2026-06-29T17:19:29.048056Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m all outputs cached, skipping build [3mdrv[0m[2m=[0mmpc-src.drv
[2m2026-06-29T17:19:29.051841Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m all outputs cached, skipping build [3mdrv[0m[2m=[0mgmp-src.drv
[2m2026-06-29T17:19:29.055462Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m all outputs cached, skipping build [3mdrv[0m[2m=[0mgcc-src.drv
[2m2026-06-29T17:19:29.058915Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m all outputs cached, skipping build [3mdrv[0m[2m=[0mdash-src.drv
[2m2026-06-29T17:19:29.062407Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m all outputs cached, skipping build [3mdrv[0m[2m=[0mmusl-gcc-raw.drv
[2m2026-06-29T17:19:29.065938Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m all outputs cached, skipping build [3mdrv[0m[2m=[0mbinutils-src.drv
[2m2026-06-29T17:19:29.069960Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m all outputs cached, skipping build [3mdrv[0m[2m=[0mmusl-src.drv
[2m2026-06-29T17:19:29.073400Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m all outputs cached, skipping build [3mdrv[0m[2m=[0mmake-src.drv
[2m2026-06-29T17:19:29.077369Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m all outputs cached, skipping build [3mdrv[0m[2m=[0mmusl-seed-toolchain.drv
[2m2026-06-29T17:19:29.080961Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m all outputs cached, skipping build [3mdrv[0m[2m=[0mgnumake.drv
[2m2026-06-29T17:19:29.084387Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m all outputs cached, skipping build [3mdrv[0m[2m=[0mdash.drv
[2m2026-06-29T17:19:29.087709Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m all outputs cached, skipping build [3mdrv[0m[2m=[0mbinutils.drv
[2m2026-06-29T17:19:29.091374Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m all outputs cached, skipping build [3mdrv[0m[2m=[0mmusl.drv
[2m2026-06-29T17:19:29.094989Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m all outputs cached, skipping build [3mdrv[0m[2m=[0mgcc.drv
[2m2026-06-29T17:19:32.176863Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m build succeeded [3mdrv[0m[2m=[0mbusybox-src.drv [3moutputs[0m[2m=[0m["/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/bplicq5hxfn60il61qrl52114dsdmx0s-busybox-src"]
[2m2026-06-29T17:19:32.177415Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m building [3mdrv[0m[2m=[0mbusybox.drv
[2m2026-06-29T17:19:32.177735Z[0m [32m INFO[0m [1mdo_build[0m[2m:[0m [2msnix_build::buildservice::bwrap[0m[2m:[0m Starting bwrap build [3mbuild_name[0m[2m=[0m832c3fb7-56ab-4ec6-b187-5437ad5d69de [3msandbox_path[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/crunch-builds/832c3fb7-56ab-4ec6-b187-5437ad5d69de-5E8HqU
[2m2026-06-29T17:19:32.241512Z[0m [32m INFO[0m [1mdo_build[0m[2m:[0m [2msnix_build::buildservice::bwrap[0m[2m:[0m CRUNCH_NO_FUSE set, materializing inputs to disk [3mpath[0m[2m=[0m"/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/crunch-builds/832c3fb7-56ab-4ec6-b187-5437ad5d69de-5E8HqU/host_inputs_dir"
[2m2026-06-29T17:19:45.872638Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m CA output path resolved [3mdrv[0m[2m=[0mbusybox.drv [3moutput[0m[2m=[0mout [3mca_path[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/wp1dyh7nl2ld4yl0ia730cdfm7js9mza-busybox
[2m2026-06-29T17:19:45.900261Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m build succeeded [3mdrv[0m[2m=[0mbusybox.drv [3moutputs[0m[2m=[0m["/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/wp1dyh7nl2ld4yl0ia730cdfm7js9mza-busybox"]
[2m2026-06-29T17:19:45.900372Z[0m [32m INFO[0m [2mcrunch_build::worker[0m[2m:[0m worker streaming finished [3mcompleted[0m[2m=[0m2 [3msucceeded[0m[2m=[0m1 [3mfailed[0m[2m=[0m0 [3mroots[0m[2m=[0m1
--- build log: busybox ---
  HOSTCC  scripts/basic/fixdep
  HOSTCC  scripts/basic/split-include
  HOSTCC  scripts/basic/docproc
  GEN     include/applets.h
  GEN     include/usage.h
  GEN     sysklogd/Kbuild
  GEN     sysklogd/Config.in
  GEN     miscutils/Kbuild
  GEN     miscutils/Config.in
  GEN     libbb/Kbuild
  GEN     libbb/Config.in
  GEN     console-tools/Kbuild
  GEN     console-tools/Config.in
  GEN     e2fsprogs/Kbuild
  GEN     e2fsprogs/Config.in
  GEN     selinux/Kbuild
  GEN     selinux/Config.in
  GEN     klibc-utils/Kbuild
  GEN     klibc-utils/Config.in
  GEN     loginutils/Kbuild
  GEN     loginutils/Config.in
  GEN     libpwdgrp/Kbuild
  GEN     coreutils/Kbuild
  GEN     coreutils/Config.in
  GEN     coreutils/libcoreutils/Kbuild
  GEN     shell/Kbuild
  GEN     shell/Config.in
  GEN     mailutils/Kbuild
  GEN     mailutils/Config.in
  GEN     printutils/Kbuild
  GEN     printutils/Config.in
  GEN     applets/Kbuild
  GEN     util-linux/Kbuild
  GEN     util-linux/Config.in
  GEN     util-linux/volume_id/Kbuild
  GEN     util-linux/volume_id/Config.in
  GEN     scripts/Kbuild
  GEN     archival/Kbuild
  GEN     archival/Config.in
  GEN     archival/libarchive/Kbuild
  GEN     procps/Kbuild
  GEN     procps/Config.in
  GEN     findutils/Kbuild
  GEN     findutils/Config.in
  GEN     editors/Kbuild
  GEN     editors/Config.in
  GEN     modutils/Kbuild
  GEN     modutils/Config.in
  GEN     debianutils/Kbuild
  GEN     debianutils/Config.in
  GEN     networking/Kbuild
  GEN     networking/Config.in
  GEN     networking/libiproute/Kbuild
  GEN     networking/udhcp/Kbuild
  GEN     networking/udhcp/Config.in
  GEN     runit/Kbuild
  GEN     runit/Config.in
  GEN     init/Kbuild
  GEN     init/Config.in
  HOSTCC  scripts/kconfig/conf.o
  HOSTCC  scripts/kconfig/kxgettext.o
  HOSTCC  scripts/kconfig/mconf.o
  SHIPPED scripts/kconfig/zconf.tab.c
  SHIPPED scripts/kconfig/lex.zconf.c
  SHIPPED scripts/kconfig/zconf.hash.c
  HOSTCC  scripts/kconfig/zconf.tab.o
  HOSTLD  scripts/kconfig/conf
scripts/kconfig/conf -d Config.in
*
* Busybox Configuration
*
*
* Settings
*
Enable compatibility for full-blown desktop systems (8kb) (DESKTOP) [Y/n/?] (NEW) y
Provide compatible behavior for rare corner cases (bigger code) (EXTRA_COMPAT) [N/y/?] (NEW) n
Building for Fedora distribution (FEDORA_COMPAT) [N/y/?] (NEW) n
Enable obsolete features removed before SUSv3 (INCLUDE_SUSv2) [Y/n/?] (NEW) y
Support --long-options (LONG_OPTS) [Y/?] (NEW) y
Show applet usage messages (SHOW_USAGE) [Y/n/?] (NEW) y
  Show verbose applet usage messages (FEATURE_VERBOSE_USAGE) [Y/n/?] (NEW) y
  Store applet usage messages in compressed form (FEATURE_COMPRESS_USAGE) [Y/n/?] (NEW) y
Support files > 2 GB (LFS) [Y/n/?] (NEW) y
  Support 64bit wide time types (TIME64) [Y/n/?] (NEW) y
Support PAM (Pluggable Authentication Modules) (PAM) [N/y/?] (NEW) n
Use the devpts filesystem for Unix98 PTYs (FEATURE_DEVPTS) [Y/n/?] (NEW) y
Support utmp file (FEATURE_UTMP) [Y/n/?] (NEW) y
  Support wtmp file (FEATURE_WTMP) [Y/n/?] (NEW) y
Support writing pidfiles (FEATURE_PIDFILE) [Y/n/?] (NEW) y
  Directory for pidfiles (PID_FILE_PATH) [/var/run] (NEW) /var/run
Include busybox applet (BUSYBOX) [Y/n/?] (NEW) y
  Support --show SCRIPT (FEATURE_SHOW_SCRIPT) [Y/n] (NEW) y
  Support --install [-s] to install applet links at runtime (FEATURE_INSTALLER) [Y/n/?] (NEW) y
Don't use /usr (INSTALL_NO_USR) [N/y/?] (NEW) n
Drop SUID state for most applets (FEATURE_SUID) [Y/n/?] (NEW) y
  Enable SUID configuration via /etc/busybox.conf (FEATURE_SUID_CONFIG) [Y/n/?] (NEW) y
    Suppress warning message if /etc/busybox.conf is not readable (FEATURE_SUID_CONFIG_QUIET) [Y/n/?] (NEW) y
exec prefers applets (FEATURE_PREFER_APPLETS) [N/y/?] (NEW) n
Path to busybox executable (BUSYBOX_EXEC_PATH) [/proc/self/exe] (NEW) /proc/self/exe
Support NSA Security Enhanced Linux (SELINUX) [N/y/?] (NEW) n
Clean up all memory before exiting (usually not needed) (FEATURE_CLEAN_UP) [N/y/?] (NEW) n
Support LOG_INFO level syslog messages (FEATURE_SYSLOG_INFO) [Y/n/?] (NEW) y
*
* Build Options
*
Build static binary (no shared libs) (STATIC) [N/y/?] (NEW) n
  Build position independent executable (PIE) [N/y/?] (NEW) n
Force NOMMU build (NOMMU) [N/y/?] (NEW) n
Build shared libbusybox (BUILD_LIBBUSYBOX) [N/y/?] (NEW) n
Cross compiler prefix (CROSS_COMPILER_PREFIX) [] (NEW) 
Path to sysroot (SYSROOT) [] (NEW) 
Additional CFLAGS (EXTRA_CFLAGS) [] (NEW) 
Additional LDFLAGS (EXTRA_LDFLAGS) [] (NEW) 
Additional LDLIBS (EXTRA_LDLIBS) [] (NEW) 
Avoid using GCC-specific code constructs (USE_PORTABLE_CODE) [N/y/?] (NEW) n
Use -mpreferred-stack-boundary=2 on i386 arch (STACK_OPTIMIZATION_386) [Y/n/?] (NEW) y
Use -static-libgcc (STATIC_LIBGCC) [Y/n/?] (NEW) y
*
* Installation Options ("make install" behavior)
*
What kind of applet links to install
> 1. as soft-links (INSTALL_APPLET_SYMLINKS) (NEW)
  2. as hard-links (INSTALL_APPLET_HARDLINKS) (NEW)
  3. as script wrappers (INSTALL_APPLET_SCRIPT_WRAPPERS) (NEW)
  4. not installed (INSTALL_APPLET_DONT) (NEW)
choice[1-4?]: 1
Destination path for 'make install' (PREFIX) [./_install] (NEW) ./_install
*
* Debugging Options
*
Build with debug information (DEBUG) [N/y/?] (NEW) n
Enable runtime sanitizers (ASAN/LSAN/USAN/etc...) (DEBUG_SANITIZE) [N/y/?] (NEW) n
Build unit tests (UNIT_TEST) [N/y/?] (NEW) n
Abort compilation on any warning (WERROR) [N/y/?] (NEW) n
Warn about single parameter bb_xx_msg calls (WARN_SIMPLE_MSG) [N/y/?] (NEW) n
Additional debugging library
> 1. None (NO_DEBUG_LIB) (NEW)
  2. Dmalloc (DMALLOC) (NEW)
  3. Electric-fence (EFENCE) (NEW)
choice[1-3?]: 1
*
* Library Tuning
*
Use the end of BSS page (FEATURE_USE_BSS_TAIL) [N/y/?] (NEW) n
Enable fractional duration arguments (FLOAT_DURATION) [Y/n/?] (NEW) y
Support RTMIN[+n] and RTMAX[-n] signal names (FEATURE_RTMINMAX) [Y/n/?] (NEW) y
  Use the definitions of SIGRTMIN/SIGRTMAX provided by libc (FEATURE_RTMINMAX_USE_LIBC_DEFINITIONS) [Y/n/?] (NEW) y
Buffer allocation policy
> 1. Allocate with Malloc (FEATURE_BUFFERS_USE_MALLOC) (NEW)
  2. Allocate on the Stack (FEATURE_BUFFERS_GO_ON_STACK) (NEW)
  3. Allocate in the .bss section (FEATURE_BUFFERS_GO_IN_BSS) (NEW)
choice[1-3?]: 1
Minimum password length (PASSWORD_MINLEN) [6] (NEW) 6
MD5: Trade bytes for speed (0:fast, 3:slow) (MD5_SMALL) [1] (NEW) 1
SHA1: Trade bytes for speed (0:fast, 3:slow) (SHA1_SMALL) [3] (NEW) 3
SHA1: Use hardware accelerated instructions if possible (SHA1_HWACCEL) [Y/n/?] (NEW) y
SHA256: Use hardware accelerated instructions if possible (SHA256_HWACCEL) [Y/n/?] (NEW) y
SHA3: Trade bytes for speed (0:fast, 1:slow) (SHA3_SMALL) [1] (NEW) 1
Non-POSIX, but safer, copying to special nodes (FEATURE_NON_POSIX_CP) [Y/n/?] (NEW) y
Give more precise messages when copy fails (cp, mv etc) (FEATURE_VERBOSE_CP_MESSAGE) [N/y/?] (NEW) n
Use sendfile system call (FEATURE_USE_SENDFILE) [Y/n/?] (NEW) y
Copy buffer size, in kilobytes (FEATURE_COPYBUF_KB) [4] (NEW) 4
Use clock_gettime(CLOCK_MONOTONIC) syscall (MONOTONIC_SYSCALL) [Y/n/?] (NEW) y
Use ioctl names rather than hex values in error messages (IOCTL_HEX2STR_ERROR) [Y/n/?] (NEW) y
Command line editing (FEATURE_EDITING) [Y/n/?] (NEW) y
  Maximum length of input (FEATURE_EDITING_MAX_LEN) [1024] (NEW) 1024
  vi-style line editing commands (FEATURE_EDITING_VI) [N/y/?] (NEW) n
  History size (FEATURE_EDITING_HISTORY) [255] (NEW) 255
  History saving (FEATURE_EDITING_SAVEHISTORY) [Y/n/?] (NEW) y
    Save history on shell exit, not after every command (FEATURE_EDITING_SAVE_ON_EXIT) [N/y/?] (NEW) n
  Reverse history search (FEATURE_REVERSE_SEARCH) [Y/n/?] (NEW) y
  Tab completion (FEATURE_TAB_COMPLETION) [Y/n] (NEW) y
    Username completion (FEATURE_USERNAME_COMPLETION) [Y/n] (NEW) y
  Fancy shell prompts (FEATURE_EDITING_FANCY_PROMPT) [Y/n/?] (NEW) y
  Enable automatic tracking of window size changes (FEATURE_EDITING_WINCH) [Y/n] (NEW) y
  Query cursor position from terminal (FEATURE_EDITING_ASK_TERMINAL) [N/y/?] (NEW) n
Enable locale support (system needs locale for this to work) (LOCALE_SUPPORT) [N/y/?] (NEW) n
Support Unicode (UNICODE_SUPPORT) [Y/n/?] (NEW) y
  Check $LC_ALL, $LC_CTYPE and $LANG environment variables (FEATURE_CHECK_UNICODE_IN_ENV) [N/y/?] (NEW) n
  Character code to substitute unprintable characters with (SUBST_WCHAR) [63] (NEW) 63
  Range of supported Unicode characters (LAST_SUPPORTED_WCHAR) [767] (NEW) 767
  Allow zero-width Unicode characters on output (UNICODE_COMBINING_WCHARS) [N/y/?] (NEW) n
  Allow wide Unicode characters on output (UNICODE_WIDE_WCHARS) [N/y/?] (NEW) n
  Bidirectional character-aware line input (UNICODE_BIDI_SUPPORT) [N/y/?] (NEW) n
  Make it possible to enter sequences of chars which are not Unicode (UNICODE_PRESERVE_BROKEN) [N/y/?] (NEW) n
Use LOOP_CONFIGURE for losetup and loop mounts
  1. use LOOP_CONFIGURE, needs kernel >= 5.8 (LOOP_CONFIGURE) (NEW)
  2. use LOOP_SET_FD + LOOP_SET_STATUS (NO_LOOP_CONFIGURE) (NEW)
> 3. try LOOP_CONFIGURE, fall back to LOOP_SET_FD + LOOP_SET_STATUS (TRY_LOOP_CONFIGURE) (NEW)
choice[1-3?]: 3
*
* Applets
*
*
* Archival Utilities
*
Make tar, rpm, modprobe etc understand .xz data (FEATURE_SEAMLESS_XZ) [Y/n] (NEW) y
Make tar, rpm, modprobe etc understand .lzma data (FEATURE_SEAMLESS_LZMA) [Y/n] (NEW) y
Make tar, rpm, modprobe etc understand .bz2 data (FEATURE_SEAMLESS_BZ2) [Y/n] (NEW) y
Make tar, rpm, modprobe etc understand .gz data (FEATURE_SEAMLESS_GZ) [Y] (NEW) y
Make tar, rpm, modprobe etc understand .Z data (FEATURE_SEAMLESS_Z) [N/y] (NEW) n
ar (9.5 kb) (AR) [N/y/?] (NEW) n
uncompress (7.1 kb) (UNCOMPRESS) [N/y/?] (NEW) n
gunzip (11 kb) (GUNZIP) [Y/n/?] (NEW) y
zcat (24 kb) (ZCAT) [Y/n/?] (NEW) y
  Enable long options (FEATURE_GUNZIP_LONG_OPTIONS) [Y/n] (NEW) y
bunzip2 (9.1 kb) (BUNZIP2) [Y/n/?] (NEW) y
bzcat (9 kb) (BZCAT) [Y/n/?] (NEW) y
unlzma (7.8 kb) (UNLZMA) [Y/n/?] (NEW) y
lzcat (7.8 kb) (LZCAT) [Y/n/?] (NEW) y
lzma -d (LZMA) [Y/n/?] (NEW) y
unxz (13 kb) (UNXZ) [Y/n/?] (NEW) y
xzcat (13 kb) (XZCAT) [Y/n/?] (NEW) y
xz -d (XZ) [Y/n/?] (NEW) y
bzip2 (16 kb) (BZIP2) [Y/n/?] (NEW) y
  Trade bytes for speed (0:fast, 9:small) (BZIP2_SMALL) [8] (NEW) 8
  Enable decompression (FEATURE_BZIP2_DECOMPRESS) [Y/?] (NEW) y
cpio (15 kb) (CPIO) [Y/n/?] (NEW) y
  Support archive creation (FEATURE_CPIO_O) [Y/n/?] (NEW) y
    Support passthrough mode (FEATURE_CPIO_P) [Y/n/?] (NEW) y
    Support --ignore-devno like GNU cpio (FEATURE_CPIO_IGNORE_DEVNO) [Y/n/?] (NEW) y
    Support --renumber-inodes like GNU cpio (FEATURE_CPIO_RENUMBER_INODES) [Y/n/?] (NEW) y
dpkg (43 kb) (DPKG) [Y/n/?] (NEW) y
dpkg-deb (29 kb) (DPKG_DEB) [Y/n/?] (NEW) y
gzip (17 kb) (GZIP) [Y/n/?] (NEW) y
  Enable long options (FEATURE_GZIP_LONG_OPTIONS) [Y/n] (NEW) y
  Trade memory for speed (0:small,slow - 2:fast,big) (GZIP_FAST) [0] (NEW) 0
  Enable compression levels (FEATURE_GZIP_LEVELS) [N/y/?] (NEW) n
  Enable decompression (FEATURE_GZIP_DECOMPRESS) [Y/?] (NEW) y
lzop (13 kb) (LZOP) [Y/n/?] (NEW) y
unlzop (13 kb) (UNLZOP) [N/y/?] (NEW) n
lzopcat (13 kb) (LZOPCAT) [N/y/?] (NEW) n
  lzop compression levels 7,8,9 (not very useful) (LZOP_COMPR_HIGH) [N/y/?] (NEW) n
rpm (32 kb) (RPM) [Y/n/?] (NEW) y
rpm2cpio (21 kb) (RPM2CPIO) [Y/n/?] (NEW) y
tar (39 kb) (TAR) [Y/n/?] (NEW) y
  Enable long options (FEATURE_TAR_LONG_OPTIONS) [Y/n] (NEW) y
  Enable -c (archive creation) (FEATURE_TAR_CREATE) [Y/n] (NEW) y
  Autodetect compressed tarballs (FEATURE_TAR_AUTODETECT) [Y/n/?] (NEW) y
  Enable -X (exclude from) and -T (include from) options (FEATURE_TAR_FROM) [Y/n/?] (NEW) y
  Support old tar header format (FEATURE_TAR_OLDGNU_COMPATIBILITY) [Y/n/?] (NEW) y
  Enable untarring of tarballs with checksums produced by buggy Sun tar (FEATURE_TAR_OLDSUN_COMPATIBILITY) [Y/n/?] (NEW) y
  Support GNU tar extensions (long filenames) (FEATURE_TAR_GNU_EXTENSIONS) [Y/n] (NEW) y
  Support writing to an external program (--to-command) (FEATURE_TAR_TO_COMMAND) [Y/n/?] (NEW) y
  Enable use of user and group names (FEATURE_TAR_UNAME_GNAME) [Y/n/?] (NEW) y
  Enable -m (do not preserve time) GNU option (FEATURE_TAR_NOPRESERVE_TIME) [Y/n] (NEW) y
unzip (26 kb) (UNZIP) [Y/n/?] (NEW) y
  Read and use Central Directory data (FEATURE_UNZIP_CDF) [Y/n/?] (NEW) y
    Support compression method 12 (bzip2) (FEATURE_UNZIP_BZIP2) [Y/n] (NEW) y
    Support compression method 14 (lzma) (FEATURE_UNZIP_LZMA) [Y/n] (NEW) y
    Support compression method 95 (xz) (FEATURE_UNZIP_XZ) [Y/n] (NEW) y
Optimize lzma for speed (FEATURE_LZMA_FAST) [N/y/?] (NEW) n
*
* Coreutils
*
Support verbose options (usually -v) for various applets (FEATURE_VERBOSE) [Y/n/?] (NEW) y
*
* Common options for date and touch
*
Allow timezone in dates (FEATURE_TIMEZONE) [Y/n/?] (NEW) y
*
* Common options for cp and mv
*
Preserve hard links (FEATURE_PRESERVE_HARDLINKS) [Y/n/?] (NEW) y
*
* Common options for df, du, ls
*
Support human readable output (example 13k, 23M, 235G) (FEATURE_HUMAN_READABLE) [Y/n/?] (NEW) y
basename (3.7 kb) (BASENAME) [Y/n/?] (NEW) y
cat (5.8 kb) (CAT) [Y/n/?] (NEW) y
  Enable -n and -b options (FEATURE_CATN) [Y/n/?] (NEW) y
  cat -v[etA] (FEATURE_CATV) [Y/n/?] (NEW) y
chgrp (7.6 kb) (CHGRP) [Y/n/?] (NEW) y
chmod (5.5 kb) (CHMOD) [Y/n/?] (NEW) y
chown (7.6 kb) (CHOWN) [Y/n/?] (NEW) y
  Enable long options (FEATURE_CHOWN_LONG_OPTIONS) [Y/n] (NEW) y
chroot (4 kb) (CHROOT) [Y/n/?] (NEW) y
cksum (4.3 kb) (CKSUM) [Y/n] (NEW) y
crc32 (4.2 kb) (CRC32) [Y/n] (NEW) y
comm (4.4 kb) (COMM) [Y/n/?] (NEW) y
cp (10 kb) (CP) [Y/n/?] (NEW) y
  Enable long options (FEATURE_CP_LONG_OPTIONS) [Y/n/?] (NEW) y
    Enable --reflink[=auto] (FEATURE_CP_REFLINK) [Y/n] (NEW) y
cut (6.7 kb) (CUT) [Y/n/?] (NEW) y
  cut -F (FEATURE_CUT_REGEX) [Y/n/?] (NEW) y
date (7.2 kb) (DATE) [Y/n/?] (NEW) y
  Enable ISO date format output (-I) (FEATURE_DATE_ISOFMT) [Y/n/?] (NEW) y
  Support %[num]N nanosecond format specifier (FEATURE_DATE_NANO) [N/y/?] (NEW) n
  Support weird 'date MMDDhhmm[[YY]YY][.ss]' format (FEATURE_DATE_COMPAT) [Y/n/?] (NEW) y
dd (8.3 kb) (DD) [Y/n/?] (NEW) y
  Enable signal handling for status reporting (FEATURE_DD_SIGNAL_HANDLING) [Y/n/?] (NEW) y
    Enable the third status line upon signal (FEATURE_DD_THIRD_STATUS_LINE) [Y/n/?] (NEW) y
  Enable ibs, obs, iflag, oflag and conv options (FEATURE_DD_IBS_OBS) [Y/n/?] (NEW) y
  Enable status display options (FEATURE_DD_STATUS) [Y/n/?] (NEW) y
df (7.1 kb) (DF) [Y/n/?] (NEW) y
  Enable -a, -i, -B (FEATURE_DF_FANCY) [Y/n/?] (NEW) y
  Skip rootfs in mount table (FEATURE_SKIP_ROOTFS) [Y/n/?] (NEW) y
dirname (611 bytes) (DIRNAME) [Y/n/?] (NEW) y
dos2unix (5.5 kb) (DOS2UNIX) [Y/n/?] (NEW) y
unix2dos (5.5 kb) (UNIX2DOS) [Y/n/?] (NEW) y
du (6.5 kb) (DU) [Y/n/?] (NEW) y
  Use default blocksize of 1024 bytes (else it's 512 bytes) (FEATURE_DU_DEFAULT_BLOCKSIZE_1K) [Y/n] (NEW) y
echo (2 kb) (ECHO) [Y/n/?] (NEW) y
  Enable -n and -e options (FEATURE_FANCY_ECHO) [Y/n] (NEW) y
env (4.3 kb) (ENV) [Y/n/?] (NEW) y
expand (5.3 kb) (EXPAND) [Y/n/?] (NEW) y
unexpand (5.5 kb) (UNEXPAND) [Y/n/?] (NEW) y
expr (6.8 kb) (EXPR) [Y/n/?] (NEW) y
  Extend Posix numbers support to 64 bit (EXPR_MATH_SUPPORT_64) [Y/n/?] (NEW) y
factor (3.2 kb) (FACTOR) [Y/n/?] (NEW) y
false (314 bytes) (FALSE) [Y/n/?] (NEW) y
fold (4.8 kb) (FOLD) [Y/n/?] (NEW) y
head (4 kb) (HEAD) [Y/n/?] (NEW) y
  Enable -c, -q, and -v (FEATURE_FANCY_HEAD) [Y/n] (NEW) y
hostid (566 bytes) (HOSTID) [Y/n/?] (NEW) y
id (7.1 kb) (ID) [Y/n/?] (NEW) y
groups (6.8 kb) (GROUPS) [Y/n/?] (NEW) y
install (12 kb) (INSTALL) [Y/n/?] (NEW) y
  Enable long options (FEATURE_INSTALL_LONG_OPTIONS) [Y/n] (NEW) y
link (3.5 kb) (LINK) [Y/n/?] (NEW) y
ln (5.1 kb) (LN) [Y/n/?] (NEW) y
logname (1.4 kb) (LOGNAME) [Y/n/?] (NEW) y
ls (14 kb) (LS) [Y/n/?] (NEW) y
  Enable filetyping options (-p and -F) (FEATURE_LS_FILETYPES) [Y/n] (NEW) y
  Enable symlinks dereferencing (-L) (FEATURE_LS_FOLLOWLINKS) [Y/n] (NEW) y
  Enable recursion (-R) (FEATURE_LS_RECURSIVE) [Y/n] (NEW) y
  Enable -w WIDTH and window size autodetection (FEATURE_LS_WIDTH) [Y/n] (NEW) y
  Sort the file names (FEATURE_LS_SORTFILES) [Y/n/?] (NEW) y
  Show file timestamps (FEATURE_LS_TIMESTAMPS) [Y/n/?] (NEW) y
  Show username/groupnames (FEATURE_LS_USERNAME) [Y/n/?] (NEW) y
  Allow use of color to identify file types (FEATURE_LS_COLOR) [Y/n/?] (NEW) y
    Produce colored ls output by default (FEATURE_LS_COLOR_IS_DEFAULT) [Y/n/?] (NEW) y
md5sum (6.7 kb) (MD5SUM) [Y/n/?] (NEW) y
sha1sum (6.7 kb) (SHA1SUM) [Y/n/?] (NEW) y
sha256sum (8.2 kb) (SHA256SUM) [Y/n/?] (NEW) y
sha512sum (7.3 kb) (SHA512SUM) [Y/n/?] (NEW) y
sha3sum (6.3 kb) (SHA3SUM) [Y/n/?] (NEW) y
  *
  * Common options for md5sum, sha1sum, sha256sum, sha512sum, sha3sum
  *
  Enable -c, -s and -w options (FEATURE_MD5_SHA1_SUM_CHECK) [Y/n/?] (NEW) y
mkdir (4.7 kb) (MKDIR) [Y/n/?] (NEW) y
mkfifo (4 kb) (MKFIFO) [Y/n/?] (NEW) y
mknod (4.6 kb) (MKNOD) [Y/n/?] (NEW) y
mktemp (4.5 kb) (MKTEMP) [Y/n/?] (NEW) y
mv (10 kb) (MV) [Y/n/?] (NEW) y
nice (2.3 kb) (NICE) [Y/n/?] (NEW) y
nl (4.9 kb) (NL) [Y/n/?] (NEW) y
nohup (2.2 kb) (NOHUP) [Y/n/?] (NEW) y
nproc (3.9 kb) (NPROC) [Y/n/?] (NEW) y
od (11 kb) (OD) [Y/n/?] (NEW) y
paste (5.1 kb) (PASTE) [Y/n/?] (NEW) y
printenv (1.6 kb) (PRINTENV) [Y/n/?] (NEW) y
printf (4.1 kb) (PRINTF) [Y/n/?] (NEW) y
pwd (4 kb) (PWD) [Y/n/?] (NEW) y
readlink (4.8 kb) (READLINK) [Y/n/?] (NEW) y
  Enable canonicalization by following all symlinks (-f) (FEATURE_READLINK_FOLLOW) [Y/n/?] (NEW) y
realpath (2.5 kb) (REALPATH) [Y/n/?] (NEW) y
rm (5.5 kb) (RM) [Y/n/?] (NEW) y
rmdir (3.8 kb) (RMDIR) [Y/n/?] (NEW) y
seq (4 kb) (SEQ) [Y/n/?] (NEW) y
shred (5.5 kb) (SHRED) [Y/n/?] (NEW) y
shuf (6 kb) (SHUF) [Y/n/?] (NEW) y
sleep (2.4 kb) (SLEEP) [Y/n/?] (NEW) y
  Enable multiple arguments and s/m/h/d suffixes (FEATURE_FANCY_SLEEP) [Y/n/?] (NEW) y
sort (8.1 kb) (SORT) [Y/n/?] (NEW) y
  Full SuSv3 compliant sort (support -ktcbdfioghM) (FEATURE_SORT_BIG) [Y/n/?] (NEW) y
  Use less memory (but might be slower) (FEATURE_SORT_OPTIMIZE_MEMORY) [N/y/?] (NEW) n
split (5.2 kb) (SPLIT) [Y/n/?] (NEW) y
  Fancy extensions (FEATURE_SPLIT_FANCY) [Y/n/?] (NEW) y
stat (11 kb) (STAT) [Y/n/?] (NEW) y
  Enable custom formats (-c) (FEATURE_STAT_FORMAT) [Y/n/?] (NEW) y
  Enable display of filesystem status (-f) (FEATURE_STAT_FILESYSTEM) [Y/n/?] (NEW) y
stty (9.2 kb) (STTY) [Y/n/?] (NEW) y
sum (4.2 kb) (SUM) [Y/n/?] (NEW) y
sync (4 kb) (SYNC) [Y/n/?] (NEW) y
  Enable -d and -f flags (requires syncfs(2) in libc) (FEATURE_SYNC_FANCY) [Y/n/?] (NEW) y
fsync (3.8 kb) (FSYNC) [Y/n/?] (NEW) y
tac (4.1 kb) (TAC) [Y/n/?] (NEW) y
tail (7.2 kb) (TAIL) [Y/n/?] (NEW) y
  Enable -q, -s, -v, and -F options (FEATURE_FANCY_TAIL) [Y/n/?] (NEW) y
tee (4.4 kb) (TEE) [Y/n/?] (NEW) y
  Enable block I/O (larger/faster) instead of byte I/O (FEATURE_TEE_USE_BLOCK_IO) [Y/n/?] (NEW) y
test (4.4 kb) (TEST) [Y/n/?] (NEW) y
test as [ (TEST1) [Y/n/?] (NEW) y
test as [[ (TEST2) [Y/n/?] (NEW) y
  Extend test to 64 bit (FEATURE_TEST_64) [Y/n/?] (NEW) y
timeout (6.5 kb) (TIMEOUT) [Y/n/?] (NEW) y
touch (6.1 kb) (TOUCH) [Y/n/?] (NEW) y
  Add support for SUSV3 features (-a -d -m -t -r) (FEATURE_TOUCH_SUSV3) [Y/n/?] (NEW) y
tr (5.3 kb) (TR) [Y/n/?] (NEW) y
  Enable character classes (such as [:upper:]) (FEATURE_TR_CLASSES) [Y/n/?] (NEW) y
  Enable equivalence classes (FEATURE_TR_EQUIV) [Y/n/?] (NEW) y
true (311 bytes) (TRUE) [Y/n/?] (NEW) y
truncate (4.4 kb) (TRUNCATE) [Y/n/?] (NEW) y
tsort (2.6 kb) (TSORT) [Y/n/?] (NEW) y
tty (3.9 kb) (TTY) [Y/n/?] (NEW) y
uname (4.2 kb) (UNAME) [Y/n/?] (NEW) y
  Operating system name (UNAME_OSNAME) [GNU/Linux] (NEW) GNU/Linux
arch (1.4 kb) (BB_ARCH) [Y/n/?] (NEW) y
uniq (5.1 kb) (UNIQ) [Y/n/?] (NEW) y
unlink (3.5 kb) (UNLINK) [Y/n/?] (NEW) y
usleep (1.6 kb) (USLEEP) [Y/n/?] (NEW) y
uudecode (5.9 kb) (UUDECODE) [Y/n/?] (NEW) y
base32 (5.5 kb) (BASE32) [Y/n/?] (NEW) y
base64 (5.3 kb) (BASE64) [Y/n/?] (NEW) y
uuencode (4.7 kb) (UUENCODE) [Y/n/?] (NEW) y
wc (4.7 kb) (WC) [Y/n/?] (NEW) y
  Support very large counts (FEATURE_WC_LARGE) [Y/n/?] (NEW) y
who (5.6 kb) (WHO) [Y/n/?] (NEW) y
w (5.5 kb) (W) [Y/n/?] (NEW) y
users (3.6 kb) (USERS) [Y/n/?] (NEW) y
whoami (3.5 kb) (WHOAMI) [Y/n/?] (NEW) y
yes (1.5 kb) (YES) [Y/n/?] (NEW) y
*
* Console Utilities
*
chvt (2.2 kb) (CHVT) [Y/n/?] (NEW) y
clear (371 bytes) (CLEAR) [Y/n/?] (NEW) y
deallocvt (2.2 kb) (DEALLOCVT) [Y/n/?] (NEW) y
dumpkmap (1.9 kb) (DUMPKMAP) [Y/n/?] (NEW) y
fgconsole (1.8 kb) (FGCONSOLE) [Y/n/?] (NEW) y
kbd_mode (4.3 kb) (KBD_MODE) [Y/n/?] (NEW) y
loadfont (5.4 kb) (LOADFONT) [Y/n/?] (NEW) y
setfont (24 kb) (SETFONT) [Y/n/?] (NEW) y
  Support reading textual screen maps (FEATURE_SETFONT_TEXTUAL_MAP) [Y/n/?] (NEW) y
  Default directory for console-tools files (DEFAULT_SETFONT_DIR) [] (NEW) 
  *
  * Common options for loadfont and setfont
  *
  Support PSF2 console fonts (FEATURE_LOADFONT_PSF2) [Y/n] (NEW) y
  Support old (raw) console fonts (FEATURE_LOADFONT_RAW) [Y/n] (NEW) y
loadkmap (2.1 kb) (LOADKMAP) [Y/n/?] (NEW) y
openvt (7.4 kb) (OPENVT) [Y/n/?] (NEW) y
reset (676 bytes) (RESET) [Y/n/?] (NEW) y
resize (1.2 kb) (RESIZE) [Y/n/?] (NEW) y
  Print environment variables (FEATURE_RESIZE_PRINT) [Y/n/?] (NEW) y
setconsole (3.8 kb) (SETCONSOLE) [Y/n/?] (NEW) y
  Enable long options (FEATURE_SETCONSOLE_LONG_OPTIONS) [Y/n] (NEW) y
setkeycodes (2.4 kb) (SETKEYCODES) [Y/n/?] (NEW) y
setlogcons (2 kb) (SETLOGCONS) [Y/n/?] (NEW) y
showkey (4.9 kb) (SHOWKEY) [Y/n/?] (NEW) y
*
* Debian Utilities
*
pipe_progress (576 bytes) (PIPE_PROGRESS) [Y/n/?] (NEW) y
run-parts (6.2 kb) (RUN_PARTS) [Y/n/?] (NEW) y
  Enable long options (FEATURE_RUN_PARTS_LONG_OPTIONS) [Y/n] (NEW) y
  Support additional arguments (FEATURE_RUN_PARTS_FANCY) [Y/n/?] (NEW) y
start-stop-daemon (12 kb) (START_STOP_DAEMON) [Y/n/?] (NEW) y
  Enable long options (FEATURE_START_STOP_DAEMON_LONG_OPTIONS) [Y/n] (NEW) y
  Support additional arguments (FEATURE_START_STOP_DAEMON_FANCY) [Y/n/?] (NEW) y
which (4 kb) (WHICH) [Y/n/?] (NEW) y
*
* klibc-utils
*
minips (11 kb) (MINIPS) [N/y/?] (NEW) n
nuke (2.9 kb) (NUKE) [N/y/?] (NEW) n
resume (3.6 kb) (RESUME) [Y/n/?] (NEW) y
run-init (8 kb) (RUN_INIT) [Y/n/?] (NEW) y
*
* Editors
*
awk (24 kb) (AWK) [Y/n/?] (NEW) y
  Enable math functions (requires libm) (FEATURE_AWK_LIBM) [Y/n/?] (NEW) y
  Enable a few GNU extensions (FEATURE_AWK_GNU_EXTENSIONS) [Y/n/?] (NEW) y
cmp (5.3 kb) (CMP) [Y/n/?] (NEW) y
diff (13 kb) (DIFF) [Y/n/?] (NEW) y
  Enable long options (FEATURE_DIFF_LONG_OPTIONS) [Y/n] (NEW) y
  Enable directory support (FEATURE_DIFF_DIR) [Y/n/?] (NEW) y
ed (16 kb) (ED) [Y/n/?] (NEW) y
patch (9.6 kb) (PATCH) [Y/n/?] (NEW) y
sed (12 kb) (SED) [Y/n/?] (NEW) y
vi (26 kb) (VI) [Y/n/?] (NEW) y
  Maximum screen width (FEATURE_VI_MAX_LEN) [4096] (NEW) 4096
  Allow to display 8-bit chars (otherwise shows dots) (FEATURE_VI_8BIT) [N/y/?] (NEW) n
  Enable ":" colon commands (no "ex" mode) (FEATURE_VI_COLON) [Y/n/?] (NEW) y
    Expand "%" and "#" in colon commands (FEATURE_VI_COLON_EXPAND) [Y/n/?] (NEW) y
  Enable yank/put commands and mark cmds (FEATURE_VI_YANKMARK) [Y/n/?] (NEW) y
  Enable search and replace cmds (FEATURE_VI_SEARCH) [Y/n/?] (NEW) y
    Enable regex in search and replace (FEATURE_VI_REGEX_SEARCH) [N/y/?] (NEW) n
  Catch signals (FEATURE_VI_USE_SIGNALS) [Y/n/?] (NEW) y
  Remember previous cmd and "." cmd (FEATURE_VI_DOT_CMD) [Y/n/?] (NEW) y
  Enable -R option and "view" mode (FEATURE_VI_READONLY) [Y/n/?] (NEW) y
  Enable settable options, ai ic showmatch (FEATURE_VI_SETOPTS) [Y/n/?] (NEW) y
  Support :set (FEATURE_VI_SET) [Y/n] (NEW) y
  Handle window resize (FEATURE_VI_WIN_RESIZE) [Y/n/?] (NEW) y
  Use 'tell me cursor position' ESC sequence to measure window (FEATURE_VI_ASK_TERMINAL) [Y/n/?] (NEW) y
  Support undo command "u" (FEATURE_VI_UNDO) [Y/n/?] (NEW) y
    Enable undo operation queuing (FEATURE_VI_UNDO_QUEUE) [Y/n/?] (NEW) y
      Maximum undo character queue size (FEATURE_VI_UNDO_QUEUE_MAX) [256] (NEW) 256
  Enable verbose status reporting (FEATURE_VI_VERBOSE_STATUS) [Y/n/?] (NEW) y
  Allow vi and awk to execute shell commands (FEATURE_ALLOW_EXEC) [Y/n/?] (NEW) y
*
* Finding Utilities
*
find (16 kb) (FIND) [Y/n/?] (NEW) y
  Enable -print0: NUL-terminated output (FEATURE_FIND_PRINT0) [Y/n/?] (NEW) y
  Enable -mtime: modification time matching (FEATURE_FIND_MTIME) [Y/n/?] (NEW) y
    Enable -atime: access time matching (FEATURE_FIND_ATIME) [Y/n/?] (NEW) y
    Enable -ctime: status change timestamp matching (FEATURE_FIND_CTIME) [Y/n/?] (NEW) y
  Enable -mmin: modification time matching by minutes (FEATURE_FIND_MMIN) [Y/n/?] (NEW) y
    Enable -amin: access time matching by minutes (FEATURE_FIND_AMIN) [Y/n/?] (NEW) y
    Enable -cmin: status change timestamp matching by minutes (FEATURE_FIND_CMIN) [Y/n/?] (NEW) y
  Enable -perm: permissions matching (FEATURE_FIND_PERM) [Y/n] (NEW) y
  Enable -type: file type matching (file/dir/link/...) (FEATURE_FIND_TYPE) [Y/n/?] (NEW) y
  Enable -executable: file is executable (FEATURE_FIND_EXECUTABLE) [Y/n] (NEW) y
  Enable -xdev: 'stay in filesystem' (FEATURE_FIND_XDEV) [Y/n] (NEW) y
  Enable -mindepth N and -maxdepth N (FEATURE_FIND_MAXDEPTH) [Y/n] (NEW) y
  Enable -newer: compare file modification times (FEATURE_FIND_NEWER) [Y/n/?] (NEW) y
  Enable -inum: inode number matching (FEATURE_FIND_INUM) [Y/n] (NEW) y
  Enable -samefile: reference file matching (FEATURE_FIND_SAMEFILE) [Y/n/?] (NEW) y
  Enable -exec: execute commands (FEATURE_FIND_EXEC) [Y/n/?] (NEW) y
    Enable -exec ... {} + (FEATURE_FIND_EXEC_PLUS) [Y/n/?] (NEW) y
    Enable -ok: execute confirmed commands (FEATURE_FIND_EXEC_OK) [Y/n/?] (NEW) y
  Enable -user: username/uid matching (FEATURE_FIND_USER) [Y/n] (NEW) y
  Enable -group: group/gid matching (FEATURE_FIND_GROUP) [Y/n] (NEW) y
  Enable the 'not' (!) operator (FEATURE_FIND_NOT) [Y/n/?] (NEW) y
  Enable -depth (FEATURE_FIND_DEPTH) [Y/n/?] (NEW) y
  Enable parens in options (FEATURE_FIND_PAREN) [Y/n/?] (NEW) y
  Enable -size: file size matching (FEATURE_FIND_SIZE) [Y/n] (NEW) y
  Enable -prune: exclude subdirectories (FEATURE_FIND_PRUNE) [Y/n/?] (NEW) y
  Enable -quit: exit (FEATURE_FIND_QUIT) [Y/n/?] (NEW) y
  Enable -delete: delete files/dirs (FEATURE_FIND_DELETE) [Y/n/?] (NEW) y
  Enable -empty: match empty files or directories (FEATURE_FIND_EMPTY) [Y/n/?] (NEW) y
  Enable -path: match pathname with shell pattern (FEATURE_FIND_PATH) [Y/n/?] (NEW) y
  Enable -regex: match pathname with regex (FEATURE_FIND_REGEX) [Y/n/?] (NEW) y
  Enable -links: link count matching (FEATURE_FIND_LINKS) [Y/n/?] (NEW) y
grep (8.9 kb) (GREP) [Y/n/?] (NEW) y
egrep (8 kb) (EGREP) [Y/n/?] (NEW) y
fgrep (8 kb) (FGREP) [Y/n/?] (NEW) y
  Enable before and after context flags (-A, -B and -C) (FEATURE_GREP_CONTEXT) [Y/n/?] (NEW) y
xargs (7.6 kb) (XARGS) [Y/n/?] (NEW) y
  Enable -p: prompt and confirmation (FEATURE_XARGS_SUPPORT_CONFIRMATION) [Y/n/?] (NEW) y
  Enable single and double quotes and backslash (FEATURE_XARGS_SUPPORT_QUOTES) [Y/n/?] (NEW) y
  Enable -x: exit if -s or -n is exceeded (FEATURE_XARGS_SUPPORT_TERMOPT) [Y/n/?] (NEW) y
  Enable -0: NUL-terminated input (FEATURE_XARGS_SUPPORT_ZERO_TERM) [Y/n/?] (NEW) y
  Enable -I STR: string to replace (FEATURE_XARGS_SUPPORT_REPL_STR) [Y/n/?] (NEW) y
  Enable -P N: processes to run in parallel (FEATURE_XARGS_SUPPORT_PARALLEL) [Y/n] (NEW) y
  Enable -a FILE: use FILE instead of stdin (FEATURE_XARGS_SUPPORT_ARGS_FILE) [Y/n] (NEW) y
*
* Init Utilities
*
bootchartd (10 kb) (BOOTCHARTD) [Y/n/?] (NEW) y
  Compatible, bloated header (FEATURE_BOOTCHARTD_BLOATED_HEADER) [Y/n/?] (NEW) y
  Support bootchartd.conf (FEATURE_BOOTCHARTD_CONFIG_FILE) [Y/n/?] (NEW) y
halt (4.3 kb) (HALT) [Y/n/?] (NEW) y
poweroff (4.3 kb) (POWEROFF) [Y/n/?] (NEW) y
reboot (4.3 kb) (REBOOT) [Y/n/?] (NEW) y
  Before signaling init, make sure it is ready for it (FEATURE_WAIT_FOR_INIT) [Y/n/?] (NEW) y
init (10 kb) (INIT) [Y/n/?] (NEW) y
linuxrc: support running init from initrd (not initramfs) (LINUXRC) [Y/n/?] (NEW) y
  Support reading an inittab file (FEATURE_USE_INITTAB) [Y/n/?] (NEW) y
    Support killing processes that have been removed from inittab (FEATURE_KILL_REMOVED) [N/y/?] (NEW) n
  Run commands with leading dash with controlling tty (FEATURE_INIT_SCTTY) [Y/n/?] (NEW) y
  Enable init to write to syslog (FEATURE_INIT_SYSLOG) [Y/n/?] (NEW) y
  Be quiet on boot (no 'init started:' message) (FEATURE_INIT_QUIET) [Y/n] (NEW) y
  Support dumping core for child processes (debugging only) (FEATURE_INIT_COREDUMPS) [N/y/?] (NEW) n
  Initial terminal type (INIT_TERMINAL_TYPE) [linux] (NEW) linux
  Clear init's command line (FEATURE_INIT_MODIFY_CMDLINE) [Y/n/?] (NEW) y
*
* Login/Password Management Utilities
*
Support shadow passwords (FEATURE_SHADOWPASSWDS) [Y/n/?] (NEW) y
Use internal password and group functions rather than system functions (USE_BB_PWD_GRP) [Y/n/?] (NEW) y
  Use internal shadow password functions (USE_BB_SHADOW) [Y/n/?] (NEW) y
Use internal crypt functions (USE_BB_CRYPT) [Y/n/?] (NEW) y
  Enable SHA256/512 crypt functions (USE_BB_CRYPT_SHA) [Y/n/?] (NEW) y
add-shell (3.3 kb) (ADD_SHELL) [Y/n/?] (NEW) y
remove-shell (3.3 kb) (REMOVE_SHELL) [Y/n/?] (NEW) y
addgroup (8.8 kb) (ADDGROUP) [Y/n/?] (NEW) y
  Support adding users to groups (FEATURE_ADDUSER_TO_GROUP) [Y/n/?] (NEW) y
adduser (15 kb) (ADDUSER) [Y/n/?] (NEW) y
  Enable sanity check on user/group names in adduser and addgroup (FEATURE_CHECK_NAMES) [N/y/?] (NEW) n
  Last valid uid or gid for adduser and addgroup (LAST_ID) [60000] (NEW) 60000
  First valid system uid or gid for adduser and addgroup (FIRST_SYSTEM_ID) [100] (NEW) 100
  Last valid system uid or gid for adduser and addgroup (LAST_SYSTEM_ID) [999] (NEW) 999
chpasswd (19 kb) (CHPASSWD) [Y/n/?] (NEW) y
  Default encryption method (passwd -a, cryptpw -m, chpasswd -c ALG) (FEATURE_DEFAULT_PASSWD_ALGO) [des] (NEW) des
cryptpw (15 kb) (CRYPTPW) [Y/n/?] (NEW) y
mkpasswd (16 kb) (MKPASSWD) [Y/n/?] (NEW) y
deluser (9.3 kb) (DELUSER) [Y/n/?] (NEW) y
delgroup (6.6 kb) (DELGROUP) [Y/n/?] (NEW) y
  Support removing users from groups (FEATURE_DEL_USER_FROM_GROUP) [Y/n/?] (NEW) y
getty (11 kb) (GETTY) [Y/n/?] (NEW) y
login (25 kb) (LOGIN) [Y/n/?] (NEW) y
  Run logged in session in a child process (LOGIN_SESSION_AS_CHILD) [N/y/?] (NEW) n
  Support login scripts (LOGIN_SCRIPTS) [Y/n/?] (NEW) y
  Support /etc/nologin (FEATURE_NOLOGIN) [Y/n/?] (NEW) y
  Support /etc/securetty (FEATURE_SECURETTY) [Y/n/?] (NEW) y
passwd (22 kb) (PASSWD) [Y/n/?] (NEW) y
  Check new passwords for weakness (FEATURE_PASSWD_WEAK_CHECK) [Y/n/?] (NEW) y
su (19 kb) (SU) [Y/n/?] (NEW) y
  Log to syslog all attempts to use su (FEATURE_SU_SYSLOG) [Y/n] (NEW) y
  If user's shell is not in /etc/shells, disallow -s PROG (FEATURE_SU_CHECKS_SHELLS) [Y/n] (NEW) y
  Allow blank passwords only on TTYs in /etc/securetty (FEATURE_SU_BLANK_PW_NEEDS_SECURE_TTY) [N/y] (NEW) n
sulogin (18 kb) (SULOGIN) [Y/n/?] (NEW) y
vlock (18 kb) (VLOCK) [Y/n/?] (NEW) y
*
* Linux Ext2 FS Progs
*
chattr (4.1 kb) (CHATTR) [Y/n/?] (NEW) y
fsck (7.6 kb) (FSCK) [Y/n/?] (NEW) y
lsattr (5.7 kb) (LSATTR) [Y/n/?] (NEW) y
tune2fs (4.4 kb) (TUNE2FS) [N/y/?] (NEW) n
*
* Linux Module Utilities
*
Simplified modutils (MODPROBE_SMALL) [Y/n/?] (NEW) y
depmod (27 kb) (DEPMOD) [Y/n/?] (NEW) y
insmod (22 kb) (INSMOD) [Y/n/?] (NEW) y
lsmod (2.1 kb) (LSMOD) [Y/n/?] (NEW) y
modinfo (24 kb) (MODINFO) [Y/n/?] (NEW) y
modprobe (27 kb) (MODPROBE) [Y/n/?] (NEW) y
rmmod (3.5 kb) (RMMOD) [Y/n/?] (NEW) y
*
* Options common to multiple modutils
*
Accept module options on modprobe command line (FEATURE_CMDLINE_MODULE_OPTIONS) [Y/n/?] (NEW) y
Skip loading of already loaded modules (FEATURE_MODPROBE_SMALL_CHECK_ALREADY_LOADED) [Y/n/?] (NEW) y
Default directory containing modules (DEFAULT_MODULES_DIR) [/lib/modules] (NEW) /lib/modules
Default name of modules.dep (DEFAULT_DEPMOD_FILE) [modules.dep] (NEW) modules.dep
*
* Linux System Utilities
*
acpid (9.3 kb) (ACPID) [Y/n/?] (NEW) y
  Accept and ignore redundant options (FEATURE_ACPID_COMPAT) [Y/n/?] (NEW) y
blkdiscard (4.6 kb) (BLKDISCARD) [Y/n/?] (NEW) y
blkid (12 kb) (BLKID) [Y/n/?] (NEW) y
  Print filesystem type (FEATURE_BLKID_TYPE) [Y/n/?] (NEW) y
blockdev (2.6 kb) (BLOCKDEV) [Y/n/?] (NEW) y
cal (6.1 kb) (CAL) [Y/n/?] (NEW) y
chrt (5.1 kb) (CHRT) [Y/n/?] (NEW) y
dmesg (3.9 kb) (DMESG) [Y/n/?] (NEW) y
  Pretty output (FEATURE_DMESG_PRETTY) [Y/n/?] (NEW) y
eject (4.3 kb) (EJECT) [Y/n/?] (NEW) y
  SCSI support (FEATURE_EJECT_SCSI) [Y/n/?] (NEW) y
fallocate (4.3 kb) (FALLOCATE) [Y/n/?] (NEW) y
fatattr (2.2 kb) (FATATTR) [Y/n/?] (NEW) y
fbset (6.2 kb) (FBSET) [Y/n/?] (NEW) y
  Enable extra options (FEATURE_FBSET_FANCY) [Y/n/?] (NEW) y
  Enable readmode support (FEATURE_FBSET_READMODE) [Y/n/?] (NEW) y
fdformat (4.7 kb) (FDFORMAT) [Y/n/?] (NEW) y
fdisk (31 kb) (FDISK) [Y/n/?] (NEW) y
  Write support (FEATURE_FDISK_WRITABLE) [Y/n/?] (NEW) y
    Support AIX disklabels (FEATURE_AIX_LABEL) [N/y/?] (NEW) n
    Support SGI disklabels (FEATURE_SGI_LABEL) [N/y/?] (NEW) n
    Support SUN disklabels (FEATURE_SUN_LABEL) [N/y/?] (NEW) n
    Support BSD disklabels (FEATURE_OSF_LABEL) [N/y/?] (NEW) n
    Support GPT disklabels (FEATURE_GPT_LABEL) [N/y/?] (NEW) n
    Support expert mode (FEATURE_FDISK_ADVANCED) [Y/n/?] (NEW) y
findfs (11 kb) (FINDFS) [Y/n/?] (NEW) y
flock (6.5 kb) (FLOCK) [Y/n/?] (NEW) y
fdflush (1.6 kb) (FDFLUSH) [Y/n/?] (NEW) y
freeramdisk (1.6 kb) (FREERAMDISK) [Y/n/?] (NEW) y
fsck.minix (13 kb) (FSCK_MINIX) [Y/n/?] (NEW) y
fsfreeze (3.7 kb) (FSFREEZE) [Y/n/?] (NEW) y
fstrim (4.6 kb) (FSTRIM) [Y/n/?] (NEW) y
getopt (6 kb) (GETOPT) [Y/n/?] (NEW) y
  Support -l LONGOPTs (FEATURE_GETOPT_LONG) [Y/n/?] (NEW) y
hexdump (8.7 kb) (HEXDUMP) [Y/n/?] (NEW) y
hd (8.3 kb) (HD) [Y/n/?] (NEW) y
xxd (11 kb) (XXD) [Y/n/?] (NEW) y
hwclock (5.9 kb) (HWCLOCK) [Y/n/?] (NEW) y
  Use FHS /var/lib/hwclock/adjtime (FEATURE_HWCLOCK_ADJTIME_FHS) [N/y/?] (NEW) n
ionice (4 kb) (IONICE) [Y/n/?] (NEW) y
ipcrm (3.5 kb) (IPCRM) [Y/n/?] (NEW) y
ipcs (12 kb) (IPCS) [Y/n/?] (NEW) y
last (7.4 kb) (LAST) [Y/n/?] (NEW) y
  Output extra information (FEATURE_LAST_FANCY) [Y/n/?] (NEW) y
losetup (6.2 kb) (LOSETUP) [Y/n/?] (NEW) y
lspci (6.4 kb) (LSPCI) [Y/n/?] (NEW) y
lsusb (4.4 kb) (LSUSB) [Y/n/?] (NEW) y
mdev (20 kb) (MDEV) [Y/n/?] (NEW) y
  Support /etc/mdev.conf (FEATURE_MDEV_CONF) [Y/n/?] (NEW) y
    Support subdirs/symlinks (FEATURE_MDEV_RENAME) [Y/n/?] (NEW) y
      Support regular expressions substitutions when renaming device (FEATURE_MDEV_RENAME_REGEXP) [Y/n/?] (NEW) y
    Support command execution at device addition/removal (FEATURE_MDEV_EXEC) [Y/n/?] (NEW) y
  Support loading of firmware (FEATURE_MDEV_LOAD_FIRMWARE) [Y/n/?] (NEW) y
  Support daemon mode (FEATURE_MDEV_DAEMON) [Y/n/?] (NEW) y
mesg (1.8 kb) (MESG) [Y/n/?] (NEW) y
  Enable writing to tty only by group, not by everybody (FEATURE_MESG_ENABLE_ONLY_GROUP) [Y/n/?] (NEW) y
mke2fs (10 kb) (MKE2FS) [Y/n/?] (NEW) y
mkfs.ext2 (10 kb) (MKFS_EXT2) [Y/n/?] (NEW) y
mkfs.minix (10 kb) (MKFS_MINIX) [Y/n/?] (NEW) y
  Support Minix fs v2 (fsck_minix/mkfs_minix) (FEATURE_MINIX2) [Y/n/?] (NEW) y
mkfs_reiser (MKFS_REISER) [N/y/?] (NEW) n
mkdosfs (7.6 kb) (MKDOSFS) [Y/n/?] (NEW) y
mkfs.vfat (7.6 kb) (MKFS_VFAT) [Y/n/?] (NEW) y
mkswap (6.6 kb) (MKSWAP) [Y/n/?] (NEW) y
  UUID support (FEATURE_MKSWAP_UUID) [Y/n/?] (NEW) y
more (7.2 kb) (MORE) [Y/n/?] (NEW) y
mount (24 kb) (MOUNT) [Y/n/?] (NEW) y
  Support -f (fake mount) (FEATURE_MOUNT_FAKE) [Y/n/?] (NEW) y
  Support -v (verbose) (FEATURE_MOUNT_VERBOSE) [Y/n/?] (NEW) y
  Support mount helpers (FEATURE_MOUNT_HELPERS) [N/y/?] (NEW) n
  Support specifying devices by label or UUID (FEATURE_MOUNT_LABEL) [Y/n/?] (NEW) y
  Support mounting NFS file systems on Linux < 2.6.23 (FEATURE_MOUNT_NFS) [N/y/?] (NEW) n
  Support mounting CIFS/SMB file systems (FEATURE_MOUNT_CIFS) [Y/n/?] (NEW) y
  Support lots of -o flags (FEATURE_MOUNT_FLAGS) [Y/n/?] (NEW) y
  Support /etc/fstab and -a (mount all) (FEATURE_MOUNT_FSTAB) [Y/n/?] (NEW) y
    Support -T <alt_fstab> (FEATURE_MOUNT_OTHERTAB) [Y/n/?] (NEW) y
mountpoint (5.1 kb) (MOUNTPOINT) [Y/n/?] (NEW) y
nologin (NOLOGIN) [Y/n/?] (NEW) y
  Enable dependencies for nologin (NOLOGIN_DEPENDENCIES) [N/y/?] (NEW) n
nsenter (6.8 kb) (NSENTER) [Y/n/?] (NEW) y
pivot_root (1.4 kb) (PIVOT_ROOT) [Y/n/?] (NEW) y
rdate (5.9 kb) (RDATE) [Y/n/?] (NEW) y
rdev (2.1 kb) (RDEV) [Y/n/?] (NEW) y
readprofile (7.5 kb) (READPROFILE) [Y/n/?] (NEW) y
renice (4.4 kb) (RENICE) [Y/n/?] (NEW) y
rev (4.6 kb) (REV) [Y/n/?] (NEW) y
rtcwake (7.5 kb) (RTCWAKE) [Y/n/?] (NEW) y
script (8.8 kb) (SCRIPT) [Y/n/?] (NEW) y
scriptreplay (2.6 kb) (SCRIPTREPLAY) [Y/n/?] (NEW) y
setarch (3.8 kb) (SETARCH) [Y/n/?] (NEW) y
linux32 (3.6 kb) (LINUX32) [Y/n/?] (NEW) y
linux64 (3.5 kb) (LINUX64) [Y/n/?] (NEW) y
setpriv (6.9 kb) (SETPRIV) [Y/n/?] (NEW) y
  Support dumping current privilege state (FEATURE_SETPRIV_DUMP) [Y/n/?] (NEW) y
  Support capabilities (FEATURE_SETPRIV_CAPABILITIES) [Y/n/?] (NEW) y
    Support capability names (FEATURE_SETPRIV_CAPABILITY_NAMES) [Y/n/?] (NEW) y
setsid (3.8 kb) (SETSID) [Y/n/?] (NEW) y
swapon (15 kb) (SWAPON) [Y/n/?] (NEW) y
  Support discard option -d (FEATURE_SWAPON_DISCARD) [Y/n/?] (NEW) y
  Support priority option -p (FEATURE_SWAPON_PRI) [Y/n/?] (NEW) y
swapoff (14 kb) (SWAPOFF) [Y/n] (NEW) y
  Support specifying devices by label or UUID (FEATURE_SWAPONOFF_LABEL) [Y/n/?] (NEW) y
switch_root (5.7 kb) (SWITCH_ROOT) [Y/n/?] (NEW) y
taskset (5.6 kb) (TASKSET) [Y/n/?] (NEW) y
  Fancy output (FEATURE_TASKSET_FANCY) [Y/n/?] (NEW) y
    CPU list support (-c option) (FEATURE_TASKSET_CPULIST) [Y/n/?] (NEW) y
uevent (3.5 kb) (UEVENT) [Y/n/?] (NEW) y
umount (5.1 kb) (UMOUNT) [Y/n/?] (NEW) y
  Support -a (unmount all) (FEATURE_UMOUNT_ALL) [Y/n/?] (NEW) y
unshare (7.3 kb) (UNSHARE) [Y/n/?] (NEW) y
wall (2.9 kb) (WALL) [Y/n/?] (NEW) y
*
* Common options for mount/umount
*
Support loopback mounts (FEATURE_MOUNT_LOOP) [Y/n/?] (NEW) y
  Create new loopback devices if needed (FEATURE_MOUNT_LOOP_CREATE) [Y/n/?] (NEW) y
Support old /etc/mtab file (FEATURE_MTAB_SUPPORT) [N/y/?] (NEW) n
*
* Filesystem/Volume identification
*
bcache filesystem (FEATURE_VOLUMEID_BCACHE) [Y/n] (NEW) y
btrfs filesystem (FEATURE_VOLUMEID_BTRFS) [Y/n] (NEW) y
cramfs filesystem (FEATURE_VOLUMEID_CRAMFS) [Y/n] (NEW) y
erofs filesystem (FEATURE_VOLUMEID_EROFS) [Y/n/?] (NEW) y
exFAT filesystem (FEATURE_VOLUMEID_EXFAT) [Y/n/?] (NEW) y
Ext filesystem (FEATURE_VOLUMEID_EXT) [Y/n] (NEW) y
f2fs filesystem (FEATURE_VOLUMEID_F2FS) [Y/n/?] (NEW) y
fat filesystem (FEATURE_VOLUMEID_FAT) [Y/n] (NEW) y
hfs filesystem (FEATURE_VOLUMEID_HFS) [Y/n] (NEW) y
iso9660 filesystem (FEATURE_VOLUMEID_ISO9660) [Y/n] (NEW) y
jfs filesystem (FEATURE_VOLUMEID_JFS) [Y/n] (NEW) y
LittleFS filesystem (FEATURE_VOLUMEID_LFS) [Y/n/?] (NEW) y
linuxraid (FEATURE_VOLUMEID_LINUXRAID) [Y/n] (NEW) y
linux swap filesystem (FEATURE_VOLUMEID_LINUXSWAP) [Y/n] (NEW) y
luks filesystem (FEATURE_VOLUMEID_LUKS) [Y/n] (NEW) y
minix filesystem (FEATURE_VOLUMEID_MINIX) [Y/n] (NEW) y
nilfs filesystem (FEATURE_VOLUMEID_NILFS) [Y/n/?] (NEW) y
ntfs filesystem (FEATURE_VOLUMEID_NTFS) [Y/n] (NEW) y
ocfs2 filesystem (FEATURE_VOLUMEID_OCFS2) [Y/n] (NEW) y
Reiser filesystem (FEATURE_VOLUMEID_REISERFS) [Y/n] (NEW) y
romfs filesystem (FEATURE_VOLUMEID_ROMFS) [Y/n] (NEW) y
SquashFS filesystem (FEATURE_VOLUMEID_SQUASHFS) [Y/n/?] (NEW) y
sysv filesystem (FEATURE_VOLUMEID_SYSV) [Y/n] (NEW) y
UBIFS filesystem (FEATURE_VOLUMEID_UBIFS) [Y/n/?] (NEW) y
udf filesystem (FEATURE_VOLUMEID_UDF) [Y/n] (NEW) y
xfs filesystem (FEATURE_VOLUMEID_XFS) [Y/n] (NEW) y
*
* Miscellaneous Utilities
*
adjtimex (4.9 kb) (ADJTIMEX) [Y/n/?] (NEW) y
ascii (784 bytes) (ASCII) [Y/n/?] (NEW) y
bbconfig (9.7 kb) (BBCONFIG) [N/y/?] (NEW) n
bc (38 kb) (BC) [Y/n/?] (NEW) y
dc (29 kb) (DC) [Y/n/?] (NEW) y
  Use bc code base for dc (larger, more features) (FEATURE_DC_BIG) [Y] (NEW) y
    Interactive mode (+4kb) (FEATURE_BC_INTERACTIVE) [Y/n/?] (NEW) y
    Enable bc/dc long options (FEATURE_BC_LONG_OPTIONS) [Y/n] (NEW) y
beep (2.7 kb) (BEEP) [Y/n/?] (NEW) y
  default frequency (FEATURE_BEEP_FREQ) [4000] (NEW) 4000
  default length (FEATURE_BEEP_LENGTH_MS) [30] (NEW) 30
chat (6.7 kb) (CHAT) [Y/n/?] (NEW) y
  Enable NOFAIL expect strings (FEATURE_CHAT_NOFAIL) [Y/n/?] (NEW) y
  Force STDIN to be a TTY (FEATURE_CHAT_TTY_HIFI) [N/y/?] (NEW) n
  Enable implicit Carriage Return (FEATURE_CHAT_IMPLICIT_CR) [Y/n/?] (NEW) y
  Swallow options (FEATURE_CHAT_SWALLOW_OPTS) [Y/n/?] (NEW) y
  Support weird SEND escapes (FEATURE_CHAT_SEND_ESCAPES) [Y/n/?] (NEW) y
  Support variable-length ABORT conditions (FEATURE_CHAT_VAR_ABORT_LEN) [Y/n/?] (NEW) y
  Support revoking of ABORT conditions (FEATURE_CHAT_CLR_ABORT) [Y/n/?] (NEW) y
conspy (10 kb) (CONSPY) [Y/n/?] (NEW) y
crond (15 kb) (CROND) [Y/n/?] (NEW) y
  Support -d (redirect output to stderr) (FEATURE_CROND_D) [Y/n/?] (NEW) y
  Report command output via email (using sendmail) (FEATURE_CROND_CALL_SENDMAIL) [Y/n/?] (NEW) y
  Support special times (@reboot, @daily, etc) in crontabs (FEATURE_CROND_SPECIAL_TIMES) [Y/n/?] (NEW) y
  crond spool directory (FEATURE_CROND_DIR) [/var/spool/cron] (NEW) /var/spool/cron
crontab (10 kb) (CRONTAB) [Y/n/?] (NEW) y
devfsd (obsolete) (DEVFSD) [N/y/?] (NEW) n
Use devfs names for all devices (obsolete) (FEATURE_DEVFS) [N/y/?] (NEW) n
devmem (2.7 kb) (DEVMEM) [Y/n/?] (NEW) y
fbsplash (26 kb) (FBSPLASH) [Y/n/?] (NEW) y
flash_eraseall (5.9 kb) (FLASH_ERASEALL) [N/y/?] (NEW) n
flash_lock (2.1 kb) (FLASH_LOCK) [N/y/?] (NEW) n
flash_unlock (1.3 kb) (FLASH_UNLOCK) [N/y/?] (NEW) n
flashcp (5.3 kb) (FLASHCP) [N/y/?] (NEW) n
getfattr (12.3 kb) (GETFATTR) [Y/n/?] (NEW) y
hdparm (25 kb) (HDPARM) [Y/n/?] (NEW) y
  Support obtaining detailed information directly from drives (FEATURE_HDPARM_GET_IDENTITY) [Y/n/?] (NEW) y
  Register an IDE interface (DANGEROUS) (FEATURE_HDPARM_HDIO_SCAN_HWIF) [Y/n/?] (NEW) y
  Un-register an IDE interface (DANGEROUS) (FEATURE_HDPARM_HDIO_UNREGISTER_HWIF) [Y/n/?] (NEW) y
  Perform device reset (DANGEROUS) (FEATURE_HDPARM_HDIO_DRIVE_RESET) [Y/n/?] (NEW) y
  Tristate device for hotswap (DANGEROUS) (FEATURE_HDPARM_HDIO_TRISTATE_HWIF) [Y/n/?] (NEW) y
  Get/set using_dma flag (FEATURE_HDPARM_HDIO_GETSET_DMA) [Y/n/?] (NEW) y
hexedit (15 kb) (HEXEDIT) [Y/n/?] (NEW) y
i2cget (5.7 kb) (I2CGET) [Y/n/?] (NEW) y
i2cset (6.9 kb) (I2CSET) [Y/n/?] (NEW) y
i2cdump (7.2 kb) (I2CDUMP) [Y/n/?] (NEW) y
i2cdetect (7.3 kb) (I2CDETECT) [Y/n/?] (NEW) y
i2ctransfer (5.5 kb) (I2CTRANSFER) [Y/n/?] (NEW) y
inotifyd (3.6 kb) (INOTIFYD) [N/y/?] (NEW) n
less (16 kb) (LESS) [Y/n/?] (NEW) y
  Max number of input lines less will try to eat (FEATURE_LESS_MAXLINES) [9999999] (NEW) 9999999
  Enable bracket searching (FEATURE_LESS_BRACKETS) [Y/n/?] (NEW) y
  Enable -m/-M (FEATURE_LESS_FLAGS) [Y/n/?] (NEW) y
  Enable -S (FEATURE_LESS_TRUNCATE) [Y/n/?] (NEW) y
  Enable marks (FEATURE_LESS_MARKS) [Y/n/?] (NEW) y
  Enable regular expressions (FEATURE_LESS_REGEXP) [Y/n/?] (NEW) y
  Enable automatic resizing on window size changes (FEATURE_LESS_WINCH) [Y/n/?] (NEW) y
    Use 'tell me cursor position' ESC sequence to measure window (FEATURE_LESS_ASK_TERMINAL) [Y/n/?] (NEW) y
  Enable flag changes ('-' command) (FEATURE_LESS_DASHCMD) [Y/n/?] (NEW) y
    Enable -N (dynamic switching of line numbers) (FEATURE_LESS_LINENUMS) [Y/n] (NEW) y
    Enable -R ('raw control characters') (FEATURE_LESS_RAW) [Y/n/?] (NEW) y
    Take options from $LESS environment variable (FEATURE_LESS_ENV) [Y/n/?] (NEW) y
lsscsi (2.9 kb) (LSSCSI) [Y/n/?] (NEW) y
makedevs (9.4 kb) (MAKEDEVS) [Y/n/?] (NEW) y
  Choose makedevs behaviour
    1. leaf (FEATURE_MAKEDEVS_LEAF) (NEW)
  > 2. table (FEATURE_MAKEDEVS_TABLE) (NEW)
  choice[1-2]: 2
man (26 kb) (MAN) [Y/n/?] (NEW) y
microcom (5.9 kb) (MICROCOM) [Y/n/?] (NEW) y
mim (0.5 kb) (MIM) [Y/n/?] (NEW) y
mt (2.7 kb) (MT) [Y/n/?] (NEW) y
nandwrite (5 kb) (NANDWRITE) [Y/n/?] (NEW) y
nanddump (5.4 kb) (NANDDUMP) [Y/n/?] (NEW) y
partprobe (3.7 kb) (PARTPROBE) [Y/n/?] (NEW) y
raidautorun (1.6 kb) (RAIDAUTORUN) [Y/n/?] (NEW) y
readahead (1.7 kb) (READAHEAD) [Y/n/?] (NEW) y
rfkill (4.4 kb) (RFKILL) [N/y/?] (NEW) n
runlevel (837 bytes) (RUNLEVEL) [Y/n/?] (NEW) y
rx (3.2 kb) (RX) [Y/n/?] (NEW) y
seedrng (9.1 kb) (SEEDRNG) [Y/n/?] (NEW) y
setfattr (3.9 kb) (SETFATTR) [Y/n/?] (NEW) y
setserial (7.1 kb) (SETSERIAL) [Y/n/?] (NEW) y
strings (4.8 kb) (STRINGS) [Y/n/?] (NEW) y
time (8.1 kb) (TIME) [Y/n/?] (NEW) y
tree (2.5 kb) (TREE) [Y/n/?] (NEW) y
ts (4.4 kb) (TS) [Y/n] (NEW) y
ttysize (718 bytes) (TTYSIZE) [Y/n/?] (NEW) y
ubiattach (4.5 kb) (UBIATTACH) [Y/n/?] (NEW) y
ubidetach (4.3 kb) (UBIDETACH) [Y/n/?] (NEW) y
ubimkvol (5.5 kb) (UBIMKVOL) [Y/n/?] (NEW) y
ubirmvol (5.1 kb) (UBIRMVOL) [Y/n/?] (NEW) y
ubirsvol (4.4 kb) (UBIRSVOL) [Y/n/?] (NEW) y
ubiupdatevol (5.6 kb) (UBIUPDATEVOL) [Y/n/?] (NEW) y
ubirename (2.7 kb) (UBIRENAME) [Y/n/?] (NEW) y
volname (1.9 kb) (VOLNAME) [Y/n/?] (NEW) y
watchdog (5.7 kb) (WATCHDOG) [Y/n/?] (NEW) y
  Open watchdog device twice, closing it gracefully in between (FEATURE_WATCHDOG_OPEN_TWICE) [N/y/?] (NEW) n
*
* Networking Utilities
*
Enable IPv6 support (FEATURE_IPV6) [Y/n/?] (NEW) y
Enable Unix domain socket support (usually not needed) (FEATURE_UNIX_LOCAL) [N/y/?] (NEW) n
Prefer IPv4 addresses from DNS queries (FEATURE_PREFER_IPV4_ADDRESS) [Y/n/?] (NEW) y
Verbose resolution errors (VERBOSE_RESOLUTION_ERRORS) [N/y/?] (NEW) n
Support /etc/networks (FEATURE_ETC_NETWORKS) [N/y/?] (NEW) n
Consult /etc/services even for well-known ports (FEATURE_ETC_SERVICES) [N/y/?] (NEW) n
Support infiniband HW (FEATURE_HWIB) [Y/n/?] (NEW) y
In TLS code, support ciphers which use deprecated SHA1 (FEATURE_TLS_SHA1) [N/y/?] (NEW) n
arp (10 kb) (ARP) [Y/n/?] (NEW) y
arping (9.1 kb) (ARPING) [Y/n/?] (NEW) y
brctl (9.9 kb) (BRCTL) [Y/n/?] (NEW) y
  Fancy options (FEATURE_BRCTL_FANCY) [Y/n/?] (NEW) y
    Support show (FEATURE_BRCTL_SHOW) [Y/n/?] (NEW) y
dnsd (10 kb) (DNSD) [Y/n/?] (NEW) y
ether-wake (5.2 kb) (ETHER_WAKE) [Y/n/?] (NEW) y
ftpd (30 kb) (FTPD) [Y/n/?] (NEW) y
  Enable -w (upload commands) (FEATURE_FTPD_WRITE) [Y/n/?] (NEW) y
  Enable workaround for RFC-violating clients (FEATURE_FTPD_ACCEPT_BROKEN_LIST) [Y/n/?] (NEW) y
  Enable authentication (FEATURE_FTPD_AUTHENTICATION) [Y/n/?] (NEW) y
ftpget (7.9 kb) (FTPGET) [Y/n/?] (NEW) y
ftpput (7.6 kb) (FTPPUT) [Y/n/?] (NEW) y
  Enable long options in ftpget/ftpput (FEATURE_FTPGETPUT_LONG_OPTIONS) [Y/n] (NEW) y
hostname (5.8 kb) (HOSTNAME) [Y/n/?] (NEW) y
dnsdomainname (3.8 kb) (DNSDOMAINNAME) [Y/n/?] (NEW) y
httpd (32 kb) (HTTPD) [Y/n/?] (NEW) y
  Default port (FEATURE_HTTPD_PORT_DEFAULT) [80] (NEW) 80
  Support 'Ranges:' header (FEATURE_HTTPD_RANGES) [Y/n/?] (NEW) y
  Enable -u <user> option (FEATURE_HTTPD_SETUID) [Y/n/?] (NEW) y
  Enable HTTP authentication (FEATURE_HTTPD_BASIC_AUTH) [Y/n/?] (NEW) y
    Support MD5-encrypted passwords in HTTP authentication (FEATURE_HTTPD_AUTH_MD5) [Y/n/?] (NEW) y
  Support Common Gateway Interface (CGI) (FEATURE_HTTPD_CGI) [Y/n/?] (NEW) y
    Support running scripts through an interpreter (FEATURE_HTTPD_CONFIG_WITH_SCRIPT_INTERPR) [Y/n/?] (NEW) y
    Set REMOTE_PORT environment variable for CGI (FEATURE_HTTPD_SET_REMOTE_PORT_TO_ENV) [Y/n/?] (NEW) y
  Enable -e option (useful for CGIs written as shell scripts) (FEATURE_HTTPD_ENCODE_URL_STR) [Y/n/?] (NEW) y
  Support custom error pages (FEATURE_HTTPD_ERROR_PAGES) [Y/n/?] (NEW) y
  Support reverse proxy (FEATURE_HTTPD_PROXY) [Y/n/?] (NEW) y
  Support GZIP content encoding (FEATURE_HTTPD_GZIP) [Y/n/?] (NEW) y
  Support caching via ETag header (FEATURE_HTTPD_ETAG) [Y/n/?] (NEW) y
  Add Last-Modified header to response (FEATURE_HTTPD_LAST_MODIFIED) [Y/n/?] (NEW) y
  Add Date header to response (FEATURE_HTTPD_DATE) [Y/n/?] (NEW) y
  ACL IP (FEATURE_HTTPD_ACL_IP) [Y/n/?] (NEW) y
ifconfig (12 kb) (IFCONFIG) [Y/n/?] (NEW) y
  Enable status reporting output (+7k) (FEATURE_IFCONFIG_STATUS) [Y/n/?] (NEW) y
  Enable slip-specific options "keepalive" and "outfill" (FEATURE_IFCONFIG_SLIP) [Y/n/?] (NEW) y
  Enable options "mem_start", "io_addr", and "irq" (FEATURE_IFCONFIG_MEMSTART_IOADDR_IRQ) [Y/n/?] (NEW) y
  Enable option "hw" (ether only) (FEATURE_IFCONFIG_HW) [Y/n/?] (NEW) y
  Set the broadcast automatically (FEATURE_IFCONFIG_BROADCAST_PLUS) [Y/n/?] (NEW) y
ifenslave (13 kb) (IFENSLAVE) [Y/n/?] (NEW) y
ifplugd (11 kb) (IFPLUGD) [Y/n/?] (NEW) y
ifup (14 kb) (IFUP) [Y/n/?] (NEW) y
ifdown (13 kb) (IFDOWN) [Y/n/?] (NEW) y
  Absolute path to ifstate file (IFUPDOWN_IFSTATE_PATH) [/var/run/ifstate] (NEW) /var/run/ifstate
  Use ip tool (else ifconfig/route is used) (FEATURE_IFUPDOWN_IP) [Y/n/?] (NEW) y
  Support IPv4 (FEATURE_IFUPDOWN_IPV4) [Y/n/?] (NEW) y
  Support IPv6 (FEATURE_IFUPDOWN_IPV6) [Y/n/?] (NEW) y
  Enable mapping support (FEATURE_IFUPDOWN_MAPPING) [Y/n/?] (NEW) y
  Support external DHCP clients (FEATURE_IFUPDOWN_EXTERNAL_DHCP) [N/y/?] (NEW) n
inetd (18 kb) (INETD) [Y/n/?] (NEW) y
  Support echo service on port 7 (FEATURE_INETD_SUPPORT_BUILTIN_ECHO) [Y/n/?] (NEW) y
  Support discard service on port 8 (FEATURE_INETD_SUPPORT_BUILTIN_DISCARD) [Y/n/?] (NEW) y
  Support time service on port 37 (FEATURE_INETD_SUPPORT_BUILTIN_TIME) [Y/n/?] (NEW) y
  Support daytime service on port 13 (FEATURE_INETD_SUPPORT_BUILTIN_DAYTIME) [Y/n/?] (NEW) y
  Support chargen service on port 19 (FEATURE_INETD_SUPPORT_BUILTIN_CHARGEN) [Y/n/?] (NEW) y
  Support RPC services (FEATURE_INETD_RPC) [N/y/?] (NEW) n
ip (35 kb) (IP) [Y/n/?] (NEW) y
ipaddr (15 kb) (IPADDR) [Y/n/?] (NEW) y
iplink (17 kb) (IPLINK) [Y/n/?] (NEW) y
iproute (15 kb) (IPROUTE) [Y/n/?] (NEW) y
iptunnel (9.8 kb) (IPTUNNEL) [Y/n/?] (NEW) y
iprule (10 kb) (IPRULE) [Y/n/?] (NEW) y
ipneigh (8.6 kb) (IPNEIGH) [Y/n/?] (NEW) y
ip address (FEATURE_IP_ADDRESS) [Y/?] (NEW) y
ip link (FEATURE_IP_LINK) [Y/?] (NEW) y
ip link set type can (FEATURE_IP_LINK_CAN) [Y/n/?] (NEW) y
ip route (FEATURE_IP_ROUTE) [Y/?] (NEW) y
  ip route configuration directory (FEATURE_IP_ROUTE_DIR) [/etc/iproute2] (NEW) /etc/iproute2
ip tunnel (FEATURE_IP_TUNNEL) [Y/?] (NEW) y
ip rule (FEATURE_IP_RULE) [Y/?] (NEW) y
ip neighbor (FEATURE_IP_NEIGH) [Y/?] (NEW) y
Support displaying rarely used link types (FEATURE_IP_RARE_PROTOCOLS) [N/y/?] (NEW) n
ipcalc (4.6 kb) (IPCALC) [Y/n/?] (NEW) y
  Enable long options (FEATURE_IPCALC_LONG_OPTIONS) [Y/n] (NEW) y
  Fancy IPCALC, more options, adds 1 kbyte (FEATURE_IPCALC_FANCY) [Y/n/?] (NEW) y
fakeidentd (9 kb) (FAKEIDENTD) [Y/n/?] (NEW) y
nameif (6.9 kb) (NAMEIF) [Y/n/?] (NEW) y
  Extended nameif (FEATURE_NAMEIF_EXTENDED) [Y/n/?] (NEW) y
nbd-client (6.3 kb) (NBDCLIENT) [Y/n/?] (NEW) y
nc (11 kb) (NC) [Y/n/?] (NEW) y
netcat (11 kb) (NETCAT) [N/y/?] (NEW) n
  Netcat server options (-l) (NC_SERVER) [Y/n/?] (NEW) y
  Netcat extensions (-eiw and -f FILE) (NC_EXTRA) [Y/n/?] (NEW) y
  Netcat 1.10 compatibility (+2.5k) (NC_110_COMPAT) [Y/n/?] (NEW) y
netstat (10 kb) (NETSTAT) [Y/n/?] (NEW) y
  Enable wide output (FEATURE_NETSTAT_WIDE) [Y/n/?] (NEW) y
  Enable PID/Program name output (FEATURE_NETSTAT_PRG) [Y/n/?] (NEW) y
nslookup (10 kb) (NSLOOKUP) [Y/n/?] (NEW) y
  Use internal resolver code instead of libc (FEATURE_NSLOOKUP_BIG) [Y/n] (NEW) y
    Enable long options (FEATURE_NSLOOKUP_LONG_OPTIONS) [Y/n] (NEW) y
ntpd (23 kb) (NTPD) [Y/n/?] (NEW) y
  Make ntpd usable as a NTP server (FEATURE_NTPD_SERVER) [Y/n/?] (NEW) y
  Make ntpd understand /etc/ntp.conf (FEATURE_NTPD_CONF) [Y/n/?] (NEW) y
  Support md5/sha1 message authentication codes (FEATURE_NTP_AUTH) [Y/n] (NEW) y
ping (10 kb) (PING) [Y/n/?] (NEW) y
ping6 (11 kb) (PING6) [Y/n/?] (NEW) y
Enable fancy ping output (FEATURE_FANCY_PING) [Y/n/?] (NEW) y
pscan (6.2 kb) (PSCAN) [Y/n/?] (NEW) y
route (9 kb) (ROUTE) [Y/n/?] (NEW) y
slattach (6.3 kb) (SLATTACH) [Y/n/?] (NEW) y
ssl_client (28 kb) (SSL_CLIENT) [Y/n/?] (NEW) y
tc (8.3 kb) (TC) [Y/n/?] (NEW) y
  Enable ingress (FEATURE_TC_INGRESS) [Y/n] (NEW) y
tcpsvd (14 kb) (TCPSVD) [Y/n/?] (NEW) y
udpsvd (13 kb) (UDPSVD) [Y/n/?] (NEW) y
telnet (8.8 kb) (TELNET) [Y/n/?] (NEW) y
  Pass TERM type to remote host (FEATURE_TELNET_TTYPE) [Y/n/?] (NEW) y
  Pass USER type to remote host (FEATURE_TELNET_AUTOLOGIN) [Y/n/?] (NEW) y
  Enable window size autodetection (FEATURE_TELNET_WIDTH) [Y/n] (NEW) y
telnetd (13 kb) (TELNETD) [Y/n/?] (NEW) y
  Support standalone telnetd (not inetd only) (FEATURE_TELNETD_STANDALONE) [Y/n/?] (NEW) y
    Default port (FEATURE_TELNETD_PORT_DEFAULT) [23] (NEW) 23
    Support -w SEC option (inetd wait mode) (FEATURE_TELNETD_INETD_WAIT) [Y/n/?] (NEW) y
tftp (11 kb) (TFTP) [Y/n/?] (NEW) y
  Enable progress bar (FEATURE_TFTP_PROGRESS_BAR) [Y/n] (NEW) y
  tftp-hpa compat (support -c get/put FILE) (FEATURE_TFTP_HPA_COMPAT) [Y/n] (NEW) y
tftpd (10 kb) (TFTPD) [Y/n/?] (NEW) y
  Enable 'tftp get' and/or tftpd upload code (FEATURE_TFTP_GET) [Y/n/?] (NEW) y
  Enable 'tftp put' and/or tftpd download code (FEATURE_TFTP_PUT) [Y/n/?] (NEW) y
  Enable 'blksize' and 'tsize' protocol options (FEATURE_TFTP_BLOCKSIZE) [Y/n/?] (NEW) y
  Enable debug (TFTP_DEBUG) [N/y/?] (NEW) n
traceroute (11 kb) (TRACEROUTE) [Y/n/?] (NEW) y
traceroute6 (12 kb) (TRACEROUTE6) [Y/n/?] (NEW) y
Enable verbose output (FEATURE_TRACEROUTE_VERBOSE) [Y/n/?] (NEW) y
Enable -I option (use ICMP instead of UDP) (FEATURE_TRACEROUTE_USE_ICMP) [Y/n] (NEW) y
tunctl (6.4 kb) (TUNCTL) [Y/n/?] (NEW) y
  Support owner:group assignment (FEATURE_TUNCTL_UG) [Y/n/?] (NEW) y
vconfig (2.6 kb) (VCONFIG) [Y/n/?] (NEW) y
wget (41 kb) (WGET) [Y/n/?] (NEW) y
  Enable long options (FEATURE_WGET_LONG_OPTIONS) [Y/n] (NEW) y
  Enable progress bar (+2k) (FEATURE_WGET_STATUSBAR) [Y/n] (NEW) y
  Enable FTP protocol (+1k) (FEATURE_WGET_FTP) [Y/n/?] (NEW) y
  Enable HTTP authentication (FEATURE_WGET_AUTHENTICATION) [Y/n/?] (NEW) y
  Enable timeout option -T SEC (FEATURE_WGET_TIMEOUT) [Y/n/?] (NEW) y
  Support HTTPS using internal TLS code (FEATURE_WGET_HTTPS) [Y/n/?] (NEW) y
  Try to connect to HTTPS using openssl (FEATURE_WGET_OPENSSL) [Y/n/?] (NEW) y
whois (6.5 kb) (WHOIS) [Y/n/?] (NEW) y
zcip (8.7 kb) (ZCIP) [Y/n/?] (NEW) y
udhcpd (21 kb) (UDHCPD) [Y/n/?] (NEW) y
  Answer to BOOTP requests as well (FEATURE_UDHCPD_BOOTP) [Y/n/?] (NEW) y
  Select IP address based on client MAC (FEATURE_UDHCPD_BASE_IP_ON_MAC) [N/y/?] (NEW) n
  Rewrite lease file at every new acknowledge (FEATURE_UDHCPD_WRITE_LEASES_EARLY) [Y/n/?] (NEW) y
  Absolute path to lease file (DHCPD_LEASES_FILE) [/var/lib/misc/udhcpd.leases] (NEW) /var/lib/misc/udhcpd.leases
dumpleases (5.3 kb) (DUMPLEASES) [Y/n/?] (NEW) y
dhcprelay (5.5 kb) (DHCPRELAY) [Y/n/?] (NEW) y
udhcpc (24 kb) (UDHCPC) [Y/n/?] (NEW) y
  Verify that the offered address is free, using ARP ping (FEATURE_UDHCPC_ARPING) [Y/n/?] (NEW) y
  Do not pass malformed host and domain names (FEATURE_UDHCPC_SANITIZEOPT) [Y/n/?] (NEW) y
  Absolute path to config script (UDHCPC_DEFAULT_SCRIPT) [/usr/share/udhcpc/default.script] (NEW) /usr/share/udhcpc/default.script
Absolute path to config script for IPv6 (UDHCPC6_DEFAULT_SCRIPT) [/usr/share/udhcpc/default6.script] (NEW) /usr/share/udhcpc/default6.script
udhcpc6 (21 kb) (UDHCPC6) [Y/n/?] (NEW) y
  Support RFC 3646 (DNS server and search list) (FEATURE_UDHCPC6_RFC3646) [Y/n/?] (NEW) y
  Support RFC 4704 (Client FQDN) (FEATURE_UDHCPC6_RFC4704) [Y/n/?] (NEW) y
  Support RFC 4833 (Timezones) (FEATURE_UDHCPC6_RFC4833) [Y/n/?] (NEW) y
  Support RFC 5970 (Network Boot) (FEATURE_UDHCPC6_RFC5970) [Y/n/?] (NEW) y
*
* Common options for DHCP applets
*
Default interface name (UDHCPC_DEFAULT_INTERFACE) [eth0] (NEW) eth0
Enable '-P port' option for udhcpd and udhcpc (FEATURE_UDHCP_PORT) [N/y/?] (NEW) n
Maximum verbosity level (0..9) (UDHCP_DEBUG) [2] (NEW) 2
DHCP options slack buffer size (UDHCPC_SLACK_FOR_BUGGY_SERVERS) [80] (NEW) 80
Support RFC 3397 domain search options (FEATURE_UDHCP_RFC3397) [Y/n/?] (NEW) y
Support 802.1Q VLAN parameters options (FEATURE_UDHCP_8021Q) [Y/n/?] (NEW) y
ifup udhcpc command line options (IFUPDOWN_UDHCPC_CMD_OPTIONS) [-R -n] (NEW) -R -n
*
* Print Utilities
*
lpd (5.7 kb) (LPD) [Y/n/?] (NEW) y
lpr (10 kb) (LPR) [Y/n/?] (NEW) y
lpq (10 kb) (LPQ) [Y/n/?] (NEW) y
*
* Mail Utilities
*
Default charset (FEATURE_MIME_CHARSET) [us-ascii] (NEW) us-ascii
makemime (5.6 kb) (MAKEMIME) [Y/n/?] (NEW) y
popmaildir (11 kb) (POPMAILDIR) [Y/n/?] (NEW) y
  Allow message filters and custom delivery program (FEATURE_POPMAILDIR_DELIVERY) [Y/n/?] (NEW) y
reformime (7.6 kb) (REFORMIME) [Y/n/?] (NEW) y
  Accept and ignore options other than -x and -X (FEATURE_REFORMIME_COMPAT) [Y/n/?] (NEW) y
sendmail (14 kb) (SENDMAIL) [Y/n/?] (NEW) y
*
* Process Utilities
*
Faster /proc scanning code (+100 bytes) (FEATURE_FAST_TOP) [N/y/?] (NEW) n
Support thread display in ps/pstree/top (FEATURE_SHOW_THREADS) [Y/n/?] (NEW) y
free (3.8 kb) (FREE) [Y/n/?] (NEW) y
fuser (7.3 kb) (FUSER) [Y/n/?] (NEW) y
iostat (8 kb) (IOSTAT) [Y/n/?] (NEW) y
kill (3.4 kb) (KILL) [Y/n/?] (NEW) y
killall (5.9 kb) (KILLALL) [Y/n/?] (NEW) y
killall5 (5.6 kb) (KILLALL5) [Y/n/?] (NEW) y
lsof (3.7 kb) (LSOF) [Y/n/?] (NEW) y
mpstat (10 kb) (MPSTAT) [Y/n/?] (NEW) y
nmeter (12 kb) (NMETER) [Y/n/?] (NEW) y
pgrep (6.8 kb) (PGREP) [Y/n/?] (NEW) y
pkill (7.8 kb) (PKILL) [Y/n/?] (NEW) y
pidof (6.5 kb) (PIDOF) [Y/n/?] (NEW) y
  Enable single shot (-s) (FEATURE_PIDOF_SINGLE) [Y/n/?] (NEW) y
  Enable omitting pids (-o PID) (FEATURE_PIDOF_OMIT) [Y/n/?] (NEW) y
pmap (6.2 kb) (PMAP) [Y/n/?] (NEW) y
powertop (9.9 kb) (POWERTOP) [Y/n/?] (NEW) y
  Accept keyboard commands (FEATURE_POWERTOP_INTERACTIVE) [Y/n/?] (NEW) y
ps (12 kb) (PS) [Y/n/?] (NEW) y
  Enable -o time and -o etime specifiers (FEATURE_PS_TIME) [Y/n] (NEW) y
    Support Linux prior to 2.4.0 and non-ELF systems (FEATURE_PS_UNUSUAL_SYSTEMS) [N/y/?] (NEW) n
  Enable -o rgroup, -o ruser, -o nice specifiers (FEATURE_PS_ADDITIONAL_COLUMNS) [Y/n] (NEW) y
pstree (9.4 kb) (PSTREE) [Y/n/?] (NEW) y
pwdx (3.9 kb) (PWDX) [Y/n/?] (NEW) y
smemcap (3 kb) (SMEMCAP) [Y/n/?] (NEW) y
sysctl (7.9 kb) (BB_SYSCTL) [Y/n/?] (NEW) y
top (18 kb) (TOP) [Y/n/?] (NEW) y
  Accept keyboard commands (FEATURE_TOP_INTERACTIVE) [Y/n/?] (NEW) y
  Show CPU per-process usage percentage (FEATURE_TOP_CPU_USAGE_PERCENTAGE) [Y/n/?] (NEW) y
    Show CPU global usage percentage (FEATURE_TOP_CPU_GLOBAL_PERCENTS) [Y/n/?] (NEW) y
      SMP CPU usage display ('c' key) (FEATURE_TOP_SMP_CPU) [Y/n/?] (NEW) y
    Show 1/10th of a percent in CPU/mem statistics (FEATURE_TOP_DECIMALS) [Y/n/?] (NEW) y
  Show CPU process runs on ('j' field) (FEATURE_TOP_SMP_PROCESS) [Y/n/?] (NEW) y
  Topmem command ('s' key) (FEATURE_TOPMEM) [Y/n/?] (NEW) y
uptime (4 kb) (UPTIME) [Y/n/?] (NEW) y
  Show the number of users (FEATURE_UPTIME_UTMP_SUPPORT) [Y/n/?] (NEW) y
watch (5.2 kb) (WATCH) [Y/n/?] (NEW) y
*
* Runit Utilities
*
chpst (9.2 kb) (CHPST) [Y/n/?] (NEW) y
setuidgid (4.2 kb) (SETUIDGID) [Y/n/?] (NEW) y
envuidgid (4.1 kb) (ENVUIDGID) [Y/n/?] (NEW) y
envdir (2.9 kb) (ENVDIR) [Y/n/?] (NEW) y
softlimit (4.7 kb) (SOFTLIMIT) [Y/n/?] (NEW) y
runsv (8.2 kb) (RUNSV) [Y/n/?] (NEW) y
runsvdir (6.6 kb) (RUNSVDIR) [Y/n/?] (NEW) y
  Enable scrolling argument log (FEATURE_RUNSVDIR_LOG) [N/y/?] (NEW) n
sv (8.7 kb) (SV) [Y/n/?] (NEW) y
  Default directory for services (SV_DEFAULT_SERVICE_DIR) [/var/service] (NEW) /var/service
svc (8.7 kb) (SVC) [Y/n/?] (NEW) y
svok (1.8 kb) (SVOK) [Y/n/?] (NEW) y
svlogd (16 kb) (SVLOGD) [Y/n/?] (NEW) y
*
* Shells
*
Choose which shell is aliased to 'sh' name
> 1. ash (SH_IS_ASH) (NEW)
  2. hush (SH_IS_HUSH) (NEW)
  3. none (SH_IS_NONE) (NEW)
choice[1-3?]: 1
Choose which shell is aliased to 'bash' name
  1. ash (BASH_IS_ASH) (NEW)
  2. hush (BASH_IS_HUSH) (NEW)
> 3. none (BASH_IS_NONE) (NEW)
choice[1-3?]: 3
ash (80 kb) (ASH) [Y/n/?] (NEW) y
  Optimize for size instead of speed (ASH_OPTIMIZE_FOR_SIZE) [Y/n] (NEW) y
  Use internal glob() implementation (ASH_INTERNAL_GLOB) [Y/n/?] (NEW) y
  bash-compatible extensions (ASH_BASH_COMPAT) [Y/n] (NEW) y
    'source' and '.' builtins search current directory after $PATH (ASH_BASH_SOURCE_CURDIR) [N/y/?] (NEW) n
    command_not_found_handle hook support (ASH_BASH_NOT_FOUND_HOOK) [Y/n/?] (NEW) y
  Job control (ASH_JOB_CONTROL) [Y/n] (NEW) y
  Alias support (ASH_ALIAS) [Y/n] (NEW) y
  Pseudorandom generator and $RANDOM variable (ASH_RANDOM_SUPPORT) [Y/n/?] (NEW) y
  Expand prompt string (ASH_EXPAND_PRMT) [Y/n/?] (NEW) y
  Idle timeout variable $TMOUT (ASH_IDLE_TIMEOUT) [Y/n/?] (NEW) y
  Check for new mail in interactive shell (ASH_MAIL) [Y/n/?] (NEW) y
  echo builtin (ASH_ECHO) [Y/n] (NEW) y
  printf builtin (ASH_PRINTF) [Y/n] (NEW) y
  test builtin (ASH_TEST) [Y/n] (NEW) y
  help builtin (ASH_HELP) [Y/n] (NEW) y
  getopts builtin (ASH_GETOPTS) [Y/n] (NEW) y
  command builtin (ASH_CMDCMD) [Y/n/?] (NEW) y
cttyhack (2.7 kb) (CTTYHACK) [Y/n/?] (NEW) y
hush (70 kb) (HUSH) [Y/n/?] (NEW) y
Internal shell for embedded script support (SHELL_HUSH) [Y] (NEW) y
  bash-compatible extensions (HUSH_BASH_COMPAT) [Y/n] (NEW) y
    Brace expansion (HUSH_BRACE_EXPANSION) [Y/n/?] (NEW) y
    'source' and '.' builtins search current directory after $PATH (HUSH_BASH_SOURCE_CURDIR) [N/y/?] (NEW) n
  $LINENO variable (bashism) (HUSH_LINENO_VAR) [Y/n] (NEW) y
  Interactive mode (HUSH_INTERACTIVE) [Y/n/?] (NEW) y
    Save command history to .hush_history (HUSH_SAVEHISTORY) [Y/n] (NEW) y
    Job control (HUSH_JOB) [Y/n/?] (NEW) y
  Support command substitution (HUSH_TICK) [Y/n/?] (NEW) y
  Support if/then/elif/else/fi (HUSH_IF) [Y/n] (NEW) y
  Support for, while and until loops (HUSH_LOOPS) [Y/n] (NEW) y
  Support case ... esac statement (HUSH_CASE) [Y/n/?] (NEW) y
  Support funcname() { commands; } syntax (HUSH_FUNCTIONS) [Y/n/?] (NEW) y
    local builtin (HUSH_LOCAL) [Y/n/?] (NEW) y
  Pseudorandom generator and $RANDOM variable (HUSH_RANDOM_SUPPORT) [Y/n/?] (NEW) y
  Support 'hush -x' option and 'set -x' command (HUSH_MODE_X) [Y/n/?] (NEW) y
  echo builtin (HUSH_ECHO) [Y/n] (NEW) y
  printf builtin (HUSH_PRINTF) [Y/n] (NEW) y
  test builtin (HUSH_TEST) [Y/n] (NEW) y
  help builtin (HUSH_HELP) [Y/n] (NEW) y
  export builtin (HUSH_EXPORT) [Y/n] (NEW) y
    Support 'export -n' option (HUSH_EXPORT_N) [Y/n/?] (NEW) y
  readonly builtin (HUSH_READONLY) [Y/n/?] (NEW) y
  kill builtin (supports kill %jobspec) (HUSH_KILL) [Y/n] (NEW) y
  wait builtin (HUSH_WAIT) [Y/n] (NEW) y
  command builtin (HUSH_COMMAND) [Y/n] (NEW) y
  trap builtin (HUSH_TRAP) [Y/n] (NEW) y
  type builtin (HUSH_TYPE) [Y/n] (NEW) y
  times builtin (HUSH_TIMES) [Y/n] (NEW) y
  read builtin (HUSH_READ) [Y/n] (NEW) y
  set builtin (HUSH_SET) [Y/n] (NEW) y
  unset builtin (HUSH_UNSET) [Y/n] (NEW) y
  ulimit builtin (HUSH_ULIMIT) [Y/n] (NEW) y
  umask builtin (HUSH_UMASK) [Y/n] (NEW) y
  getopts builtin (HUSH_GETOPTS) [Y/n] (NEW) y
  memleak builtin (debugging) (HUSH_MEMLEAK) [N/y] (NEW) n
*
* Options common to all shells
*
POSIX math support (FEATURE_SH_MATH) [Y/n/?] (NEW) y
  Extend POSIX math support to 64 bit (FEATURE_SH_MATH_64) [Y/n/?] (NEW) y
  Support BASE#nnnn literals (FEATURE_SH_MATH_BASE) [Y/n] (NEW) y
Hide message on interactive shell startup (FEATURE_SH_EXTRA_QUIET) [Y/n/?] (NEW) y
Standalone shell (FEATURE_SH_STANDALONE) [N/y/?] (NEW) n
Run 'nofork' applets directly (FEATURE_SH_NOFORK) [N/y/?] (NEW) n
read -t N.NNN support (+110 bytes) (FEATURE_SH_READ_FRAC) [Y/n/?] (NEW) y
Use $HISTFILESIZE (FEATURE_SH_HISTFILESIZE) [Y/n/?] (NEW) y
Embed scripts in the binary (FEATURE_SH_EMBEDDED_SCRIPTS) [Y/n/?] (NEW) y
*
* System Logging Utilities
*
klogd (6.2 kb) (KLOGD) [Y/n/?] (NEW) y
  *
  * klogd should not be used together with syslog to kernel printk buffer
  *
  Use the klogctl() interface (FEATURE_KLOGD_KLOGCTL) [Y/n/?] (NEW) y
logger (6.5 kb) (LOGGER) [Y/n/?] (NEW) y
logread (5 kb) (LOGREAD) [Y/n/?] (NEW) y
  Double buffering (FEATURE_LOGREAD_REDUCED_LOCKING) [Y/n/?] (NEW) y
syslogd (14 kb) (SYSLOGD) [Y/n/?] (NEW) y
  Rotate message files (FEATURE_ROTATE_LOGFILE) [Y/n/?] (NEW) y
  Remote Log support (FEATURE_REMOTE_LOG) [Y/n/?] (NEW) y
  Support -D (drop dups) option (FEATURE_SYSLOGD_DUP) [Y/n/?] (NEW) y
  Support syslog.conf (FEATURE_SYSLOGD_CFG) [Y/n/?] (NEW) y
  Include milliseconds in timestamps (FEATURE_SYSLOGD_PRECISE_TIMESTAMPS) [N/y/?] (NEW) n
  Read buffer size in bytes (FEATURE_SYSLOGD_READ_BUFFER_SIZE) [256] (NEW) 256
  Circular Buffer support (FEATURE_IPC_SYSLOG) [Y/n/?] (NEW) y
    Circular buffer size in Kbytes (minimum 4KB) (FEATURE_IPC_SYSLOG_BUFFER_SIZE) [16] (NEW) 16
  Linux kernel printk buffer support (FEATURE_KMSG_SYSLOG) [Y/n/?] (NEW) y
Building busybox with static musl linking...
scripts/kconfig/conf -s Config.in
#
# using defaults found in .config
#
Your linker does not support --sort-section,alignment
Your linker does not support --sort-common
Your linker does not support -Wl,--gc-sections
Trying libraries: crypt m resolv rt
 Library crypt is not needed, excluding it
 Library m is not needed, excluding it
 Library resolv is not needed, excluding it
 Library rt is not needed, excluding it
Final link with: <none>
Verifying busybox...
[
[[
acpid
add-shell
addgroup
busybox-ok
Busybox build complete: /nix/store/3abxry0k8hihbhr83p5bq6y788jvgihq-busybox/bin/busybox

In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
/bin/sh: pod2text: not found
make: [/tmp/build/Makefile.custom:159: docs/BusyBox.txt] Error 127 (ignored)
/bin/sh: pod2man: not found
make: [/tmp/build/Makefile.custom:164: docs/busybox.1] Error 127 (ignored)
/bin/sh: pod2html: not found
make: [/tmp/build/Makefile.custom:174: docs/busybox.net/BusyBox.html] Error 127 (ignored)
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
<built-in>: note: this is the location of the previous definition
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
modutils/modutils.c: In function 'filename2modname':
modutils/modutils.c:118:1: warning: function may return address of local variable [-Wreturn-local-addr]
  118 | }
      | ^
modutils/modutils.c:97:14: note: declared here
   97 |         char local_modname[MODULE_NAME_LEN];
      |              ^~~~~~~~~~~~~
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from networking/brctl.c:71:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/sockios.h:42: warning: "SIOCGSTAMP" redefined
   42 | #define SIOCGSTAMP      SIOCGSTAMP_OLD
      | 
In file included from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/sys/ioctl.h:10,
                 from include/libbb.h:42,
                 from networking/brctl.c:69:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:111: note: this is the location of the previous definition
  111 | #define SIOCGSTAMP      0x8906
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/sockios.h:43: warning: "SIOCGSTAMPNS" redefined
   43 | #define SIOCGSTAMPNS    SIOCGSTAMPNS_OLD
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:112: note: this is the location of the previous definition
  112 | #define SIOCGSTAMPNS    0x8907
      | 
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from libbb/loop.c:11:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/version.h:2: warning: "KERNEL_VERSION" redefined
    2 | #define KERNEL_VERSION(a,b,c) (((a) << 16) + ((b) << 8) + ((c) > 255 ? 255 : (c)))
      | 
In file included from include/libbb.h:13,
                 from libbb/loop.c:10:
include/platform.h:305: note: this is the location of the previous definition
  305 | #define KERNEL_VERSION(a,b,c) (((a) << 16) + ((b) << 8) + (c))
      | 
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from networking/ifenslave.c:143:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/sockios.h:42: warning: "SIOCGSTAMP" redefined
   42 | #define SIOCGSTAMP      SIOCGSTAMP_OLD
      | 
In file included from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/sys/ioctl.h:10,
                 from include/libbb.h:42,
                 from networking/ifenslave.c:137:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:111: note: this is the location of the previous definition
  111 | #define SIOCGSTAMP      0x8906
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/sockios.h:43: warning: "SIOCGSTAMPNS" redefined
   43 | #define SIOCGSTAMPNS    SIOCGSTAMPNS_OLD
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:112: note: this is the location of the previous definition
  112 | #define SIOCGSTAMPNS    0x8907
      | 
In file included from networking/libiproute/iproute.c:17:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/version.h:2: warning: "KERNEL_VERSION" redefined
    2 | #define KERNEL_VERSION(a,b,c) (((a) << 16) + ((b) << 8) + ((c) > 255 ? 255 : (c)))
      | 
In file included from include/libbb.h:13,
                 from networking/libiproute/ip_common.h:5,
                 from networking/libiproute/iproute.c:12:
include/platform.h:305: note: this is the location of the previous definition
  305 | #define KERNEL_VERSION(a,b,c) (((a) << 16) + ((b) << 8) + (c))
      | 
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from networking/libiproute/iprule.c:29:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/version.h:2: warning: "KERNEL_VERSION" redefined
    2 | #define KERNEL_VERSION(a,b,c) (((a) << 16) + ((b) << 8) + ((c) > 255 ? 255 : (c)))
      | 
In file included from include/libbb.h:13,
                 from networking/libiproute/ip_common.h:5,
                 from networking/libiproute/iprule.c:25:
include/platform.h:305: note: this is the location of the previous definition
  305 | #define KERNEL_VERSION(a,b,c) (((a) << 16) + ((b) << 8) + (c))
      | 
In file included from networking/ifplugd.c:65:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/sockios.h:42: warning: "SIOCGSTAMP" redefined
   42 | #define SIOCGSTAMP      SIOCGSTAMP_OLD
      | 
In file included from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/sys/ioctl.h:10,
                 from include/libbb.h:42,
                 from networking/ifplugd.c:44:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:111: note: this is the location of the previous definition
  111 | #define SIOCGSTAMP      0x8906
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/sockios.h:43: warning: "SIOCGSTAMPNS" redefined
   43 | #define SIOCGSTAMPNS    SIOCGSTAMPNS_OLD
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:112: note: this is the location of the previous definition
  112 | #define SIOCGSTAMPNS    0x8907
      | 
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from include/platform.h:157,
                 from include/libbb.h:13,
                 from miscutils/partprobe.c:18:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/limits.h:48: warning: "NGROUPS_MAX" redefined
   48 | #define NGROUPS_MAX 32
      | 
In file included from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/fs.h:13,
                 from miscutils/partprobe.c:17:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/limits.h:7: note: this is the location of the previous definition
    7 | #define NGROUPS_MAX    65536    /* supplemental group IDs are available */
      | 
In file included from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/sys/ioctl.h:10,
                 from include/libbb.h:42:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:1: warning: "_IOC" redefined
    1 | #define _IOC(a,b,c,d) ( ((a)<<30) | ((b)<<8) | (c) | ((d)<<16) )
      | 
In file included from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm/ioctl.h:1,
                 from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/ioctl.h:5,
                 from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/fs.h:14:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:69: note: this is the location of the previous definition
   69 | #define _IOC(dir,type,nr,size) \
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:6: warning: "_IO" redefined
    6 | #define _IO(a,b) _IOC(_IOC_NONE,(a),(b),0)
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:83: note: this is the location of the previous definition
   83 | #define _IO(type,nr)            _IOC(_IOC_NONE,(type),(nr),0)
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:7: warning: "_IOW" redefined
    7 | #define _IOW(a,b,c) _IOC(_IOC_WRITE,(a),(b),sizeof(c))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:85: note: this is the location of the previous definition
   85 | #define _IOW(type,nr,size)      _IOC(_IOC_WRITE,(type),(nr),(_IOC_TYPECHECK(size)))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:8: warning: "_IOR" redefined
    8 | #define _IOR(a,b,c) _IOC(_IOC_READ,(a),(b),sizeof(c))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:84: note: this is the location of the previous definition
   84 | #define _IOR(type,nr,size)      _IOC(_IOC_READ,(type),(nr),(_IOC_TYPECHECK(size)))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:9: warning: "_IOWR" redefined
    9 | #define _IOWR(a,b,c) _IOC(_IOC_READ|_IOC_WRITE,(a),(b),sizeof(c))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:86: note: this is the location of the previous definition
   86 | #define _IOWR(type,nr,size)     _IOC(_IOC_READ|_IOC_WRITE,(type),(nr),(_IOC_TYPECHECK(size)))
      | 
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from networking/nameif.c:78:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/sockios.h:42: warning: "SIOCGSTAMP" redefined
   42 | #define SIOCGSTAMP      SIOCGSTAMP_OLD
      | 
In file included from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/sys/ioctl.h:10,
                 from include/libbb.h:42,
                 from networking/nameif.c:74:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:111: note: this is the location of the previous definition
  111 | #define SIOCGSTAMP      0x8906
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/sockios.h:43: warning: "SIOCGSTAMPNS" redefined
   43 | #define SIOCGSTAMPNS    SIOCGSTAMPNS_OLD
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:112: note: this is the location of the previous definition
  112 | #define SIOCGSTAMPNS    0x8907
      | 
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/fs.h:13,
                 from networking/nbd-client.c:18:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/limits.h:7: warning: "NGROUPS_MAX" redefined
    7 | #define NGROUPS_MAX    65536    /* supplemental group IDs are available */
      | 
In file included from include/platform.h:157,
                 from include/libbb.h:13,
                 from networking/nbd-client.c:16:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/limits.h:48: note: this is the location of the previous definition
   48 | #define NGROUPS_MAX 32
      | 
In file included from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm/ioctl.h:1,
                 from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/ioctl.h:5,
                 from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/fs.h:14:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:69: warning: "_IOC" redefined
   69 | #define _IOC(dir,type,nr,size) \
      | 
In file included from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/sys/ioctl.h:10,
                 from include/libbb.h:42:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:1: note: this is the location of the previous definition
    1 | #define _IOC(a,b,c,d) ( ((a)<<30) | ((b)<<8) | (c) | ((d)<<16) )
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:83: warning: "_IO" redefined
   83 | #define _IO(type,nr)            _IOC(_IOC_NONE,(type),(nr),0)
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:6: note: this is the location of the previous definition
    6 | #define _IO(a,b) _IOC(_IOC_NONE,(a),(b),0)
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:84: warning: "_IOR" redefined
   84 | #define _IOR(type,nr,size)      _IOC(_IOC_READ,(type),(nr),(_IOC_TYPECHECK(size)))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:8: note: this is the location of the previous definition
    8 | #define _IOR(a,b,c) _IOC(_IOC_READ,(a),(b),sizeof(c))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:85: warning: "_IOW" redefined
   85 | #define _IOW(type,nr,size)      _IOC(_IOC_WRITE,(type),(nr),(_IOC_TYPECHECK(size)))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:7: note: this is the location of the previous definition
    7 | #define _IOW(a,b,c) _IOC(_IOC_WRITE,(a),(b),sizeof(c))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:86: warning: "_IOWR" redefined
   86 | #define _IOWR(type,nr,size)     _IOC(_IOC_READ|_IOC_WRITE,(type),(nr),(_IOC_TYPECHECK(size)))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:9: note: this is the location of the previous definition
    9 | #define _IOWR(a,b,c) _IOC(_IOC_READ|_IOC_WRITE,(a),(b),sizeof(c))
      | 
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm/ioctl.h:1,
                 from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/ioctl.h:5,
                 from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/random.h:12,
                 from miscutils/seedrng.c:44:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:69: warning: "_IOC" redefined
   69 | #define _IOC(dir,type,nr,size) \
      | 
In file included from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/sys/ioctl.h:10,
                 from include/libbb.h:42,
                 from miscutils/seedrng.c:42:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:1: note: this is the location of the previous definition
    1 | #define _IOC(a,b,c,d) ( ((a)<<30) | ((b)<<8) | (c) | ((d)<<16) )
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:83: warning: "_IO" redefined
   83 | #define _IO(type,nr)            _IOC(_IOC_NONE,(type),(nr),0)
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:6: note: this is the location of the previous definition
    6 | #define _IO(a,b) _IOC(_IOC_NONE,(a),(b),0)
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:84: warning: "_IOR" redefined
   84 | #define _IOR(type,nr,size)      _IOC(_IOC_READ,(type),(nr),(_IOC_TYPECHECK(size)))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:8: note: this is the location of the previous definition
    8 | #define _IOR(a,b,c) _IOC(_IOC_READ,(a),(b),sizeof(c))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:85: warning: "_IOW" redefined
   85 | #define _IOW(type,nr,size)      _IOC(_IOC_WRITE,(type),(nr),(_IOC_TYPECHECK(size)))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:7: note: this is the location of the previous definition
    7 | #define _IOW(a,b,c) _IOC(_IOC_WRITE,(a),(b),sizeof(c))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:86: warning: "_IOWR" redefined
   86 | #define _IOWR(type,nr,size)     _IOC(_IOC_READ|_IOC_WRITE,(type),(nr),(_IOC_TYPECHECK(size)))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:9: note: this is the location of the previous definition
    9 | #define _IOWR(a,b,c) _IOC(_IOC_READ|_IOC_WRITE,(a),(b),sizeof(c))
      | 
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm/ioctl.h:1,
                 from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/ioctl.h:5,
                 from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/watchdog.h:13,
                 from miscutils/watchdog.c:53:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:69: warning: "_IOC" redefined
   69 | #define _IOC(dir,type,nr,size) \
      | 
In file included from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/sys/ioctl.h:10,
                 from include/libbb.h:42,
                 from miscutils/watchdog.c:51:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:1: note: this is the location of the previous definition
    1 | #define _IOC(a,b,c,d) ( ((a)<<30) | ((b)<<8) | (c) | ((d)<<16) )
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:83: warning: "_IO" redefined
   83 | #define _IO(type,nr)            _IOC(_IOC_NONE,(type),(nr),0)
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:6: note: this is the location of the previous definition
    6 | #define _IO(a,b) _IOC(_IOC_NONE,(a),(b),0)
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:84: warning: "_IOR" redefined
   84 | #define _IOR(type,nr,size)      _IOC(_IOC_READ,(type),(nr),(_IOC_TYPECHECK(size)))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:8: note: this is the location of the previous definition
    8 | #define _IOR(a,b,c) _IOC(_IOC_READ,(a),(b),sizeof(c))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:85: warning: "_IOW" redefined
   85 | #define _IOW(type,nr,size)      _IOC(_IOC_WRITE,(type),(nr),(_IOC_TYPECHECK(size)))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:7: note: this is the location of the previous definition
    7 | #define _IOW(a,b,c) _IOC(_IOC_WRITE,(a),(b),sizeof(c))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:86: warning: "_IOWR" redefined
   86 | #define _IOWR(type,nr,size)     _IOC(_IOC_READ|_IOC_WRITE,(type),(nr),(_IOC_TYPECHECK(size)))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:9: note: this is the location of the previous definition
    9 | #define _IOWR(a,b,c) _IOC(_IOC_READ|_IOC_WRITE,(a),(b),sizeof(c))
      | 
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from networking/zcip.c:61:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/sockios.h:42: warning: "SIOCGSTAMP" redefined
   42 | #define SIOCGSTAMP      SIOCGSTAMP_OLD
      | 
In file included from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/sys/ioctl.h:10,
                 from include/libbb.h:42,
                 from networking/zcip.c:56:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:111: note: this is the location of the previous definition
  111 | #define SIOCGSTAMP      0x8906
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/sockios.h:43: warning: "SIOCGSTAMPNS" redefined
   43 | #define SIOCGSTAMPNS    SIOCGSTAMPNS_OLD
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:112: note: this is the location of the previous definition
  112 | #define SIOCGSTAMPNS    0x8907
      | 
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
<built-in>: note: this is the location of the previous definition
In file included from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/fs.h:13,
                 from util-linux/blockdev.c:48:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/limits.h:7: warning: "NGROUPS_MAX" redefined
    7 | #define NGROUPS_MAX    65536    /* supplemental group IDs are available */
      | 
In file included from include/platform.h:157,
                 from include/libbb.h:13,
                 from util-linux/blockdev.c:47:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/limits.h:48: note: this is the location of the previous definition
   48 | #define NGROUPS_MAX 32
      | 
In file included from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm/ioctl.h:1,
                 from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/ioctl.h:5,
                 from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/fs.h:14:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:69: warning: "_IOC" redefined
   69 | #define _IOC(dir,type,nr,size) \
      | 
In file included from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/sys/ioctl.h:10,
                 from include/libbb.h:42:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:1: note: this is the location of the previous definition
    1 | #define _IOC(a,b,c,d) ( ((a)<<30) | ((b)<<8) | (c) | ((d)<<16) )
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:83: warning: "_IO" redefined
   83 | #define _IO(type,nr)            _IOC(_IOC_NONE,(type),(nr),0)
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:6: note: this is the location of the previous definition
    6 | #define _IO(a,b) _IOC(_IOC_NONE,(a),(b),0)
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:84: warning: "_IOR" redefined
   84 | #define _IOR(type,nr,size)      _IOC(_IOC_READ,(type),(nr),(_IOC_TYPECHECK(size)))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:8: note: this is the location of the previous definition
    8 | #define _IOR(a,b,c) _IOC(_IOC_READ,(a),(b),sizeof(c))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:85: warning: "_IOW" redefined
   85 | #define _IOW(type,nr,size)      _IOC(_IOC_WRITE,(type),(nr),(_IOC_TYPECHECK(size)))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:7: note: this is the location of the previous definition
    7 | #define _IOW(a,b,c) _IOC(_IOC_WRITE,(a),(b),sizeof(c))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:86: warning: "_IOWR" redefined
   86 | #define _IOWR(type,nr,size)     _IOC(_IOC_READ|_IOC_WRITE,(type),(nr),(_IOC_TYPECHECK(size)))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:9: note: this is the location of the previous definition
    9 | #define _IOWR(a,b,c) _IOC(_IOC_READ|_IOC_WRITE,(a),(b),sizeof(c))
      | 
In file included from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/fs.h:13,
                 from util-linux/blkdiscard.c:32:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/limits.h:7: warning: "NGROUPS_MAX" redefined
    7 | #define NGROUPS_MAX    65536    /* supplemental group IDs are available */
      | 
In file included from include/platform.h:157,
                 from include/libbb.h:13,
                 from util-linux/blkdiscard.c:31:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/limits.h:48: note: this is the location of the previous definition
   48 | #define NGROUPS_MAX 32
      | 
In file included from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm/ioctl.h:1,
                 from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/ioctl.h:5,
                 from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/fs.h:14:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:69: warning: "_IOC" redefined
   69 | #define _IOC(dir,type,nr,size) \
      | 
In file included from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/sys/ioctl.h:10,
                 from include/libbb.h:42:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:1: note: this is the location of the previous definition
    1 | #define _IOC(a,b,c,d) ( ((a)<<30) | ((b)<<8) | (c) | ((d)<<16) )
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:83: warning: "_IO" redefined
   83 | #define _IO(type,nr)            _IOC(_IOC_NONE,(type),(nr),0)
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:6: note: this is the location of the previous definition
    6 | #define _IO(a,b) _IOC(_IOC_NONE,(a),(b),0)
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:84: warning: "_IOR" redefined
   84 | #define _IOR(type,nr,size)      _IOC(_IOC_READ,(type),(nr),(_IOC_TYPECHECK(size)))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:8: note: this is the location of the previous definition
    8 | #define _IOR(a,b,c) _IOC(_IOC_READ,(a),(b),sizeof(c))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:85: warning: "_IOW" redefined
   85 | #define _IOW(type,nr,size)      _IOC(_IOC_WRITE,(type),(nr),(_IOC_TYPECHECK(size)))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:7: note: this is the location of the previous definition
    7 | #define _IOW(a,b,c) _IOC(_IOC_WRITE,(a),(b),sizeof(c))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:86: warning: "_IOWR" redefined
   86 | #define _IOWR(type,nr,size)     _IOC(_IOC_READ|_IOC_WRITE,(type),(nr),(_IOC_TYPECHECK(size)))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:9: note: this is the location of the previous definition
    9 | #define _IOWR(a,b,c) _IOC(_IOC_READ|_IOC_WRITE,(a),(b),sizeof(c))
      | 
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/fs.h:13,
                 from util-linux/fsfreeze.c:24:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/limits.h:7: warning: "NGROUPS_MAX" redefined
    7 | #define NGROUPS_MAX    65536    /* supplemental group IDs are available */
      | 
In file included from include/platform.h:157,
                 from include/libbb.h:13,
                 from util-linux/fsfreeze.c:23:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/limits.h:48: note: this is the location of the previous definition
   48 | #define NGROUPS_MAX 32
      | 
In file included from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm/ioctl.h:1,
                 from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/ioctl.h:5,
                 from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/fs.h:14:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:69: warning: "_IOC" redefined
   69 | #define _IOC(dir,type,nr,size) \
      | 
In file included from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/sys/ioctl.h:10,
                 from include/libbb.h:42:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:1: note: this is the location of the previous definition
    1 | #define _IOC(a,b,c,d) ( ((a)<<30) | ((b)<<8) | (c) | ((d)<<16) )
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:83: warning: "_IO" redefined
   83 | #define _IO(type,nr)            _IOC(_IOC_NONE,(type),(nr),0)
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:6: note: this is the location of the previous definition
    6 | #define _IO(a,b) _IOC(_IOC_NONE,(a),(b),0)
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:84: warning: "_IOR" redefined
   84 | #define _IOR(type,nr,size)      _IOC(_IOC_READ,(type),(nr),(_IOC_TYPECHECK(size)))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:8: note: this is the location of the previous definition
    8 | #define _IOR(a,b,c) _IOC(_IOC_READ,(a),(b),sizeof(c))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:85: warning: "_IOW" redefined
   85 | #define _IOW(type,nr,size)      _IOC(_IOC_WRITE,(type),(nr),(_IOC_TYPECHECK(size)))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:7: note: this is the location of the previous definition
    7 | #define _IOW(a,b,c) _IOC(_IOC_WRITE,(a),(b),sizeof(c))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:86: warning: "_IOWR" redefined
   86 | #define _IOWR(type,nr,size)     _IOC(_IOC_READ|_IOC_WRITE,(type),(nr),(_IOC_TYPECHECK(size)))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:9: note: this is the location of the previous definition
    9 | #define _IOWR(a,b,c) _IOC(_IOC_READ|_IOC_WRITE,(a),(b),sizeof(c))
      | 
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/fs.h:13,
                 from util-linux/fstrim.c:29:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/limits.h:7: warning: "NGROUPS_MAX" redefined
    7 | #define NGROUPS_MAX    65536    /* supplemental group IDs are available */
      | 
In file included from include/platform.h:157,
                 from include/libbb.h:13,
                 from util-linux/fstrim.c:28:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/limits.h:48: note: this is the location of the previous definition
   48 | #define NGROUPS_MAX 32
      | 
In file included from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm/ioctl.h:1,
                 from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/ioctl.h:5,
                 from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/fs.h:14:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:69: warning: "_IOC" redefined
   69 | #define _IOC(dir,type,nr,size) \
      | 
In file included from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/sys/ioctl.h:10,
                 from include/libbb.h:42:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:1: note: this is the location of the previous definition
    1 | #define _IOC(a,b,c,d) ( ((a)<<30) | ((b)<<8) | (c) | ((d)<<16) )
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:83: warning: "_IO" redefined
   83 | #define _IO(type,nr)            _IOC(_IOC_NONE,(type),(nr),0)
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:6: note: this is the location of the previous definition
    6 | #define _IO(a,b) _IOC(_IOC_NONE,(a),(b),0)
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:84: warning: "_IOR" redefined
   84 | #define _IOR(type,nr,size)      _IOC(_IOC_READ,(type),(nr),(_IOC_TYPECHECK(size)))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:8: note: this is the location of the previous definition
    8 | #define _IOR(a,b,c) _IOC(_IOC_READ,(a),(b),sizeof(c))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:85: warning: "_IOW" redefined
   85 | #define _IOW(type,nr,size)      _IOC(_IOC_WRITE,(type),(nr),(_IOC_TYPECHECK(size)))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:7: note: this is the location of the previous definition
    7 | #define _IOW(a,b,c) _IOC(_IOC_WRITE,(a),(b),sizeof(c))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:86: warning: "_IOWR" redefined
   86 | #define _IOWR(type,nr,size)     _IOC(_IOC_READ|_IOC_WRITE,(type),(nr),(_IOC_TYPECHECK(size)))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:9: note: this is the location of the previous definition
    9 | #define _IOWR(a,b,c) _IOC(_IOC_READ|_IOC_WRITE,(a),(b),sizeof(c))
      | 
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/wp1dyh7nl2ld4yl0ia730cdfm7js9mza-busybox
this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/fs.h:13,
                 from util-linux/mkfs_ext2.c:69:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/limits.h:7: warning: "NGROUPS_MAX" redefined
    7 | #define NGROUPS_MAX    65536    /* supplemental group IDs are available */
      | 
In file included from include/platform.h:157,
                 from include/libbb.h:13,
                 from util-linux/mkfs_ext2.c:68:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/limits.h:48: note: this is the location of the previous definition
   48 | #define NGROUPS_MAX 32
      | 
In file included from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm/ioctl.h:1,
                 from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/ioctl.h:5,
                 from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/fs.h:14:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:69: warning: "_IOC" redefined
   69 | #define _IOC(dir,type,nr,size) \
      | 
In file included from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/sys/ioctl.h:10,
                 from include/libbb.h:42:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:1: note: this is the location of the previous definition
    1 | #define _IOC(a,b,c,d) ( ((a)<<30) | ((b)<<8) | (c) | ((d)<<16) )
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:83: warning: "_IO" redefined
   83 | #define _IO(type,nr)            _IOC(_IOC_NONE,(type),(nr),0)
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:6: note: this is the location of the previous definition
    6 | #define _IO(a,b) _IOC(_IOC_NONE,(a),(b),0)
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:84: warning: "_IOR" redefined
   84 | #define _IOR(type,nr,size)      _IOC(_IOC_READ,(type),(nr),(_IOC_TYPECHECK(size)))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:8: note: this is the location of the previous definition
    8 | #define _IOR(a,b,c) _IOC(_IOC_READ,(a),(b),sizeof(c))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:85: warning: "_IOW" redefined
   85 | #define _IOW(type,nr,size)      _IOC(_IOC_WRITE,(type),(nr),(_IOC_TYPECHECK(size)))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:7: note: this is the location of the previous definition
    7 | #define _IOW(a,b,c) _IOC(_IOC_WRITE,(a),(b),sizeof(c))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:86: warning: "_IOWR" redefined
   86 | #define _IOWR(type,nr,size)     _IOC(_IOC_READ|_IOC_WRITE,(type),(nr),(_IOC_TYPECHECK(size)))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:9: note: this is the location of the previous definition
    9 | #define _IOWR(a,b,c) _IOC(_IOC_READ|_IOC_WRITE,(a),(b),sizeof(c))
      | 
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm/ioctl.h:1,
                 from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/ioctl.h:5,
                 from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/linux/fd.h:5,
                 from util-linux/mkfs_vfat.c:47:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:69: warning: "_IOC" redefined
   69 | #define _IOC(dir,type,nr,size) \
      | 
In file included from /nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/sys/ioctl.h:10,
                 from include/libbb.h:42,
                 from util-linux/mkfs_vfat.c:44:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:1: note: this is the location of the previous definition
    1 | #define _IOC(a,b,c,d) ( ((a)<<30) | ((b)<<8) | (c) | ((d)<<16) )
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:83: warning: "_IO" redefined
   83 | #define _IO(type,nr)            _IOC(_IOC_NONE,(type),(nr),0)
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:6: note: this is the location of the previous definition
    6 | #define _IO(a,b) _IOC(_IOC_NONE,(a),(b),0)
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:84: warning: "_IOR" redefined
   84 | #define _IOR(type,nr,size)      _IOC(_IOC_READ,(type),(nr),(_IOC_TYPECHECK(size)))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:8: note: this is the location of the previous definition
    8 | #define _IOR(a,b,c) _IOC(_IOC_READ,(a),(b),sizeof(c))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:85: warning: "_IOW" redefined
   85 | #define _IOW(type,nr,size)      _IOC(_IOC_WRITE,(type),(nr),(_IOC_TYPECHECK(size)))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:7: note: this is the location of the previous definition
    7 | #define _IOW(a,b,c) _IOC(_IOC_WRITE,(a),(b),sizeof(c))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/asm-generic/ioctl.h:86: warning: "_IOWR" redefined
   86 | #define _IOWR(type,nr,size)     _IOC(_IOC_READ|_IOC_WRITE,(type),(nr),(_IOC_TYPECHECK(size)))
      | 
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/bits/ioctl.h:9: note: this is the location of the previous definition
    9 | #define _IOWR(a,b,c) _IOC(_IOC_READ|_IOC_WRITE,(a),(b),sizeof(c))
      | 
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
In file included from <command-line>:
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:10: warning: "__STDC_UTF_16__" redefined
   10 | #define __STDC_UTF_16__ 1
      | 
<built-in>: note: this is the location of the previous definition
/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/include/stdc-predef.h:11: warning: "__STDC_UTF_32__" redefined
   11 | #define __STDC_UTF_32__ 1
      | 
<built-in>: note: this is the location of the previous definition
/tmp/build/scripts/trylink: line 50: mktemp: not found
/tmp/build/scripts/trylink: line 50: mktemp: not found
/tmp/build/scripts/trylink: line 61: mktemp: not found
/tmp/build/scripts/trylink: line 50: mktemp: not found

--- end log ---
hermeticity: practical (no degraded facts)
self-build-proof: progress=bootstrap-tool-done:busybox.ncl

[3/4] Building mantle...
self-build-proof: progress=mantle-build-start
[2m2026-06-29T17:19:45.984572Z[0m [32m INFO[0m [2mcrunch_store::handle[0m[2m:[0m blob service opened [3mpath[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/state0/blobs
[2m2026-06-29T17:19:45.984965Z[0m [32m INFO[0m [2mredb::db[0m[2m:[0m Opening database "FileBackend { lock_supported: true, file: File { fd: 10, path: \"/home/brittonr/.cargo-target/repo-targets/mantle/self-hosting-proof/work/tmp/.tmp9xxeXr/state0/directories.redb\", read: true, write: true } }"
[2m2026-06-29T17:19:45.985301Z[0m [32m INFO[0m [2mredb::db[0m[2m:[0m Found valid allocator state, full repair not needed
[2m2026-06-29T17:19:46.002832Z[0m [32m INFO[0m [2mredb::db[0m[2m:[0m Opening database "FileBackend { lock_supported: true, file: File { fd: 11, path: \"/home/brittonr/.cargo-target/repo-targets/mantle/self-hosting-proof/work/tmp/.tmp9xxeXr/state0/pathinfo.redb\", read: true, write: true } }"
[2m2026-06-29T17:19:46.003043Z[0m [32m INFO[0m [2mredb::db[0m[2m:[0m Found valid allocator state, full repair not needed
[2m2026-06-29T17:19:46.004874Z[0m [32m INFO[0m [2mcrunch_store::handle[0m[2m:[0m PathInfo database opened [3mpath[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/state0/pathinfo.redb
[2m2026-06-29T17:19:46.005174Z[0m [32m INFO[0m [2mcrunch_build::worker[0m[2m:[0m worker streaming started [3mjobs[0m[2m=[0m4
[2m2026-06-29T17:19:46.165072Z[0m [32m INFO[0m [2mcrunch_pipeline[0m[2m:[0m converted, sending to worker [3mdrv[0m[2m=[0msy7vv7vcdgcprfasd89hf3m55hdnr084-mantle.drv [3mlabel[0m[2m=[0mmantle [3mentries[0m[2m=[0m18
[2m2026-06-29T17:19:46.170164Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m all outputs cached, skipping build [3mdrv[0m[2m=[0mmpfr-src.drv
[2m2026-06-29T17:19:46.173968Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m all outputs cached, skipping build [3mdrv[0m[2m=[0mmpc-src.drv
[2m2026-06-29T17:19:46.177899Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m all outputs cached, skipping build [3mdrv[0m[2m=[0mgmp-src.drv
[2m2026-06-29T17:19:46.181714Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m all outputs cached, skipping build [3mdrv[0m[2m=[0mgcc-src.drv
[2m2026-06-29T17:19:46.185094Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m all outputs cached, skipping build [3mdrv[0m[2m=[0mdash-src.drv
[2m2026-06-29T17:19:46.188557Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m all outputs cached, skipping build [3mdrv[0m[2m=[0mmusl-gcc-raw.drv
[2m2026-06-29T17:19:46.192079Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m all outputs cached, skipping build [3mdrv[0m[2m=[0mbinutils-src.drv
[2m2026-06-29T17:19:46.195564Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m all outputs cached, skipping build [3mdrv[0m[2m=[0mmusl-src.drv
[2m2026-06-29T17:19:46.199173Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m all outputs cached, skipping build [3mdrv[0m[2m=[0mmake-src.drv
[2m2026-06-29T17:19:46.199454Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m building [3mdrv[0m[2m=[0mrust-standalone.drv
[2m2026-06-29T17:19:46.202819Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m all outputs cached, skipping build [3mdrv[0m[2m=[0mmusl-seed-toolchain.drv
[2m2026-06-29T17:19:46.206225Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m all outputs cached, skipping build [3mdrv[0m[2m=[0mgnumake.drv
[2m2026-06-29T17:19:46.209621Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m all outputs cached, skipping build [3mdrv[0m[2m=[0mdash.drv
[2m2026-06-29T17:19:46.209735Z[0m [32m INFO[0m [2mcrunch_build::fetch_build_service[0m[2m:[0m fetching tarball [3murl[0m[2m=[0mhttps://static.rust-lang.org/dist/rust-1.94.1-x86_64-unknown-linux-musl.tar.xz
[2m2026-06-29T17:19:46.213592Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m all outputs cached, skipping build [3mdrv[0m[2m=[0mbinutils.drv
[2m2026-06-29T17:19:46.217408Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m all outputs cached, skipping build [3mdrv[0m[2m=[0mmusl.drv
[2m2026-06-29T17:19:46.220847Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m all outputs cached, skipping build [3mdrv[0m[2m=[0mgcc.drv
[2m2026-06-29T17:23:59.153789Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m build succeeded [3mdrv[0m[2m=[0mrust-standalone.drv [3moutputs[0m[2m=[0m["/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/jw1z67aamhnc39dl83jms0ikmics4dg8-rust-standalone"]
[2m2026-06-29T17:23:59.154276Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m building [3mdrv[0m[2m=[0mrust.drv
[2m2026-06-29T17:23:59.154639Z[0m [32m INFO[0m [1mdo_build[0m[2m:[0m [2msnix_build::buildservice::bwrap[0m[2m:[0m Starting bwrap build [3mbuild_name[0m[2m=[0me9ec6e39-0476-419d-8905-8aac760f3edf [3msandbox_path[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/crunch-builds/e9ec6e39-0476-419d-8905-8aac760f3edf-HioGx9
[2m2026-06-29T17:23:59.235842Z[0m [32m INFO[0m [1mdo_build[0m[2m:[0m [2msnix_build::buildservice::bwrap[0m[2m:[0m CRUNCH_NO_FUSE set, materializing inputs to disk [3mpath[0m[2m=[0m"/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/crunch-builds/e9ec6e39-0476-419d-8905-8aac760f3edf-HioGx9/host_inputs_dir"
[2m2026-06-29T17:25:56.877762Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m CA output path resolved [3mdrv[0m[2m=[0mrust.drv [3moutput[0m[2m=[0mout [3mca_path[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/7z9n20ipqp2z26lp9ywl8xkfb0ddn37v-rust
[2m2026-06-29T17:25:56.881946Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m build succeeded [3mdrv[0m[2m=[0mrust.drv [3moutputs[0m[2m=[0m["/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/7z9n20ipqp2z26lp9ywl8xkfb0ddn37v-rust"]
[2m2026-06-29T17:26:28.434749Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m building [3mdrv[0m[2m=[0mmantle.drv
[2m2026-06-29T17:26:28.435089Z[0m [32m INFO[0m [1mdo_build[0m[2m:[0m [2msnix_build::buildservice::bwrap[0m[2m:[0m Starting bwrap build [3mbuild_name[0m[2m=[0m9d2d84bc-a928-41e4-af2f-0feb8bb4ad7e [3msandbox_path[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/crunch-builds/9d2d84bc-a928-41e4-af2f-0feb8bb4ad7e-iwlQZN
[2m2026-06-29T17:26:28.500057Z[0m [32m INFO[0m [1mdo_build[0m[2m:[0m [2msnix_build::buildservice::bwrap[0m[2m:[0m CRUNCH_NO_FUSE set, materializing inputs to disk [3mpath[0m[2m=[0m"/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/crunch-builds/9d2d84bc-a928-41e4-af2f-0feb8bb4ad7e-iwlQZN/host_inputs_dir"
[2m2026-06-29T17:39:14.150482Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m CA output path resolved [3mdrv[0m[2m=[0mmantle.drv [3moutput[0m[2m=[0mout [3mca_path[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/ymhmp1lbams1c2dgn748vklzbj2zmzkd-mantle
[2m2026-06-29T17:39:14.435288Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m build succeeded [3mdrv[0m[2m=[0mmantle.drv [3moutputs[0m[2m=[0m["/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/ymhmp1lbams1c2dgn748vklzbj2zmzkd-mantle"]
[2m2026-06-29T17:39:14.436063Z[0m [32m INFO[0m [2mcrunch_build::worker[0m[2m:[0m worker streaming finished [3mcompleted[0m[2m=[0m3 [3msucceeded[0m[2m=[0m1 [3mfailed[0m[2m=[0m0 [3mroots[0m[2m=[0m1
--- build log: mantle ---
GCC=/nix/store/rcpdzyz11r40n4wh2pq6q26hxs279kfy-gcc
GCC_ALIAS=/tmp/bootstrap/gcc
BINUTILS=/nix/store/qvc3wivg193x36v92pcdf1xq51b4jjky-binutils
BINUTILS_ALIAS=/tmp/bootstrap/binutils
MUSL=/nix/store/lpr77qmlvx3yq3dxgavrlgpc64m7p0b2-musl
MUSL_ALIAS=/tmp/bootstrap/musl
DASH=/nix/store/hvmnd5pnb2zg3l92ycff9p8ms7msigmm-dash
DASH_ALIAS=/tmp/bootstrap/dash
MAKE=/nix/store/096b231pbs9cqw67p32sdk011lpm60xl-gnumake
MAKE_ALIAS=/tmp/bootstrap/gnumake
RUST=/nix/store/7z9n20ipqp2z26lp9ywl8xkfb0ddn37v-rust
RUST_ALIAS=/tmp/bootstrap/rust
BWRAP=/nix/store/pys6ig7mb98pf53s7cq35iafz2wll35f-bwrap
BWRAP_ALIAS=/tmp/bootstrap/bwrap
BUSYBOX=/nix/store/wp1dyh7nl2ld4yl0ia730cdfm7js9mza-busybox
BUSYBOX_ALIAS=/tmp/bootstrap/busybox
Using mantle-built bwrap: /tmp/bootstrap/bwrap/bin
CRUNCH_SRC=/nix/store/zi7ps2paplpbljym0i0z8jkdq0ymibxq-mantle-src
SEED_LIB=/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/lib
SEED_LIB_ALIAS=/tmp/bootstrap/seed-lib
=== Tool versions ===
rustc 1.94.1 (e408947bf 2026-03-25)
cargo 1.94.1 (29ea6fb6a 2026-03-24)
gcc (GCC) 13.3.0
GNU Make 4.4.1
Using mantle-built busybox: /tmp/bootstrap/busybox/bin/busybox
=== Building mantle ===
warning: /tmp/build/crunch/Cargo.toml: file `/tmp/build/crunch/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling proc-macro2 v1.0.106
   Compiling unicode-ident v1.0.24
   Compiling quote v1.0.45
   Compiling serde_core v1.0.228
   Compiling serde v1.0.228
   Compiling memchr v2.8.0
   Compiling cfg-if v1.0.4
   Compiling libc v0.2.184
   Compiling syn v2.0.117
   Compiling smallvec v1.15.1
   Compiling parking_lot_core v0.9.12
   Compiling version_check v0.9.5
   Compiling scopeguard v1.2.0
   Compiling lock_api v0.4.14
   Compiling aho-corasick v1.1.4
   Compiling regex-syntax v0.8.10
   Compiling typenum v1.19.0
   Compiling itoa v1.0.18
   Compiling generic-array v0.14.7
   Compiling once_cell v1.21.4
   Compiling parking_lot v0.12.5
   Compiling regex-automata v0.4.14
   Compiling serde_derive v1.0.228
   Compiling allocator-api2 v0.2.21
   Compiling equivalent v1.0.2
   Compiling subtle v2.6.1
   Compiling thiserror v2.0.18
   Compiling thiserror-impl v2.0.18
   Compiling bytes v1.11.1
   Compiling jobserver v0.1.34
   Compiling find-msvc-tools v0.1.9
   Compiling shlex v1.3.0
   Compiling bstr v1.12.1
   Compiling cc v1.2.59
   Compiling stable_deref_trait v1.2.1
   Compiling crossbeam-utils v0.8.21
   Compiling bitflags v2.11.0
   Compiling foldhash v0.2.0
   Compiling hashbrown v0.16.1
   Compiling fastrand v2.4.1
   Compiling block-buffer v0.10.4
   Compiling crypto-common v0.1.7
   Compiling errno v0.3.14
   Compiling digest v0.10.7
   Compiling signal-hook-registry v1.4.8
   Compiling crc32fast v1.5.0
   Compiling tinyvec_macros v0.1.1
   Compiling tinyvec v1.11.0
   Compiling gix-trace v0.1.18
   Compiling gix-validate v0.11.0
   Compiling same-file v1.0.6
   Compiling walkdir v2.5.0
   Compiling unicode-normalization v0.1.25
   Compiling gix-path v0.11.2
   Compiling gix-utils v0.3.1
   Compiling cpufeatures v0.2.17
   Compiling byteorder v1.5.0
   Compiling pin-project-lite v0.2.17
   Compiling human_format v1.2.1
   Compiling bytesize v2.3.1
   Compiling prodash v31.0.0
   Compiling crossbeam-channel v0.5.15
   Compiling gix-error v0.2.1
   Compiling zlib-rs v0.6.3
   Compiling futures-core v0.3.32
   Compiling winnow v0.7.15
   Compiling heapless v0.8.0
   Compiling hash32 v0.3.1
   Compiling sha1 v0.10.6
   Compiling gix-features v0.46.2
   Compiling rustix v1.1.4
   Compiling faster-hex v0.10.0
   Compiling tokio-macros v2.7.0
   Compiling mio v1.2.0
   Compiling socket2 v0.6.3
   Compiling zmij v1.0.21
   Compiling linux-raw-sys v0.12.1
   Compiling tokio v1.51.0
   Compiling synstructure v0.13.2
   Compiling jiff v0.2.23
   Compiling serde_json v1.0.149
   Compiling sha1-checked v0.10.0
   Compiling futures-sink v0.3.32
   Compiling rand_core v0.10.0
   Compiling futures-io v0.3.32
   Compiling getrandom v0.4.2
   Compiling futures-channel v0.3.32
   Compiling gix-hash v0.23.0
   Compiling zerofrom-derive v0.1.7
   Compiling gix-date v0.15.1
   Compiling futures-macro v0.3.32
   Compiling zeroize v1.8.2
   Compiling futures-task v0.3.32
   Compiling slab v0.4.12
   Compiling zerofrom v0.1.7
   Compiling futures-util v0.3.32
   Compiling yoke-derive v0.8.2
   Compiling indexmap v2.13.1
   Compiling rustversion v1.0.22
   Compiling yoke v0.8.2
   Compiling gix-actor v0.40.0
   Compiling cmake v0.1.58
   Compiling dunce v1.0.5
   Compiling fs_extra v1.3.0
   Compiling log v0.4.29
   Compiling aws-lc-sys v0.39.1
   Compiling gix-hashtable v0.13.0
   Compiling zerovec-derive v0.11.3
   Compiling tracing-core v0.1.36
   Compiling fnv v1.0.7
   Compiling semver v1.0.28
   Compiling foldhash v0.1.5
   Compiling rustc_version v0.4.1
   Compiling hashbrown v0.15.5
   Compiling zerovec v0.11.6
   Compiling gix-object v0.58.0
   Compiling displaydoc v0.2.5
   Compiling memmap2 v0.9.10
   Compiling getrandom v0.2.17
   Compiling percent-encoding v2.3.2
   Compiling ring v0.17.14
   Compiling http v1.4.0
   Compiling tracing-attributes v0.1.31
   Compiling aws-lc-rs v1.16.2
   Compiling arrayvec v0.7.6
   Compiling tracing v0.1.44
   Compiling rustls-pki-types v1.14.0
   Compiling tinystr v0.8.3
   Compiling gix-fs v0.19.2
   Compiling gix-chunk v0.7.0
   Compiling litemap v0.8.2
   Compiling signal-hook v0.4.4
   Compiling untrusted v0.9.0
   Compiling writeable v0.6.3
   Compiling httparse v1.10.1
   Compiling base64 v0.22.1
   Compiling icu_locale_core v2.2.0
   Compiling zerotrie v0.2.4
   Compiling potential_utf v0.1.5
   Compiling tokio-util v0.7.18
   Compiling tempfile v3.27.0
   Compiling gix-quote v0.7.0
   Compiling rustls v0.23.37
   Compiling icu_normalizer_data v2.2.0
   Compiling static_assertions v1.1.0
   Compiling nonempty v0.12.0
   Compiling hashbrown v0.14.5
   Compiling utf8_iter v1.0.4
   Compiling pkg-config v0.3.32
   Compiling icu_properties_data v2.2.0
   Compiling dashmap v6.1.0
   Compiling icu_collections v2.2.0
   Compiling icu_provider v2.2.0
   Compiling http-body v1.0.1
   Compiling encoding_rs v0.8.35
   Compiling ident_case v1.0.1
   Compiling strsim v0.11.1
   Compiling gix-tempfile v21.0.2
   Compiling gix-commitgraph v0.35.0
   Compiling gix-glob v0.24.0
   Compiling unicode-xid v0.2.6
   Compiling try-lock v0.2.5
   Compiling tower-service v0.3.3
   Compiling ryu v1.0.23
   Compiling atomic-waker v1.1.2
   Compiling h2 v0.4.13
   Compiling want v0.3.1
   Compiling gix-revwalk v0.29.0
   Compiling icu_properties v2.2.0
   Compiling icu_normalizer v2.2.0
   Compiling rayon-core v1.13.0
   Compiling openssl-probe v0.2.1
   Compiling unicode-width v0.2.2
   Compiling either v1.15.0
   Compiling hyper v1.9.0
   Compiling rustls-native-certs v0.8.3
   Compiling idna_adapter v1.2.1
   Compiling form_urlencoded v1.2.2
   Compiling sync_wrapper v1.0.2
   Compiling crossbeam-epoch v0.9.18
   Compiling tower-layer v0.3.3
   Compiling unicode-bom v2.0.3
   Compiling ipnet v2.12.0
   Compiling hyper-util v0.1.20
   Compiling tower v0.5.3
   Compiling crossbeam-deque v0.8.6
   Compiling idna v1.1.0
   Compiling itertools v0.14.0
   Compiling gix-lock v21.0.2
   Compiling blake3 v1.8.2
   Compiling filetime v0.2.27
   Compiling iri-string v0.7.12
   Compiling autocfg v1.5.0
   Compiling url v2.5.8
   Compiling tower-http v0.6.8
   Compiling http-body-util v0.1.3
   Compiling kstring v2.0.2
   Compiling arrayref v0.3.9
   Compiling shell-words v1.1.1
   Compiling constant_time_eq v0.3.1
   Compiling gix-command v0.8.0
   Compiling gix-attributes v0.31.0
   Compiling gix-config-value v0.17.1
   Compiling colorchoice v1.0.5
   Compiling mime v0.3.17
   Compiling cfg_aliases v0.2.1
   Compiling gix-traverse v0.55.0
   Compiling darling_core v0.23.0
   Compiling zstd-sys v2.0.16+zstd.1.5.7
   Compiling gix-packetline v0.21.2
   Compiling portable-atomic v1.13.1
   Compiling heck v0.5.0
   Compiling libm v0.2.16
   Compiling siphasher v1.0.2
   Compiling utf8parse v0.2.2
   Compiling data-encoding v2.10.0
   Compiling anstyle-parse v1.0.0
   Compiling darling_macro v0.23.0
   Compiling phf_shared v0.11.3
   Compiling gix-sec v0.13.2
   Compiling fixedbitset v0.5.7
   Compiling precomputed-hash v0.1.1
   Compiling winnow v1.0.1
   Compiling anstyle-query v1.1.5
   Compiling keccak v0.1.6
   Compiling bytemuck v1.25.0
   Compiling anstyle v1.0.14
   Compiling lazy_static v1.5.0
   Compiling is_terminal_polyfill v1.70.2
   Compiling anyhow v1.0.102
   Compiling new_debug_unreachable v1.0.6
   Compiling unicode-segmentation v1.13.2
   Compiling bit-vec v0.8.0
   Compiling term v1.2.1
   Compiling convert_case v0.10.0
   Compiling ascii-canvas v4.0.0
   Compiling bit-set v0.8.0
   Compiling string_cache v0.8.9
   Compiling anstream v1.0.0
   Compiling safe_arch v0.7.4
   Compiling sha3 v0.10.8
   Compiling toml_parser v1.1.2+spec-1.1.0
   Compiling ena v0.14.4
   Compiling petgraph v0.7.1
   Compiling darling v0.23.0
   Compiling lalrpop-util v0.22.2
   Compiling regex v1.12.3
   Compiling memoffset v0.6.5
   Compiling num-traits v0.2.19
   Compiling lzma-sys v0.1.20
   Compiling bzip2-sys v0.1.13+1.0.8
   Compiling vte v0.15.0
   Compiling gix-bitmap v0.3.0
   Compiling sha2 v0.10.9
   Compiling libmimalloc-sys v0.1.44
   Compiling async-trait v0.1.89
   Compiling clap_lex v1.1.0
   Compiling zstd-safe v7.2.4
   Compiling typeid v1.0.3
   Compiling pico-args v0.5.0
   Compiling toml_datetime v1.1.1+spec-1.1.0
   Compiling malachite-nz v0.6.1
   Compiling toml_edit v0.25.10+spec-1.1.0
   Compiling lalrpop v0.22.2
   Compiling clap_builder v4.6.0
   Compiling gix-index v0.49.0
   Compiling vt100 v0.16.2
   Compiling malachite-base v0.6.1
   Compiling serde_with_macros v3.18.0
   Compiling wide v0.7.33
   Compiling derive_more-impl v2.1.1
   Compiling sharded-slab v0.1.7
   Compiling clap_derive v4.6.0
   Compiling gix-filter v0.28.0
   Compiling n0-future v0.3.2
   Compiling gix-ref v0.61.0
   Compiling darling_core v0.20.11
   Compiling gix-ignore v0.19.1
   Compiling console v0.16.3
   Compiling heapless v0.7.17
   Compiling curve25519-dalek v4.1.3
   Compiling logos-codegen v0.15.1
   Compiling tracing-log v0.2.0
   Compiling arc-swap v1.9.1
   Compiling futures-executor v0.3.32
   Compiling matchers v0.2.0
   Compiling pin-project-internal v1.1.11
   Compiling proc-macro2-diagnostics v0.10.1
   Compiling thread_local v1.1.9
   Compiling adler2 v2.0.1
   Compiling spin v0.10.0
   Compiling thiserror v1.0.69
   Compiling diatomic-waker v0.2.3
   Compiling nu-ansi-term v0.50.3
   Compiling cpufeatures v0.3.0
   Compiling erased-serde v0.4.10
   Compiling unit-prefix v0.5.2
   Compiling simd-adler32 v0.3.9
   Compiling bitflags v1.3.2
   Compiling cordyceps v0.3.4
   Compiling parking v2.2.1
   Compiling futures-buffered v0.2.13
   Compiling futures-lite v2.6.1
   Compiling miniz_oxide v0.8.9
   Compiling tracing-subscriber v0.3.23
   Compiling indicatif v0.18.4
   Compiling chacha20 v0.10.0
   Compiling pin-project v1.1.11
   Compiling futures v0.3.32
   Compiling gix-worktree v0.50.0
   Compiling rustls-webpki v0.103.12
   Compiling darling_macro v0.20.11
   Compiling clap v4.6.0
   Compiling derive_more v2.1.1
   Compiling serde_with v3.18.0
   Compiling proc-macro-crate v3.5.0
   Compiling gix-pathspec v0.16.1
   Compiling serde_urlencoded v0.7.1
   Compiling tokio-stream v0.1.18
   Compiling xattr v1.6.1
   Compiling hash32 v0.2.1
   Compiling tokio-rustls v0.26.4
   Compiling hyper-rustls v0.27.7
   Compiling rustls-platform-verifier v0.6.2
   Compiling reqwest v0.13.2
   Compiling curve25519-dalek-derive v0.1.1
   Compiling async-stream-impl v0.3.6
   Compiling thiserror-impl v1.0.69
   Compiling reqwest-middleware v0.5.1
   Compiling n0-error-macros v0.1.3
   Compiling spez v0.1.2
   Compiling spin v0.9.8
   Compiling nibble_vec v0.1.0
   Compiling beef v0.5.2
   Compiling matchit v0.8.4
   Compiling fuse-backend-rs v0.12.0 (/tmp/build/crunch/vendor/fuse-backend-rs)
   Compiling iana-time-zone v0.1.65
   Compiling endian-type v0.1.2
   Compiling yansi v1.0.1
   Compiling redb v3.1.3
   Compiling signature v2.2.0
   Compiling ed25519 v2.2.3
   Compiling radix_trie v0.2.1
   Compiling chrono v0.4.44
   Compiling reqwest-tracing v0.6.0
   Compiling n0-error v0.1.3
   Compiling malachite-q v0.6.1
   Compiling async-stream v0.3.6
   Compiling reqwest v0.12.28
   Compiling num_enum_derive v0.7.6
   Compiling mimalloc v0.1.48
   Compiling bzip2 v0.5.2
   Compiling zstd v0.13.3
   Compiling nix v0.24.3
   Compiling xz2 v0.1.7
   Compiling clap-verbosity-flag v3.0.4
   Compiling darling v0.20.11
   Compiling rand v0.10.1
   Compiling tracing-indicatif v0.3.14
   Compiling flate2 v1.1.9
   Compiling vmm-sys-util v0.11.2
   Compiling crunch-attestation-core v0.1.0 (/tmp/build/crunch/crates/crunch-attestation-core)
   Compiling gix-url v0.35.2
   Compiling mio v0.8.11
   Compiling md-5 v0.10.6
   Compiling cobs v0.3.0
   Compiling quick-xml v0.39.2
   Compiling irpc-derive v0.10.0
   Compiling toml_datetime v0.7.5+spec-1.1.0
   Compiling vm-memory v0.10.0
   Compiling num_cpus v1.17.0
   Compiling caps v0.5.6
   Compiling termcolor v1.4.1
   Compiling toml_writer v1.1.1+spec-1.1.0
   Compiling heck v0.4.1
   Compiling snix-castore v0.1.0 (/tmp/build/crunch/vendor/snix-castore)
   Compiling fixedbitset v0.4.2
   Compiling rustc-hash v2.1.2
   Compiling zerocopy v0.8.48
   Compiling nix-compat v0.1.0 (/tmp/build/crunch/vendor/nix-compat)
   Compiling humantime v2.3.0
   Compiling bitmaps v3.2.1
   Compiling petgraph v0.6.5
   Compiling object_store v0.13.2
   Compiling imbl-sized-chunks v0.1.3
   Compiling astral-tokio-tar v0.6.0
   Compiling ouroboros_macro v0.18.5
   Compiling codespan-reporting v0.13.1
   Compiling threadpool v1.8.1
   Compiling irpc v0.13.0
   Compiling postcard v1.1.3
   Compiling async-compression v0.4.19
   Compiling snix-tracing v0.1.0 (/tmp/build/crunch/vendor/snix-tracing)
   Compiling derive_builder_core v0.20.2
   Compiling num_enum v0.7.6
   Compiling serde_tagged v0.3.0
   Compiling ed25519-dalek v2.2.0
   Compiling fastcdc v3.2.1
   Compiling serde_qs v0.12.0
   Compiling malachite-float v0.6.1
   Compiling logos-derive v0.15.1
   Compiling nickel-lang-parser v0.1.1
   Compiling gix-prompt v0.14.1
   Compiling gix-revision v0.43.0
   Compiling hashlink v0.10.0
   Compiling imara-diff v0.2.0
   Compiling imara-diff v0.1.8
   Compiling auto_impl v1.3.0
   Compiling nix-compat-derive v0.1.0 (/tmp/build/crunch/vendor/nix-compat-derive)
   Compiling serde_bytes v0.11.19
   Compiling proc-macro-error-attr2 v2.0.0
   Compiling nom v8.0.0
   Compiling typed-arena v2.0.2
   Compiling paste v1.0.15
   Compiling aliasable v0.1.3
   Compiling wu-manber v0.1.0 (https://github.com/tvlfyi/wu-manber.git#0d5b22be)
   Compiling arrayvec v0.5.2
   Compiling arraydeque v0.5.1
   Compiling saphyr-parser v0.0.6
   Compiling pretty v0.12.5
   Compiling ouroboros v0.18.5
   Compiling proc-macro-error2 v2.0.1
   Compiling gix-diff v0.61.0
   Compiling gix-refspec v0.39.0
   Compiling gix-credentials v0.37.1
   Compiling logos v0.15.1
   Compiling malachite v0.6.1
   Compiling derive_builder_macro v0.20.2
   Compiling codespan v0.13.1
   Compiling nickel-lang-vector v0.1.0
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling crunch-attestation v0.1.0 (/tmp/build/crunch/crates/crunch-attestation)
   Compiling gix-discover v0.49.0
   Compiling nickel-lang-core v0.16.1
   Compiling nix v0.29.0
   Compiling uluru v3.1.0
   Compiling clru v0.6.3
   Compiling serde_spanned v1.1.1
   Compiling vte v0.14.1
   Compiling bumpalo v3.20.2
   Compiling simple-counter v0.1.0
   Compiling rustix v0.38.44
   Compiling snix-store v0.1.0 (/tmp/build/crunch/vendor/snix-store)
   Compiling unsafe-libyaml v0.2.11
   Compiling strip-ansi-escapes v0.2.1
   Compiling serde_yaml v0.9.34+deprecated
   Compiling gix-pack v0.68.0
   Compiling toml v0.9.12+spec-1.1.0
   Compiling ppv-lite86 v0.2.21
   Compiling gix-dir v0.23.0
   Compiling derive_builder v0.20.2
   Compiling gix-transport v0.55.1
   Compiling getset v0.1.6
   Compiling gix-config v0.54.0
   Compiling gix-worktree-stream v0.30.0
   Compiling strum_macros v0.26.4
   Compiling gix-shallow v0.10.0
   Compiling gix-negotiate v0.29.0
   Compiling rand_core v0.6.4
   Compiling sha-1 v0.10.1
   Compiling lru v0.16.4
   Compiling maybe-async v0.2.10
   Compiling typed-builder-macro v0.22.0
   Compiling io-close v0.3.7
   Compiling json_scanner v0.1.0
   Compiling crc-catalog v2.4.0
   Compiling snix-build v0.1.0 (/tmp/build/crunch/vendor/snix-build)
   Compiling linux-raw-sys v0.4.15
   Compiling strum v0.26.3
   Compiling indoc v2.0.7
   Compiling count-write v0.1.0
   Compiling oci-spec v0.7.1
   Compiling typed-builder v0.22.0
   Compiling crc v3.4.0
   Compiling gix-worktree-state v0.28.0
   Compiling gix-protocol v0.59.0
   Compiling rand_chacha v0.3.1
   Compiling gix-submodule v0.28.0
   Compiling gix-archive v0.30.0
   Compiling gix-status v0.28.0
   Compiling gix-odb v0.78.0
   Compiling gix-blame v0.11.0
   Compiling itertools v0.12.1
   Compiling ureq-proto v0.6.0
   Compiling webpki-roots v1.0.6
   Compiling gix-mailmap v0.32.0
   Compiling uuid v1.23.0
   Compiling fs2 v0.4.3
   Compiling twox-hash v2.1.2
   Compiling utf8-zero v0.8.1
   Compiling ruzstd v0.8.2
   Compiling ureq v3.3.0
   Compiling crunch-store v0.1.0 (/tmp/build/crunch/crates/crunch-store)
   Compiling gix v0.81.0
   Compiling rand v0.8.6
   Compiling lzma-rs v0.3.0
   Compiling gethostname v0.5.0
   Compiling tar v0.4.45
   Compiling bzip2-rs v0.1.2
   Compiling toml_datetime v0.6.11
   Compiling serde_spanned v0.6.9
   Compiling toml_write v0.1.2
   Compiling toml_edit v0.22.27
   Compiling crunch-build v0.1.0 (/tmp/build/crunch/crates/crunch-build)
   Compiling crunch-glue v0.1.0 (/tmp/build/crunch/crates/crunch-glue)
   Compiling crunch-project-core v0.1.0 (/tmp/build/crunch/crates/crunch-project-core)
   Compiling crunch-shell-core v0.1.0 (/tmp/build/crunch/crates/crunch-shell-core)
   Compiling crunch-shell v0.1.0 (/tmp/build/crunch/crates/crunch-shell)
   Compiling crunch-project v0.1.0 (/tmp/build/crunch/crates/crunch-project)
   Compiling toml v0.8.23
   Compiling crunch-release-core v0.1.0 (/tmp/build/crunch/crates/crunch-release-core)
   Compiling crunch-bootstrap-core v0.1.0 (/tmp/build/crunch/crates/crunch-bootstrap-core)
   Compiling nickel-lang v2.0.0
   Compiling crunch-eval v0.1.0 (/tmp/build/crunch/crates/crunch-eval)
   Compiling crunch-pipeline v0.1.0 (/tmp/build/crunch/crates/crunch-pipeline)
   Compiling mantle v0.1.0 (/tmp/build/crunch)
    Finished `release` profile [optimized] target(s) in 12m 02s
=== Installing ===
=== Path leak scan ===
=== Verify ===
-rwxr-xr-x    1 nixbld   nixbld    63530144 Jun 29 17:38 /nix/store/jfnzr7khgspxcivcdpaxr94rvgifbr58-mantle/bin/mantle
mantle 0.1.0

--- end log ---
hermeticity: practical (no degraded facts)
self-build-proof: progress=crunch-build-done
/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/ymhmp1lbams1c2dgn748vklzbj2zmzkd-mantle

[4/4] Verifying output...
verifying /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/ymhmp1lbams1c2dgn748vklzbj2zmzkd-mantle/bin/mantle...
  binary OK
self-build-proof: provider-mode=legacy-fetch
self-build-proof: hermeticity-mode=practical
self-build-proof: invoking-binary=/home/brittonr/.cargo-target/repo-targets/mantle/self-hosting-proof/work/cargo-target/debug/crunch
self-build-proof: staged-source=/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/zi7ps2paplpbljym0i0z8jkdq0ymibxq-mantle-src
self-build-proof: bwrap-source=mantle-built:/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/pys6ig7mb98pf53s7cq35iafz2wll35f-bwrap/bin
self-build-proof: fallback-event=bwrap-host-fallback:/nix/store/gr9l6ql3wg70idpqlqhnfdx81hak22c8-bubblewrap-0.11.0/bin/bwrap
self-build-proof: fallback-event=source-host-discovery:/home/brittonr/git/mantle
self-build-proof: protected-transition=bootstrap-tools-selected
self-build-proof: protected-transition-bwrap-path=/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/pys6ig7mb98pf53s7cq35iafz2wll35f-bwrap/bin/bwrap
self-build-proof: protected-transition-bwrap-digest=93cacbf9a523c439d160745228073492be1121b62074889345e676858fd1adf2
self-build-proof: protected-transition-bwrap-store-name=pys6ig7mb98pf53s7cq35iafz2wll35f-bwrap
self-build-proof: protected-transition-busybox-path=/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/wp1dyh7nl2ld4yl0ia730cdfm7js9mza-busybox/bin/busybox
self-build-proof: protected-transition-busybox-digest=ce321b321d0f06650ebbb24b834f4d2d600f5765d9f5828fb09c41a3ccd31ed5
self-build-proof: protected-transition-busybox-store-name=wp1dyh7nl2ld4yl0ia730cdfm7js9mza-busybox
self-build-proof: protected-seccomp-event=none
self-build-proof: busybox-path=/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/wp1dyh7nl2ld4yl0ia730cdfm7js9mza-busybox/bin/busybox
self-build-proof: output-binary=/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/ymhmp1lbams1c2dgn748vklzbj2zmzkd-mantle/bin/mantle
self-build-proof: stagex-metadata=none

=== self-build complete ===
stage0 audit: /home/brittonr/git/mantle/target/test-audit/self-hosting/stage0-2021944-1782754754649599906
stage0 diagnostics: /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/stage0-diagnostics.txt
stage0 stdout: /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/stage0-stdout.txt
stage0 stderr: /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/stage0-stderr.txt
stage1 binary: /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/ymhmp1lbams1c2dgn748vklzbj2zmzkd-mantle/bin/mantle
bwrap on disk: /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/pys6ig7mb98pf53s7cq35iafz2wll35f-bwrap/bin/bwrap
busybox on disk: /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/wp1dyh7nl2ld4yl0ia730cdfm7js9mza-busybox/bin/busybox

=== PROOF: Stage 2 (stage1 -> stage2) ===

stage2 store: /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store
stage2 state: /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/state2
stage2 stdout: /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/stage2-stdout.txt
stage2 stderr: /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/stage2-stderr.txt
=== crunch self-build ===
  invoking binary: /home/brittonr/.cargo-target/repo-targets/mantle/self-hosting-proof/work/tmp/.tmp9xxeXr/stage1-mantle
source: /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/zi7ps2paplpbljym0i0z8jkdq0ymibxq-mantle-src

[1/4] Staging source...
  reusing staged source: /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/zi7ps2paplpbljym0i0z8jkdq0ymibxq-mantle-src
Generated signing key: crunch-britton-desktop-1 (/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/state2/signing-key)

[2/4] Building bootstrap tools...
self-build-proof: progress=bootstrap-tool-start:bwrap.ncl
  reusing bwrap.ncl from /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/pys6ig7mb98pf53s7cq35iafz2wll35f-bwrap...
self-build-proof: progress=bootstrap-tool-done:bwrap.ncl
self-build-proof: progress=bootstrap-tool-start:busybox.ncl
  reusing busybox.ncl from /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/wp1dyh7nl2ld4yl0ia730cdfm7js9mza-busybox...
self-build-proof: progress=bootstrap-tool-done:busybox.ncl

[3/4] Building mantle...
self-build-proof: progress=mantle-build-start
[2m2026-06-29T17:39:21.934031Z[0m [32m INFO[0m [2mcrunch_store::handle[0m[2m:[0m blob service opened [3mpath[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/state2/blobs
[2m2026-06-29T17:39:21.935472Z[0m [32m INFO[0m [2mredb::db[0m[2m:[0m Opening database "FileBackend { lock_supported: true, file: File { fd: 10, path: \"/home/brittonr/.cargo-target/repo-targets/mantle/self-hosting-proof/work/tmp/.tmp9xxeXr/state2/directories.redb\", read: true, write: true } }"
[2m2026-06-29T17:39:21.937659Z[0m [33m WARN[0m [2mredb::db[0m[2m:[0m Database "FileBackend { lock_supported: true, file: File { fd: 10, path: \"/home/brittonr/.cargo-target/repo-targets/mantle/self-hosting-proof/work/tmp/.tmp9xxeXr/state2/directories.redb\", read: true, write: true } }" not shutdown cleanly. Repairing
[2m2026-06-29T17:39:21.942720Z[0m [32m INFO[0m [2mredb::db[0m[2m:[0m Opening database "FileBackend { lock_supported: true, file: File { fd: 11, path: \"/home/brittonr/.cargo-target/repo-targets/mantle/self-hosting-proof/work/tmp/.tmp9xxeXr/state2/pathinfo.redb\", read: true, write: true } }"
[2m2026-06-29T17:39:21.944746Z[0m [33m WARN[0m [2mredb::db[0m[2m:[0m Database "FileBackend { lock_supported: true, file: File { fd: 11, path: \"/home/brittonr/.cargo-target/repo-targets/mantle/self-hosting-proof/work/tmp/.tmp9xxeXr/state2/pathinfo.redb\", read: true, write: true } }" not shutdown cleanly. Repairing
[2m2026-06-29T17:39:21.949574Z[0m [32m INFO[0m [2mcrunch_store::handle[0m[2m:[0m PathInfo database opened [3mpath[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/state2/pathinfo.redb
[2m2026-06-29T17:39:21.950177Z[0m [32m INFO[0m [2mcrunch_build::worker[0m[2m:[0m worker streaming started [3mjobs[0m[2m=[0m4
[2m2026-06-29T17:39:21.997908Z[0m [32m INFO[0m [2mcrunch_pipeline[0m[2m:[0m converted, sending to worker [3mdrv[0m[2m=[0msy7vv7vcdgcprfasd89hf3m55hdnr084-mantle.drv [3mlabel[0m[2m=[0mmantle [3mentries[0m[2m=[0m18
[2m2026-06-29T17:39:22.000130Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m building [3mdrv[0m[2m=[0mmpfr-src.drv
[2m2026-06-29T17:39:22.000437Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m building [3mdrv[0m[2m=[0mmpc-src.drv
[2m2026-06-29T17:39:22.000494Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m building [3mdrv[0m[2m=[0mgmp-src.drv
[2m2026-06-29T17:39:22.000572Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m building [3mdrv[0m[2m=[0mgcc-src.drv
[2m2026-06-29T17:39:22.000713Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m building [3mdrv[0m[2m=[0mdash-src.drv
[2m2026-06-29T17:39:22.000806Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m building [3mdrv[0m[2m=[0mmusl-gcc-raw.drv
[2m2026-06-29T17:39:22.000910Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m building [3mdrv[0m[2m=[0mbinutils-src.drv
[2m2026-06-29T17:39:22.001026Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m building [3mdrv[0m[2m=[0mmusl-src.drv
[2m2026-06-29T17:39:22.001069Z[0m [32m INFO[0m [2mcrunch_build::fetch_build_service[0m[2m:[0m fetching tarball [3murl[0m[2m=[0mhttps://mirrors.kernel.org/gnu/mpc/mpc-1.2.1.tar.gz
[2m2026-06-29T17:39:22.001137Z[0m [32m INFO[0m [2mcrunch_build::fetch_build_service[0m[2m:[0m fetching tarball [3murl[0m[2m=[0mhttps://mirrors.kernel.org/gnu/mpfr/mpfr-4.1.0.tar.bz2
[2m2026-06-29T17:39:22.001179Z[0m [32m INFO[0m [2mcrunch_build::fetch_build_service[0m[2m:[0m fetching tarball [3murl[0m[2m=[0mhttps://mirrors.kernel.org/gnu/gmp/gmp-6.2.1.tar.bz2
[2m2026-06-29T17:39:22.001191Z[0m [32m INFO[0m [2mcrunch_build::fetch_build_service[0m[2m:[0m fetching tarball [3murl[0m[2m=[0mhttps://mirrors.kernel.org/gnu/gcc/gcc-13.3.0/gcc-13.3.0.tar.gz
[2m2026-06-29T17:39:22.001217Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m building [3mdrv[0m[2m=[0mmake-src.drv
[2m2026-06-29T17:39:22.001261Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m building [3mdrv[0m[2m=[0mrust-standalone.drv
[2m2026-06-29T17:39:22.669468Z[0m [32m INFO[0m [2mcrunch_build::fetch_build_service[0m[2m:[0m fetching tarball [3murl[0m[2m=[0mhttp://gondor.apana.org.au/~herbert/dash/files/dash-0.5.12.tar.gz
[2m2026-06-29T17:39:22.698638Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m build succeeded [3mdrv[0m[2m=[0mmpc-src.drv [3moutputs[0m[2m=[0m["/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/p9dxx2mn8fkfllpw2j7dsza5ws54ghwr-mpc-src"]
[2m2026-06-29T17:39:23.213987Z[0m [32m INFO[0m [2mcrunch_build::fetch_build_service[0m[2m:[0m fetching tarball [3murl[0m[2m=[0mhttps://musl.cc/x86_64-linux-musl-native.tgz
[2m2026-06-29T17:39:23.277157Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m build succeeded [3mdrv[0m[2m=[0mmpfr-src.drv [3moutputs[0m[2m=[0m["/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/66yb9y96cjyqkr3hyzgdrhzk9vh48ijg-mpfr-src"]
[2m2026-06-29T17:39:23.643683Z[0m [32m INFO[0m [2mcrunch_build::fetch_build_service[0m[2m:[0m fetching tarball [3murl[0m[2m=[0mhttps://mirrors.kernel.org/gnu/binutils/binutils-2.42.tar.gz
[2m2026-06-29T17:39:23.654268Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m build succeeded [3mdrv[0m[2m=[0mdash-src.drv [3moutputs[0m[2m=[0m["/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/zb9gm8bmwzrsxshr1qb85ppf2m6gp1cf-dash-src"]
[2m2026-06-29T17:39:24.147281Z[0m [32m INFO[0m [2mcrunch_build::fetch_build_service[0m[2m:[0m fetching tarball [3murl[0m[2m=[0mhttps://musl.libc.org/releases/musl-1.2.5.tar.gz
[2m2026-06-29T17:39:24.279407Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m build succeeded [3mdrv[0m[2m=[0mgmp-src.drv [3moutputs[0m[2m=[0m["/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/lrijcfa79mzlayc975ahqfh05lrpxdaz-gmp-src"]
[2m2026-06-29T17:39:29.240454Z[0m [32m INFO[0m [2mcrunch_build::fetch_build_service[0m[2m:[0m fetching tarball [3murl[0m[2m=[0mhttps://mirrors.kernel.org/gnu/make/make-4.4.1.tar.gz
[2m2026-06-29T17:39:30.078632Z[0m [32m INFO[0m [2mcrunch_build::fetch_build_service[0m[2m:[0m fetching tarball [3murl[0m[2m=[0mhttps://static.rust-lang.org/dist/rust-1.94.1-x86_64-unknown-linux-musl.tar.xz
[2m2026-06-29T17:39:30.368878Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m build succeeded [3mdrv[0m[2m=[0mmusl-gcc-raw.drv [3moutputs[0m[2m=[0m["/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/a0qkrlw0f9jm2l716xl67cb16fc3ggn0-musl-gcc-raw"]
[2m2026-06-29T17:39:30.369491Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m building [3mdrv[0m[2m=[0mmusl-seed-toolchain.drv
[2m2026-06-29T17:39:30.439152Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m build succeeded [3mdrv[0m[2m=[0mmake-src.drv [3moutputs[0m[2m=[0m["/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/sadgqs1ykvs2m6zs35qn8q4qg5s38rr3-make-src"]
[2m2026-06-29T17:39:32.265821Z[0m [32m INFO[0m [1mdo_build[0m[2m:[0m [2msnix_build::buildservice::bwrap[0m[2m:[0m Starting bwrap build [3mbuild_name[0m[2m=[0m93a5a688-15b7-4308-9ef2-6a44ef3ba0db [3msandbox_path[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/crunch-builds/93a5a688-15b7-4308-9ef2-6a44ef3ba0db-90PtJu
[2m2026-06-29T17:39:32.336989Z[0m [32m INFO[0m [1mdo_build[0m[2m:[0m [2msnix_build::buildservice::bwrap[0m[2m:[0m CRUNCH_NO_FUSE set, materializing inputs to disk [3mpath[0m[2m=[0m"/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/crunch-builds/93a5a688-15b7-4308-9ef2-6a44ef3ba0db-90PtJu/host_inputs_dir"
[2m2026-06-29T17:39:32.614640Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m build succeeded [3mdrv[0m[2m=[0mmusl-src.drv [3moutputs[0m[2m=[0m["/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/p36m6lwn63zllahxrm756vd760ss7i7c-musl-src"]
[2m2026-06-29T17:39:40.264656Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m CA output path resolved [3mdrv[0m[2m=[0mmusl-seed-toolchain.drv [3moutput[0m[2m=[0mout [3mca_path[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain
[2m2026-06-29T17:39:40.270179Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m build succeeded [3mdrv[0m[2m=[0mmusl-seed-toolchain.drv [3moutputs[0m[2m=[0m["/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain"]
[2m2026-06-29T17:39:40.270483Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m building [3mdrv[0m[2m=[0mgnumake.drv
[2m2026-06-29T17:39:40.270673Z[0m [32m INFO[0m [1mdo_build[0m[2m:[0m [2msnix_build::buildservice::bwrap[0m[2m:[0m Starting bwrap build [3mbuild_name[0m[2m=[0m2bf693d0-2c3b-4b14-bf71-c96dbbac3492 [3msandbox_path[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/crunch-builds/2bf693d0-2c3b-4b14-bf71-c96dbbac3492-JssHu3
[2m2026-06-29T17:39:40.328114Z[0m [32m INFO[0m [1mdo_build[0m[2m:[0m [2msnix_build::buildservice::bwrap[0m[2m:[0m CRUNCH_NO_FUSE set, materializing inputs to disk [3mpath[0m[2m=[0m"/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/crunch-builds/2bf693d0-2c3b-4b14-bf71-c96dbbac3492-JssHu3/host_inputs_dir"
[2m2026-06-29T17:39:45.832769Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m CA output path resolved [3mdrv[0m[2m=[0mgnumake.drv [3moutput[0m[2m=[0mout [3mca_path[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/096b231pbs9cqw67p32sdk011lpm60xl-gnumake
[2m2026-06-29T17:39:45.838668Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m build succeeded [3mdrv[0m[2m=[0mgnumake.drv [3moutputs[0m[2m=[0m["/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/096b231pbs9cqw67p32sdk011lpm60xl-gnumake"]
[2m2026-06-29T17:39:45.838792Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m building [3mdrv[0m[2m=[0mdash.drv
[2m2026-06-29T17:39:45.838888Z[0m [32m INFO[0m [1mdo_build[0m[2m:[0m [2msnix_build::buildservice::bwrap[0m[2m:[0m Starting bwrap build [3mbuild_name[0m[2m=[0me46cd580-c10c-4784-a28e-29a1824d99e7 [3msandbox_path[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/crunch-builds/e46cd580-c10c-4784-a28e-29a1824d99e7-dIz0TF
[2m2026-06-29T17:39:45.896474Z[0m [32m INFO[0m [1mdo_build[0m[2m:[0m [2msnix_build::buildservice::bwrap[0m[2m:[0m CRUNCH_NO_FUSE set, materializing inputs to disk [3mpath[0m[2m=[0m"/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/crunch-builds/e46cd580-c10c-4784-a28e-29a1824d99e7-dIz0TF/host_inputs_dir"
[2m2026-06-29T17:39:50.608170Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m build succeeded [3mdrv[0m[2m=[0mbinutils-src.drv [3moutputs[0m[2m=[0m["/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/2mgxyaw8zwivswh3kgak7nvbkxrprih7-binutils-src"]
[2m2026-06-29T17:39:50.613764Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m CA output path resolved [3mdrv[0m[2m=[0mdash.drv [3moutput[0m[2m=[0mout [3mca_path[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/hvmnd5pnb2zg3l92ycff9p8ms7msigmm-dash
[2m2026-06-29T17:39:50.615130Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m build succeeded [3mdrv[0m[2m=[0mdash.drv [3moutputs[0m[2m=[0m["/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/hvmnd5pnb2zg3l92ycff9p8ms7msigmm-dash"]
[2m2026-06-29T17:39:50.615245Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m building [3mdrv[0m[2m=[0mbinutils.drv
[2m2026-06-29T17:39:50.615310Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m building [3mdrv[0m[2m=[0mmusl.drv
[2m2026-06-29T17:39:50.615331Z[0m [32m INFO[0m [1mdo_build[0m[2m:[0m [2msnix_build::buildservice::bwrap[0m[2m:[0m Starting bwrap build [3mbuild_name[0m[2m=[0m17e2aeef-c379-4b84-91fb-5878109e5a45 [3msandbox_path[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/crunch-builds/17e2aeef-c379-4b84-91fb-5878109e5a45-iXqegF
[2m2026-06-29T17:39:50.615404Z[0m [32m INFO[0m [1mdo_build[0m[2m:[0m [2msnix_build::buildservice::bwrap[0m[2m:[0m Starting bwrap build [3mbuild_name[0m[2m=[0mf854d310-db1a-48b3-bb2f-ad044d4fe3a8 [3msandbox_path[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/crunch-builds/f854d310-db1a-48b3-bb2f-ad044d4fe3a8-maYYjD
[2m2026-06-29T17:39:50.691888Z[0m [32m INFO[0m [1mdo_build[0m[2m:[0m [2msnix_build::buildservice::bwrap[0m[2m:[0m CRUNCH_NO_FUSE set, materializing inputs to disk [3mpath[0m[2m=[0m"/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/crunch-builds/f854d310-db1a-48b3-bb2f-ad044d4fe3a8-maYYjD/host_inputs_dir"
[2m2026-06-29T17:39:50.691906Z[0m [32m INFO[0m [1mdo_build[0m[2m:[0m [2msnix_build::buildservice::bwrap[0m[2m:[0m CRUNCH_NO_FUSE set, materializing inputs to disk [3mpath[0m[2m=[0m"/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/crunch-builds/17e2aeef-c379-4b84-91fb-5878109e5a45-iXqegF/host_inputs_dir"
[2m2026-06-29T17:40:01.140607Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m CA output path resolved [3mdrv[0m[2m=[0mmusl.drv [3moutput[0m[2m=[0mout [3mca_path[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/lpr77qmlvx3yq3dxgavrlgpc64m7p0b2-musl
[2m2026-06-29T17:40:01.143553Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m build succeeded [3mdrv[0m[2m=[0mmusl.drv [3moutputs[0m[2m=[0m["/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/lpr77qmlvx3yq3dxgavrlgpc64m7p0b2-musl"]
[2m2026-06-29T17:40:33.833934Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m CA output path resolved [3mdrv[0m[2m=[0mbinutils.drv [3moutput[0m[2m=[0mout [3mca_path[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/qvc3wivg193x36v92pcdf1xq51b4jjky-binutils
[2m2026-06-29T17:40:33.837441Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m build succeeded [3mdrv[0m[2m=[0mbinutils.drv [3moutputs[0m[2m=[0m["/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/qvc3wivg193x36v92pcdf1xq51b4jjky-binutils"]
[2m2026-06-29T17:41:43.579626Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m build succeeded [3mdrv[0m[2m=[0mrust-standalone.drv [3moutputs[0m[2m=[0m["/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/jw1z67aamhnc39dl83jms0ikmics4dg8-rust-standalone"]
[2m2026-06-29T17:41:43.579997Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m building [3mdrv[0m[2m=[0mrust.drv
[2m2026-06-29T17:41:43.580268Z[0m [32m INFO[0m [1mdo_build[0m[2m:[0m [2msnix_build::buildservice::bwrap[0m[2m:[0m Starting bwrap build [3mbuild_name[0m[2m=[0m412fe4cf-e872-4759-9b75-b8d94ffce424 [3msandbox_path[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/crunch-builds/412fe4cf-e872-4759-9b75-b8d94ffce424-guujvj
[2m2026-06-29T17:41:43.700441Z[0m [32m INFO[0m [1mdo_build[0m[2m:[0m [2msnix_build::buildservice::bwrap[0m[2m:[0m CRUNCH_NO_FUSE set, materializing inputs to disk [3mpath[0m[2m=[0m"/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/crunch-builds/412fe4cf-e872-4759-9b75-b8d94ffce424-guujvj/host_inputs_dir"
[2m2026-06-29T17:42:01.024425Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m build succeeded [3mdrv[0m[2m=[0mgcc-src.drv [3moutputs[0m[2m=[0m["/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/kqv0h44wl9in55dlgwd9l9amlgbp0qp8-gcc-src"]
[2m2026-06-29T17:42:01.024614Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m building [3mdrv[0m[2m=[0mgcc.drv
[2m2026-06-29T17:42:01.024816Z[0m [32m INFO[0m [1mdo_build[0m[2m:[0m [2msnix_build::buildservice::bwrap[0m[2m:[0m Starting bwrap build [3mbuild_name[0m[2m=[0mbfb2e607-6439-4a7c-b59d-409c87d6e50a [3msandbox_path[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/crunch-builds/bfb2e607-6439-4a7c-b59d-409c87d6e50a-Tbp73B
[2m2026-06-29T17:42:01.070763Z[0m [32m INFO[0m [1mdo_build[0m[2m:[0m [2msnix_build::buildservice::bwrap[0m[2m:[0m CRUNCH_NO_FUSE set, materializing inputs to disk [3mpath[0m[2m=[0m"/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/crunch-builds/bfb2e607-6439-4a7c-b59d-409c87d6e50a-Tbp73B/host_inputs_dir"
[2m2026-06-29T17:42:31.084529Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m CA output path resolved [3mdrv[0m[2m=[0mrust.drv [3moutput[0m[2m=[0mout [3mca_path[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/7z9n20ipqp2z26lp9ywl8xkfb0ddn37v-rust
[2m2026-06-29T17:42:31.097692Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m build succeeded [3mdrv[0m[2m=[0mrust.drv [3moutputs[0m[2m=[0m["/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/7z9n20ipqp2z26lp9ywl8xkfb0ddn37v-rust"]
[2m2026-06-29T17:47:05.907177Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m CA output path resolved [3mdrv[0m[2m=[0mgcc.drv [3moutput[0m[2m=[0mout [3mca_path[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/rcpdzyz11r40n4wh2pq6q26hxs279kfy-gcc
[2m2026-06-29T17:47:05.908815Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m build succeeded [3mdrv[0m[2m=[0mgcc.drv [3moutputs[0m[2m=[0m["/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/rcpdzyz11r40n4wh2pq6q26hxs279kfy-gcc"]
[2m2026-06-29T17:47:29.026004Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m building [3mdrv[0m[2m=[0mmantle.drv
[2m2026-06-29T17:47:29.026179Z[0m [32m INFO[0m [1mdo_build[0m[2m:[0m [2msnix_build::buildservice::bwrap[0m[2m:[0m Starting bwrap build [3mbuild_name[0m[2m=[0m3d7bab02-b381-4fdc-b9a7-bae2eb321a16 [3msandbox_path[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/crunch-builds/3d7bab02-b381-4fdc-b9a7-bae2eb321a16-YR7SK1
[2m2026-06-29T17:47:29.064510Z[0m [32m INFO[0m [1mdo_build[0m[2m:[0m [2msnix_build::buildservice::bwrap[0m[2m:[0m CRUNCH_NO_FUSE set, materializing inputs to disk [3mpath[0m[2m=[0m"/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/crunch-builds/3d7bab02-b381-4fdc-b9a7-bae2eb321a16-YR7SK1/host_inputs_dir"
[2m2026-06-29T17:57:01.993760Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m CA output path resolved [3mdrv[0m[2m=[0mmantle.drv [3moutput[0m[2m=[0mout [3mca_path[0m[2m=[0m/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/ymhmp1lbams1c2dgn748vklzbj2zmzkd-mantle
[2m2026-06-29T17:57:02.136110Z[0m [32m INFO[0m [2mcrunch_build::orchestrate[0m[2m:[0m build succeeded [3mdrv[0m[2m=[0mmantle.drv [3moutputs[0m[2m=[0m["/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/ymhmp1lbams1c2dgn748vklzbj2zmzkd-mantle"]
[2m2026-06-29T17:57:02.136594Z[0m [32m INFO[0m [2mcrunch_build::worker[0m[2m:[0m worker streaming finished [3mcompleted[0m[2m=[0m18 [3msucceeded[0m[2m=[0m1 [3mfailed[0m[2m=[0m0 [3mroots[0m[2m=[0m1
--- build log: mantle ---
GCC=/nix/store/rcpdzyz11r40n4wh2pq6q26hxs279kfy-gcc
GCC_ALIAS=/tmp/bootstrap/gcc
BINUTILS=/nix/store/qvc3wivg193x36v92pcdf1xq51b4jjky-binutils
BINUTILS_ALIAS=/tmp/bootstrap/binutils
MUSL=/nix/store/lpr77qmlvx3yq3dxgavrlgpc64m7p0b2-musl
MUSL_ALIAS=/tmp/bootstrap/musl
DASH=/nix/store/hvmnd5pnb2zg3l92ycff9p8ms7msigmm-dash
DASH_ALIAS=/tmp/bootstrap/dash
MAKE=/nix/store/096b231pbs9cqw67p32sdk011lpm60xl-gnumake
MAKE_ALIAS=/tmp/bootstrap/gnumake
RUST=/nix/store/7z9n20ipqp2z26lp9ywl8xkfb0ddn37v-rust
RUST_ALIAS=/tmp/bootstrap/rust
BWRAP=/nix/store/pys6ig7mb98pf53s7cq35iafz2wll35f-bwrap
BWRAP_ALIAS=/tmp/bootstrap/bwrap
BUSYBOX=/nix/store/wp1dyh7nl2ld4yl0ia730cdfm7js9mza-busybox
BUSYBOX_ALIAS=/tmp/bootstrap/busybox
Using mantle-built bwrap: /tmp/bootstrap/bwrap/bin
CRUNCH_SRC=/nix/store/zi7ps2paplpbljym0i0z8jkdq0ymibxq-mantle-src
SEED_LIB=/nix/store/vailqg2jdmk7m3af9farciy1vpmsb5bp-musl-seed-toolchain/x86_64-linux-musl/lib
SEED_LIB_ALIAS=/tmp/bootstrap/seed-lib
=== Tool versions ===
rustc 1.94.1 (e408947bf 2026-03-25)
cargo 1.94.1 (29ea6fb6a 2026-03-24)
gcc (GCC) 13.3.0
GNU Make 4.4.1
Using mantle-built busybox: /tmp/bootstrap/busybox/bin/busybox
=== Building mantle ===
warning: /tmp/build/crunch/Cargo.toml: file `/tmp/build/crunch/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling proc-macro2 v1.0.106
   Compiling unicode-ident v1.0.24
   Compiling quote v1.0.45
   Compiling serde_core v1.0.228
   Compiling serde v1.0.228
   Compiling memchr v2.8.0
   Compiling cfg-if v1.0.4
   Compiling libc v0.2.184
   Compiling syn v2.0.117
   Compiling smallvec v1.15.1
   Compiling parking_lot_core v0.9.12
   Compiling version_check v0.9.5
   Compiling scopeguard v1.2.0
   Compiling lock_api v0.4.14
   Compiling aho-corasick v1.1.4
   Compiling regex-syntax v0.8.10
   Compiling itoa v1.0.18
   Compiling typenum v1.19.0
   Compiling generic-array v0.14.7
   Compiling once_cell v1.21.4
   Compiling parking_lot v0.12.5
   Compiling regex-automata v0.4.14
   Compiling serde_derive v1.0.228
   Compiling allocator-api2 v0.2.21
   Compiling equivalent v1.0.2
   Compiling bytes v1.11.1
   Compiling subtle v2.6.1
   Compiling thiserror v2.0.18
   Compiling thiserror-impl v2.0.18
   Compiling jobserver v0.1.34
   Compiling shlex v1.3.0
   Compiling find-msvc-tools v0.1.9
   Compiling cc v1.2.59
   Compiling bstr v1.12.1
   Compiling stable_deref_trait v1.2.1
   Compiling crossbeam-utils v0.8.21
   Compiling bitflags v2.11.0
   Compiling foldhash v0.2.0
   Compiling fastrand v2.4.1
   Compiling hashbrown v0.16.1
   Compiling errno v0.3.14
   Compiling signal-hook-registry v1.4.8
   Compiling crypto-common v0.1.7
   Compiling block-buffer v0.10.4
   Compiling digest v0.10.7
   Compiling crc32fast v1.5.0
   Compiling tinyvec_macros v0.1.1
   Compiling tinyvec v1.11.0
   Compiling gix-trace v0.1.18
   Compiling gix-validate v0.11.0
   Compiling same-file v1.0.6
   Compiling walkdir v2.5.0
   Compiling unicode-normalization v0.1.25
   Compiling gix-path v0.11.2
   Compiling gix-utils v0.3.1
   Compiling byteorder v1.5.0
   Compiling cpufeatures v0.2.17
   Compiling pin-project-lite v0.2.17
   Compiling bytesize v2.3.1
   Compiling human_format v1.2.1
   Compiling prodash v31.0.0
   Compiling crossbeam-channel v0.5.15
   Compiling gix-error v0.2.1
   Compiling futures-core v0.3.32
   Compiling zlib-rs v0.6.3
   Compiling winnow v0.7.15
   Compiling heapless v0.8.0
   Compiling hash32 v0.3.1
   Compiling sha1 v0.10.6
   Compiling rustix v1.1.4
   Compiling gix-features v0.46.2
   Compiling faster-hex v0.10.0
   Compiling tokio-macros v2.7.0
   Compiling mio v1.2.0
   Compiling socket2 v0.6.3
   Compiling linux-raw-sys v0.12.1
   Compiling zmij v1.0.21
   Compiling tokio v1.51.0
   Compiling synstructure v0.13.2
   Compiling jiff v0.2.23
   Compiling serde_json v1.0.149
   Compiling sha1-checked v0.10.0
   Compiling rand_core v0.10.0
   Compiling futures-io v0.3.32
   Compiling futures-sink v0.3.32
   Compiling getrandom v0.4.2
   Compiling futures-channel v0.3.32
   Compiling gix-hash v0.23.0
   Compiling gix-date v0.15.1
   Compiling zerofrom-derive v0.1.7
   Compiling futures-macro v0.3.32
   Compiling slab v0.4.12
   Compiling futures-task v0.3.32
   Compiling zeroize v1.8.2
   Compiling zerofrom v0.1.7
   Compiling futures-util v0.3.32
   Compiling yoke-derive v0.8.2
   Compiling indexmap v2.13.1
   Compiling rustversion v1.0.22
   Compiling yoke v0.8.2
   Compiling gix-actor v0.40.0
   Compiling cmake v0.1.58
   Compiling fs_extra v1.3.0
   Compiling log v0.4.29
   Compiling dunce v1.0.5
   Compiling aws-lc-sys v0.39.1
   Compiling gix-hashtable v0.13.0
   Compiling zerovec-derive v0.11.3
   Compiling tracing-core v0.1.36
   Compiling fnv v1.0.7
   Compiling semver v1.0.28
   Compiling foldhash v0.1.5
   Compiling rustc_version v0.4.1
   Compiling hashbrown v0.15.5
   Compiling zerovec v0.11.6
   Compiling gix-object v0.58.0
   Compiling displaydoc v0.2.5
   Compiling memmap2 v0.9.10
   Compiling getrandom v0.2.17
   Compiling percent-encoding v2.3.2
   Compiling ring v0.17.14
   Compiling http v1.4.0
   Compiling tracing-attributes v0.1.31
   Compiling arrayvec v0.7.6
   Compiling aws-lc-rs v1.16.2
   Compiling tracing v0.1.44
   Compiling rustls-pki-types v1.14.0
   Compiling tinystr v0.8.3
   Compiling gix-fs v0.19.2
   Compiling gix-chunk v0.7.0
   Compiling signal-hook v0.4.4
   Compiling untrusted v0.9.0
   Compiling litemap v0.8.2
   Compiling base64 v0.22.1
   Compiling writeable v0.6.3
   Compiling httparse v1.10.1
   Compiling icu_locale_core v2.2.0
   Compiling zerotrie v0.2.4
   Compiling potential_utf v0.1.5
   Compiling tokio-util v0.7.18
   Compiling tempfile v3.27.0
   Compiling gix-quote v0.7.0
   Compiling rustls v0.23.37
   Compiling pkg-config v0.3.32
   Compiling icu_normalizer_data v2.2.0
   Compiling hashbrown v0.14.5
   Compiling icu_properties_data v2.2.0
   Compiling utf8_iter v1.0.4
   Compiling static_assertions v1.1.0
   Compiling nonempty v0.12.0
   Compiling icu_collections v2.2.0
   Compiling dashmap v6.1.0
   Compiling icu_provider v2.2.0
   Compiling http-body v1.0.1
   Compiling encoding_rs v0.8.35
   Compiling strsim v0.11.1
   Compiling ident_case v1.0.1
   Compiling gix-tempfile v21.0.2
   Compiling gix-commitgraph v0.35.0
   Compiling gix-glob v0.24.0
   Compiling ryu v1.0.23
   Compiling tower-service v0.3.3
   Compiling try-lock v0.2.5
   Compiling unicode-xid v0.2.6
   Compiling atomic-waker v1.1.2
   Compiling want v0.3.1
   Compiling h2 v0.4.13
   Compiling gix-revwalk v0.29.0
   Compiling icu_properties v2.2.0
   Compiling icu_normalizer v2.2.0
   Compiling rayon-core v1.13.0
   Compiling unicode-width v0.2.2
   Compiling either v1.15.0
   Compiling openssl-probe v0.2.1
   Compiling rustls-native-certs v0.8.3
   Compiling idna_adapter v1.2.1
   Compiling form_urlencoded v1.2.2
   Compiling sync_wrapper v1.0.2
   Compiling crossbeam-epoch v0.9.18
   Compiling hyper v1.9.0
   Compiling ipnet v2.12.0
   Compiling tower-layer v0.3.3
   Compiling unicode-bom v2.0.3
   Compiling hyper-util v0.1.20
   Compiling tower v0.5.3
   Compiling crossbeam-deque v0.8.6
   Compiling idna v1.1.0
   Compiling itertools v0.14.0
   Compiling gix-lock v21.0.2
   Compiling blake3 v1.8.2
   Compiling filetime v0.2.27
   Compiling autocfg v1.5.0
   Compiling iri-string v0.7.12
   Compiling url v2.5.8
   Compiling tower-http v0.6.8
   Compiling http-body-util v0.1.3
   Compiling kstring v2.0.2
   Compiling arrayref v0.3.9
   Compiling shell-words v1.1.1
   Compiling constant_time_eq v0.3.1
   Compiling gix-command v0.8.0
   Compiling gix-attributes v0.31.0
   Compiling gix-config-value v0.17.1
   Compiling colorchoice v1.0.5
   Compiling cfg_aliases v0.2.1
   Compiling mime v0.3.17
   Compiling gix-traverse v0.55.0
   Compiling darling_core v0.23.0
   Compiling zstd-sys v2.0.16+zstd.1.5.7
   Compiling gix-packetline v0.21.2
   Compiling libm v0.2.16
   Compiling portable-atomic v1.13.1
   Compiling data-encoding v2.10.0
   Compiling utf8parse v0.2.2
   Compiling heck v0.5.0
   Compiling siphasher v1.0.2
   Compiling phf_shared v0.11.3
   Compiling darling_macro v0.23.0
   Compiling anstyle-parse v1.0.0
   Compiling gix-sec v0.13.2
   Compiling is_terminal_polyfill v1.70.2
   Compiling anstyle v1.0.14
   Compiling keccak v0.1.6
   Compiling lazy_static v1.5.0
   Compiling unicode-segmentation v1.13.2
   Compiling fixedbitset v0.5.7
   Compiling anstyle-query v1.1.5
   Compiling bytemuck v1.25.0
   Compiling term v1.2.1
   Compiling anyhow v1.0.102
   Compiling bit-vec v0.8.0
   Compiling precomputed-hash v0.1.1
   Compiling new_debug_unreachable v1.0.6
   Compiling winnow v1.0.1
   Compiling string_cache v0.8.9
   Compiling bit-set v0.8.0
   Compiling ascii-canvas v4.0.0
   Compiling safe_arch v0.7.4
   Compiling toml_parser v1.1.2+spec-1.1.0
   Compiling anstream v1.0.0
   Compiling petgraph v0.7.1
   Compiling convert_case v0.10.0
   Compiling ena v0.14.4
   Compiling sha3 v0.10.8
   Compiling darling v0.23.0
   Compiling regex v1.12.3
   Compiling lalrpop-util v0.22.2
   Compiling memoffset v0.6.5
   Compiling num-traits v0.2.19
   Compiling bzip2-sys v0.1.13+1.0.8
   Compiling lzma-sys v0.1.20
   Compiling vte v0.15.0
   Compiling gix-bitmap v0.3.0
   Compiling sha2 v0.10.9
   Compiling libmimalloc-sys v0.1.44
   Compiling async-trait v0.1.89
   Compiling clap_lex v1.1.0
   Compiling zstd-safe v7.2.4
   Compiling typeid v1.0.3
   Compiling toml_datetime v1.1.1+spec-1.1.0
   Compiling malachite-nz v0.6.1
   Compiling pico-args v0.5.0
   Compiling toml_edit v0.25.10+spec-1.1.0
   Compiling lalrpop v0.22.2
   Compiling clap_builder v4.6.0
   Compiling gix-index v0.49.0
   Compiling vt100 v0.16.2
   Compiling malachite-base v0.6.1
   Compiling serde_with_macros v3.18.0
   Compiling derive_more-impl v2.1.1
   Compiling wide v0.7.33
   Compiling sharded-slab v0.1.7
   Compiling clap_derive v4.6.0
   Compiling gix-filter v0.28.0
   Compiling n0-future v0.3.2
   Compiling gix-ref v0.61.0
   Compiling rustls-webpki v0.103.12
   Compiling gix-ignore v0.19.1
   Compiling darling_core v0.20.11
   Compiling console v0.16.3
   Compiling heapless v0.7.17
   Compiling curve25519-dalek v4.1.3
   Compiling logos-codegen v0.15.1
   Compiling tracing-log v0.2.0
   Compiling arc-swap v1.9.1
   Compiling futures-executor v0.3.32
   Compiling matchers v0.2.0
   Compiling pin-project-internal v1.1.11
   Compiling proc-macro2-diagnostics v0.10.1
   Compiling thread_local v1.1.9
   Compiling spin v0.10.0
   Compiling thiserror v1.0.69
   Compiling diatomic-waker v0.2.3
   Compiling adler2 v2.0.1
   Compiling bitflags v1.3.2
   Compiling erased-serde v0.4.10
   Compiling nu-ansi-term v0.50.3
   Compiling cpufeatures v0.3.0
   Compiling cordyceps v0.3.4
   Compiling tokio-rustls v0.26.4
   Compiling rustls-platform-verifier v0.6.2
   Compiling parking v2.2.1
   Compiling unit-prefix v0.5.2
   Compiling hyper-rustls v0.27.7
   Compiling simd-adler32 v0.3.9
   Compiling reqwest v0.13.2
   Compiling miniz_oxide v0.8.9
   Compiling indicatif v0.18.4
   Compiling reqwest-middleware v0.5.1
   Compiling futures-lite v2.6.1
   Compiling futures-buffered v0.2.13
   Compiling tracing-subscriber v0.3.23
   Compiling chacha20 v0.10.0
   Compiling pin-project v1.1.11
   Compiling futures v0.3.32
   Compiling darling_macro v0.20.11
   Compiling gix-worktree v0.50.0
   Compiling clap v4.6.0
   Compiling derive_more v2.1.1
   Compiling serde_with v3.18.0
   Compiling proc-macro-crate v3.5.0
   Compiling gix-pathspec v0.16.1
   Compiling serde_urlencoded v0.7.1
   Compiling tokio-stream v0.1.18
   Compiling xattr v1.6.1
   Compiling hash32 v0.2.1
   Compiling spez v0.1.2
   Compiling curve25519-dalek-derive v0.1.1
   Compiling async-stream-impl v0.3.6
   Compiling thiserror-impl v1.0.69
   Compiling n0-error-macros v0.1.3
   Compiling spin v0.9.8
   Compiling nibble_vec v0.1.0
   Compiling iana-time-zone v0.1.65
   Compiling matchit v0.8.4
   Compiling endian-type v0.1.2
   Compiling yansi v1.0.1
   Compiling redb v3.1.3
   Compiling beef v0.5.2
   Compiling signature v2.2.0
   Compiling fuse-backend-rs v0.12.0 (/tmp/build/crunch/vendor/fuse-backend-rs)
   Compiling ed25519 v2.2.3
   Compiling radix_trie v0.2.1
   Compiling reqwest-tracing v0.6.0
   Compiling chrono v0.4.44
   Compiling n0-error v0.1.3
   Compiling async-stream v0.3.6
   Compiling reqwest v0.12.28
   Compiling malachite-q v0.6.1
   Compiling num_enum_derive v0.7.6
   Compiling mimalloc v0.1.48
   Compiling bzip2 v0.5.2
   Compiling nix v0.24.3
   Compiling xz2 v0.1.7
   Compiling zstd v0.13.3
   Compiling clap-verbosity-flag v3.0.4
   Compiling tracing-indicatif v0.3.14
   Compiling darling v0.20.11
   Compiling rand v0.10.1
   Compiling flate2 v1.1.9
   Compiling vmm-sys-util v0.11.2
   Compiling crunch-attestation-core v0.1.0 (/tmp/build/crunch/crates/crunch-attestation-core)
   Compiling gix-url v0.35.2
   Compiling mio v0.8.11
   Compiling md-5 v0.10.6
   Compiling cobs v0.3.0
   Compiling quick-xml v0.39.2
   Compiling irpc-derive v0.10.0
   Compiling toml_datetime v0.7.5+spec-1.1.0
   Compiling vm-memory v0.10.0
   Compiling num_cpus v1.17.0
   Compiling caps v0.5.6
   Compiling heck v0.4.1
   Compiling fixedbitset v0.4.2
   Compiling snix-castore v0.1.0 (/tmp/build/crunch/vendor/snix-castore)
   Compiling rustc-hash v2.1.2
   Compiling termcolor v1.4.1
   Compiling bitmaps v3.2.1
   Compiling nix-compat v0.1.0 (/tmp/build/crunch/vendor/nix-compat)
   Compiling toml_writer v1.1.1+spec-1.1.0
   Compiling humantime v2.3.0
   Compiling zerocopy v0.8.48
   Compiling imbl-sized-chunks v0.1.3
   Compiling object_store v0.13.2
   Compiling codespan-reporting v0.13.1
   Compiling petgraph v0.6.5
   Compiling astral-tokio-tar v0.6.0
   Compiling ouroboros_macro v0.18.5
   Compiling threadpool v1.8.1
   Compiling irpc v0.13.0
   Compiling postcard v1.1.3
   Compiling async-compression v0.4.19
   Compiling derive_builder_core v0.20.2
   Compiling snix-tracing v0.1.0 (/tmp/build/crunch/vendor/snix-tracing)
   Compiling serde_tagged v0.3.0
   Compiling num_enum v0.7.6
   Compiling malachite-float v0.6.1
   Compiling ed25519-dalek v2.2.0
   Compiling fastcdc v3.2.1
   Compiling serde_qs v0.12.0
   Compiling logos-derive v0.15.1
   Compiling nickel-lang-parser v0.1.1
   Compiling gix-prompt v0.14.1
   Compiling gix-revision v0.43.0
   Compiling hashlink v0.10.0
   Compiling imara-diff v0.2.0
   Compiling imara-diff v0.1.8
   Compiling auto_impl v1.3.0
   Compiling nix-compat-derive v0.1.0 (/tmp/build/crunch/vendor/nix-compat-derive)
   Compiling serde_bytes v0.11.19
   Compiling proc-macro-error-attr2 v2.0.0
   Compiling nom v8.0.0
   Compiling paste v1.0.15
   Compiling arraydeque v0.5.1
   Compiling aliasable v0.1.3
   Compiling arrayvec v0.5.2
   Compiling wu-manber v0.1.0 (https://github.com/tvlfyi/wu-manber.git#0d5b22be)
   Compiling typed-arena v2.0.2
   Compiling pretty v0.12.5
   Compiling ouroboros v0.18.5
   Compiling saphyr-parser v0.0.6
   Compiling proc-macro-error2 v2.0.1
   Compiling gix-diff v0.61.0
   Compiling gix-refspec v0.39.0
   Compiling gix-credentials v0.37.1
   Compiling logos v0.15.1
   Compiling malachite v0.6.1
   Compiling derive_builder_macro v0.20.2
   Compiling codespan v0.13.1
   Compiling nickel-lang-vector v0.1.0
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling crunch-attestation v0.1.0 (/tmp/build/crunch/crates/crunch-attestation)
   Compiling gix-discover v0.49.0
   Compiling nickel-lang-core v0.16.1
   Compiling nix v0.29.0
   Compiling uluru v3.1.0
   Compiling clru v0.6.3
   Compiling serde_spanned v1.1.1
   Compiling vte v0.14.1
   Compiling snix-store v0.1.0 (/tmp/build/crunch/vendor/snix-store)
   Compiling bumpalo v3.20.2
   Compiling simple-counter v0.1.0
   Compiling rustix v0.38.44
   Compiling unsafe-libyaml v0.2.11
   Compiling serde_yaml v0.9.34+deprecated
   Compiling strip-ansi-escapes v0.2.1
   Compiling gix-pack v0.68.0
   Compiling toml v0.9.12+spec-1.1.0
   Compiling ppv-lite86 v0.2.21
   Compiling gix-dir v0.23.0
   Compiling derive_builder v0.20.2
   Compiling gix-transport v0.55.1
   Compiling getset v0.1.6
   Compiling gix-config v0.54.0
   Compiling gix-worktree-stream v0.30.0
   Compiling strum_macros v0.26.4
   Compiling gix-shallow v0.10.0
   Compiling gix-negotiate v0.29.0
   Compiling rand_core v0/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/ymhmp1lbams1c2dgn748vklzbj2zmzkd-mantle
.6.4
   Compiling sha-1 v0.10.1
   Compiling lru v0.16.4
   Compiling typed-builder-macro v0.22.0
   Compiling maybe-async v0.2.10
   Compiling io-close v0.3.7
   Compiling json_scanner v0.1.0
   Compiling strum v0.26.3
   Compiling count-write v0.1.0
   Compiling linux-raw-sys v0.4.15
   Compiling snix-build v0.1.0 (/tmp/build/crunch/vendor/snix-build)
   Compiling indoc v2.0.7
   Compiling crc-catalog v2.4.0
   Compiling crc v3.4.0
   Compiling typed-builder v0.22.0
   Compiling oci-spec v0.7.1
   Compiling gix-worktree-state v0.28.0
   Compiling gix-protocol v0.59.0
   Compiling rand_chacha v0.3.1
   Compiling gix-submodule v0.28.0
   Compiling gix-archive v0.30.0
   Compiling gix-status v0.28.0
   Compiling gix-odb v0.78.0
   Compiling gix-blame v0.11.0
   Compiling itertools v0.12.1
   Compiling ureq-proto v0.6.0
   Compiling webpki-roots v1.0.6
   Compiling gix-mailmap v0.32.0
   Compiling uuid v1.23.0
   Compiling fs2 v0.4.3
   Compiling twox-hash v2.1.2
   Compiling utf8-zero v0.8.1
   Compiling ureq v3.3.0
   Compiling ruzstd v0.8.2
   Compiling crunch-store v0.1.0 (/tmp/build/crunch/crates/crunch-store)
   Compiling gix v0.81.0
   Compiling rand v0.8.6
   Compiling gethostname v0.5.0
   Compiling lzma-rs v0.3.0
   Compiling tar v0.4.45
   Compiling bzip2-rs v0.1.2
   Compiling toml_datetime v0.6.11
   Compiling serde_spanned v0.6.9
   Compiling toml_write v0.1.2
   Compiling toml_edit v0.22.27
   Compiling crunch-build v0.1.0 (/tmp/build/crunch/crates/crunch-build)
   Compiling crunch-glue v0.1.0 (/tmp/build/crunch/crates/crunch-glue)
   Compiling crunch-project-core v0.1.0 (/tmp/build/crunch/crates/crunch-project-core)
   Compiling crunch-shell-core v0.1.0 (/tmp/build/crunch/crates/crunch-shell-core)
   Compiling crunch-shell v0.1.0 (/tmp/build/crunch/crates/crunch-shell)
   Compiling crunch-project v0.1.0 (/tmp/build/crunch/crates/crunch-project)
   Compiling toml v0.8.23
   Compiling crunch-bootstrap-core v0.1.0 (/tmp/build/crunch/crates/crunch-bootstrap-core)
   Compiling crunch-release-core v0.1.0 (/tmp/build/crunch/crates/crunch-release-core)
   Compiling nickel-lang v2.0.0
   Compiling crunch-eval v0.1.0 (/tmp/build/crunch/crates/crunch-eval)
   Compiling crunch-pipeline v0.1.0 (/tmp/build/crunch/crates/crunch-pipeline)
   Compiling mantle v0.1.0 (/tmp/build/crunch)
    Finished `release` profile [optimized] target(s) in 9m 08s
=== Installing ===
=== Path leak scan ===
=== Verify ===
-rwxr-xr-x    1 nixbld   nixbld    63530144 Jun 29 17:56 /nix/store/jfnzr7khgspxcivcdpaxr94rvgifbr58-mantle/bin/mantle
mantle 0.1.0

--- end log ---
hermeticity: strict (no degraded facts)
self-build-proof: progress=crunch-build-done

[4/4] Verifying output...
verifying /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/ymhmp1lbams1c2dgn748vklzbj2zmzkd-mantle/bin/mantle...
  binary OK
self-build-proof: provider-mode=legacy-fetch
self-build-proof: hermeticity-mode=strict
self-build-proof: invoking-binary=/home/brittonr/.cargo-target/repo-targets/mantle/self-hosting-proof/work/tmp/.tmp9xxeXr/stage1-mantle
self-build-proof: staged-source=/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/zi7ps2paplpbljym0i0z8jkdq0ymibxq-mantle-src
self-build-proof: bwrap-source=mantle-built:/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/pys6ig7mb98pf53s7cq35iafz2wll35f-bwrap/bin
self-build-proof: fallback-event=none
self-build-proof: protected-transition=bootstrap-tools-selected
self-build-proof: protected-transition-bwrap-path=/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/pys6ig7mb98pf53s7cq35iafz2wll35f-bwrap/bin/bwrap
self-build-proof: protected-transition-bwrap-digest=93cacbf9a523c439d160745228073492be1121b62074889345e676858fd1adf2
self-build-proof: protected-transition-bwrap-store-name=pys6ig7mb98pf53s7cq35iafz2wll35f-bwrap
self-build-proof: protected-transition-busybox-path=/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/wp1dyh7nl2ld4yl0ia730cdfm7js9mza-busybox/bin/busybox
self-build-proof: protected-transition-busybox-digest=ce321b321d0f06650ebbb24b834f4d2d600f5765d9f5828fb09c41a3ccd31ed5
self-build-proof: protected-transition-busybox-store-name=wp1dyh7nl2ld4yl0ia730cdfm7js9mza-busybox
self-build-proof: protected-seccomp-event=none
self-build-proof: busybox-path=/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/wp1dyh7nl2ld4yl0ia730cdfm7js9mza-busybox/bin/busybox
self-build-proof: output-binary=/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/ymhmp1lbams1c2dgn748vklzbj2zmzkd-mantle/bin/mantle
self-build-proof: stagex-metadata=none

=== self-build complete ===
stage2 audit: /home/brittonr/git/mantle/target/test-audit/self-hosting/stage2-2021944-1782755822176160354
stage2 diagnostics: /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/stage2-diagnostics.txt
stage2 stdout: /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/stage2-stdout.txt
stage2 stderr: /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/stage2-stderr.txt
stage2 binary: /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/ymhmp1lbams1c2dgn748vklzbj2zmzkd-mantle/bin/mantle

thread 'self_hosting_stage0_stage1_stage2' (2021945) panicked at tests/self_hosting.rs:3656:5:
stage2 build log must use the exact reported mantle-built bwrap.
expected: Using mantle-built bwrap: /nix/store/pys6ig7mb98pf53s7cq35iafz2wll35f-bwrap/bin
audit bundle: /home/brittonr/git/mantle/target/test-audit/self-hosting/stage2-2021944-1782755822176160354
diagnostics: /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/stage2-diagnostics.txt
stdout: /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/stage2-stdout.txt
stderr: /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/stage2-stderr.txt
saved diagnostics snapshot:
stage: stage2
exit_code: 0
command: /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/stage1-mantle --verbose --log-level info --store /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store --state-dir /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/state2 --nix-compat self-build --source-store-path /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/zi7ps2paplpbljym0i0z8jkdq0ymibxq-mantle-src --bootstrap-bwrap-path /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/pys6ig7mb98pf53s7cq35iafz2wll35f-bwrap/bin/bwrap --bootstrap-busybox-path /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/wp1dyh7nl2ld4yl0ia730cdfm7js9mza-busybox/bin/busybox --no-substitute -j 4 --strict-hermetic
store_dir: /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store
state_dir: /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/state2
logs_dir: /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/state2/logs
proof lines:
  self-build-proof: progress=bootstrap-tool-start:bwrap.ncl
  self-build-proof: progress=bootstrap-tool-done:bwrap.ncl
  self-build-proof: progress=bootstrap-tool-start:busybox.ncl
  self-build-proof: progress=bootstrap-tool-done:busybox.ncl
  self-build-proof: progress=mantle-build-start
  self-build-proof: progress=crunch-build-done
  self-build-proof: provider-mode=legacy-fetch
  self-build-proof: hermeticity-mode=strict
  self-build-proof: invoking-binary=/home/brittonr/.cargo-target/repo-targets/mantle/self-hosting-proof/work/tmp/.tmp9xxeXr/stage1-mantle
  self-build-proof: staged-source=/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/zi7ps2paplpbljym0i0z8jkdq0ymibxq-mantle-src
  self-build-proof: bwrap-source=mantle-built:/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/pys6ig7mb98pf53s7cq35iafz2wll35f-bwrap/bin
  self-build-proof: fallback-event=none
  self-build-proof: protected-transition=bootstrap-tools-selected
  self-build-proof: protected-transition-bwrap-path=/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/pys6ig7mb98pf53s7cq35iafz2wll35f-bwrap/bin/bwrap
  self-build-proof: protected-transition-bwrap-digest=93cacbf9a523c439d160745228073492be1121b62074889345e676858fd1adf2
  self-build-proof: protected-transition-bwrap-store-name=pys6ig7mb98pf53s7cq35iafz2wll35f-bwrap
  self-build-proof: protected-transition-busybox-path=/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/wp1dyh7nl2ld4yl0ia730cdfm7js9mza-busybox/bin/busybox
  self-build-proof: protected-transition-busybox-digest=ce321b321d0f06650ebbb24b834f4d2d600f5765d9f5828fb09c41a3ccd31ed5
  self-build-proof: protected-transition-busybox-store-name=wp1dyh7nl2ld4yl0ia730cdfm7js9mza-busybox
  self-build-proof: protected-seccomp-event=none
  self-build-proof: busybox-path=/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/wp1dyh7nl2ld4yl0ia730cdfm7js9mza-busybox/bin/busybox
  self-build-proof: output-binary=/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/ymhmp1lbams1c2dgn748vklzbj2zmzkd-mantle/bin/mantle
  self-build-proof: stagex-metadata=none
store entries:
  00000000000000000000000000000000-stale-busybox
  00000000000000000000000000000000-stale-bwrap
  pys6ig7mb98pf53s7cq35iafz2wll35f-bwrap
  wp1dyh7nl2ld4yl0ia730cdfm7js9mza-busybox
  ymhmp1lbams1c2dgn748vklzbj2zmzkd-mantle
  zi7ps2paplpbljym0i0z8jkdq0ymibxq-mantle-src
log entries:
  sy7vv7vcdgcprfasd89hf3m55hdnr084-mantle.drv.log
stderr tail:
  ... 617 earlier lines omitted ...
     Compiling utf8-zero v0.8.1
     Compiling ureq v3.3.0
     Compiling ruzstd v0.8.2
     Compiling crunch-store v0.1.0 (/tmp/build/crunch/crates/crunch-store)
     Compiling gix v0.81.0
     Compiling rand v0.8.6
     Compiling gethostname v0.5.0
     Compiling lzma-rs v0.3.0
     Compiling tar v0.4.45
     Compiling bzip2-rs v0.1.2
     Compiling toml_datetime v0.6.11
     Compiling serde_spanned v0.6.9
     Compiling toml_write v0.1.2
     Compiling toml_edit v0.22.27
     Compiling crunch-build v0.1.0 (/tmp/build/crunch/crates/crunch-build)
     Compiling crunch-glue v0.1.0 (/tmp/build/crunch/crates/crunch-glue)
     Compiling crunch-project-core v0.1.0 (/tmp/build/crunch/crates/crunch-project-core)
     Compiling crunch-shell-core v0.1.0 (/tmp/build/crunch/crates/crunch-shell-core)
     Compiling crunch-shell v0.1.0 (/tmp/build/crunch/crates/crunch-shell)
     Compiling crunch-project v0.1.0 (/tmp/build/crunch/crates/crunch-project)
     Compiling toml v0.8.23
     Compiling crunch-bootstrap-core v0.1.0 (/tmp/build/crunch/crates/crunch-bootstrap-core)
     Compiling crunch-release-core v0.1.0 (/tmp/build/crunch/crates/crunch-release-core)
     Compiling nickel-lang v2.0.0
     Compiling crunch-eval v0.1.0 (/tmp/build/crunch/crates/crunch-eval)
     Compiling crunch-pipeline v0.1.0 (/tmp/build/crunch/crates/crunch-pipeline)
     Compiling mantle v0.1.0 (/tmp/build/crunch)
      Finished `release` profile [optimized] target(s) in 9m 08s
  === Installing ===
  === Path leak scan ===
  === Verify ===
  -rwxr-xr-x    1 nixbld   nixbld    63530144 Jun 29 17:56 /nix/store/jfnzr7khgspxcivcdpaxr94rvgifbr58-mantle/bin/mantle
  mantle 0.1.0
  
  --- end log ---
  hermeticity: strict (no degraded facts)
  self-build-proof: progress=crunch-build-done
  
  [4/4] Verifying output...
  verifying /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/ymhmp1lbams1c2dgn748vklzbj2zmzkd-mantle/bin/mantle...
    binary OK
  self-build-proof: provider-mode=legacy-fetch
  self-build-proof: hermeticity-mode=strict
  self-build-proof: invoking-binary=/home/brittonr/.cargo-target/repo-targets/mantle/self-hosting-proof/work/tmp/.tmp9xxeXr/stage1-mantle
  self-build-proof: staged-source=/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/zi7ps2paplpbljym0i0z8jkdq0ymibxq-mantle-src
  self-build-proof: bwrap-source=mantle-built:/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/pys6ig7mb98pf53s7cq35iafz2wll35f-bwrap/bin
  self-build-proof: fallback-event=none
  self-build-proof: protected-transition=bootstrap-tools-selected
  self-build-proof: protected-transition-bwrap-path=/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/pys6ig7mb98pf53s7cq35iafz2wll35f-bwrap/bin/bwrap
  self-build-proof: protected-transition-bwrap-digest=93cacbf9a523c439d160745228073492be1121b62074889345e676858fd1adf2
  self-build-proof: protected-transition-bwrap-store-name=pys6ig7mb98pf53s7cq35iafz2wll35f-bwrap
  self-build-proof: protected-transition-busybox-path=/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/wp1dyh7nl2ld4yl0ia730cdfm7js9mza-busybox/bin/busybox
  self-build-proof: protected-transition-busybox-digest=ce321b321d0f06650ebbb24b834f4d2d600f5765d9f5828fb09c41a3ccd31ed5
  self-build-proof: protected-transition-busybox-store-name=wp1dyh7nl2ld4yl0ia730cdfm7js9mza-busybox
  self-build-proof: protected-seccomp-event=none
  self-build-proof: busybox-path=/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/wp1dyh7nl2ld4yl0ia730cdfm7js9mza-busybox/bin/busybox
  self-build-proof: output-binary=/home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/ymhmp1lbams1c2dgn748vklzbj2zmzkd-mantle/bin/mantle
  self-build-proof: stagex-metadata=none
  
  === self-build complete ===
stdout tail:
  /home/brittonr/git/mantle/target/self-hosting-proof/work/tmp/.tmp9xxeXr/store/ymhmp1lbams1c2dgn748vklzbj2zmzkd-mantle

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test self_hosting_stage0_stage1_stage2 ... FAILED

failures:

failures:
    self_hosting_stage0_stage1_stage2

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 49 filtered out; finished in 2805.89s

error: test failed, to rerun pass `-p mantle --test self_hosting`
~~~

Exit status: 101
