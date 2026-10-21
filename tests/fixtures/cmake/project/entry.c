#include "answer.h"
__attribute__((noreturn)) void _start(void) {
  long code = answer() == 42 ? 0 : 1;
#if defined(__x86_64__)
  __asm__ volatile("syscall" : : "a"(60L), "D"(code) : "rcx", "r11", "memory");
#elif defined(__aarch64__)
  register long x0 __asm__("x0") = code;
  register long x8 __asm__("x8") = 93;
  __asm__ volatile("svc 0" : : "r"(x0), "r"(x8) : "memory");
#else
#error unsupported fixture architecture
#endif
  __builtin_unreachable();
}
