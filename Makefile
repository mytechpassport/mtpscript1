# MTPScript Makefile - Core Runtime Build
# After successful migration, this Makefile contains only working targets

CONFIG_SMALL=y

ifdef CONFIG_WIN32
  ifdef CONFIG_X86_32
    CROSS_PREFIX?=i686-w64-mingw32-
  else
    CROSS_PREFIX?=x86_64-w64-mingw32-
  endif
  EXE=.exe
else
  CROSS_PREFIX?=
  EXE=
endif

HOST_CC=gcc
CC=$(CROSS_PREFIX)gcc
MYSQL_CFLAGS=$(shell mysql_config --include)
MYSQL_LDFLAGS=$(shell mysql_config --libs)

# Core include paths for migrated structure
CFLAGS=-Wall -g -MMD -D_GNU_SOURCE -DMTPSCRIPT_DETERMINISTIC -fno-math-errno -fno-trapping-math -I/usr/local/opt/openssl@1.1/include $(MYSQL_CFLAGS) -I. -Icore/runtime -Icore/stdlib -Icore/crypto -Icore/db -Icore/http -Icore/effects -Icore/utils -Ibuild/generated -Ibuild/templates
HOST_CFLAGS=-Wall -g -MMD -D_GNU_SOURCE -DMTPSCRIPT_DETERMINISTIC -fno-math-errno -fno-trapping-math $(MYSQL_CFLAGS) -Icore/runtime -Icore/stdlib -Icore/crypto -Icore/db -Icore/http -Icore/effects -Icore/utils -Ibuild/generated

ifdef CONFIG_SMALL
CFLAGS+=-Os
else
CFLAGS+=-O1
endif

HOST_CFLAGS+=-O3 -DHOST_BUILD
LDFLAGS=-g
HOST_LDFLAGS=-g

PROGS=mtpjs$(EXE) example$(EXE)
TEST_PROGS=dtoa_test libm_test

all: tools/mtpjs_stdlib build/generated/mquickjs_atom.h tools/example_stdlib build/generated/example_stdlib.h $(PROGS)

# Core runtime object files (migrated structure)
# Core runtime object files (migrated structure)
MTPJS_OBJS=build/objects/mtpjs.o build/objects/readline_tty.o build/objects/readline.o build/objects/mquickjs.o build/objects/mquickjs_crypto.o build/objects/mquickjs_effects.o build/objects/mquickjs_db.o build/objects/mquickjs_http.o build/objects/mquickjs_log.o build/objects/mquickjs_api.o build/objects/mquickjs_errors.o build/objects/dtoa.o build/objects/libm.o build/objects/cutils.o
LIBS=-lm -L/usr/local/opt/openssl@1.1/lib -lcrypto $(MYSQL_LDFLAGS) -lcurl

mtpjs$(EXE): $(MTPJS_OBJS)
	$(CC) $(LDFLAGS) -o $@ $^ $(LIBS)

# Stdlib generation for runtime
tools/mtpjs_stdlib: build/objects/mtpjs_stdlib.host.o build/objects/mquickjs_build.host.o
	$(HOST_CC) $(HOST_LDFLAGS) -o $@ $^

build/generated/mquickjs_atom.h: tools/mtpjs_stdlib
	./tools/mtpjs_stdlib -a > $@

build/generated/mtpjs_stdlib.h: tools/mtpjs_stdlib
	./tools/mtpjs_stdlib > $@

src/main/mtpjs.o: build/generated/mtpjs_stdlib.h

# Example program
example$(EXE): build/objects/example.o build/objects/mquickjs.o build/objects/mquickjs_crypto.o build/objects/mquickjs_effects.o build/objects/mquickjs_db.o build/objects/mquickjs_http.o build/objects/mquickjs_log.o build/objects/mquickjs_api.o build/objects/mquickjs_errors.o build/objects/dtoa.o build/objects/libm.o build/objects/cutils.o
	$(CC) $(LDFLAGS) -o $@ $^ $(LIBS)

tools/example_stdlib: build/objects/example_stdlib.host.o build/objects/mquickjs_build.host.o
	$(HOST_CC) $(HOST_LDFLAGS) -o $@ $^

build/generated/example_stdlib.h: tools/example_stdlib
	./tools/example_stdlib > $@

