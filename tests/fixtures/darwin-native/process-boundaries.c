#include <errno.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/event.h>
#include <sys/mman.h>
#include <sys/resource.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>

/* Bounded native feasibility experiments. No files or network connections. */
static int descendant_escape(void) {
    int ready[2], release[2];
    if (pipe(ready) || pipe(release)) return 70;
    pid_t child = fork();
    if (child < 0) return 71;
    if (!child) {
        close(ready[0]); close(release[1]);
        pid_t sid = setsid();
        int error = errno;
        char status = sid < 0 ? 'N' : 'Y';
        if (write(ready[1], &status, 1) != 1) _exit(72);
        if (sid < 0) _exit(error == EPERM ? 77 : 73);
        /* Operator interruption cannot leave this fixture indefinitely alive. */
        alarm(5);
        char byte;
        if (read(release[0], &byte, 1) != 1) _exit(74);
        _exit(0);
    }
    close(ready[1]); close(release[0]);
    char status = 'N';
    if (read(ready[0], &status, 1) != 1) { kill(child, SIGKILL); waitpid(child, NULL, 0); return 75; }
    pid_t parent_group = getpgrp();
    pid_t child_group = getpgid(child);
    printf("{\"parent_group\":%d,\"child_group\":%d,\"setsid_succeeded\":%s,\"escaped\":%s}\n",
           parent_group, child_group, status == 'Y' ? "true" : "false",
           status == 'Y' && child_group != parent_group ? "true" : "false");
    /* Keep the child alive during observation, then release and reap it. */
    if (write(release[1], "x", 1) != 1) kill(child, SIGKILL);
    int result;
    if (waitpid(child, &result, 0) != child) return 76;
    return WIFEXITED(result) ? WEXITSTATUS(result) : 78;
}

static int track(void) {
    int queue = kqueue();
    if (queue < 0) return 70;
    struct kevent change, result;
    struct timespec timeout = { 0, 0 };
    EV_SET(&change, getpid(), EVFILT_PROC, EV_ADD | EV_RECEIPT, NOTE_EXIT | NOTE_TRACK, 0, NULL);
    errno = 0;
    int count = kevent(queue, &change, 1, &result, 1, &timeout);
    int error = errno;
    if (count == 1 && (result.flags & EV_ERROR)) error = (int)result.data;
    printf("{\"kevent_count\":%d,\"error\":%d,\"unsupported\":%s}\n", count, error, error == ENOTSUP ? "true" : "false");
    close(queue);
    return 0;
}

static int memory(void) {
    const size_t cap = 16 * 1024 * 1024;
    const size_t allocation = 64 * 1024 * 1024;
    struct rlimit limit = { cap, cap };
    errno = 0;
    int result = setrlimit(RLIMIT_AS, &limit);
    int error = errno;
    void *mapping = mmap(NULL, allocation, PROT_READ | PROT_WRITE, MAP_ANON | MAP_PRIVATE, -1, 0);
    int allocated = mapping != MAP_FAILED;
    if (allocated) {
        volatile char *bytes = mapping;
        for (size_t offset = 0; offset < allocation; offset += 4096) bytes[offset] = 1;
    }
    printf("{\"setrlimit_status\":%d,\"setrlimit_errno\":%d,\"cap_bytes\":%zu,\"allocation_bytes\":%zu,\"allocated_and_touched\":%s}\n",
           result, error, cap, allocation, allocated ? "true" : "false");
    if (allocated) munmap(mapping, allocation);
    return 0;
}

int main(int argc, char **argv) {
    if (argc != 2) return 64;
    if (!strcmp(argv[1], "escape")) return descendant_escape();
    if (!strcmp(argv[1], "track")) return track();
    if (!strcmp(argv[1], "memory")) return memory();
    return 64;
}
