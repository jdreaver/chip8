# Run like "make DEBUG=1 ..." to turn on debugging symbols
DEBUG ?= 0
ifeq ($(DEBUG), 1)
    CFLAGS=-g3 -ggdb -DDEBUG
else
    CFLAGS=-DNDEBUG
endif

CC=gcc
CFLAGS+=-Wall
CFLAGS+=-Wextra
CFLAGS+=-pedantic

exe = bin/chip8
sources = $(wildcard src/*.c)
headers = $(wildcard src/*.h)

.PHONY: all
all: $(exe)

$(exe): $(sources) $(headers)
	@mkdir -p bin
	$(CC) $(CFLAGS) -o $@ $(sources)
