#include <assert.h>
#include <errno.h>
#include <fcntl.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

enum {
  MANTLE_AUTHORITY_FAILURE = 125,
  MANTLE_SELF_TEST_ARGUMENT_COUNT = 2,
  MANTLE_EXPECTED_ARGUMENT_COUNT = 10,
  MANTLE_GATE_FLAG_ARGUMENT_INDEX = 1,
  MANTLE_GATE_PROGRAM_ARGUMENT_INDEX = 2,
  MANTLE_PATH_FLAG_ARGUMENT_INDEX = 3,
  MANTLE_PATH_PROGRAM_ARGUMENT_INDEX = 4,
  MANTLE_NAME_FLAG_ARGUMENT_INDEX = 5,
  MANTLE_NAME_PROGRAM_ARGUMENT_INDEX = 6,
  MANTLE_GUARD_FLAG_ARGUMENT_INDEX = 7,
  MANTLE_GUARD_PROGRAM_ARGUMENT_INDEX = 8,
  MANTLE_INPUT_ARGUMENT_INDEX = 9,
  MANTLE_YLWRAP_PROGRAM_COUNT = 4,
  MANTLE_SUBSTITUTION_PREFIX_BYTES = 2,
  MANTLE_PARENT_SLASH_TERMINATOR_BYTES = 2,
  MANTLE_PRINTABLE_ASCII_MIN = 0x20,
  MANTLE_PRINTABLE_ASCII_MAX = 0x7e,
  MANTLE_REPLACEMENT_COUNT_MAX = 7,
  MANTLE_FILENAME_REPLACEMENT_COUNT_MAX = 3,
  MANTLE_BISON_OUTPUT_PAIR_INDEX = 2,
  MANTLE_TEXT_BYTES_MAX = 1024,
  MANTLE_PATH_BYTES_MAX = 4096,
  MANTLE_KIBIBYTE_BYTES = 1024,
  MANTLE_MEBIBYTE_BYTES = MANTLE_KIBIBYTE_BYTES * MANTLE_KIBIBYTE_BYTES,
  MANTLE_FILE_MEBIBYTES_MAX = 8,
  MANTLE_FILE_BYTES_MAX = MANTLE_FILE_MEBIBYTES_MAX * MANTLE_MEBIBYTE_BYTES
};

struct MantleReplacement {
  char from[MANTLE_TEXT_BYTES_MAX];
  char to[MANTLE_TEXT_BYTES_MAX];
  size_t from_len;
  size_t to_len;
  int global;
};

struct MantlePlan {
  struct MantleReplacement replacements[MANTLE_REPLACEMENT_COUNT_MAX];
  size_t replacement_count;
};

struct MantleBuffer {
  unsigned char *bytes;
  size_t len;
};

static int mantle_error(const char *message) {
  const size_t message_len = strlen(message);
  const ssize_t written = write(STDERR_FILENO, message, message_len);
  (void)written;
  return MANTLE_AUTHORITY_FAILURE;
}

static int mantle_safe_filename(const char *text) {
  size_t index;
  const size_t len = text == NULL ? 0 : strlen(text);
  if (len == 0 || len >= MANTLE_TEXT_BYTES_MAX) return 0;
  if (strcmp(text, ".") == 0 || strcmp(text, "..") == 0) return 0;
  for (index = 0; index < len; index++) {
    const unsigned char byte = (unsigned char)text[index];
    const int safe = (byte >= 'a' && byte <= 'z') || (byte >= 'A' && byte <= 'Z') ||
                     (byte >= '0' && byte <= '9') || byte == '.' || byte == '_' || byte == '-';
    if (!safe) return 0;
  }
  assert(strchr(text, '/') == NULL);
  assert(len < MANTLE_TEXT_BYTES_MAX);
  return 1;
}

static int mantle_pattern_byte(unsigned char byte, int escaped) {
  if (escaped) return byte == '.' || byte == '[' || byte == ']' || byte == '*' || byte == '\\';
  if (byte == '.' || byte == '[' || byte == ']' || byte == '*' || byte == '\\') return 0;
  if (byte == '^' || byte == '$' || byte == '(' || byte == ')' || byte == '{' || byte == '}') return 0;
  if (byte == '+' || byte == '?' || byte == '|') return 0;
  return byte >= MANTLE_PRINTABLE_ASCII_MIN && byte <= MANTLE_PRINTABLE_ASCII_MAX;
}

