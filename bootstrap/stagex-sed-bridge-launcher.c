#include <assert.h>
#include <stdlib.h>
#include <string.h>
#include <sys/resource.h>
#include <sys/syscall.h>
#include <unistd.h>

extern char **environ;

enum {
  MANTLE_ARGUMENT_COUNT_MAX = 128,
  MANTLE_CHILD_ARGUMENT_EXTRA = 2,
  MANTLE_SELF_TEST_ARGUMENT_COUNT = 2,
  MANTLE_KIBIBYTE_BYTES = 1024,
  MANTLE_MEBIBYTE_BYTES = MANTLE_KIBIBYTE_BYTES * MANTLE_KIBIBYTE_BYTES,
  MANTLE_SPOOL_MEBIBYTES_MAX = 8,
  MANTLE_SPOOL_BYTES_MAX = MANTLE_SPOOL_MEBIBYTES_MAX * MANTLE_MEBIBYTE_BYTES,
  MANTLE_EXEC_FAILURE = 126,
  MANTLE_AUTHORITY_FAILURE = 125
};

static int mantle_absolute_path(const char *path) {
  return path != NULL && path[0] == '/' && path[1] != '\0';
}

static int mantle_self_test(void) {
  assert(mantle_absolute_path("/declared/tool") == 1);
  assert(mantle_absolute_path("relative/tool") == 0);
  assert(mantle_absolute_path("") == 0);
  assert(mantle_absolute_path(NULL) == 0);
  return 0;
}

static int mantle_error(const char *message, int exit_code) {
  const size_t message_len = strlen(message);
  const ssize_t written = write(STDERR_FILENO, message, message_len);
  (void)written;
  return exit_code;
}

static int mantle_set_file_limit(void) {
  struct rlimit limit;
  limit.rlim_cur = MANTLE_SPOOL_BYTES_MAX;
  limit.rlim_max = MANTLE_SPOOL_BYTES_MAX;
  return (int)syscall(SYS_setrlimit, RLIMIT_FSIZE, &limit);
}

int main(int argc, char **argv) {
  char *child_argv[MANTLE_ARGUMENT_COUNT_MAX + MANTLE_CHILD_ARGUMENT_EXTRA];
  const char *bash;
  const char *script;
  int index;

  if (argc == MANTLE_SELF_TEST_ARGUMENT_COUNT && strcmp(argv[1], "--mantle-self-test") == 0) {
    return mantle_self_test();
  }
  if (argc < 1 || argc > MANTLE_ARGUMENT_COUNT_MAX) {
    return mantle_error("stagex-sed-bridge: argument limit exceeded\n", MANTLE_AUTHORITY_FAILURE);
  }
  bash = getenv("MANTLE_STAGE_X_SED_BRIDGE_BASH");
  script = getenv("MANTLE_STAGE_X_SED_BRIDGE_SCRIPT");
  if (!mantle_absolute_path(bash) || !mantle_absolute_path(script)) {
    return mantle_error("stagex-sed-bridge: missing absolute launcher authority\n", MANTLE_AUTHORITY_FAILURE);
  }
  if (mantle_set_file_limit() != 0) {
    return mantle_error("stagex-sed-bridge: file limit setup failed\n", MANTLE_AUTHORITY_FAILURE);
  }

  child_argv[0] = (char *)bash;
  child_argv[1] = (char *)script;
  for (index = 1; index < argc; index++) {
    child_argv[index + 1] = argv[index];
  }
  child_argv[argc + 1] = NULL;
  assert(child_argv[0] != NULL);
  assert(child_argv[argc + 1] == NULL);
  execve(bash, child_argv, environ);
  return mantle_error("stagex-sed-bridge: exact Bash launch failed\n", MANTLE_EXEC_FAILURE);
}
