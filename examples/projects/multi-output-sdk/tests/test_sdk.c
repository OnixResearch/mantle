#include "mantle_sdk.h"

#include <assert.h>
#include <string.h>

static const size_t MESSAGE_CAPACITY_BYTES = 128u;

int main(void) {
    char message[MESSAGE_CAPACITY_BYTES];
    const MantleSdkStatus positive_status = mantle_sdk_format("Mantle", message, sizeof(message));
    assert(positive_status == MANTLE_SDK_STATUS_OK);
    assert(strcmp(message, "Hello, Mantle!") == 0);

    const MantleSdkStatus null_output_status = mantle_sdk_format("Mantle", NULL, sizeof(message));
    assert(null_output_status == MANTLE_SDK_STATUS_INVALID_ARGUMENT);

    char tiny_buffer[1];
    const MantleSdkStatus tiny_buffer_status = mantle_sdk_format("Mantle", tiny_buffer, sizeof(tiny_buffer));
    assert(tiny_buffer_status == MANTLE_SDK_STATUS_BUFFER_TOO_SMALL);
    assert(tiny_buffer[0] == '\0');

    return 0;
}
