#include <assert.h>
#include <errno.h>
#include <fcntl.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/syscall.h>
#include <time.h>
#include <unistd.h>

enum {
  MANTLE_AUTHORITY_FAILURE = 125,
  MANTLE_SINGLE_ARGUMENT_COUNT = 2,
  MANTLE_FLAGGED_ARGUMENT_COUNT = 3,
  MANTLE_DECIMAL_RADIX = 10,
  MANTLE_BITS_PER_BYTE = 8,
  MANTLE_ELF_MAGIC_FIRST = 0x7f,
  MANTLE_SLEEP_SECONDS_MAX = 10,
  MANTLE_PATH_BYTES_MAX = 255,
  MANTLE_FILE_MEBIBYTES_MAX = 64,
  MANTLE_KIBIBYTE_BYTES = 1024,
  MANTLE_MEBIBYTE_BYTES = MANTLE_KIBIBYTE_BYTES * MANTLE_KIBIBYTE_BYTES,
  MANTLE_FILE_BYTES_MAX = MANTLE_FILE_MEBIBYTES_MAX * MANTLE_MEBIBYTE_BYTES,
  MANTLE_HEADER_BYTES_MAX = 64,
  MANTLE_EMIT_BUFFER_BYTES = 4096,
  MANTLE_ELF_CLASS_INDEX = 4,
  MANTLE_ELF_DATA_INDEX = 5,
  MANTLE_ELF_TYPE_INDEX = 16,
  MANTLE_ELF_MACHINE_LOW_INDEX = 18,
  MANTLE_ELF_MACHINE_HIGH_INDEX = 19,
  MANTLE_ELF_CLASS_32 = 1,
  MANTLE_ELF_CLASS_64 = 2,
  MANTLE_ELF_DATA_LSB = 1,
  MANTLE_ELF_DATA_MSB = 2,
  MANTLE_ELF_MACHINE_X86_64 = 62,
  MANTLE_ELF_TYPE_RELOCATABLE = 1,
  MANTLE_ELF_TYPE_EXECUTABLE = 2,
  MANTLE_ELF_TYPE_SHARED = 3
};

static int mantle_error(const char *message, int exit_code) {
  const size_t message_len = strlen(message);
  const ssize_t written = write(STDERR_FILENO, message, message_len);
  (void)written;
  return exit_code;
}

static const char *mantle_basename(const char *path) {
  const char *last_slash = strrchr(path, '/');
  return last_slash == NULL ? path : last_slash + 1;
}

static int mantle_safe_relative_path(const char *path) {
  size_t path_len;
  if (path == NULL) return 0;
  path_len = strlen(path);
  if (path_len == 0) return 0;
  if (path_len > MANTLE_PATH_BYTES_MAX) return 0;
  if (path[0] == '/') return 0;
  if (strchr(path, '/') != NULL) return 0;
  if (strcmp(path, "..") == 0) return 0;
  return 1;
}

static int mantle_parse_seconds(const char *text, unsigned *seconds_out) {
  char *end = NULL;
  unsigned long value;
  if (text == NULL || text[0] == '\0' || text[0] == '-') return 0;
  value = strtoul(text, &end, MANTLE_DECIMAL_RADIX);
  if (end == NULL || end[0] != '\0' || value > MANTLE_SLEEP_SECONDS_MAX) return 0;
  *seconds_out = (unsigned)value;
  return 1;
}

static int mantle_sleep_main(int argc, char **argv) {
  struct timespec duration;
  unsigned seconds = 0;
  if (argc != MANTLE_SINGLE_ARGUMENT_COUNT || !mantle_parse_seconds(argv[1], &seconds)) {
    return mantle_error("stagex-configure-sleep: rejected arguments\n", MANTLE_AUTHORITY_FAILURE);
  }
  duration.tv_sec = (time_t)seconds;
  duration.tv_nsec = 0;
  if (syscall(SYS_nanosleep, &duration, NULL) != 0) {
    return mantle_error("stagex-configure-sleep: nanosleep failed\n", MANTLE_AUTHORITY_FAILURE);
  }
  assert(seconds <= MANTLE_SLEEP_SECONDS_MAX);
  assert(duration.tv_nsec == 0);
  return 0;
}

static const char *mantle_elf_kind(unsigned type) {
  if (type == MANTLE_ELF_TYPE_RELOCATABLE) return "relocatable";
  if (type == MANTLE_ELF_TYPE_EXECUTABLE) return "executable";
  if (type == MANTLE_ELF_TYPE_SHARED) return "shared object";
  return "unknown";
}

