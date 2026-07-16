#include "greeting.h"

#include <stdio.h>

static const size_t MESSAGE_CAPACITY_BYTES = 128u;
static const int MAX_POSITIONAL_ARGUMENTS = 1;
static const int EXIT_USAGE = 64;
static const int EXIT_FORMAT = 65;

int main(int argc, char **argv) {
    if (argc > MAX_POSITIONAL_ARGUMENTS + 1) {
        fprintf(stderr, "usage: schema-c-app [name]\n");
        return EXIT_USAGE;
    }

    const char *name = argc == MAX_POSITIONAL_ARGUMENTS + 1 ? argv[1] : NULL;
    char message[MESSAGE_CAPACITY_BYTES];
    const GeneratedGreetingStatus status = generated_greeting_format(name, message, sizeof(message));
    if (status != GENERATED_GREETING_STATUS_OK) {
        fprintf(stderr, "generated greeting failed: %d\n", status);
        return EXIT_FORMAT;
    }

    puts(message);
    return 0;
}