static int mantle_copy_pattern(const char **cursor, char *output, size_t *output_len) {
  size_t len = 0;
  while (**cursor != '\0' && **cursor != '|') {
    unsigned char byte = (unsigned char)**cursor;
    int escaped = 0;
    if (byte == '\\') {
      *cursor += 1;
      byte = (unsigned char)**cursor;
      if (byte == '\0') return 0;
      escaped = 1;
    }
    if (!mantle_pattern_byte(byte, escaped) || len + 1 >= MANTLE_TEXT_BYTES_MAX) return 0;
    output[len++] = (char)byte;
    *cursor += 1;
  }
  if (**cursor != '|' || len == 0) return 0;
  output[len] = '\0';
  *output_len = len;
  *cursor += 1;
  assert(len < MANTLE_TEXT_BYTES_MAX);
  assert(output[len] == '\0');
  return 1;
}

static int mantle_copy_replacement(const char **cursor, char *output, size_t *output_len) {
  size_t len = 0;
  while (**cursor != '\0' && **cursor != '|') {
    const unsigned char byte = (unsigned char)**cursor;
    if (byte < MANTLE_PRINTABLE_ASCII_MIN || byte > MANTLE_PRINTABLE_ASCII_MAX ||
        byte == '&' || byte == '\\' || len + 1 >= MANTLE_TEXT_BYTES_MAX) return 0;
    output[len++] = (char)byte;
    *cursor += 1;
  }
  if (**cursor != '|') return 0;
  output[len] = '\0';
  *output_len = len;
  *cursor += 1;
  assert(len < MANTLE_TEXT_BYTES_MAX);
  assert(output[len] == '\0');
  return 1;
}

static int mantle_parse_substitution(const char **cursor, struct MantleReplacement *replacement) {
  if ((*cursor)[0] != 's' || (*cursor)[1] != '|') return 0;
  *cursor += MANTLE_SUBSTITUTION_PREFIX_BYTES;
  if (!mantle_copy_pattern(cursor, replacement->from, &replacement->from_len)) return 0;
  if (!mantle_copy_replacement(cursor, replacement->to, &replacement->to_len)) return 0;
  replacement->global = 0;
  if (**cursor == 'g') {
    replacement->global = 1;
    *cursor += 1;
  }
  if (**cursor == ';') *cursor += 1;
  else if (**cursor != '\0') return 0;
  assert(replacement->from_len > 0);
  assert(replacement->to_len < MANTLE_TEXT_BYTES_MAX);
  return 1;
}

static int mantle_parse_program(const char *text, struct MantleReplacement *output, size_t output_max, size_t *count_out) {
  const char *cursor = text;
  size_t count = 0;
  if (text == NULL || text[0] == '\0') return 0;
  while (*cursor != '\0') {
    if (count >= output_max) return 0;
    if (!mantle_parse_substitution(&cursor, &output[count])) return 0;
    count += 1;
  }
  if (count == 0) return 0;
  *count_out = count;
  assert(count <= output_max);
  assert(*cursor == '\0');
  return 1;
}

static int mantle_supported_generated_name(const char *name) {
  return strcmp(name, "y.tab.c") == 0 || strcmp(name, "y.tab.h") == 0 ||
         strcmp(name, "y.output") == 0 || strcmp(name, "lex.yy.c") == 0;
}

static int mantle_guard_name(const char *name, char *guard) {
  size_t input_index;
  size_t output_len = 0;
  int previous_underscore = 0;
  if (!mantle_safe_filename(name)) return 0;
  for (input_index = 0; name[input_index] != '\0'; input_index++) {
    unsigned char byte = (unsigned char)name[input_index];
    if (byte >= 'a' && byte <= 'z') byte = (unsigned char)(byte - 'a' + 'A');
    if (byte < 'A' || byte > 'Z') byte = '_';
    if (byte == '_' && previous_underscore) continue;
    if (output_len + 1 >= MANTLE_TEXT_BYTES_MAX) return 0;
    guard[output_len++] = (char)byte;
    previous_underscore = byte == '_';
  }
  guard[output_len] = '\0';
  assert(output_len > 0);
  assert(output_len < MANTLE_TEXT_BYTES_MAX);
  return 1;
}

static int mantle_name_has_suffix(const char *name, const char *suffix) {
  const size_t name_len = strlen(name);
  const size_t suffix_len = strlen(suffix);
  if (suffix_len > name_len) return 0;
  assert(name_len > 0);
  assert(suffix_len > 0);
  return strcmp(name + name_len - suffix_len, suffix) == 0;
}

