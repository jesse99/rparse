RUSTC ?= rustc

# ------------------
# Internal variables
dummy1 := $(shell mkdir bin 2> /dev/null)

# ------------------
# Primary targets
all: parser

parser: lib

check: bin/test-rparse
	export RUST_LOG=rparse=3 && ./bin/test-rparse

# ------------------
# Binary targets 
# We always build the lib because:
# 1) We don't do it that often.
# 2) It's fast.
# 3) The compiler gives it some crazy name like "librparse-da45653350eb4f90-0.1.dylib"
# which is dependent on some hash(?) as well as the current platform. (And -o works when
# setting an executable's name, but not libraries).
.PHONY : lib
lib:
	$(RUSTC) --out-dir bin -O src/parser.rc

bin/test-rparse: src/parser.rc src/*.rs src/tests/*.rs
	$(RUSTC) -g --test -o $@ src/parser.rc
