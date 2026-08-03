use std::ffi::OsString;
use std::fs;
use std::io::ErrorKind;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::process::Output;
use std::process::Stdio;

use tokio::process::Command;

use crate::sandbox::InputsProvider;
use crate::sandbox::SandboxSpec;

const BWRAP_PROGRAM: &str = "bwrap";
const BWRAP_PATH_ENV: &str = "SNIX_BUILD_BWRAP";

const COMMON_BWRAP_ARGS: &[&str] = &[
    "--unshare-uts",
    "--hostname",
    "localhost",
    "--unshare-ipc",
    "--unshare-pid",
    // NOTE: --die-with-parent remains omitted because PR_SET_PDEATHSIG(SIGKILL)
    // fires when the spawning tokio worker thread exits, killing long-running
    // sandbox builds. Keep --as-pid-1 omitted too: bubblewrap's PID-1 reaper
    // must own command teardown, especially for short-lived builders inside a
    // nested user namespace.
    "--unshare-user",
    "--unshare-cgroup-try",
    // Prevent sandbox from gaining new privileges via setuid/setgid binaries
    // or other capability escalation.
    "--new-session",
    "--uid",
    "1000",
    "--gid",
    "100",
    "--clearenv",
    "--tmpfs",
    "/",
    "--dev",
    "/dev",
    "--tmpfs",
    "/dev/shm",
    "--proc",
    "/proc",
    "--tmpfs",
    "/tmp",
];

const RANDOM_DEVICE_MASK_ARGS: &[&str] = &[
    "--ro-bind",
    "/dev/null",
    "/dev/random",
    "--ro-bind",
    "/dev/null",
    "/dev/urandom",
];

const PROC_METADATA_MASK_ARGS: &[&str] = &[
    "--ro-bind-try",
    "/dev/null",
    "/proc/cpuinfo",
    "--ro-bind-try",
    "/dev/null",
    "/proc/meminfo",
    "--ro-bind-try",
    "/dev/null",
    "/proc/stat",
    "--ro-bind-try",
    "/dev/null",
    "/proc/loadavg",
    "--ro-bind-try",
    "/dev/null",
    "/proc/uptime",
    "--ro-bind-try",
    "/dev/null",
    "/proc/version",
];

const ETC_PASSWD: &[u8] = b"
root:x:0:0:Nix build user:/build:/noshell
nixbld:x:1000:100:Nix build user:/build:/noshell
nobody:x:65534:65534:Nobody:/:/noshell
";

const ETC_GROUP: &[u8] = b"
root:x:0:
nixbld:!:100:
nogroup:x:65534:
";

const ETC_HOSTS: &[u8] = b"
127.0.0.1 localhost
::1 localhost
";

const ETC_NSSWITCH: &[u8] = b"
hosts: files dns
services: files
";

const ETC_RESOLV_CONF: &[u8] = b"
nameserver 127.0.0.1
nameserver 8.8.8.8
";

const ETC_SERVICES: &[u8] = b"
tcpmux          1/tcp
echo            7/tcp
echo            7/udp
discard         9/tcp
discard         9/udp
systat          11/tcp
daytime         13/tcp
daytime         13/udp
qotd            17/tcp
chargen         19/tcp
chargen         19/udp
ftp-data        20/tcp
ftp             21/tcp
ssh             22/tcp
telnet          23/tcp
smtp            25/tcp
time            37/tcp
time            37/udp
nameserver      42/tcp
nicname         43/tcp
domain          53/tcp
domain          53/udp
bootps          67/udp
bootpc          68/udp
tftp            69/udp
gopher          70/tcp
http            80/tcp
kerberos        88/tcp
kerberos        88/udp
pop3            110/tcp
ident           113/tcp
sftp            115/tcp
nntp            119/tcp
ntp             123/udp
imap            143/tcp
snmp            161/udp
snmp-trap       162/udp
bgp             179/tcp
irc             194/tcp
ldap            389/tcp
https           443/tcp
smtps           465/tcp
submission      587/tcp
ldaps           636/tcp
imaps           993/tcp
pop3s           995/tcp
";