static int mantle_same_stem(const char *left, const char *left_suffix, const char *right, const char *right_suffix) {
  const size_t left_len = strlen(left);
  const size_t right_len = strlen(right);
  const size_t left_suffix_len = strlen(left_suffix);
  const size_t right_suffix_len = strlen(right_suffix);
  if (!mantle_name_has_suffix(left, left_suffix) || !mantle_name_has_suffix(right, right_suffix)) return 0;
  if (left_len - left_suffix_len != right_len - right_suffix_len) return 0;
  assert(left_len >= left_suffix_len);
  assert(right_len >= right_suffix_len);
  return memcmp(left, right, left_len - left_suffix_len) == 0;
}

static int mantle_validate_filename_pairs(const struct MantleReplacement *pairs, size_t count) {
  size_t index;
  if (count == 0 || count > MANTLE_FILENAME_REPLACEMENT_COUNT_MAX) return 0;
  for (index = 0; index < count; index++) {
    if (!pairs[index].global) return 0;
    if (!mantle_supported_generated_name(pairs[index].from)) return 0;
    if (!mantle_safe_filename(pairs[index].to)) return 0;
  }
  if (count == 1) {
    if (strcmp(pairs[0].from, "lex.yy.c") != 0 || !mantle_name_has_suffix(pairs[0].to, ".c")) return 0;
  } else if (count == MANTLE_FILENAME_REPLACEMENT_COUNT_MAX) {
    if (strcmp(pairs[0].from, "y.tab.c") != 0 || strcmp(pairs[1].from, "y.tab.h") != 0) return 0;
    if (strcmp(pairs[MANTLE_BISON_OUTPUT_PAIR_INDEX].from, "y.output") != 0) return 0;
    if (!mantle_same_stem(pairs[0].to, ".c", pairs[1].to, ".h")) return 0;
    if (!mantle_same_stem(pairs[0].to, ".c", pairs[MANTLE_BISON_OUTPUT_PAIR_INDEX].to, ".output")) return 0;
  } else {
    return 0;
  }
  assert(count == 1 || count == MANTLE_FILENAME_REPLACEMENT_COUNT_MAX);
  assert(pairs[0].from_len > 0);
  return 1;
}

static int mantle_validate_guard_pairs(
    const struct MantleReplacement *names,
    const struct MantleReplacement *guards,
    size_t count) {
  size_t index;
  char expected_from[MANTLE_TEXT_BYTES_MAX];
  char expected_to[MANTLE_TEXT_BYTES_MAX];
  for (index = 0; index < count; index++) {
    if (!guards[index].global) return 0;
    if (!mantle_guard_name(names[index].from, expected_from)) return 0;
    if (!mantle_guard_name(names[index].to, expected_to)) return 0;
    if (strcmp(guards[index].from, expected_from) != 0) return 0;
    if (strcmp(guards[index].to, expected_to) != 0) return 0;
  }
  assert(count > 0);
  assert(count <= MANTLE_FILENAME_REPLACEMENT_COUNT_MAX);
  return 1;
}

static int mantle_expected_source_prefix(char *output) {
  char cwd[MANTLE_PATH_BYTES_MAX];
  char *last_slash;
  const char *leaf;
  size_t parent_len;
  size_t leaf_index;
  if (getcwd(cwd, sizeof(cwd)) == NULL) return 0;
  last_slash = strrchr(cwd, '/');
  if (last_slash == NULL || last_slash == cwd) return 0;
  leaf = last_slash + 1;
  if (strncmp(leaf, "ylwrap", strlen("ylwrap")) != 0) return 0;
  for (leaf_index = strlen("ylwrap"); leaf[leaf_index] != '\0'; leaf_index++) {
    if (leaf[leaf_index] < '0' || leaf[leaf_index] > '9') return 0;
  }
  if (leaf_index == strlen("ylwrap")) return 0;
  parent_len = (size_t)(last_slash - cwd);
  if (parent_len + MANTLE_PARENT_SLASH_TERMINATOR_BYTES > MANTLE_PATH_BYTES_MAX) return 0;
  memcpy(output, cwd, parent_len);
  output[parent_len] = '/';
  output[parent_len + 1] = '\0';
  assert(output[0] == '/');
  assert(output[parent_len] == '/');
  return 1;
}

