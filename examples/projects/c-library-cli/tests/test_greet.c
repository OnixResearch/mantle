#include "greet.h"

#include <assert.h>
#include <string.h>

#define EXPECTED_GREETING "Hello, Mantle!"

enum {
    TEST_BUFFER_CAPACITY = 128,
};

int main(void) {
    char buffer[TEST_BUFFER_CAPACITY];

    const GreetStatus positive_status = greet_format(buffer, TEST_BUFFER_CAPACITY, "Mantle");
    assert(positive_status == GREET_STATUS_OK);
    assert(strcmp(buffer, EXPECTED_GREETING) == 0);

    const GreetStatus zero_capacity_status = greet_format(buffer, 0, "Mantle");
    assert(zero_capacity_status == GREET_STATUS_INVALID_ARGUMENT);
    assert(buffer[0] == 'H');

    return 0;
}
