CARGO = cargo

.PHONY: all
all: src/*
    ${CARGO} build --release