static int mantle_build_plan(char **argv, struct MantlePlan *plan) {
  struct MantleReplacement path_pair[1];
  struct MantleReplacement name_pairs[MANTLE_FILENAME_REPLACEMENT_COUNT_MAX];
  struct MantleReplacement guard_pairs[MANTLE_FILENAME_REPLACEMENT_COUNT_MAX];
  char expected_prefix[MANTLE_PATH_BYTES_MAX];
  size_t path_count = 0;
  size_t name_count = 0;
  size_t guard_count = 0;
  size_t index;
  if (!mantle_parse_program(argv[MANTLE_PATH_PROGRAM_ARGUMENT_INDEX], path_pair, 1, &path_count)) return 0;
  if (path_count != 1 || path_pair[0].global || path_pair[0].to_len != 0) return 0;
  if (!mantle_expected_source_prefix(expected_prefix)) return 0;
  if (strcmp(path_pair[0].from, expected_prefix) != 0) return 0;
  if (!mantle_parse_program(
          argv[MANTLE_NAME_PROGRAM_ARGUMENT_INDEX], name_pairs, MANTLE_FILENAME_REPLACEMENT_COUNT_MAX, &name_count)) return 0;
  if (!mantle_validate_filename_pairs(name_pairs, name_count)) return 0;
  if (!mantle_parse_program(
          argv[MANTLE_GUARD_PROGRAM_ARGUMENT_INDEX], guard_pairs, MANTLE_FILENAME_REPLACEMENT_COUNT_MAX, &guard_count)) return 0;
  if (guard_count != name_count || !mantle_validate_guard_pairs(name_pairs, guard_pairs, name_count)) return 0;
  if (!mantle_safe_filename(argv[MANTLE_INPUT_ARGUMENT_INDEX]) ||
      !mantle_supported_generated_name(argv[MANTLE_INPUT_ARGUMENT_INDEX])) return 0;
  if (name_count == 1 && strcmp(argv[MANTLE_INPUT_ARGUMENT_INDEX], "lex.yy.c") != 0) return 0;
  if (name_count == MANTLE_FILENAME_REPLACEMENT_COUNT_MAX &&
      strcmp(argv[MANTLE_INPUT_ARGUMENT_INDEX], "lex.yy.c") == 0) return 0;
  plan->replacement_count = 0;
  plan->replacements[plan->replacement_count++] = path_pair[0];
  for (index = 0; index < name_count; index++) plan->replacements[plan->replacement_count++] = name_pairs[index];
  for (index = 0; index < guard_count; index++) plan->replacements[plan->replacement_count++] = guard_pairs[index];
  assert(plan->replacement_count <= MANTLE_REPLACEMENT_COUNT_MAX);
  assert(plan->replacement_count == 1 + name_count + guard_count);
  return 1;
}

static int mantle_validate_arguments(int argc, char **argv) {
  if (argc != MANTLE_EXPECTED_ARGUMENT_COUNT) return 0;
  if (strcmp(argv[MANTLE_GATE_FLAG_ARGUMENT_INDEX], "-e") != 0 ||
      strcmp(argv[MANTLE_GATE_PROGRAM_ARGUMENT_INDEX], "/^#/!b") != 0) return 0;
  if (strcmp(argv[MANTLE_PATH_FLAG_ARGUMENT_INDEX], "-e") != 0 ||
      strcmp(argv[MANTLE_NAME_FLAG_ARGUMENT_INDEX], "-e") != 0 ||
      strcmp(argv[MANTLE_GUARD_FLAG_ARGUMENT_INDEX], "-e") != 0) return 0;
  assert(MANTLE_YLWRAP_PROGRAM_COUNT == 4);
  assert(argc == MANTLE_EXPECTED_ARGUMENT_COUNT);
  return 1;
}

static int mantle_read_input(const char *path, struct MantleBuffer *input) {
  struct stat path_metadata;
  struct stat metadata;
  ssize_t read_len;
  size_t offset = 0;
  int descriptor;
  if (!mantle_safe_filename(path)) return 0;
  if (lstat(path, &path_metadata) != 0 || S_ISLNK(path_metadata.st_mode)) return 0;
  descriptor = open(path, O_RDONLY | O_NOFOLLOW);
  if (descriptor < 0) return 0;
  if (fstat(descriptor, &metadata) != 0 || !S_ISREG(metadata.st_mode) ||
      metadata.st_size < 0 || metadata.st_size > MANTLE_FILE_BYTES_MAX) {
    close(descriptor);
    return 0;
  }
  input->len = (size_t)metadata.st_size;
  input->bytes = malloc(input->len == 0 ? 1 : input->len);
  if (input->bytes == NULL) { close(descriptor); return 0; }
  while (offset < input->len) {
    read_len = read(descriptor, input->bytes + offset, input->len - offset);
    if (read_len < 0 && errno == EINTR) continue;
    if (read_len <= 0) { close(descriptor); free(input->bytes); return 0; }
    offset += (size_t)read_len;
  }
  if (close(descriptor) != 0 || memchr(input->bytes, '\0', input->len) != NULL) { free(input->bytes); return 0; }
  assert(offset == input->len);
  assert(input->len <= MANTLE_FILE_BYTES_MAX);
  return 1;
}

