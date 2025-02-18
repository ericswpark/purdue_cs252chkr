.DEFAULT_GOAL := all

CARGO = cargo
BUILD_OPTS = --release
DEV_BUILD_OPTS =
SOURCE_FILES = src/*

.PHONY: all
all: ${SOURCE_FILES}
	${CARGO} build ${BUILD_OPTS}

# Clean build files
# Caution: this will also remove downloaded dependencies! It is usually never necessary to do a clean Rust build.
.PHONY: clean
clean:
	${CARGO} clean

# Build development version with debug symbols intact
.PHONY: dev
dev: ${SOURCE_FILES}
	${CARGO} build ${DEV_BUILD_OPTS}