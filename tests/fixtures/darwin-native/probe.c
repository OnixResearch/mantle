#include <errno.h>
#include <fcntl.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/resource.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <unistd.h>

/* Native negative fixture: status zero means the expected operation occurred. */
int main(int argc, char **argv) {
    if (argc < 2) return 64;
    if (!strcmp(argv[1], "hello")) {
        puts("mantle-darwin-native");
        return 0;
    }
    if (!strcmp(argv[1], "read") && argc == 3) {
        int fd = open(argv[2], O_RDONLY);
        if (fd < 0) return errno == EPERM || errno == EACCES ? 77 : 70;
        char byte;
        int result = read(fd, &byte, 1) == 1 ? 0 : 71;
        close(fd);
        return result;
    }
    if (!strcmp(argv[1], "write") && argc == 3) {
        int fd = open(argv[2], O_WRONLY | O_CREAT | O_EXCL, 0600);
        if (fd < 0) return errno == EPERM || errno == EACCES ? 77 : 70;
        int result = write(fd, "native-output\n", 14) == 14 ? 0 : 71;
        close(fd);
        return result;
    }
    if (!strcmp(argv[1], "network")) {
        int fd = socket(AF_INET, SOCK_STREAM, 0);
        if (fd < 0) return errno == EPERM || errno == EACCES ? 77 : 70;
        close(fd);
        return 0;
    }
    if (!strcmp(argv[1], "fork")) {
        pid_t child = fork();
        if (child < 0) return errno == EPERM || errno == EACCES ? 77 : 70;
        if (!child) { setsid(); _exit(0); }
        int status;
        if (waitpid(child, &status, 0) != child) return 71;
        return WIFEXITED(status) ? WEXITSTATUS(status) : 72;
    }
    if (!strcmp(argv[1], "sleep")) {
        for (;;) pause();
    }
    if (!strcmp(argv[1], "cpu")) {
        for (;;) { __asm__ volatile("" ::: "memory"); }
    }
    if (!strcmp(argv[1], "fail")) return 42;
    return 64;
}
