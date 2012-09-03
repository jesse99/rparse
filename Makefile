RUSTC ?= rustc

# ------------------
# Internal variables
dummy1 := $(shell mkdir bin 2> /dev/null)

# ------------------
# Primary targets
all: parser

parser: lib

check: bin/test-rparse
	export RUST_LOG=rparse=1 && ./bin/test-rparse

check1: bin/test-rparse
	export RUST_LOG=rparse=3 && ./bin/test-rparse test_expr::test_expr

# Run unit tests with optimizations enabled (which is how we build the lib).
check-release: bin/test-rparse-release
	export RUST_LOG=rparse=1 && ./bin/test-rparse-release

install:
	install `find bin -name "librparse*" -type f -maxdepth 1` /usr/local/lib/rust/

clean:
	rm -rf bin
	
dist: lib
	tar --create --compress --exclude \*/.git --exclude \*/.git/\* --file=rparse-0.6.tar.gz \
		CHANGES MIT.X11 Makefile README.md rparse.rtf src

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
	$(RUSTC) --lib --out-dir bin -O src/crate.rc

bin/test-rparse: src/crate.rc src/*.rs src/tests/*.rs
	$(RUSTC) --test -o $@ $<

bin/test-rparse-release: src/crate.rc src/*.rs src/tests/*.rs
	$(RUSTC) --test -O -o $@ $<
