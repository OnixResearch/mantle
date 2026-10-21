/* Small libc surface for compiling the original BusyBox bzip2 library as an
 * auxiliary usage-header generator before the complete bootstrap BusyBox.
 * Compression and CRC implementation bytes come from the admitted source tree.
 */
#ifndef MANTLE_BUSYBOX_BZIP2_LIBC_H
#define MANTLE_BUSYBOX_BZIP2_LIBC_H
#include <stdint.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <endian.h>
#define FAST_FUNC
#define ALWAYS_INLINE inline
#define BZ_DEBUG 0
#define NOINLINE __attribute__((noinline))
#define ALIGN4 __attribute__((aligned(4)))
#define ENABLE_FEATURE_CLEAN_UP 1
#define BZ_LIGHT_DEBUG 0
#define BZIP2_SPEED 1
#if __BYTE_ORDER == __LITTLE_ENDIAN
#define BB_LITTLE_ENDIAN 1
#define BB_BIG_ENDIAN 0
#else
#define BB_LITTLE_ENDIAN 0
#define BB_BIG_ENDIAN 1
#endif
#if defined(__i386__) || defined(__x86_64__)
#define BB_UNALIGNED_MEMACCESS_OK 1
#else
#define BB_UNALIGNED_MEMACCESS_OK 0
#endif
typedef uint32_t bb__aliased_uint32_t __attribute__((may_alias));
typedef uint64_t bb__aliased_uint64_t __attribute__((may_alias));
static void *xmalloc(size_t n){void *p=malloc(n);if(!p){fputs("BusyBox bzip2 allocation failed\n",stderr);exit(1);}return p;}
static void *xzalloc(size_t n){void *p=xmalloc(n);memset(p,0,n);return p;}
#endif