static int mantle_classify_file(const char *path, const unsigned char *header, size_t header_len) {
  static const unsigned char elf_magic[] = {MANTLE_ELF_MAGIC_FIRST, 'E', 'L', 'F'};
  static const unsigned char archive_magic[] = {'!', '<', 'a', 'r', 'c', 'h', '>', '\n'};
  int written;
  if (header_len > MANTLE_ELF_MACHINE_HIGH_INDEX && memcmp(header, elf_magic, sizeof(elf_magic)) == 0) {
    const unsigned elf_class = header[MANTLE_ELF_CLASS_INDEX];
    const unsigned elf_data = header[MANTLE_ELF_DATA_INDEX];
    const unsigned elf_type = header[MANTLE_ELF_TYPE_INDEX];
    unsigned elf_machine;
    const char *class_text;
    const char *data_text;
    if (elf_class != MANTLE_ELF_CLASS_32) {
      if (elf_class != MANTLE_ELF_CLASS_64) return MANTLE_AUTHORITY_FAILURE;
    }
    if (elf_data == MANTLE_ELF_DATA_LSB) {
      elf_machine = header[MANTLE_ELF_MACHINE_LOW_INDEX] |
                    (header[MANTLE_ELF_MACHINE_HIGH_INDEX] << MANTLE_BITS_PER_BYTE);
      data_text = "LSB";
    } else if (elf_data == MANTLE_ELF_DATA_MSB) {
      elf_machine = (header[MANTLE_ELF_MACHINE_LOW_INDEX] << MANTLE_BITS_PER_BYTE) |
                    header[MANTLE_ELF_MACHINE_HIGH_INDEX];
      data_text = "MSB";
    } else {
      return MANTLE_AUTHORITY_FAILURE;
    }
    if (elf_machine != MANTLE_ELF_MACHINE_X86_64) return MANTLE_AUTHORITY_FAILURE;
    class_text = elf_class == MANTLE_ELF_CLASS_64 ? "64-bit" : "32-bit";
    written = printf("%s: ELF %s %s %s, x86-64\n", path, class_text, data_text, mantle_elf_kind(elf_type));
    return written < 0 ? MANTLE_AUTHORITY_FAILURE : 0;
  }
  if (header_len >= sizeof(archive_magic) && memcmp(header, archive_magic, sizeof(archive_magic)) == 0) {
    written = printf("%s: current ar archive\n", path);
    return written < 0 ? MANTLE_AUTHORITY_FAILURE : 0;
  }
  written = printf("%s: data\n", path);
  return written < 0 ? MANTLE_AUTHORITY_FAILURE : 0;
}

static int mantle_file_main(int argc, char **argv) {
  unsigned char header[MANTLE_HEADER_BYTES_MAX];
  const char *path;
  struct stat path_metadata;
  struct stat metadata;
  ssize_t header_len;
  int descriptor;
  if (argc == MANTLE_SINGLE_ARGUMENT_COUNT) path = argv[1];
  else if (argc == MANTLE_FLAGGED_ARGUMENT_COUNT && strcmp(argv[1], "-L") == 0) path = argv[2];
  else return mantle_error("stagex-configure-file: rejected arguments\n", MANTLE_AUTHORITY_FAILURE);
  if (!mantle_safe_relative_path(path)) {
    return mantle_error("stagex-configure-file: rejected path\n", MANTLE_AUTHORITY_FAILURE);
  }
  if (lstat(path, &path_metadata) != 0) return 1;
  if (S_ISLNK(path_metadata.st_mode)) {
    return mantle_error("stagex-configure-file: rejected symlink\n", MANTLE_AUTHORITY_FAILURE);
  }
  descriptor = open(path, O_RDONLY | O_NOFOLLOW);
  if (descriptor < 0) return MANTLE_AUTHORITY_FAILURE;
  if (fstat(descriptor, &metadata) != 0) {
    close(descriptor);
    return mantle_error("stagex-configure-file: metadata failed\n", MANTLE_AUTHORITY_FAILURE);
  }
  if (!S_ISREG(metadata.st_mode) || metadata.st_size < 0 || metadata.st_size > MANTLE_FILE_BYTES_MAX) {
    close(descriptor);
    return mantle_error("stagex-configure-file: rejected file\n", MANTLE_AUTHORITY_FAILURE);
  }
  header_len = read(descriptor, header, sizeof(header));
  if (close(descriptor) != 0 || header_len < 0) {
    return mantle_error("stagex-configure-file: read failed\n", MANTLE_AUTHORITY_FAILURE);
  }
  assert(metadata.st_size >= 0);
  assert((size_t)header_len <= sizeof(header));
  return mantle_classify_file(path, header, (size_t)header_len);
}

