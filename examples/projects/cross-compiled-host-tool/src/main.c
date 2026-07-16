#include <stdio.h>
#include "build_target.h"

enum {
    EXIT_OK = 0,
    EXIT_USAGE = 64,
    EXPECTED_ARGC = 1,
};

int main(int argc, char **argv) {
    if (argc != EXPECTED_ARGC) {
        (void)fprintf(stderr, "usage: %s\n", argv[0]);
        return EXIT_USAGE;
    }
    (void)printf("host-generated header -> %s (%s)\n", BUILD_TARGET_TRIPLE, BUILD_TARGET_ROLE);
    return EXIT_OK;
}
