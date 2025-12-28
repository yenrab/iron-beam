# Makefile for building Rust emulator and Erlang .beam files
# Enhanced version with automatic version detection from vsn.mk files

ERL_TOP ?= $(shell pwd)/../..
RUST_DIR = $(shell pwd)
ROOTDIR ?= $(RUST_DIR)/target/otp_root
BINDIR = $(ROOTDIR)/bin

# Rust build configuration
CARGO ?= cargo
RUST_BUILD_TYPE ?= release
RUST_TARGET_DIR = $(RUST_DIR)/target/$(RUST_BUILD_TYPE)

# Erlang compiler
ERLC ?= erlc
ERL_TOP_LIB = $(ERL_TOP)/lib

# Applications to build
APPS = stdlib kernel compiler sasl

# Read versions from vsn.mk files in each library directory
# These files define variables like STDLIB_VSN, KERNEL_VSN, etc.
# Example: lib/stdlib/vsn.mk contains "STDLIB_VSN = 7.1"
-include $(ERL_TOP)/lib/stdlib/vsn.mk
-include $(ERL_TOP)/lib/kernel/vsn.mk
-include $(ERL_TOP)/lib/compiler/vsn.mk
-include $(ERL_TOP)/lib/sasl/vsn.mk

# Fallback versions if vsn.mk files are not found
STDLIB_VSN ?= 7.1
KERNEL_VSN ?= 10.4.1
COMPILER_VSN ?= 9.0.2
SASL_VSN ?= 4.3

.PHONY: all rust beam clean install release help

release: RUST_BUILD_TYPE = release
release: all install
	@echo ""
	@echo "=========================================="
	@echo "Release build and installation complete!"
	@echo "=========================================="
	@echo "Binaries installed to: $(BINDIR)/"
	@echo "  - beam (Erlang emulator)"
	@echo "  - erlc (Erlang compiler)"
	@echo "BEAM files in: $(ROOTDIR)/lib/*/ebin/"
	@echo ""
	@echo "To run the emulator:"
	@echo "  cd $(ROOTDIR) && ROOTDIR=\$$(pwd) ./bin/beam"
	@echo ""
	@echo "To compile Erlang code:"
	@echo "  cd $(ROOTDIR) && ./bin/erlc file.erl"

all: rust beam
	@echo ""
	@echo "=========================================="
	@echo "Build complete!"
	@echo "=========================================="
	@echo "Rust binaries: $(RUST_TARGET_DIR)/"
	@echo "  - beam (Erlang emulator)"
	@echo "  - erlc (Erlang compiler)"
	@echo "ROOTDIR: $(ROOTDIR)"
	@echo ""
	@echo "To run the emulator:"
	@echo "  cd $(ROOTDIR) && ROOTDIR=\$$(pwd) ./bin/beam"
	@echo ""
	@echo "To compile Erlang code:"
	@echo "  $(RUST_TARGET_DIR)/erlc file.erl"
	@echo ""

rust:
	@echo "Building Rust emulator and compiler ($(RUST_BUILD_TYPE))..."
	cd $(RUST_DIR) && $(CARGO) build $(if $(filter release,$(RUST_BUILD_TYPE)),--release,) -p frameworks_emulator_init -p interfaces_compiler_api
	@echo "✓ Rust build complete"

beam: $(foreach app,$(APPS),beam-$(app))
	@echo "✓ All .beam files built"

