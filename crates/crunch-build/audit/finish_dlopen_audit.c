#define _GNU_SOURCE
#include <link.h>
#include <stdatomic.h>
#include <stdint.h>
#include <stddef.h>
#include <sys/syscall.h>
#include <fcntl.h>
#include <unistd.h>

/* Loader callbacks must neither allocate nor run a shell. The fixed scratch
 * path is read only by the builder after the audited command exits. */
#ifndef AUDIT_LOG
#define AUDIT_LOG "/build/.mantle-finish-audit-events"
#endif
#define MAX_EVENTS 128u
#define MAX_NAME_BYTES 256u
static _Atomic unsigned int event_count;

static void audit_event(char kind, const char *name) {
    unsigned int index = atomic_fetch_add_explicit(&event_count, 1u, memory_order_relaxed);
    if (index > MAX_EVENTS) return;
    if (index == MAX_EVENTS) kind = 'X';
    char row[2u * MAX_NAME_BYTES + 4u];
    static const char hex[] = "0123456789abcdef";
    size_t offset = 0u;
    row[offset++] = kind;
    if (name != NULL && kind != 'X') {
        row[offset++] = '\t';
        for (size_t i = 0u; name[i] != '\0'; ++i) {
            if (i >= MAX_NAME_BYTES) {
                row[0] = 'X';
                offset = 1u;
                break;
            }
            unsigned char c = (unsigned char)name[i];
            row[offset++] = hex[c >> 4];
            row[offset++] = hex[c & 15u];
        }
    }
    row[offset++] = '\n';
    int fd = (int)syscall(SYS_openat, AT_FDCWD, AUDIT_LOG, O_WRONLY | O_CREAT | O_APPEND | O_CLOEXEC, 0600);
    if (fd < 0) return;
    (void)syscall(SYS_write, fd, row, offset);
    (void)syscall(SYS_close, fd);
}

unsigned int la_version(unsigned int version) {
    (void)version;
    audit_event('V', NULL);
    return LAV_CURRENT;
}

char *la_objsearch(const char *name, uintptr_t *cookie, unsigned int flag) {
    (void)cookie;
    if (flag == LA_SER_ORIG) audit_event('S', name);
    return (char *)name;
}

unsigned int la_objopen(struct link_map *map, Lmid_t lmid, uintptr_t *cookie) {
    (void)lmid;
    (void)cookie;
    if (map != NULL && map->l_name != NULL && map->l_name[0] != '\0') {
        audit_event('O', map->l_name);
    }
    return 0u;
}