static size_t mantle_match_count(const unsigned char *input, size_t input_len, const struct MantleReplacement *item) {
  size_t count = 0;
  size_t offset = 0;
  while (offset + item->from_len <= input_len) {
    if (memcmp(input + offset, item->from, item->from_len) == 0) {
      count += 1;
      offset += item->from_len;
      if (!item->global) break;
    } else {
      offset += 1;
    }
  }
  assert(item->from_len > 0);
  assert(count <= input_len);
  return count;
}

static int mantle_replace_literal(
    const unsigned char *input,
    size_t input_len,
    const struct MantleReplacement *item,
    struct MantleBuffer *output) {
  const size_t count = mantle_match_count(input, input_len, item);
  size_t output_len = input_len;
  size_t input_offset = 0;
  size_t output_offset = 0;
  size_t replaced = 0;
  if (item->to_len > item->from_len) {
    const size_t growth = item->to_len - item->from_len;
    if (count > (MANTLE_FILE_BYTES_MAX - input_len) / growth) return 0;
    output_len += count * growth;
  } else {
    output_len -= count * (item->from_len - item->to_len);
  }
  output->bytes = malloc(output_len == 0 ? 1 : output_len);
  if (output->bytes == NULL) return 0;
  output->len = output_len;
  while (input_offset < input_len) {
    const int match = replaced < count && input_offset + item->from_len <= input_len &&
                      memcmp(input + input_offset, item->from, item->from_len) == 0;
    if (match) {
      memcpy(output->bytes + output_offset, item->to, item->to_len);
      input_offset += item->from_len;
      output_offset += item->to_len;
      replaced += 1;
    } else {
      output->bytes[output_offset++] = input[input_offset++];
    }
  }
  assert(replaced == count);
  assert(output_offset == output_len);
  return 1;
}

static int mantle_transform_line(
    const unsigned char *line,
    size_t line_len,
    const struct MantlePlan *plan,
    struct MantleBuffer *output) {
  struct MantleBuffer current;
  size_t index;
  current.bytes = malloc(line_len == 0 ? 1 : line_len);
  if (current.bytes == NULL) return 0;
  memcpy(current.bytes, line, line_len);
  current.len = line_len;
  for (index = 0; index < plan->replacement_count; index++) {
    struct MantleBuffer next;
    if (!mantle_replace_literal(current.bytes, current.len, &plan->replacements[index], &next)) {
      free(current.bytes);
      return 0;
    }
    free(current.bytes);
    current = next;
  }
  *output = current;
  assert(output->len <= MANTLE_FILE_BYTES_MAX);
  assert(plan->replacement_count > 0);
  return 1;
}

static int mantle_append(struct MantleBuffer *output, const unsigned char *bytes, size_t len) {
  unsigned char *grown;
  if (len > MANTLE_FILE_BYTES_MAX - output->len) return 0;
  grown = realloc(output->bytes, output->len + len == 0 ? 1 : output->len + len);
  if (grown == NULL) return 0;
  output->bytes = grown;
  memcpy(output->bytes + output->len, bytes, len);
  output->len += len;
  assert(output->len <= MANTLE_FILE_BYTES_MAX);
  assert(output->bytes != NULL);
  return 1;
}

static int mantle_transform(const struct MantleBuffer *input, const struct MantlePlan *plan, struct MantleBuffer *output) {
  size_t offset = 0;
  output->bytes = NULL;
  output->len = 0;
  while (offset < input->len) {
    size_t line_end = offset;
    struct MantleBuffer line_output;
    while (line_end < input->len && input->bytes[line_end] != '\n') line_end += 1;
    if (line_end < input->len) line_end += 1;
    if (input->bytes[offset] == '#') {
      if (!mantle_transform_line(input->bytes + offset, line_end - offset, plan, &line_output)) return 0;
      if (!mantle_append(output, line_output.bytes, line_output.len)) { free(line_output.bytes); return 0; }
      free(line_output.bytes);
    } else if (!mantle_append(output, input->bytes + offset, line_end - offset)) {
      return 0;
    }
    offset = line_end;
  }
  if (input->len == 0 && !mantle_append(output, (const unsigned char *)"", 0)) return 0;
  assert(offset == input->len);
  assert(output->len <= MANTLE_FILE_BYTES_MAX);
  return 1;
}