const SANDBOX_UMASK_BITS: u32 = 0o022;
const _: () = assert!(SANDBOX_UMASK_BITS <= 0o777);

/// Bubblewrap based sandbox executor.
///
/// It executes the sandbox command in separate uts, ipc, pid and user namespaces,
/// always runs as uid=1000(nixbld) and gid=100(nixbld) inside the namespace. Provides sane
/// defaults for various `/etc` files.
///
/// Network is optionally disabled with a separate network namespace based on the value of
/// [SandboxSpec::allow_network].
///
/// The root filesystem is tmpfs, has /dev and /proc.
///
/// The rest of the filesystem is based on the [SandboxSpec::scratches],
/// [SandboxSpec::additional_files] and [SandboxSpec::inputs_provider].
///
/// # Scratches
///
/// A list of read-write directories available inside the sandbox, these directories are also left
/// available on the host after the sandbox has finished.
///
/// # Additional files
///
/// A list of read-write files whose path currently *must* resolve into one of the Scratches.
///
/// # Build Inputs([SandboxSpec::inputs_provider])
///
/// A read-only directory that contains any files required by the sandboxed command, e.g
/// `/nix/store`.
/// Before the sandbox starts, the [SandboxSpec::inputs_provider] will have a chance to populate
/// this directory and clean up after the sandbox is stopped.
///
/// **Note**: If the build inputs directory overlaps with any of the scratches, an overlayfs mount
/// will be created for that scratch so it remains writable, i.e. the sandboxed command can create
/// new files/directories.
pub struct Bwrap {
    host_workdir: PathBuf,
    args: Vec<OsString>,
    inputs_provider: InputsProvider,
}

/// The result of running the sandbox.
pub struct SandboxOutcome {
    output: Output,
    scratch_dir: PathBuf,
}

impl SandboxOutcome {
    /// Status code, stderr, stdout, etc.
    pub fn output(&self) -> &Output {
        &self.output
    }

    /// Allows finding outputs produced by the sandboxed command.
    ///
    /// The command must write into one of the scratches.
    pub fn find_path(&self, path: impl AsRef<Path>) -> Option<PathBuf> {
        let path = self.scratch_dir.join(path);
        // Exists follows symlinks so may return false incorrectly, as nix builds are apparently
        // allowed to produce broken symlinks as their $out...
        // i.e. `runCommand "test" {} "ln -s IdontExist $out"` is a valid nix build.
        //
        // Additionally, SandboxOutcome values are handed out by builds **after** unmounting the
        // fuse store, which means that even valid symlinks can be "broken" during ingestion.
        if path.is_symlink() || path.exists() {
            Some(path)
        } else {
            None
        }
    }
}

#[cfg(unix)]
fn set_sandbox_umask(command: &mut Command) {
    // SAFETY: `pre_exec` runs in the forked child immediately before `execve`.
    // The closure performs one libc syscall (`umask`) and does not allocate or
    // touch shared Rust state.
    unsafe {
        command.pre_exec(|| {
            nix::libc::umask(SANDBOX_UMASK_BITS as nix::libc::mode_t);
            Ok(())
        });
    }
}

#[cfg(not(unix))]
fn set_sandbox_umask(_command: &mut Command) {}

fn choose_bwrap_program(env_value: Option<OsString>) -> OsString {
    match env_value {
        Some(value) if !value.is_empty() => value,
        _ => OsString::from(BWRAP_PROGRAM),
    }
}

fn bwrap_program() -> OsString {
    choose_bwrap_program(std::env::var_os(BWRAP_PATH_ENV))
}

