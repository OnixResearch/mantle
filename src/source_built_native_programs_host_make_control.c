/* Appended to the shared shellvm library_source(), never a second shell VM.
 * Original GNU Make supplies every argv, environment and child completion.
 * No process is launched here: a missing real result pauses the source Make.
 */
#include <fcntl.h>
#include <sys/wait.h>
#include <signal.h>
#undef dup
#define MANTLE_NATIVE_MAKE_PHYSICAL 1
#include "mantle_make_control.h"

extern int mantle_original_make_main(int, char **);
static int make_control_active;
static Request make_initial_request;
static int make_control_status = -1;
struct MakeCompletion { pid_t token; int status; struct MakeCompletion *next; };
static struct MakeCompletion *make_completed, **make_completion_tail = &make_completed;

int mantle_make_active(void) { return make_control_active; }
int mantle_make_paths_active(void) { return make_control_active && mantle_paths_ready; }

static FILE *make_descriptor(int descriptor, int writable) {
    if (descriptor < 0) return NULL;
    int flags = mantle_descriptor_flags(descriptor);
    int fd = fcntl(descriptor, F_DUPFD_CLOEXEC, 64);
    if (fd < 0) die("duplicate Make descriptor: %s", strerror(errno));
    if (lseek(fd, 0, SEEK_CUR) < 0) {
        if (errno != ESPIPE || writable) die("nonseekable Make descriptor");
        FILE *input = fdopen(fd, "rb"), *snapshot = tmpfile();
        if (!input || !snapshot) die("Make stdin snapshot");
        transfer(input, snapshot);
        if (fclose(input) || fseek(snapshot, 0, SEEK_SET)) die("Make stdin snapshot close");
        return snapshot;
    }
    const char *mode = (flags & O_ACCMODE) == O_RDONLY ? "rb" : (flags & O_ACCMODE) == O_WRONLY ? "wb" : "r+b";
    FILE *stream = fdopen(fd, mode);
    if (!stream) die("Make descriptor stream: %s", strerror(errno));
    return stream;
}

static void make_close_environment(Env *environment) {
    for (int i = 0; i < 64; ++i) {
        if (!environment->fd[i]) continue;
        int first = 1;
        for (int j = 0; j < i; ++j)
            if (environment->fd[i] == environment->fd[j]) first = 0;
        if (first && fclose(environment->fd[i])) die("Make descriptor close");
    }
    clearvars(environment);
    free(environment->cwd);
}

/* Comparing inodes is insufficient: separate opens of one file have separate
 * OFDs. Observe shared F_SETFL state and restore it before original evaluation
 * resumes. No other operating-system child is running in this source helper. */
