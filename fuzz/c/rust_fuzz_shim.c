#include <stddef.h>
#include <stdint.h>

extern void fuzz_elf_rust(const uint8_t *data, size_t size);
extern void fuzz_archive_rust(const uint8_t *data, size_t size);
extern void fuzz_elf_diff(const uint8_t *data, size_t size);
extern void fuzz_store_archive_list(const uint8_t *data, size_t size);

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size)
{
#if defined(FUZZ_TARGET_ELF_RUST)
    fuzz_elf_rust(data, size);
#elif defined(FUZZ_TARGET_ARCHIVE_RUST)
    fuzz_archive_rust(data, size);
#elif defined(FUZZ_TARGET_ELF_DIFF)
    fuzz_elf_diff(data, size);
#elif defined(FUZZ_TARGET_STORE_ARCHIVE_LIST)
    fuzz_store_archive_list(data, size);
#else
#error "Define one FUZZ_TARGET_* macro"
#endif
    return 0;
}