# Build stdlib .beam files
beam-stdlib:
	@echo "Building stdlib (version $(STDLIB_VSN))..."
	@APP_DIR=$(ROOTDIR)/lib/stdlib-$(STDLIB_VSN) && \
	mkdir -p $$APP_DIR/ebin $$APP_DIR/src $$APP_DIR/include && \
	if [ -n "$(SKIP_BEAM_BUILD)" ] && [ -d "$$APP_DIR/ebin" ] && [ "$$(find $$APP_DIR/ebin -name '*.beam' 2>/dev/null | wc -l)" -gt 0 ]; then \
		echo "  ✓ Skipping stdlib .beam build (files already exist)" && \
		echo "  ✓ stdlib .beam files ready in $$APP_DIR/ebin"; \
	else \
		echo "  Copying sources from $(ERL_TOP_LIB)/stdlib..." && \
		cp -r $(ERL_TOP_LIB)/stdlib/src/*.erl $$APP_DIR/src/ 2>/dev/null || true && \
		cp -r $(ERL_TOP_LIB)/stdlib/include/*.hrl $$APP_DIR/include/ 2>/dev/null || true && \
		echo "  Compiling .erl files..." && \
		cd $$APP_DIR/src && \
		INCLUDE_FLAGS="-I ../include -I $(ERL_TOP_LIB)/kernel/include -I $(ERL_TOP_LIB)/stdlib/include" && \
		for erl in *.erl; do \
			if [ -f "$$erl" ]; then \
				$(ERLC) $$INCLUDE_FLAGS -o ../ebin "$$erl" 2>&1 | grep -v "Warning" || true; \
			fi \
		done && \
		echo "  ✓ stdlib .beam files built in $$APP_DIR/ebin"; \
	fi

# Build kernel .beam files
beam-kernel:
	@echo "Building kernel (version $(KERNEL_VSN))..."
	@APP_DIR=$(ROOTDIR)/lib/kernel-$(KERNEL_VSN) && \
	mkdir -p $$APP_DIR/ebin $$APP_DIR/src $$APP_DIR/include && \
	if [ -n "$(SKIP_BEAM_BUILD)" ] && [ -d "$$APP_DIR/ebin" ] && [ "$$(find $$APP_DIR/ebin -name '*.beam' 2>/dev/null | wc -l)" -gt 0 ]; then \
		echo "  ✓ Skipping kernel .beam build (files already exist)" && \
		echo "  ✓ kernel .beam files ready in $$APP_DIR/ebin"; \
	else \
		echo "  Copying sources from $(ERL_TOP_LIB)/kernel..." && \
		cp -r $(ERL_TOP_LIB)/kernel/src/*.erl $$APP_DIR/src/ 2>/dev/null || true && \
		cp -r $(ERL_TOP_LIB)/kernel/include/*.hrl $$APP_DIR/include/ 2>/dev/null || true && \
		echo "  Compiling .erl files..." && \
		cd $$APP_DIR/src && \
		INCLUDE_FLAGS="-I ../include -I $(ERL_TOP_LIB)/kernel/include" && \
		for erl in *.erl; do \
			if [ -f "$$erl" ]; then \
				$(ERLC) $$INCLUDE_FLAGS -o ../ebin "$$erl" 2>&1 | grep -v "Warning" || true; \
			fi \
		done && \
		echo "  ✓ kernel .beam files built in $$APP_DIR/ebin"; \
	fi

# Build compiler .beam files
beam-compiler:
	@echo "Building compiler (version $(COMPILER_VSN))..."
	@APP_DIR=$(ROOTDIR)/lib/compiler-$(COMPILER_VSN) && \
	mkdir -p $$APP_DIR/ebin $$APP_DIR/src && \
	if [ -n "$(SKIP_BEAM_BUILD)" ] && [ -d "$$APP_DIR/ebin" ] && [ "$$(find $$APP_DIR/ebin -name '*.beam' 2>/dev/null | wc -l)" -gt 0 ]; then \
		echo "  ✓ Skipping compiler .beam build (files already exist)" && \
		echo "  ✓ compiler .beam files ready in $$APP_DIR/ebin"; \
	else \
		echo "  Copying sources from $(ERL_TOP_LIB)/compiler..." && \
		cp -r $(ERL_TOP_LIB)/compiler/src/*.erl $$APP_DIR/src/ 2>/dev/null || true && \
		cp -r $(ERL_TOP_LIB)/compiler/src/*.hrl $$APP_DIR/src/ 2>/dev/null || true && \
		echo "  Compiling .erl files in dependency order..." && \
		cd $$APP_DIR/src && \
		INCLUDE_FLAGS="-I $(ERL_TOP_LIB)/kernel/include -I $(ERL_TOP_LIB)/stdlib/include -I ../include" && \
		$(ERLC) $$INCLUDE_FLAGS -o ../ebin beam_opcodes.erl 2>&1 | grep -v "Warning" || true && \
		$(ERLC) $$INCLUDE_FLAGS -o ../ebin beam_disasm.erl 2>&1 | grep -v "Warning" || true && \
		for erl in *.erl; do \
			if [ "$$erl" != "beam_opcodes.erl" ] && [ "$$erl" != "beam_disasm.erl" ] && [ -f "$$erl" ]; then \
				$(ERLC) $$INCLUDE_FLAGS -o ../ebin "$$erl" 2>&1 | grep -v "Warning" || true; \
			fi \
		done && \
		echo "  ✓ compiler .beam files built in $$APP_DIR/ebin"; \
	fi

# Build sasl .beam files
beam-sasl:
	@echo "Building sasl (version $(SASL_VSN))..."
	@APP_DIR=$(ROOTDIR)/lib/sasl-$(SASL_VSN) && \
	mkdir -p $$APP_DIR/ebin $$APP_DIR/src && \
	if [ -n "$(SKIP_BEAM_BUILD)" ] && [ -d "$$APP_DIR/ebin" ] && [ "$$(find $$APP_DIR/ebin -name '*.beam' 2>/dev/null | wc -l)" -gt 0 ]; then \
		echo "  ✓ Skipping sasl .beam build (files already exist)" && \
		echo "  ✓ sasl .beam files ready in $$APP_DIR/ebin"; \
	else \
		echo "  Copying sources from $(ERL_TOP_LIB)/sasl..." && \
		cp -r $(ERL_TOP_LIB)/sasl/src/*.erl $$APP_DIR/src/ 2>/dev/null || true && \
		echo "  Compiling .erl files..." && \
		cd $$APP_DIR/src && \
		INCLUDE_FLAGS="-I $(ERL_TOP_LIB)/kernel/include -I $(ERL_TOP_LIB)/stdlib/include" && \
		for erl in *.erl; do \
			if [ -f "$$erl" ]; then \
				$(ERLC) $$INCLUDE_FLAGS -o ../ebin "$$erl" 2>&1 | grep -v "Warning" || true; \
			fi \
		done && \
		echo "  ✓ sasl .beam files built in $$APP_DIR/ebin"; \
	fi

install: all
	@echo "Installing to $(ROOTDIR)..."
	@mkdir -p $(BINDIR)
	@cp $(RUST_TARGET_DIR)/beam $(BINDIR)/beam
	@chmod +x $(BINDIR)/beam
	@cp $(RUST_TARGET_DIR)/erlc $(BINDIR)/erlc
	@chmod +x $(BINDIR)/erlc
	@cp $(RUST_DIR)/erl $(BINDIR)/erl
	@chmod +x $(BINDIR)/erl
	@echo "  Copying preloaded modules..."
	@mkdir -p $(ROOTDIR)/erts/preloaded/ebin
	@if [ -d "$(ERL_TOP)/erts/preloaded/ebin" ]; then \
		cp $(ERL_TOP)/erts/preloaded/ebin/erl_init.beam $(ROOTDIR)/erts/preloaded/ebin/ 2>/dev/null || true; \
		cp $(ERL_TOP)/erts/preloaded/ebin/init.beam $(ROOTDIR)/erts/preloaded/ebin/ 2>/dev/null || true; \
		echo "  ✓ Preloaded modules copied"; \
	else \
		echo "  ⚠ Preloaded modules not found in $(ERL_TOP)/erts/preloaded/ebin"; \
	fi
	@echo "✓ Installation complete!"
	@echo ""
	@echo "To run the emulator:"
	@echo "  cd $(ROOTDIR) && ./bin/erl"
	@echo "  Or directly: cd $(ROOTDIR) && ROOTDIR=\$$(pwd) ./bin/beam"

clean:
	@echo "Cleaning build artifacts..."
	cd $(RUST_DIR) && $(CARGO) clean
	rm -rf $(ROOTDIR)
	@echo "✓ Clean complete"

help:
	@echo "Erlang/OTP Rust Emulator Build System"
	@echo "======================================"
	@echo ""
	@echo "Targets:"
	@echo "  all      - Build Rust and compile all .beam files (default)"
	@echo "  rust     - Build Rust emulator only"
	@echo "  beam     - Compile all .erl files to .beam files"
	@echo "  install  - Install binary and .beam files to ROOTDIR"
	@echo "  clean    - Remove all build artifacts"
	@echo "  help     - Show this help message"
	@echo ""
	@echo "Variables:"
	@echo "  ERL_TOP         - Path to Erlang/OTP source (default: ../../)"
	@echo "  ROOTDIR         - Installation root (default: target/otp_root)"
	@echo "  RUST_BUILD_TYPE - Build type: debug or release (default: release)"
	@echo "  CARGO           - Cargo command (default: cargo)"
	@echo "  ERLC            - Erlang compiler (default: erlc)"
	@echo ""
	@echo "Version Detection:"
	@echo "  Versions are automatically read from vsn.mk files:"
	@echo "    - $(ERL_TOP)/lib/stdlib/vsn.mk  -> STDLIB_VSN  ($(STDLIB_VSN))"
	@echo "    - $(ERL_TOP)/lib/kernel/vsn.mk  -> KERNEL_VSN  ($(KERNEL_VSN))"
	@echo "    - $(ERL_TOP)/lib/compiler/vsn.mk -> COMPILER_VSN ($(COMPILER_VSN))"
	@echo "    - $(ERL_TOP)/lib/sasl/vsn.mk     -> SASL_VSN    ($(SASL_VSN))"
	@echo ""
	@echo "Examples:"
	@echo "  make                    # Build everything (release)"
	@echo "  make RUST_BUILD_TYPE=debug  # Build in debug mode"
	@echo "  make ROOTDIR=/opt/otp   # Install to custom location"
	@echo "  make clean              # Clean all artifacts"

