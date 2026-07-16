#include "mantle_sdk.h"

#include <stdio.h>

static const size_t MESSAGE_CAPACITY_BYTES = 128u;
static const int MAX_POSITIONAL_ARGUMENTS = 1;
static const int EXIT_USAGE = 64;
static const int EXIT_FORMAT = 65;

int main(int argc, char **argv) {
    if (argc > MAX_POSITIONAL_ARGUMENTS + 1) {
        fprintf(stderr, "usage: mantle-sdk-greet [name]\n");
        return EXIT_USAGE;
    }

    const char *name = argc == MAX_POSITIONAL_ARGUMENTS + 1 ? argv[1] : NULL;
    char message[MESSAGE_CAPACITY_BYTES];
    const MantleSdkStatus status = mantle_sdk_format(name, message, sizeof(message));
    if (status != MANTLE_SDK_STATUS_OK) {
        fprintf(stderr, "failed to format SDK greeting: %d\n", status);
        return EXIT_FORMAT;
    }

    puts(message);
    return 0;
}