static int mantle_emit_main(int argc, char **argv) {
  unsigned char buffer[MANTLE_EMIT_BUFFER_BYTES];
  struct stat metadata;
  size_t emitted_bytes = 0;
  ssize_t read_len;
  int descriptor;
  if (argc != MANTLE_SINGLE_ARGUMENT_COUNT || !mantle_safe_relative_path(argv[1])) {
    return mantle_error("stagex-configure-emit: rejected arguments\n", MANTLE_AUTHORITY_FAILURE);
  }
  descriptor = open(argv[1], O_RDONLY | O_NOFOLLOW);
  if (descriptor < 0) return MANTLE_AUTHORITY_FAILURE;
  if (fstat(descriptor, &metadata) != 0) {
    close(descriptor);
    return mantle_error("stagex-configure-emit: metadata failed\n", MANTLE_AUTHORITY_FAILURE);
  }
  if (!S_ISREG(metadata.st_mode) || metadata.st_size < 0 || metadata.st_size > MANTLE_FILE_BYTES_MAX) {
    close(descriptor);
    return mantle_error("stagex-configure-emit: rejected file\n", MANTLE_AUTHORITY_FAILURE);
  }
  if (signal(SIGPIPE, SIG_IGN) == SIG_ERR) {
    close(descriptor);
    return mantle_error("stagex-configure-emit: signal setup failed\n", MANTLE_AUTHORITY_FAILURE);
  }
  while ((read_len = read(descriptor, buffer, sizeof(buffer))) > 0) {
    ssize_t offset = 0;
    if ((size_t)read_len > MANTLE_FILE_BYTES_MAX - emitted_bytes) {
      close(descriptor);
      return mantle_error("stagex-configure-emit: file grew past limit\n", MANTLE_AUTHORITY_FAILURE);
    }
    emitted_bytes += (size_t)read_len;
    while (offset < read_len) {
      const ssize_t written = write(STDOUT_FILENO, buffer + offset, (size_t)(read_len - offset));
      if (written < 0) {
        if (errno == EPIPE) { close(descriptor); return 0; }
        if (errno == EINTR) continue;
        close(descriptor);
        return MANTLE_AUTHORITY_FAILURE;
      }
      if (written == 0) { close(descriptor); return MANTLE_AUTHORITY_FAILURE; }
      offset += written;
    }
  }
  if (close(descriptor) != 0 || read_len < 0) return MANTLE_AUTHORITY_FAILURE;
  assert(emitted_bytes <= MANTLE_FILE_BYTES_MAX);
  assert(read_len == 0);
  return 0;
}

static int mantle_self_test(void) {
  unsigned seconds = 0;
  assert(mantle_safe_relative_path("conftest.o") == 1);
  assert(mantle_safe_relative_path("/usr/bin/file") == 0);
  assert(mantle_safe_relative_path("../escape") == 0);
  assert(mantle_parse_seconds("0", &seconds) == 1);
  assert(mantle_parse_seconds("11", &seconds) == 0);
  return 0;
}

int main(int argc, char **argv) {
  const char *name;
  if (argc <= 0) return MANTLE_AUTHORITY_FAILURE;
  if (argv == NULL) return MANTLE_AUTHORITY_FAILURE;
  if (argv[0] == NULL) return MANTLE_AUTHORITY_FAILURE;
  name = mantle_basename(argv[0]);
  if (argc == MANTLE_SINGLE_ARGUMENT_COUNT && strcmp(argv[1], "--mantle-self-test") == 0) return mantle_self_test();
  if (strcmp(name, "sleep") == 0) return mantle_sleep_main(argc, argv);
  if (strcmp(name, "file") == 0) return mantle_file_main(argc, argv);
  if (strcmp(name, "emit") == 0) return mantle_emit_main(argc, argv);
  return mantle_error("stagex-configure-utility: unknown invocation name\n", MANTLE_AUTHORITY_FAILURE);
}