fn annotate_bwrap_spawn_error(program: &OsString, error: std::io::Error) -> std::io::Error {
    if error.kind() != ErrorKind::NotFound {
        return error;
    }

    std::io::Error::new(
        ErrorKind::NotFound,
        format!(
            "bubblewrap executable '{}' not found; put '{}' on PATH or set {} to an executable bwrap path: {error}",
            program.to_string_lossy(),
            BWRAP_PROGRAM,
            BWRAP_PATH_ENV
        ),
    )
}

fn append_prevalidated_mount_args<'a>(
    args: &mut Vec<OsString>,
    mounts: impl IntoIterator<Item = &'a crate::sandbox::SandboxMount>,
) -> std::io::Result<()> {
    for mount in mounts {
        if !mount.host_path.is_absolute() || !mount.guest_path.is_absolute() {
            return Err(std::io::Error::other("sandbox mount paths must be absolute"));
        }
        if mount
            .guest_path
            .components()
            .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
        {
            return Err(std::io::Error::other("sandbox guest mount path must be clean"));
        }
        let bind_flag = if mount.read_only { "--ro-bind" } else { "--bind" };
        args.extend([
            "--dir".into(),
            mount.guest_path.clone().into(),
            bind_flag.into(),
            mount.host_path.clone().into(),
            mount.guest_path.clone().into(),
        ]);
    }
    Ok(())
}

impl Bwrap {
    // TODO(#132): support streaming std{err,out}
    /// Run the sandbox and return the result.
    pub async fn run(mut self) -> std::io::Result<SandboxOutcome> {
        let _guard = self.inputs_provider.provide_inputs(self.host_workdir.join("host_inputs_dir"))?;

        let program = bwrap_program();
        let mut command = Command::new(&program);
        command.args(self.args);
        // Make sure we've closed stdin otherwise builds can hang forever blocked on std io.
        command.stdin(Stdio::null());
        set_sandbox_umask(&mut command);
        let output = command.output().await.map_err(|error| annotate_bwrap_spawn_error(&program, error))?;

        Ok(SandboxOutcome {
            output,
            scratch_dir: self.host_workdir.join("scratches"),
        })
    }

