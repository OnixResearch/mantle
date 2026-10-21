// The proof creates this file on the host; it must remain absent in the sandbox.
#include "/tmp/mantle-gpu-undeclared-secret.h"
#include "kernel_api.h"
extern "C" MANTLE_GPU_KERNEL void mantle_add_one(float *values) {
    values[0] += MANTLE_INCREMENT;
}
