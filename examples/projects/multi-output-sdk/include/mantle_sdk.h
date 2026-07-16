#ifndef MANTLE_SDK_H
#define MANTLE_SDK_H

#include <stddef.h>

#define MANTLE_SDK_API_VERSION 1u

typedef enum MantleSdkStatus {
    MANTLE_SDK_STATUS_OK = 0,
    MANTLE_SDK_STATUS_INVALID_ARGUMENT = 1,
    MANTLE_SDK_STATUS_BUFFER_TOO_SMALL = 2,
} MantleSdkStatus;

MantleSdkStatus mantle_sdk_format(const char *name, char *output, size_t output_capacity);

#endif