    /// Constructor.
    pub fn initialize(spec: SandboxSpec) -> std::io::Result<Bwrap> {
        let scratch_dir = spec.host_workdir().join("scratches");
        fs::create_dir_all(&scratch_dir)?;
        let mut args: Vec<OsString> = COMMON_BWRAP_ARGS.iter().map(|s| s.into()).collect();
        if !spec.provide_random_devices() {
            args.extend(RANDOM_DEVICE_MASK_ARGS.iter().map(OsString::from));
        }
        if !spec.provide_proc_metadata() {
            args.extend(PROC_METADATA_MASK_ARGS.iter().map(OsString::from));
        }
        if !spec.allow_network() {
            args.push("--unshare-net".into());
        }
        for env in spec.env_vars() {
            args.extend([
                "--setenv".into(),
                env.key.clone().into(),
                str::from_utf8(&env.value).expect("invalid string in env").into(),
            ]);
        }

        let host_inputs_dir = spec.host_workdir().join("host_inputs_dir");
        fs::create_dir_all(&host_inputs_dir)?;
        args.extend([
            "--ro-bind".into(),
            Path::new("/").join(&host_inputs_dir).into(),
            Path::new("/").join(spec.inputs_provider().inputs_dir()).into(),
        ]);
        for scratch in spec.scratches() {
            let scratch_path = scratch_dir.join(scratch);
            fs::create_dir_all(&scratch_path)?;
            if scratch == spec.inputs_provider().inputs_dir() {
                let overlay_workdir = spec.host_workdir().join("overlay_workdir");
                fs::create_dir_all(&overlay_workdir)?;
                args.extend([
                    "--overlay-src".into(),
                    OsString::from(&host_inputs_dir),
                    "--overlay".into(),
                    scratch_path.into(),
                    overlay_workdir.into(),
                    Path::new("/").join(spec.inputs_provider().inputs_dir()).into(),
                ]);
            } else {
                args.extend([
                    "--bind".into(),
                    scratch_path.into(),
                    Path::new("/").join(scratch).into(),
                ]);
            }
        }
        append_prevalidated_mount_args(&mut args, spec.mounts())?;
        args.extend(["--chdir".into(), Path::new("/").join(spec.sandbox_workdir()).into()]);

        if let Some(shell) = spec.provide_shell() {
            args.extend_from_slice(&["--ro-bind".into(), shell.into(), "/bin/sh".into()]);
            // Also mount as /bin/busybox so that busybox applets can be
            // invoked via `busybox <applet>` or via symlinks named after
            // the applet pointing to /bin/busybox.
            args.extend_from_slice(&["--ro-bind".into(), shell.into(), "/bin/busybox".into()]);
        }

        for file in spec.additional_files() {
            let mut found = false;
            for scratch in spec.scratches() {
                if file.path.starts_with(scratch) {
                    found = true;
                }
            }
            if !found {
                return Err(std::io::Error::other(format!(
                    "Additional file does not belong to any scratch: {:?}",
                    file.path
                )));
            }
            // TODO: prevent files from escaping the sandbox, i.e. don't allow additional files
            // of this form: build/../../hello.
            let file_path = scratch_dir.join(&file.path);
            fs::create_dir_all(file_path.parent().expect("parent"))?;
            fs::write(&file_path, &file.contents)?;
        }
        let etc = &spec.host_workdir().join("etc");
        fs::create_dir_all(etc)?;
        fs::write(etc.join("passwd"), ETC_PASSWD)?;
        fs::write(etc.join("group"), ETC_GROUP)?;
        fs::write(etc.join("hosts"), ETC_HOSTS)?;
        fs::write(etc.join("nsswitch.conf"), ETC_NSSWITCH)?;

        args.extend([
            "--ro-bind".into(),
            etc.join("passwd").into(),
            "/etc/passwd".into(),
            "--ro-bind".into(),
            etc.join("group").into(),
            "/etc/group".into(),
        ]);
        if spec.allow_network() {
            fs::write(etc.join("resolv.conf"), ETC_RESOLV_CONF)?;
            fs::write(etc.join("services"), ETC_SERVICES)?;
            args.extend([
                "--ro-bind".into(),
                etc.join("hosts").into(),
                "/etc/hosts".into(),
                "--ro-bind".into(),
                etc.join("resolv.conf").into(),
                "/etc/resolv.conf".into(),
                "--ro-bind".into(),
                etc.join("services").into(),
                "/etc/services".into(),
                "--ro-bind".into(),
                etc.join("nsswitch.conf").into(),
                "/etc/nsswitch.conf".into(),
            ]);
        } else {
            // Use predefined /etc/hosts like nix does.
            // Among other things it is required for libuv getaddrinfo() tests to pass.
            args.extend(["--ro-bind".into(), etc.join("hosts").into(), "/etc/hosts".into()]);
        }
        args.extend(spec.command().into_iter().map(|s| s.into()));

        Ok(Self {
            host_workdir: spec.host_workdir().into(),
            args,
            inputs_provider: spec.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXPECTED_PROC_MASKS: &[&str] = &[
        "/proc/cpuinfo",
        "/proc/meminfo",
        "/proc/stat",
        "/proc/loadavg",
        "/proc/uptime",
        "/proc/version",
    ];

    fn minimal_spec(host_workdir: &Path, allow_network: bool) -> SandboxSpec {
        SandboxSpec::builder()
            .host_workdir(host_workdir)
            .command(["/bin/sh", "-c", "true"])
            .sandbox_workdir("build")
            .scratches(["build"])
            .allow_network(allow_network)
            .build()
    }

    fn arg_strings(bwrap: &Bwrap) -> Vec<String> {
        bwrap.args.iter().map(|arg| arg.to_string_lossy().into_owned()).collect()
    }

    fn bind_source_for(args: &[String], dest: &str) -> Option<String> {
        args.windows(3)
            .find(|window| window[0] == "--ro-bind" && window[2] == dest)
            .map(|window| window[1].clone())
    }

    #[test]
    fn choose_bwrap_program_prefers_explicit_env_path() {
        let explicit = OsString::from("/tools/bwrap");
        let selected = choose_bwrap_program(Some(explicit.clone()));

        assert_eq!(selected, explicit);
    }

    #[test]
    fn choose_bwrap_program_falls_back_for_empty_or_missing_env() {
        assert_eq!(choose_bwrap_program(None), OsString::from(BWRAP_PROGRAM));
        assert_eq!(choose_bwrap_program(Some(OsString::new())), OsString::from(BWRAP_PROGRAM));
    }

    #[test]
    fn annotate_bwrap_spawn_error_mentions_env_for_missing_program() {
        let error = std::io::Error::new(ErrorKind::NotFound, "missing");
        let annotated = annotate_bwrap_spawn_error(&OsString::from("/missing/bwrap"), error);
        let rendered = annotated.to_string();

        assert_eq!(annotated.kind(), ErrorKind::NotFound);
        assert!(rendered.contains("/missing/bwrap"));
        assert!(rendered.contains(BWRAP_PATH_ENV));
    }

    #[test]
    fn annotate_bwrap_spawn_error_preserves_non_not_found_errors() {
        let error = std::io::Error::new(ErrorKind::PermissionDenied, "denied");
        let annotated = annotate_bwrap_spawn_error(&OsString::from("/locked/bwrap"), error);

        assert_eq!(annotated.kind(), ErrorKind::PermissionDenied);
        assert_eq!(annotated.to_string(), "denied");
    }

    #[test]
    fn common_bwrap_args_contains_hostname() {
        assert!(COMMON_BWRAP_ARGS.windows(2).any(|w| w == ["--hostname", "localhost"]));
    }

    #[test]
    fn common_bwrap_args_install_pid_namespace_reaper() {
        assert!(COMMON_BWRAP_ARGS.contains(&"--unshare-pid"));
        assert!(!COMMON_BWRAP_ARGS.contains(&"--as-pid-1"));
    }

    #[test]
    fn default_sandbox_contains_exact_proc_masks() {
        let tempdir = tempfile::tempdir().expect("tempdir must be available");
        let bwrap = Bwrap::initialize(minimal_spec(tempdir.path(), false)).expect("sandbox should initialize");
        let args = arg_strings(&bwrap);
        let observed: Vec<&str> = args
            .windows(3)
            .filter(|window| window[0] == "--ro-bind-try" && window[1] == "/dev/null")
            .map(|window| window[2].as_str())
            .filter(|target| target.starts_with("/proc/"))
            .collect();

        assert_eq!(observed, EXPECTED_PROC_MASKS);
        assert!(args.windows(2).any(|window| window == ["--proc", "/proc"]));
    }

    #[test]
    fn requested_proc_metadata_omits_proc_masks() {
        let tempdir = tempfile::tempdir().expect("tempdir must be available");
        let spec = SandboxSpec::builder()
            .host_workdir(tempdir.path())
            .command(["/bin/sh", "-c", "true"])
            .sandbox_workdir("build")
            .scratches(["build"])
            .provide_proc_metadata(true)
            .build();
        let bwrap = Bwrap::initialize(spec).expect("sandbox should initialize");
        let args = arg_strings(&bwrap);

        for target in EXPECTED_PROC_MASKS {
            assert!(bind_source_for(&args, target).is_none(), "unexpected mask for {target}");
        }
        assert!(args.windows(2).any(|window| window == ["--proc", "/proc"]));
    }

    #[test]
    fn proc_masks_keep_proc_self_accessible() {
        for target in EXPECTED_PROC_MASKS {
            assert!(!target.starts_with("/proc/self/"));
        }
        assert!(!PROC_METADATA_MASK_ARGS.iter().any(|arg| arg.starts_with("/proc/self/")));
    }

    #[test]
    fn default_sandbox_contains_random_device_masks() {
        let tempdir = tempfile::tempdir().expect("tempdir must be available");
        let bwrap = Bwrap::initialize(minimal_spec(tempdir.path(), false)).expect("sandbox should initialize");
        let args = arg_strings(&bwrap);
        let masked = ["/dev/random", "/dev/urandom"];

        for path in &masked {
            assert_eq!(bind_source_for(&args, path).as_deref(), Some("/dev/null"), "/dev mask missing for {path}");
        }
    }

    #[test]
    fn requested_random_devices_omit_random_device_masks() {
        let tempdir = tempfile::tempdir().expect("tempdir must be available");
        let spec = SandboxSpec::builder()
            .host_workdir(tempdir.path())
            .command(["/bin/sh", "-c", "true"])
            .sandbox_workdir("build")
            .scratches(["build"])
            .provide_random_devices(true)
            .build();
        let bwrap = Bwrap::initialize(spec).expect("sandbox should initialize");
        let args = arg_strings(&bwrap);

        assert!(bind_source_for(&args, "/dev/random").is_none());
        assert!(bind_source_for(&args, "/dev/urandom").is_none());
    }

    #[test]
    fn common_bwrap_args_contains_dev_shm_isolation() {
        assert!(COMMON_BWRAP_ARGS.windows(2).any(|w| w == ["--tmpfs", "/dev/shm"]));
    }

    #[test]
    fn common_bwrap_args_contains_cgroup_unshare() {
        assert!(COMMON_BWRAP_ARGS.contains(&"--unshare-cgroup-try"));
    }

    #[test]
    fn common_bwrap_args_does_not_mount_sys() {
        for window in COMMON_BWRAP_ARGS.windows(2) {
            let bind_types = ["--bind", "--ro-bind", "--ro-bind-try", "--dev-bind"];
            if bind_types.contains(&window[0]) {
                assert!(!window[1].starts_with("/sys"), "must not bind-mount host /sys: found {:?}", window);
            }
        }
    }

    #[test]
    fn synthetic_etc_files_are_not_empty() {
        assert!(!ETC_RESOLV_CONF.is_empty());
        assert!(!ETC_SERVICES.is_empty());
        assert!(ETC_RESOLV_CONF.windows("nameserver".len()).any(|w| w == b"nameserver"));
        assert!(ETC_SERVICES.windows("https".len()).any(|w| w == b"https"));
    }

    #[test]
    fn network_sandbox_binds_synthetic_resolv_conf_and_services() {
        let tempdir = tempfile::tempdir().expect("tempdir must be available");
        let bwrap = Bwrap::initialize(minimal_spec(tempdir.path(), true)).expect("network sandbox should initialize");
        let args = arg_strings(&bwrap);

        let resolv_source = bind_source_for(&args, "/etc/resolv.conf").expect("resolv.conf must be bound");
        let services_source = bind_source_for(&args, "/etc/services").expect("services must be bound");
        assert_ne!(resolv_source, "/etc/resolv.conf");
        assert_ne!(services_source, "/etc/services");
        assert_eq!(fs::read(&resolv_source).expect("synthetic resolv.conf readable"), ETC_RESOLV_CONF);
        assert_eq!(fs::read(&services_source).expect("synthetic services readable"), ETC_SERVICES);
    }

    #[test]
    fn non_network_sandbox_omits_resolv_conf_and_services() {
        let tempdir = tempfile::tempdir().expect("tempdir must be available");
        let bwrap =
            Bwrap::initialize(minimal_spec(tempdir.path(), false)).expect("non-network sandbox should initialize");
        let args = arg_strings(&bwrap);

        assert!(args.iter().any(|arg| arg == "--unshare-net"));
        assert!(bind_source_for(&args, "/etc/resolv.conf").is_none());
        assert!(bind_source_for(&args, "/etc/services").is_none());
        assert!(!tempdir.path().join("etc/resolv.conf").exists());
        assert!(!tempdir.path().join("etc/services").exists());
    }
}
