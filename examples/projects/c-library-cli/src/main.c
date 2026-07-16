#include "greet.h"

#include <assert.h>
#include <stdio.h>

#define DEFAULT_NAME "world"

enum {
    EXPECTED_ARGC_DEFAULT = 1,
    EXPECTED_ARGC_WITH_NAME = 2,
    EXIT_USAGE_CODE = 64,
};

int main(int argc, char **argv) {
    if (argc < EXPECTED_ARGC_DEFAULT || argc > EXPECTED_ARGC_WITH_NAME) {
        fprintf(stderr, "usage: greet [name]\n");
        return EXIT_USAGE_CODE;
    }

    assert(argv != NULL);
    assert(argv[0] != NULL);

    const char *name = argc == EXPECTED_ARGC_WITH_NAME ? argv[1] : DEFAULT_NAME;
    char message[GREET_BUFFER_CAPACITY];
    const GreetStatus status = greet_format(message, GREET_BUFFER_CAPACITY, name);
    if (status != GREET_STATUS_OK) {
        fprintf(stderr, "greet formatting failed: %d\n", status);
        return status;
    }

    puts(message);
    return GREET_STATUS_OK;
}