examples/example.o: build/generated/example_stdlib.h

# Build rules
build/objects/%.host.o: %.c
	$(HOST_CC) $(HOST_CFLAGS) -c -o $@ $<

build/objects/%.o: %.c
	$(CC) $(CFLAGS) -c -o $@ $<

# Specific rules for main objects
build/objects/mtpjs.o: src/main/mtpjs.c
	$(CC) $(CFLAGS) -c -o $@ $<

build/objects/readline.o: src/main/readline.c
	$(CC) $(CFLAGS) -c -o $@ $<

build/objects/readline_tty.o: src/main/readline_tty.c
	$(CC) $(CFLAGS) -c -o $@ $<

build/objects/example.o: examples/example.c
	$(CC) $(CFLAGS) -c -o $@ $<

# Specific rules for core objects
build/objects/mquickjs.o: core/runtime/mquickjs.c
	$(CC) $(CFLAGS) -c -o $@ $<

build/objects/mquickjs_errors.o: core/runtime/mquickjs_errors.c
	$(CC) $(CFLAGS) -c -o $@ $<

build/objects/mquickjs_api.o: core/stdlib/mquickjs_api.c
	$(CC) $(CFLAGS) -c -o $@ $<

build/objects/mquickjs_crypto.o: core/crypto/mquickjs_crypto.c
	$(CC) $(CFLAGS) -c -o $@ $<

build/objects/mquickjs_effects.o: core/effects/mquickjs_effects.c
	$(CC) $(CFLAGS) -c -o $@ $<

build/objects/mquickjs_db.o: core/db/mquickjs_db.c
	$(CC) $(CFLAGS) -c -o $@ $<

build/objects/mquickjs_http.o: core/http/mquickjs_http.c
	$(CC) $(CFLAGS) -c -o $@ $<

build/objects/mquickjs_log.o: core/effects/mquickjs_log.c
	$(CC) $(CFLAGS) -c -o $@ $<

build/objects/dtoa.o: core/utils/dtoa.c
	$(CC) $(CFLAGS) -c -o $@ $<

build/objects/libm.o: core/utils/libm.c
	$(CC) $(CFLAGS) -c -o $@ $<

build/objects/cutils.o: core/utils/cutils.c
	$(CC) $(CFLAGS) -c -o $@ $<

# Specific rules for host objects
build/objects/mtpjs_stdlib.host.o: src/stdlib/mtpjs_stdlib.c
	$(HOST_CC) $(HOST_CFLAGS) -c -o $@ $<

build/objects/mquickjs_build.host.o: src/stdlib/mquickjs_build.c
	$(HOST_CC) $(HOST_CFLAGS) -c -o $@ $<

build/objects/example_stdlib.host.o: examples/example_stdlib.c
	$(HOST_CC) $(HOST_CFLAGS) -c -o $@ $<

# Test targets
test: mtpjs example
	./mtpjs tests/integration/test_closure.js
	./mtpjs tests/integration/test_language.js
	./mtpjs tests/integration/test_loop.js
	./mtpjs tests/integration/test_builtin.js
	./mtpjs -o test_builtin.bin tests/integration/test_builtin.js
	./mtpjs -b test_builtin.bin
	./example tests/integration/test_rect.js

microbench: mtpjs
	./mtpjs tests/microbench.js

octane: mtpjs
	./mtpjs --memory-limit 256M tests/octane/run.js

size: mtpjs
	size mtpjs mtpjs.o

# Unit tests
dtoa_test: tests/dtoa_test.o core/utils/dtoa.o core/utils/cutils.o tests/gay-fixed.o tests/gay-precision.o tests/gay-shortest.o
	$(CC) $(LDFLAGS) -o $@ $^ $(LIBS)

libm_test: tests/libm_test.o core/utils/libm.o
	$(CC) $(LDFLAGS) -o $@ $^ $(LIBS)

# Cleanup
clean:
	rm -f *.o *.d *~ tests/*.o tests/*.d tests/*~ test_builtin.bin
	rm -rf build/generated/* build/artifacts/* build/objects/* build/*.json
	rm -f tools/mtpjs_stdlib tools/example_stdlib
	rm -f $(PROGS) $(TEST_PROGS)

-include $(wildcard *.d)
