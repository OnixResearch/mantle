# Build a multi-file C project using make.
#
# Demonstrates the full mkDerivation pipeline:
#   - gcc + gnumake from seed
#   - Makefile with separate compilation
#   - Library (libgreet.a) + binary (greet)
#   - make install into $out
#
# Usage:
#   1. Generate seed: mantle bootstrap -o examples/seed.ncl
#   2. Build: mantle build examples/build-from-source.ncl -I examples -I builders

let mantle = import "lib.ncl" in
let builders = import "builders/lib.ncl" in
let seed = import "seed.ncl" in
let stdenv = builders.mkStdenv seed in

stdenv.mkDerivation {
  pname = "greet",
  version = "1.0",
  buildInputs = [seed.gcc, seed.gnumake],
  buildPhase = m%"
    cat > greet.h << 'EOF'
    #ifndef GREET_H
    #define GREET_H
    void greet(const char *name);
    void greet_formal(const char *name);
    int greet_count(void);
    #endif
    EOF

    cat > greet.c << 'EOF'
    #include <stdio.h>
    #include "greet.h"
    static int count = 0;
    void greet(const char *name) { printf("Hello, %s!\n", name); count++; }
    void greet_formal(const char *name) { printf("Good day, %s.\n", name); count++; }
    int greet_count(void) { return count; }
    EOF

    cat > main.c << 'EOF'
    #include <stdio.h>
    #include <string.h>
    #include "greet.h"
    int main(int argc, char *argv[]) {
        if (argc > 1 && strcmp(argv[1], "--version") == 0) {
            printf("greet 1.0 (built by mantle)\n");
            return 0;
        }
        const char *name = argc > 1 ? argv[1] : "world";
        int formal = argc > 2 && strcmp(argv[2], "--formal") == 0;
        if (formal) greet_formal(name); else greet(name);
        printf("Total greetings: %d\n", greet_count());
        return 0;
    }
    EOF

    cat > Makefile << 'MAKEOF'
    CC ?= gcc
    CFLAGS ?= -Wall -O2
    AR ?= ar

    all: greet

    greet.o: greet.c greet.h
    	$(CC) $(CFLAGS) -c -o $@ greet.c

    libgreet.a: greet.o
    	$(AR) rcs $@ $<

    main.o: main.c greet.h
    	$(CC) $(CFLAGS) -c -o $@ main.c

    greet: main.o libgreet.a
    	$(CC) $(CFLAGS) -o $@ main.o -L. -lgreet

    install: greet libgreet.a greet.h
    	mkdir -p $(DESTDIR)/bin $(DESTDIR)/lib $(DESTDIR)/include
    	cp greet $(DESTDIR)/bin/
    	cp libgreet.a $(DESTDIR)/lib/
    	cp greet.h $(DESTDIR)/include/

    clean:
    	rm -f *.o *.a greet
    MAKEOF

    make
  "%,
  installPhase = m%"
    make install DESTDIR="$out"
  "%,
}
