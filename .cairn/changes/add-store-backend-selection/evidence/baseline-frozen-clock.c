#define _GNU_SOURCE
#include <sys/syscall.h>
#include <sys/time.h>
#include <time.h>
#include <unistd.h>

/* Test fixture only. Never preload this into a production Mantle process. */
#define FIXTURE_EPOCH_SECONDS ((time_t)1790810000)

int clock_gettime(clockid_t clock_id, struct timespec *value) {
    if (clock_id == CLOCK_REALTIME || clock_id == CLOCK_REALTIME_COARSE) {
        value->tv_sec = FIXTURE_EPOCH_SECONDS;
        value->tv_nsec = 0;
        return 0;
    }
    return (int)syscall(SYS_clock_gettime, clock_id, value);
}

time_t time(time_t *value) {
    if (value != 0) *value = FIXTURE_EPOCH_SECONDS;
    return FIXTURE_EPOCH_SECONDS;
}

int gettimeofday(struct timeval *value, void *zone) {
    (void)zone;
    value->tv_sec = FIXTURE_EPOCH_SECONDS;
    value->tv_usec = 0;
    return 0;
}
