#include <errno.h>
#include <inttypes.h>
#include <mach/mach.h>
#include <stdint.h>
#include <stdio.h>
#include <sys/mman.h>
#include <sys/resource.h>

/* Bounded follow-up when a tiny RLIMIT_AS is below existing VM usage.
 * This is a host resource mechanism experiment, never executor admission. */
int main(void) {
    mach_task_basic_info_data_t info;
    mach_msg_type_number_t count = MACH_TASK_BASIC_INFO_COUNT;
    kern_return_t query = task_info(mach_task_self(), MACH_TASK_BASIC_INFO,
                                    (task_info_t)&info, &count);
    if (query != KERN_SUCCESS) {
        printf("{\"task_info_status\":%d}\n", query);
        return 70;
    }
    const uint64_t headroom = UINT64_C(32) * 1024 * 1024;
    const size_t allocation = 64 * 1024 * 1024;
    uint64_t baseline = (uint64_t)info.virtual_size;
    if (baseline > UINT64_MAX - headroom) return 71;
    struct rlimit requested = { baseline + headroom, baseline + headroom };
    errno = 0;
    int result = setrlimit(RLIMIT_AS, &requested);
    int set_error = errno;
    struct rlimit observed = { 0, 0 };
    errno = 0;
    int read_result = getrlimit(RLIMIT_AS, &observed);
    int read_error = errno;
    errno = 0;
    void *mapping = mmap(NULL, allocation, PROT_READ | PROT_WRITE,
                         MAP_ANON | MAP_PRIVATE, -1, 0);
    int map_error = errno;
    int allocated = mapping != MAP_FAILED;
    if (allocated) {
        volatile char *bytes = mapping;
        for (size_t offset = 0; offset < allocation; offset += 4096) bytes[offset] = 1;
        munmap(mapping, allocation);
    }
    printf("{\"schema\":\"mantle-darwin-address-space-probe-v1\","
           "\"baseline_vm_bytes\":%" PRIu64 ",\"requested_bytes\":%" PRIu64 ","
           "\"setrlimit_status\":%d,\"setrlimit_errno\":%d,"
           "\"getrlimit_status\":%d,\"getrlimit_errno\":%d,"
           "\"observed_soft_bytes\":%" PRIu64 ",\"observed_hard_bytes\":%" PRIu64 ","
           "\"allocation_bytes\":%zu,\"mmap_errno\":%d,\"allocated_and_touched\":%s}\n",
           baseline, (uint64_t)requested.rlim_cur, result, set_error, read_result, read_error,
           (uint64_t)observed.rlim_cur, (uint64_t)observed.rlim_max,
           allocation, map_error, allocated ? "true" : "false");
    return 0;
}