static int make_same_ofd(int first, int second) {
    if (first == second) return 1;
    int before = fcntl(first, F_GETFL), other = fcntl(second, F_GETFL);
    if (before < 0 || other < 0) die("Make OFD flags");
    if ((before & ~0x8000) != (other & ~0x8000)) return 0;
    if (fcntl(first, F_SETFL, before ^ O_NONBLOCK)) die("observe Make shared OFD");
    int changed = fcntl(first, F_GETFL), observed = fcntl(second, F_GETFL);
    if (fcntl(first, F_SETFL, before)) die("restore Make shared OFD flags");
    if (changed < 0 || observed < 0 || !((changed ^ before) & O_NONBLOCK)) die("Make OFD observation failed");
    return ((observed ^ other) & O_NONBLOCK) != 0;
}
pid_t mantle_make_child(char **argv, char **environment, int input, int output, int error) {
    if (!make_control_active || !argv || !argv[0]) die("invalid Make continuation entry");
    if (fflush(NULL)) die("flush Make streams");
    Env state = {0};
    state.cwd = mantle_make_paths_active() ? mantle_paths_getcwd(NULL, 0) : getcwd(NULL, 0);
    state.zero = argv[0];
    state.creation_mask = umask(0); umask(state.creation_mask); state.mask_valid = 1;
    if (!state.cwd) die("Make request cwd");
    for (char **entry = environment; entry && *entry; ++entry) {
        const char *equal = strchr(*entry, '=');
        if (!strncmp(*entry, "MANTLE_COMMAND_", 15)) continue;
        if (!equal || equal == *entry) die("invalid original Make environment");
        char *name = slice(*entry, equal - *entry);
        set(&state, name, equal + 1);
        variable(&state, name)->exported = 1;
        free(name);
    }
    int descriptors[64]; struct stat identities[64];
    for (int i = 0; i < 64; ++i) descriptors[i] = i;
    descriptors[0] = input; descriptors[1] = output; descriptors[2] = error;
    for (int i = 0; i < 64; ++i) {
        int descriptor = descriptors[i];
        if (descriptor < 0) continue;
        int cloexec = fcntl(descriptor, F_GETFD);
        if (cloexec < 0) { if (errno == EBADF && i >= 3) continue; die("original Make source descriptor"); }
        if ((cloexec & FD_CLOEXEC) && !(i < 3 && descriptor != i)) continue;
        state.fd_flags[i] = mantle_descriptor_flags(descriptor);
        state.fd_cloexec[i] = 0;
        if (fstat(descriptor, &identities[i])) die("Make source descriptor inode");
        int alias = -1;
        for (int j = 0; j < i; ++j) {
            if (!state.fd[j] || identities[j].st_dev != identities[i].st_dev || identities[j].st_ino != identities[i].st_ino) continue;
            if (make_same_ofd(descriptors[j], descriptor)) { alias = j; break; }
        }
        state.fd[i] = alias >= 0 ? state.fd[alias] : make_descriptor(descriptor, (state.fd_flags[i] & O_ACCMODE) != O_RDONLY);
    }
    state.in = state.fd[0]; state.out = state.fd[1]; state.err = state.fd[2];
    Args command = {0};
    while (argv[command.n]) ++command.n;
    command.v = argv;
    int status = external(&state, &command);
    make_close_environment(&state);
    if (request_ordinal > 100000000UL) die("Make continuation ordinal overflow");
    struct MakeCompletion *completion = alloc(sizeof *completion);
    completion->token = (pid_t)(request_ordinal + 1);
    completion->status = status;
    *make_completion_tail = completion;
    make_completion_tail = &completion->next;
    return completion->token;
}

pid_t mantle_make_wait(int *status, int nonblocking) {
    if (!make_control_active) return nonblocking ? waitpid(-1, status, WNOHANG) : wait(status);
    if (!make_completed) { if (nonblocking) return 0; errno = ECHILD; return -1; }
    struct MakeCompletion *completion = make_completed;
    make_completed = completion->next;
    if (!make_completed) make_completion_tail = &make_completed;
    /* The journal contains an observed normal process exit, not a guessed
     * success. Encode it only for the original source's WEXITSTATUS logic. */
    *status = completion->status << 8;
    pid_t token = completion->token;
    free(completion);
    return token;
}

void mantle_make_replace(char **argv, char **environment) {
    pid_t token = mantle_make_child(argv, environment, 0, 1, 2);
    int status;
    if (mantle_make_wait(&status, 0) != token) die("Make replacement completion order");
    mantle_make_exit(WEXITSTATUS(status));
}

int mantle_make_cancel(pid_t token, int signal) {
    if (!make_control_active) return kill(token, signal);
    /* No operating-system child exists for these completion tokens. Every
     * queued result is already an observed completed protected leaf. */
    for (struct MakeCompletion *completion = make_completed; completion; completion = completion->next)
        if (completion->token == token) return 0;
    errno = ESRCH;
    return -1;
}

int mantle_make_pipe(int descriptors[2]) {
    if (!make_control_active) return pipe(descriptors);
    /* $(shell) must read after source Make has received the actual result.
     * Independent file offsets preserve its pipe API without a hidden child
     * or a bounded-capacity pipe that would block before the reader starts. */
    char path[] = "/tmp/mantle-make-output-XXXXXX";
    int output = mkstemp(path);
    if (output < 0) return -1;
    int writer = open(path, O_WRONLY);
    if (writer < 0 || dup2(writer, output) < 0) { int saved = errno; if (writer >= 0) close(writer); close(output); unlink(path); errno = saved; return -1; }
    if (close(writer)) { int saved = errno; close(output); unlink(path); errno = saved; return -1; }
    int input = open(path, O_RDONLY);
    int saved = errno;
    if (unlink(path)) { saved = errno; close(output); if (input >= 0) close(input); errno = saved; return -1; }
    if (input < 0) { close(output); errno = saved; return -1; }
    descriptors[0] = input; descriptors[1] = output;
    return 0;
}

