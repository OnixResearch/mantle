CC ?= cc
AR ?= ar
OPTIMIZATION_LEVEL ?= 2
C_STANDARD ?= c11
CPPFLAGS ?= -Iinclude
CFLAGS ?= -O$(OPTIMIZATION_LEVEL) -std=$(C_STANDARD) -Wall -Wextra -Werror
BUILD_DIR ?= build
PREFIX ?= out

LIB_OBJECT := $(BUILD_DIR)/greet.o
CLI_OBJECT := $(BUILD_DIR)/main.o
TEST_OBJECT := $(BUILD_DIR)/test_greet.o
STATIC_LIBRARY := $(BUILD_DIR)/libgreet.a
CLI := $(BUILD_DIR)/greet
TEST_BINARY := $(BUILD_DIR)/test_greet

.PHONY: all test install clean

all: $(CLI)

$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

$(LIB_OBJECT): src/greet.c include/greet.h | $(BUILD_DIR)
	$(CC) $(CPPFLAGS) $(CFLAGS) -c src/greet.c -o $@

$(CLI_OBJECT): src/main.c include/greet.h | $(BUILD_DIR)
	$(CC) $(CPPFLAGS) $(CFLAGS) -c src/main.c -o $@

$(TEST_OBJECT): tests/test_greet.c include/greet.h | $(BUILD_DIR)
	$(CC) $(CPPFLAGS) $(CFLAGS) -c tests/test_greet.c -o $@

$(STATIC_LIBRARY): $(LIB_OBJECT)
	$(AR) rcs $@ $<

$(CLI): $(CLI_OBJECT) $(STATIC_LIBRARY)
	$(CC) $(CFLAGS) $(CLI_OBJECT) $(STATIC_LIBRARY) -o $@

$(TEST_BINARY): $(TEST_OBJECT) $(STATIC_LIBRARY)
	$(CC) $(CFLAGS) $(TEST_OBJECT) $(STATIC_LIBRARY) -o $@

test: $(TEST_BINARY)
	$(TEST_BINARY)

install: all
	mkdir -p $(PREFIX)/bin $(PREFIX)/lib $(PREFIX)/include
	cp $(CLI) $(PREFIX)/bin/greet
	cp $(STATIC_LIBRARY) $(PREFIX)/lib/libgreet.a
	cp include/greet.h $(PREFIX)/include/greet.h

clean:
	rm -rf $(BUILD_DIR)
