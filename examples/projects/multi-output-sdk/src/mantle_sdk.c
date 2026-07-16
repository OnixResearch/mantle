#include "mantle_sdk.h"

#include <stdio.h>

static const char *const DEFAULT_NAME = "SDK user";

MantleSdkStatus mantle_sdk_format(const char *name, char *output, size_t output_capacity) {
    if (output == NULL || output_capacity == 0u) {
        return MANTLE_SDK_STATUS_INVALID_ARGUMENT;
    }

    const char *selected_name = name;
    if (selected_name == NULL || selected_name[0] == '\0') {
        selected_name = DEFAULT_NAME;
    }

    const int written = snprintf(output, output_capacity, "Hello, %s!", selected_name);
    if (written < 0 || (size_t)written >= output_capacity) {
        output[0] = '\0';
        return MANTLE_SDK_STATUS_BUFFER_TOO_SMALL;
    }

    return MANTLE_SDK_STATUS_OK;
}
