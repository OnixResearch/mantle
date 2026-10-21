#pragma once
// Deliberately no HIP runtime: compilation alone never loads a driver/runtime.
#define MANTLE_GPU_KERNEL __attribute__((global))
