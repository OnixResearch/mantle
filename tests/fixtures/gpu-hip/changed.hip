#include "kernel_api.h"
extern "C" MANTLE_GPU_KERNEL void mantle_add_one(float *values) {
    values[0] += 2.0f * MANTLE_INCREMENT;
}
