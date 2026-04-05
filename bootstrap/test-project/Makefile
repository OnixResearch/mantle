CC ?= gcc
CFLAGS ?= -Wall -Wextra -O2 -static
LDFLAGS ?= -static

SRCS = main.c vec.c hash.c
OBJS = $(SRCS:.c=.o)
TARGET = test-project

all: $(TARGET)
	@echo "=== running tests ==="
	./$(TARGET)

$(TARGET): $(OBJS)
	$(CC) $(LDFLAGS) -o $@ $^

%.o: %.c
	$(CC) $(CFLAGS) -c -o $@ $<

clean:
	rm -f $(OBJS) $(TARGET)

install: $(TARGET)
	mkdir -p $(DESTDIR)/bin
	cp $(TARGET) $(DESTDIR)/bin/

.PHONY: all clean install