static int mantle_write_output(const struct MantleBuffer *output) {
  size_t offset = 0;
  while (offset < output->len) {
    const ssize_t written = write(STDOUT_FILENO, output->bytes + offset, output->len - offset);
    if (written < 0 && errno == EINTR) continue;
    if (written <= 0) return 0;
    offset += (size_t)written;
  }
  assert(offset == output->len);
  assert(output->len <= MANTLE_FILE_BYTES_MAX);
  return 1;
}

static int mantle_self_test(void) {
  struct MantleReplacement names[MANTLE_FILENAME_REPLACEMENT_COUNT_MAX];
  struct MantleReplacement guards[MANTLE_FILENAME_REPLACEMENT_COUNT_MAX];
  const char *valid_names =
      "s|y\\.tab\\.c|arparse.c|g;s|y\\.tab\\.h|arparse.h|g;s|y\\.output|arparse.output|g;";
  const char *valid_guards =
      "s|Y_TAB_C|ARPARSE_C|g;s|Y_TAB_H|ARPARSE_H|g;s|Y_OUTPUT|ARPARSE_OUTPUT|g;";
  const char *nested_name = "s|y\\.tab\\.c|nested/arparse.c|g;";
  const char *wrong_guard =
      "s|Y_TAB_C|WRONG_GUARD|g;s|Y_TAB_H|ARPARSE_H|g;s|Y_OUTPUT|ARPARSE_OUTPUT|g;";
  size_t name_count = 0;
  size_t guard_count = 0;
  assert(mantle_parse_program(valid_names, names, MANTLE_FILENAME_REPLACEMENT_COUNT_MAX, &name_count) == 1);
  assert(mantle_parse_program(valid_guards, guards, MANTLE_FILENAME_REPLACEMENT_COUNT_MAX, &guard_count) == 1);
  assert(name_count == guard_count);
  assert(mantle_validate_filename_pairs(names, name_count) == 1);
  assert(mantle_validate_guard_pairs(names, guards, name_count) == 1);
  assert(mantle_parse_program("s|.*|arparse.c|g;", names, MANTLE_FILENAME_REPLACEMENT_COUNT_MAX, &name_count) == 0);
  assert(mantle_parse_program(nested_name, names, MANTLE_FILENAME_REPLACEMENT_COUNT_MAX, &name_count) == 1);
  assert(mantle_validate_filename_pairs(names, name_count) == 0);
  assert(mantle_parse_program(valid_names, names, MANTLE_FILENAME_REPLACEMENT_COUNT_MAX, &name_count) == 1);
  assert(mantle_parse_program(wrong_guard, guards, MANTLE_FILENAME_REPLACEMENT_COUNT_MAX, &guard_count) == 1);
  assert(mantle_validate_guard_pairs(names, guards, name_count) == 0);
  assert(mantle_safe_filename("../escape") == 0);
  return 0;
}

int main(int argc, char **argv) {
  struct MantlePlan plan;
  struct MantleBuffer input;
  struct MantleBuffer output;
  int status = 0;
  if (argc == MANTLE_SELF_TEST_ARGUMENT_COUNT && strcmp(argv[1], "--mantle-self-test") == 0) return mantle_self_test();
  if (!mantle_validate_arguments(argc, argv)) return mantle_error("stagex-ylwrap-sed: rejected arguments\n");
  if (!mantle_build_plan(argv, &plan)) return mantle_error("stagex-ylwrap-sed: rejected transformation plan\n");
  if (!mantle_read_input(argv[MANTLE_INPUT_ARGUMENT_INDEX], &input)) {
    return mantle_error("stagex-ylwrap-sed: rejected input\n");
  }
  if (!mantle_transform(&input, &plan, &output)) {
    free(input.bytes);
    return mantle_error("stagex-ylwrap-sed: transformation failed\n");
  }
  if (!mantle_write_output(&output)) status = mantle_error("stagex-ylwrap-sed: output failed\n");
  free(output.bytes);
  free(input.bytes);
  assert(plan.replacement_count > 0);
  assert(input.len <= MANTLE_FILE_BYTES_MAX);
  return status;
}
