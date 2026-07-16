#ifndef MANTLE_EXAMPLE_GREET_H
#define MANTLE_EXAMPLE_GREET_H

#include <stdint.h>

typedef enum {
    GREET_STATUS_OK = 0,
    GREET_STATUS_INVALID_ARGUMENT = 1,
    GREET_STATUS_CAPACITY_EXCEEDED,
    GREET_STATUS_FORMAT_ERROR,
} GreetStatus;

enum {
    GREET_BUFFER_CAPACITY = 128,
    GREET_CAPACITY_MAX = 4096,
};

GreetStatus greet_format(char *buffer, uint32_t capacity_bytes, const char *name);

#endif
