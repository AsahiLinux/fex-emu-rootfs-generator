DESTDIR ?=
PREFIX ?= /usr
GENERATORSDIR ?= $(PREFIX)/lib/systemd/system-generators

RUSTFLAGS ?= --release

ROOTDIR := $(dir $(realpath $(lastword $(MAKEFILE_LIST))))

all: build

build:
	cargo build $(RUSTFLAGS)

check:
	cargo test $(RUSTFLAGS)

clean:
	rm -rf target

install:
	install -Dpm0755 -t $(DESTDIR)$(GENERATORSDIR)/ target/release/fex-emu-rootfs-generator

uninstall:
	rm -f $(DESTDIR)$(GENERATORSDIR)/fex-emu-rootfs-generator


.PHONY: check install uninstall