static void make_capture_source_result(void) {
    if (!make_control_active || make_control_status < 0) return;
    if (fflush(NULL)) die("flush original Make descriptors");
    for (int i = 0; i < 64; ++i) if (make_initial_request.env.fd[i])
        make_initial_request.env.fd_flags[i] = mantle_descriptor_flags(fileno(make_initial_request.env.fd[i]));
    char *result = join(output_root, "source-result");
    write_result(result, &make_initial_request, make_control_status, NULL, NULL);
    free(result);
}

void mantle_make_exit(int status) {
    if (make_control_active) {
        if (status < 0 || status > 255) die("invalid original Make status");
        make_control_status = status;
        if (request_ordinal != journal_count(journal_root)) die("unused Make command results");
        text_file(output_root, "state", "done\n");
        char value[32]; snprintf(value, sizeof value, "%d\n", status);
        text_file(output_root, "exit-status", value);
        char *request = join(output_root, "request");
        remove_tree(request); mkdir_checked(request);
        text_file(request, "state", "not-active\n");
        free(request);
    }
    /* Like the shared shell VM, a completed source controller returns its
     * interpreted status in the authenticated envelope. Ordinary Make keeps
     * its original process exit semantics. */
    exit(make_control_active ? 1 : status);
}

/* A source request constructor, not a replacement version implementation.
 * Original GNU Make still evaluates --version and writes its actual result. */
static int make_version_request(const char *destination) {
    mkdir_checked(destination); output_root = realpath(destination, NULL);
    if (!output_root) die("Make version request output");
    workspace_root = join(output_root, "workspace"); mkdir_checked(workspace_root);
    char *temporary = join(workspace_root, "tmp"); mkdir_checked(temporary); free(temporary);
    FILE *manifest = open_record(output_root, "controller-roots", "wb");
    for (const char **part = (const char *[]){"MANTLE-CONTROLLER-ROOTS-1", "mutable", "/tmp", "tmp", NULL}; *part; ++part) record(manifest, *part);
    if (fclose(manifest)) die("Make version roots close");
    Env initial = {0}; initial.cwd = slice("/tmp", 4); initial.zero = "make";
    initial.creation_mask = umask(0); umask(initial.creation_mask); initial.mask_valid = 1;
    set(&initial, "LC_ALL", "C"); variable(&initial, "LC_ALL")->exported = 1;
    set(&initial, "SOURCE_DATE_EPOCH", "1"); variable(&initial, "SOURCE_DATE_EPOCH")->exported = 1;
    for (int i = 0; i < 3; ++i) {
        initial.fd[i] = tmpfile(); if (!initial.fd[i]) die("Make version source stream");
        initial.fd_flags[i] = mantle_descriptor_flags(fileno(initial.fd[i]));
    }
    initial.in = initial.fd[0]; initial.out = initial.fd[1]; initial.err = initial.fd[2];
    char *arguments[] = {"make", "--version", NULL}; Args command = {.v = arguments, .n = 2};
    char *request = join(output_root, "call"); write_request(request, &initial, &command); free(request);
    make_close_environment(&initial);
    return 0;
}

