# Diagnosis: the post-build stall in route-v2 realizations

Diagnostic only, not evidence of a fix. On 2026-11-22 I re-ran the Octet Aeneas consumer job on route
`cache-inputs-build-roots-v2` with `--build-cores 4`. The inputs were the same as
`build-or-reuse/pilot-v2-cores4`, and the binary was the checked Mantle 8ae03535b
(`/nix/store/0j2489gc…-mantle-0.1.0/bin/mantle`, identical realize code to `main` 35b90f9b3).
`ptracer_any.so` was preloaded only so `eu-stack` could read thread stacks under Yama
`ptrace_scope=1`. `run.sh` launched the run and `watch.sh` sampled it.

## Timeline

| Time (EST) | Observation |
|---|---|
| 17:55:39 | realize started (`logs/realize.txt`) |
| 18:19:04 | sandbox started; castore FUSE mount `0:78` at `<state>/build-work/sandbox-remote/…/host_inputs_dir`, `shared:2408`, `user_id=1555`, no `allow_other` |
| ≈18:58:40 | build finished: no bwrap processes, realize CPU time frozen at 44:03 (`samples/io.txt`) |
| 18:58:55, 19:00:06 | stall samples 1 and 2 (`samples/stall-*`) |
| 19:06:11.708 | `/sys/fs/fuse/connections/78` disappeared (`samples/conn78.txt`); the FUSE threads exited and the tokio workers resumed |
| 19:06:19 | realize exited 0; output ingestion and receipt took 8 s |

The stall lasted about 7.5 minutes. The first Octet run showed the same gap (output written 16:33:27, castore ingestion
16:41). The Terrane WP8 job showed 42 s.

## Where the realize waits (`samples/stall-2/eu-stack.txt`, `threads.txt`)

- One tokio worker: `snix_build::bwrap::Bwrap::run_inner` → drop of the inputs guard →
  `<snix_castore::fs::fuse::FuseDaemon as Drop>::drop` → `FuseDaemon::unmount` →
  `threadpool::ThreadPool::join` → futex wait.
- Four `fuse_server` threads: `fuse_backend_rs::…::FuseChannel::get_request` → `mio::Poll::poll` → `epoll_wait`.
- Main thread: runtime `block_on`. The other 31 workers are idle-parked.

`FuseDaemon::unmount` calls `FuseSession::umount`, a lazy unmount (`umount2(MNT_DETACH)`, or
`fusermount3 --unmount --lazy`), and then joins the server threads. It never calls `FuseSession::wake`,
so the threads exit only when the kernel aborts the FUSE connection. The kernel aborts it only once the
lazily detached filesystem is no longer referenced anywhere.

## What still referenced the filesystem

During the stall:
- connection `78` still existed in fusectl, with `waiting=0`;
- no task anywhere was in uninterruptible sleep on FUSE;
- no process I can read had the mount in its mountinfo (1,185 readable, 28 unreadable: zombies,
  processes that exited mid-scan, and one service), or any descriptor, cwd, root or mapping on device
  `0:78`.

No live process held the filesystem. Because the mount was not `allow_other`, only processes with uid 1555 and
gid 998 could open files in it, and all of those were readable. The holder was therefore a mount copy in a
namespace that no process occupies.

### Holder: a concurrent Nix build's saved sandbox namespace

Nix 2.36.0 (the running `nix-daemon`) keeps each sandboxed build's pre-`pivot_root` mount namespace open
as `sandboxMountNamespace`. That namespace is a full copy of the host mount tree, which the builder
makes private (`mount(0, "/", 0, MS_PRIVATE | MS_REC, 0)`). The daemon closes it only in
`unprepareBuild()`, at the end of the build (`src/libstore/linux/build/linux-derivation-builder.cc`
at tag 2.36.0, lines 683-688, 745-746, 903-914 and 957-959). Because the copy is private, our lazy
unmount does not propagate to it. Any FUSE mount that exists when a Nix build starts therefore stays
referenced until that build ends.

Both stalls ended right after a Nix build that started while the inputs mount existed. Log birth and
modification times come from `/nix/var/log/nix/drvs`:

| Run | Inputs mount lifetime | Nix build (log created → finalized) | Stall ended |
|---|---|---|---|
| this run | 18:19:04 → ≈18:58:40 | `cairn-radicle-cob-format-compatibility-0.1.0.drv`, 18:57:34.098 → 19:06:11.509 | connection gone by 19:06:11.708 |
| first pilot run | ≈15:53 → 16:33:27 | `kiln-0.1.0.drv`, 16:17:51.964 → 16:41:33.826 | first post-stall state write 16:41:37 |

The stall length therefore depends on unrelated Nix builds on the host (42 s for the Terrane WP8 job).
Mantle cannot control those references, so it must not wait for them.

## Fix direction

After the lazy unmount, wake the session's channel threads (`FuseSession::wake`) so they exit
immediately. That closes their `/dev/fuse` descriptors, which aborts the connection: any late access
through a stale reference gets `ENOTCONN` instead of holding up the realization, and `join` returns at
once.
