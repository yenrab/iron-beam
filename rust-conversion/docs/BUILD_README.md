# Build System for Rust Emulator + Erlang .beam Files

This Makefile builds the Rust emulator and compiles Erlang source files to `.beam` files, placing them in the correct directory structure for the Rust code to load.

## Version Detection

Versions are automatically detected from `vsn.mk` files in each Erlang/OTP library directory:

- **STDLIB_VSN**: Read from `$(ERL_TOP)/lib/stdlib/vsn.mk`
- **KERNEL_VSN**: Read from `$(ERL_TOP)/lib/kernel/vsn.mk`
- **COMPILER_VSN**: Read from `$(ERL_TOP)/lib/compiler/vsn.mk`
- **SASL_VSN**: Read from `$(ERL_TOP)/lib/sasl/vsn.mk`

### How Version Detection Works

The Makefile uses Make's `include` directive to read version variables:

```makefile
-include $(ERL_TOP)/lib/stdlib/vsn.mk
-include $(ERL_TOP)/lib/kernel/vsn.mk
-include $(ERL_TOP)/lib/compiler/vsn.mk
-include $(ERL_TOP)/lib/sasl/vsn.mk
```

The `-` prefix means the Makefile won't fail if a `vsn.mk` file is missing (it will use fallback values).

### Example vsn.mk File

Each `vsn.mk` file contains a simple variable definition:

```makefile
STDLIB_VSN = 7.1
```

When the Makefile includes this file, it can use `$(STDLIB_VSN)` to get the version.

### Fallback Versions

If a `vsn.mk` file is not found, the Makefile uses fallback versions:

- `STDLIB_VSN ?= 7.1`
- `KERNEL_VSN ?= 10.4.1`
- `COMPILER_VSN ?= 9.0.2`
- `SASL_VSN ?= 4.3`

## Directory Structure

The build system creates the following structure:

```
target/otp_root/
├── bin/
│   └── beam                    # Rust emulator binary
└── lib/
    ├── stdlib-7.1/
    │   ├── ebin/               # Compiled .beam files go here
    │   ├── src/                # Copied .erl source files
    │   └── include/            # Copied .hrl header files
    ├── kernel-10.4.1/
    │   ├── ebin/
    │   ├── src/
    │   └── include/
    ├── compiler-9.0.2/
    │   ├── ebin/
    │   └── src/
    └── sasl-4.3/
        ├── ebin/
        └── src/
```

The Rust code expects `.beam` files in `$ROOTDIR/lib/<app>-<version>/ebin/`, which matches this structure.

## Usage

### Basic Build

```bash
cd rust-conversion/rust
make
```

This will:
1. Build the Rust emulator using `cargo build --release`
2. Compile all `.erl` files to `.beam` files
3. Place them in the correct directory structure

### Build in Debug Mode

```bash
make RUST_BUILD_TYPE=debug
```

### Custom Installation Directory

```bash
make ROOTDIR=/opt/otp
```

### Custom Erlang/OTP Source Location

```bash
make ERL_TOP=/path/to/erlang/otp
```

### Run the Emulator

After building:

```bash
cd target/otp_root
ROOTDIR=$(pwd) ./bin/beam
```

Or set ROOTDIR as an environment variable:

```bash
export ROOTDIR=/path/to/target/otp_root
./bin/beam
```

## Targets

- `make` or `make all` - Build everything (Rust + .beam files)
- `make rust` - Build only the Rust emulator
- `make beam` - Compile only the .beam files
- `make install` - Install binary to BINDIR (same as `all` but explicit)
- `make clean` - Remove all build artifacts
- `make help` - Show help message with version information

## How It Works

1. **Rust Build**: Uses `cargo build` to compile the Rust emulator
2. **Version Detection**: Reads `vsn.mk` files to get application versions
3. **Source Copying**: Copies `.erl` and `.hrl` files from Erlang/OTP source
4. **Compilation**: Uses `erlc` to compile `.erl` files to `.beam` files
5. **Installation**: Copies the Rust binary to `$ROOTDIR/bin/beam`

## Troubleshooting

### Versions Not Detected

If versions are not being detected, check:

1. `ERL_TOP` is set correctly (defaults to `../../`)
2. The `vsn.mk` files exist in `$(ERL_TOP)/lib/*/vsn.mk`
3. The files contain the correct variable names (e.g., `STDLIB_VSN`)

You can verify versions with:

```bash
make help
```

This will show the detected versions.

### Missing .beam Files

If `.beam` files are not being generated:

1. Check that `erlc` is in your PATH
2. Verify that `.erl` source files exist in `$(ERL_TOP)/lib/*/src/`
3. Check compilation errors (they may be hidden by grep filtering)

To see compilation output:

```bash
make beam 2>&1 | grep -v "Warning"
```

### Rust Code Can't Find .beam Files

Ensure:

1. `ROOTDIR` environment variable is set when running the emulator
2. The directory structure matches: `$ROOTDIR/lib/<app>-<version>/ebin/`
3. The `.beam` files were actually created (check `target/otp_root/lib/*/ebin/`)