int main(int argc, char **argv) {
    if (argc > 1 && !strcmp(argv[1], "--mantle-source-version-request")) {
        if (argc != 3) die("usage: make --mantle-source-version-request OUTPUT");
        return make_version_request(argv[2]);
    }
    if (argc > 1 && !strcmp(argv[1], "--mantle-source-shell"))
        return mantle_shell_run(argc - 1, argv + 1);
    if (argc < 2 || strcmp(argv[1], "--mantle-source-control")) {
        return mantle_original_make_main(argc, argv);
    }
    if (argc < 7) die("usage: make --mantle-source-control INITIAL-REQUEST PREVIOUS-CHECKPOINT RESULT OUTPUT ORIGINAL-ARGV0 [ARGS...]");
    char *initial = realpath(argv[2], NULL);
    char *previous = strcmp(argv[3], "-") ? realpath(argv[3], NULL) : NULL;
    char *result = strcmp(argv[4], "-") ? realpath(argv[4], NULL) : NULL;
    if (!initial || (strcmp(argv[3], "-") && !previous) || (strcmp(argv[4], "-") && !result)) die("Make continuation inputs");
    mkdir_checked(argv[5]); output_root = realpath(argv[5], NULL);
    if (!output_root) die("Make checkpoint output");
    journal_root = join(output_root, "journal"); mkdir_checked(journal_root);
    if (previous) {
        char *old = join(previous, "journal"); copy_tree(old, journal_root); free(old);
        if (result) {
            if (!text_is(previous, "state", "paused\n")) die("result for completed Make");
            unsigned long count = journal_count(journal_root);
            char name[64]; snprintf(name, sizeof name, "%lu", count);
            char *destination = join(journal_root, name);
            copy_tree(result, destination); free(destination);
        }
    } else if (result) die("Make result without checkpoint");
    workspace_root = join(output_root, "workspace");
    read_request_data_at(initial, &make_initial_request, workspace_root);
    if (!make_initial_request.active) die("Make initial source call is inactive");
    const char *manifest = getenv("MANTLE_CONTROLLER_ROOTS");
    if (!manifest || mantle_paths_init(workspace_root, manifest, make_initial_request.env.cwd) || mantle_paths_chdir(make_initial_request.env.cwd)) die("Make source logical cwd");
    FILE *before[64]; memcpy(before, make_initial_request.env.fd, sizeof before);
    for (int i = 0; i < 64; ++i) if (before[i]) {
        int alias = -1; for (int j = 0; j < i; ++j) if (before[j] == before[i]) { alias = j; break; }
        if (alias >= 0) { make_initial_request.env.fd[i] = make_initial_request.env.fd[alias]; continue; }
        int flags = make_initial_request.env.fd_flags[i]; long position = ftell(before[i]);
        if (position < 0) die("Make initial descriptor position");
        char proc[64]; snprintf(proc, sizeof proc, "/proc/self/fd/%d", fileno(before[i]));
        int opened = open(proc, flags | O_CLOEXEC), staged;
        if (opened < 0 || (staged = fcntl(opened, F_DUPFD_CLOEXEC, 64)) < 0) die("Make stage initial source descriptor");
        if (close(opened) || lseek(staged, position, SEEK_SET) < 0) die("Make source descriptor staging position");
        const char *mode = (flags & O_ACCMODE) == O_RDONLY ? "rb" : (flags & O_ACCMODE) == O_WRONLY ? "wb" : "r+b";
        make_initial_request.env.fd[i] = fdopen(staged, mode);
        if (!make_initial_request.env.fd[i]) die("Make initial source stream");
    }
    for (int i = 0; i < 64; ++i) if (before[i]) { int first = 1; for (int j = 0; j < i; ++j) if (before[j] == before[i]) first = 0; if (first && fclose(before[i])) die("Make initial stream close"); }
    make_initial_request.env.in = make_initial_request.env.fd[0]; make_initial_request.env.out = make_initial_request.env.fd[1]; make_initial_request.env.err = make_initial_request.env.fd[2];
    for (int i = 0; i < 64; ++i) {
        if (!make_initial_request.env.fd[i] || make_initial_request.env.fd_cloexec[i]) { if (close(i) && errno != EBADF) die("Make source closed descriptor"); }
        else if (dup2(fileno(make_initial_request.env.fd[i]), i) < 0 || fcntl(i, F_SETFD, 0)) die("Make install original source descriptor");
    }
    make_control_active = 1;
    if (!make_initial_request.env.mask_valid) die("Make initial source creation mask is missing");
    umask(make_initial_request.env.creation_mask);
    clearerr(stdin); clearerr(stdout); clearerr(stderr);
    if (atexit(make_capture_source_result)) die("Make source-result finalizer");
    int status = mantle_original_make_main(argc - 6, argv + 6);
    mantle_make_exit(status);
}
