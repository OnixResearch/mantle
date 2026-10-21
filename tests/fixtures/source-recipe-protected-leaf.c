/* Parent-built smoke input, not a bootstrap compiler or an admitted provider.
 * cc -O2 -nostdlib -static -fno-builtin -fno-stack-protector -fno-pie -no-pie
 *    -Wl,-e,_start source-recipe-protected-leaf.c -o protected-leaf
 * No interpreter, libc, runtime mounts, environment, or ambient tools needed.
 */
#if defined(__x86_64__)
#define NR_READ 0
#define NR_LSEEK 8
#define NR_FCNTL 72
#define NR_FCHMOD 91
#define NR_WRITE 1
#define NR_OPENAT 257
#define NR_READLINKAT 267
#define NR_GETCWD 79
#define NR_CLOSE 3
#define NR_EXECVE 59
#define NR_CLONE 56
#define NR_EXIT 60
__asm__(".global _start\n_start:\nmov %rsp,%rdi\nand $-16,%rsp\ncall entry\nud2\n");
static long call3(long nr, long a, long b, long c) {
    long result;
    __asm__ volatile("syscall" : "=a"(result) : "a"(nr), "D"(a), "S"(b), "d"(c) : "rcx", "r11", "memory");
    return result;
}
#elif defined(__aarch64__)
#define NR_READ 63
#define NR_LSEEK 62
#define NR_FCNTL 25
#define NR_FCHMOD 52
#define NR_WRITE 64
#define NR_OPENAT 56
#define NR_READLINKAT 78
#define NR_GETCWD 17
#define NR_CLOSE 57
#define NR_EXECVE 221
#define NR_CLONE 220
#define NR_EXIT 93
__asm__(".global _start\n_start:\nmov x0,sp\nbl entry\nbrk #0\n");
static long call3(long nr, long a, long b, long c) {
    register long x0 __asm__("x0") = a;
    register long x1 __asm__("x1") = b;
    register long x2 __asm__("x2") = c;
    register long x8 __asm__("x8") = nr;
    __asm__ volatile("svc #0" : "+r"(x0) : "r"(x1), "r"(x2), "r"(x8) : "memory");
    return x0;
}
#else
#error unsupported protected-leaf smoke architecture
#endif
static long create_output_mode(const char *path, long permissions) {
#if defined(__x86_64__)
    long result;
    register long mode __asm__("r10") = permissions;
    __asm__ volatile("syscall" : "=a"(result) : "a"(NR_OPENAT), "D"(-100L), "S"(path), "d"(193L), "r"(mode) : "rcx", "r11", "memory");
    return result;
#else
    register long x0 __asm__("x0") = -100;
    register const char *x1 __asm__("x1") = path;
    register long x2 __asm__("x2") = 193;
    register long x3 __asm__("x3") = permissions;
    register long x8 __asm__("x8") = NR_OPENAT;
    __asm__ volatile("svc #0" : "+r"(x0) : "r"(x1), "r"(x2), "r"(x3), "r"(x8) : "memory");
    return x0;
#endif
}
static long create_output(const char *path) {
    return create_output_mode(path, 0600);
}
static long read_executable(char *path, long capacity) {
#if defined(__x86_64__)
    long result;
    register long fourth __asm__("r10") = capacity;
    __asm__ volatile("syscall" : "=a"(result) : "a"(NR_READLINKAT), "D"(-100L), "S"("/proc/self/exe"), "d"(path), "r"(fourth) : "rcx", "r11", "memory");
    return result;
#else
    register long x0 __asm__("x0") = -100;
    register const char *x1 __asm__("x1") = "/proc/self/exe";
    register char *x2 __asm__("x2") = path;
    register long x3 __asm__("x3") = capacity;
    register long x8 __asm__("x8") = NR_READLINKAT;
    __asm__ volatile("svc #0" : "+r"(x0) : "r"(x1), "r"(x2), "r"(x3), "r"(x8) : "memory");
    return x0;
#endif
}
static void finish(long status) {
    call3(NR_EXIT, status, 0, 0);
    __builtin_unreachable();
}
static void message(long fd, const char *bytes, long count) {
    if (call3(NR_WRITE, fd, (long)bytes, count) != count) finish(91);
}
static int same(const char *left, const char *right) {
    for (unsigned long i = 0; i < 65536; ++i) {
        if (left[i] != right[i]) return 0;
        if (left[i] == 0) return 1;
    }
    return 0;
}
__attribute__((used, noreturn)) void entry(const long *stack) {
    const char *const *argv = (const char *const *)(stack + 1);
    if (stack[0] != 2 && (stack[0] != 4 || argv[1][0] != 'd')) finish(92);
    if (argv[1][0] != 'm' && argv[1][0] != 'b') message(2, "leaf-stderr\n", 12);
    switch (argv[1][0]) {
    case 's':
        message(1, "leaf-ok\n", 8);
        finish(0);
    case 'd': {
        static const char expected[] = "repo=https://example.invalid/pkg\nlicense=MIT/Apache-2.0\n";
        const char *const *environment = argv + stack[0] + 1;
        unsigned long found = 0, count = 0;
        if (!same(argv[2], "https://example.invalid/pkg") || !same(argv[3], "/mantle-rust-runtime/metadata.txt")) finish(99);
        for (; count < 5 && environment[count]; ++count) {
            if (same(environment[count], "CARGO_PKG_REPOSITORY=https://example.invalid/pkg")) found |= 1;
            else if (same(environment[count], "CARGO_PKG_LICENSE=MIT/Apache-2.0")) found |= 2;
            else if (same(environment[count], "LD_LIBRARY_PATH=/mantle-rust-runtime")) found |= 4;
            else if (same(environment[count], "PWD=/work")) found |= 8;
            else finish(99);
        }
        if (count != 4 || found != 15) finish(99);
        long input = call3(NR_OPENAT, -100, (long)argv[3], 0);
        char bytes[sizeof(expected)];
        if (input < 0 || call3(NR_READ, input, (long)bytes, sizeof(bytes)) != sizeof(expected) - 1) finish(99);
        for (unsigned long i = 0; i < sizeof(expected) - 1; ++i) {
            if (bytes[i] != expected[i]) finish(99);
        }
        if (call3(NR_READ, input, (long)bytes, 1) != 0 || call3(NR_CLOSE, input, 0, 0) != 0) finish(99);
        message(1, "operands-ok\n", sizeof("operands-ok\n") - 1);
        finish(0);
    }
    case 'i': {
        static const char image[] = "/inputs/installation/bin/fixture-image";
        static const char metadata[] = "actual-cwd-source\n";
        char bytes[128];
        if (!same(argv[0], image)) finish(96);
        long length = read_executable(bytes, sizeof(bytes));
        if (length <= 0 || length >= sizeof(bytes)) finish(96);
        bytes[length] = 0;
        if (!same(bytes, image)) finish(96);
        length = call3(NR_GETCWD, (long)bytes, sizeof(bytes), 0);
        if (length <= 0 || length > sizeof(bytes) || !same(bytes, "/inputs/workspace/package")) finish(96);
        long input = call3(NR_OPENAT, -100, (long)"metadata.txt", 0);
        if (input < 0 || call3(NR_READ, input, (long)bytes, sizeof(bytes)) != sizeof(metadata) - 1) finish(99);
        for (unsigned long i = 0; i < sizeof(metadata) - 1; ++i) {
            if (bytes[i] != metadata[i]) finish(99);
        }
        if (call3(NR_READ, input, (long)bytes, 1) != 0 || call3(NR_CLOSE, input, 0, 0) != 0) finish(99);
        message(1, "image-context-ok\n", sizeof("image-context-ok\n") - 1);
        finish(0);
    }
    case 'l': {
        const char *directory = same(argv[1], "local-output")
            ? "/outputs/generated/package" : "/inputs/workspace/package";
        static const char expected[] = "actual-cwd-source\n";
        char cwd[64];
        char bytes[sizeof(expected)];
        long length = call3(NR_GETCWD, (long)cwd, sizeof(cwd), 0);
        if (length <= 0 || length > sizeof(cwd) || !same(cwd, directory)) finish(96);
        long input = call3(NR_OPENAT, -100, (long)"metadata.txt", 0);
        if (input < 0 || call3(NR_READ, input, (long)bytes, sizeof(bytes)) != sizeof(expected) - 1) finish(99);
        for (unsigned long i = 0; i < sizeof(expected) - 1; ++i) {
            if (bytes[i] != expected[i]) finish(99);
        }
        if (call3(NR_READ, input, (long)bytes, 1) != 0 || call3(NR_CLOSE, input, 0, 0) != 0) finish(99);
        message(1, "cwd-operands-ok\n", sizeof("cwd-operands-ok\n") - 1);
        finish(0);
    }
    case 't': {
        static const char bytes[] = "actual-tmp-output\n";
        long output = create_output("/tmp/perl-io-positive");
        if (output < 0) finish(97);
        message(output, bytes, sizeof(bytes) - 1);
        if (call3(NR_CLOSE, output, 0, 0) != 0) finish(98);
        message(1, "tmp-write-ok\n", sizeof("tmp-write-ok\n") - 1);
        finish(0);
    }
    case 'm':
        message(1, "out-a\n", 6);
        message(2, "err-a\n", 6);
        message(1, "out-b\n", 6);
        message(2, "err-b\n", 6);
        finish(0);
    case 'u': {
        long output = create_output_mode("/outputs/generated/created", 0666);
        if (output < 0) finish(97);
        message(output, "original-mask\n", sizeof("original-mask\n") - 1);
        if (call3(NR_CLOSE, output, 0, 0) != 0) finish(98);
        finish(0);
    }
    case 'b': {
        char input[3], next;
        if (call3(NR_READ, 0, (long)input, sizeof(input)) != sizeof(input)) finish(97);
        if (input[0] != 'a' || input[1] != 'b' || input[2] != 'c') finish(97);
        if (call3(NR_READ, 9, (long)&next, 1) != 1 || next != 'd') finish(97);
        if (call3(NR_LSEEK, 0, 0, 1) != 4) finish(98);
        if (call3(NR_READ, 4, (long)&next, 1) != -9 || call3(NR_READ, 6, (long)&next, 1) != -9) finish(98);
        message(3, "discarded", sizeof("discarded") - 1);
        if (call3(NR_READ, 3, (long)&next, 1) != 0 || call3(NR_LSEEK, 3, 17, 0) != 0) finish(98);
        message(1, "out", 3);
        message(2, "err", 3);
        if (call3(NR_LSEEK, 1, 2, 0) != 2 || call3(NR_LSEEK, 2, 0, 1) != 2) finish(98);
        if (call3(NR_FCNTL, 5, 3, 0) & (131072L | 2097152L)) finish(98);
        if (call3(NR_FCNTL, 5, 4, 3072) != 0) finish(98);
        message(5, "fd-five\n", sizeof("fd-five\n") - 1);
        if (call3(NR_LSEEK, 63, 0, 1) != 13) finish(98);
        finish(0);
    }
    case 'n':
        message(1, "normal-probe\n", sizeof("normal-probe\n") - 1);
        finish(123);
    case 'o': {
        static const char source[] = "int observed(void) { return 17; }\n";
        char input[3];
        if (call3(NR_READ, 0, (long)input, sizeof(input)) != sizeof(input)) finish(97);
        if (input[0] != 'a' || input[1] != 'b' || input[2] != 'c') finish(97);
        long output = create_output("observed.c");
        if (output < 0) finish(97);
        message(output, source, sizeof(source) - 1);
        if (call3(NR_CLOSE, output, 0, 0) != 0) finish(98);
        message(1, "observed-output\n", sizeof("observed-output\n") - 1);
        message(2, "observed-error\n", sizeof("observed-error\n") - 1);
        if (call3(NR_LSEEK, 1, 2, 0) != 2 || call3(NR_LSEEK, 2, 4, 0) != 4) finish(98);
        finish(0);
    }
    case 'r': {
        static const char request[] = "{\"arguments\":[{\"parts\":[{\"kind\":\"literal\",\"text\":\"probe0\"}]}],\"environment\":{}}";
        char bytes[4096];
        unsigned long copied = 0;
        long input = call3(NR_OPENAT, -100, (long)"/inputs/seed", 0);
        long output = create_output("/outputs/executable");
        if (input < 0 || output < 0) finish(97);
        for (;;) {
            long count = call3(NR_READ, input, (long)bytes, sizeof(bytes));
            if (count < 0) finish(97);
            if (count == 0) break;
            copied += (unsigned long)count;
            if (copied > 1048576) finish(97);
            message(output, bytes, count);
        }
        if (copied == 0 || call3(NR_FCHMOD, output, 0755, 0) != 0) finish(97);
        if (call3(NR_CLOSE, input, 0, 0) != 0) finish(98);
        if (call3(NR_CLOSE, output, 0, 0) != 0) finish(98);
        message(1, request, sizeof(request) - 1);
        finish(0);
    }
    case 'w': {
        static const char workspace[] = "/outputs/generated";
        static const char source[] = "int generated(void) { return 42; }\n";
        char cwd[sizeof(workspace)];
        if (call3(NR_GETCWD, (long)cwd, sizeof(cwd), 0) != sizeof(workspace)) finish(96);
        for (unsigned long i = 0; i < sizeof(workspace); ++i) {
            if (cwd[i] != workspace[i]) finish(96);
        }
        long fd = create_output("generated.c");
        if (fd < 0) finish(97);
        message(fd, source, sizeof(source) - 1);
        if (call3(NR_CLOSE, fd, 0, 0) != 0) finish(98);
        message(1, "workspace-ok\n", sizeof("workspace-ok\n") - 1);
        finish(0);
    }
    case 'v': {
        char executable[128];
        if (!same(argv[0], "/logical/source-program")) finish(96);
        long length = read_executable(executable, sizeof(executable));
        if (length <= 0 || length >= sizeof(executable)) finish(96);
        executable[length] = 0;
        if (!same(executable, "/inputs/program")) finish(96);
        message(1, "actual-argv0-ok\n", sizeof("actual-argv0-ok\n") - 1);
        finish(0);
    }
    case 'p': {
        static const char prefix[] = "probe";
        for (unsigned long i = 0; i < sizeof(prefix) - 1; ++i) {
            if (argv[1][i] != prefix[i]) finish(95);
        }
        if (argv[1][5] < '0' || argv[1][5] > '2' || argv[1][6] != 0) finish(95);
        message(1, "probe-ok\n", sizeof("probe-ok\n") - 1);
        finish(argv[1][5] - '0');
    }
    case 'e': {
        const char *next[] = {"/inputs/seed", "s", 0};
        if (call3(NR_EXECVE, (long)next[0], (long)next, 0) != -1) finish(93);
        message(1, "exec-denied\n", 12);
        finish(0);
    }
    case 'f':
        /* SIGCHLD with no CLONE_THREAD is a process child, not a thread. */
        if (call3(NR_CLONE, 17, 0, 0) != -1) finish(94);
        message(1, "clone-denied\n", 13);
        finish(0);
    case 'c':
        message(1, "cancel-ready\n", 13);
        for (;;) __asm__ volatile("" ::: "memory");
    default:
        finish(95);
    }
}
