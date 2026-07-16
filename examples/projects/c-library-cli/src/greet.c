#include "greet.h"

#include <assert.h>
#include <stddef.h>
#include <stdio.h>

GreetStatus greet_format(char *buffer, uint32_t capacity_bytes, const char *name) {
    if (buffer == NULL || name == NULL || capacity_bytes == 0 || capacity_bytes > GREET_CAPACITY_MAX) {
        return GREET_STATUS_INVALID_ARGUMENT;
    }

    assert(buffer != NULL);
    assert(name != NULL);
    assert(capacity_bytes > 0);
    assert(capacity_bytes <= GREET_CAPACITY_MAX);

    const int written_bytes = snprintf(buffer, capacity_bytes, "Hello, %s!", name);
    if (written_bytes < 0) {
        return GREET_STATUS_FORMAT_ERROR;
    }
    if ((uint32_t)written_bytes >= capacity_bytes) {
        return GREET_STATUS_CAPACITY_EXCEEDED;
    }
    return GREET_STATUS_OK;
}
