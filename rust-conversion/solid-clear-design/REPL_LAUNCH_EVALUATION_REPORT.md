# Erlang REPL Launch Path and Runtime Integration Evaluation Report

**Tool:** startup_suggest  
**Date:** Generated  
**Purpose:** Evaluate Erlang REPL launch path and runtime integration with Rust code, providing recommendations for pure Rust implementation following Rust best practices

---

## Executive Summary

This report evaluates the current C-based Erlang REPL launch sequence and provides recommendations for migrating to a **pure Rust implementation with zero C code dependencies**. The analysis covers:

- **Current Launch Path:** `erl` script → `erlexec` (C) → `erl_main.c` (C) → `erl_start()` → `erl_init()`
- **Rust Implementation Status:** Partial Rust implementation exists in `frameworks_emulator_init` crate with `erl_start()` and `erl_init()` functions
- **Key Finding:** No Rust binary entry point (`main.rs`) exists; the Rust code is currently designed as a library that would be called from C
- **Critical Gap:** The launch path requires two C programs (`erlexec` and `erl_main.c`) to start the REPL

**Recommendation:** Create a **single Rust binary** that combines the functionality of both `erlexec` (C) and `erl_main.c` (C), allowing the `erl` script to launch a pure Rust binary directly with **zero C code in the launch path**. All `erlexec` functionality (argument parsing, environment setup, epmd management, boot/config resolution, etc.) must be implemented in Rust and integrated into the `main()` function, completely eliminating the need for the C `erlexec` program.

---

## Current State Analysis

### Launch Sequence Overview

#### Current C-Based Launch Path

The current Erlang REPL launch sequence follows this path:

1. **User invokes `erl` script** (`erts/etc/unix/erl.src`)
   - Shell script that sets up environment and calls `erlexec`

2. **`erlexec` executable** (`erts/etc/common/erlexec.c`) - **C CODE**
   - C program that:
     - Parses command-line arguments
     - Handles distribution setup (epmd)
     - Manages boot script and config file paths
     - Sets up environment variables (ROOTDIR, BINDIR, PROGNAME, PATH)
     - Constructs argument array for emulator
     - Calls `start_emulator()` which uses `execv()`/`execvp()` to launch the emulator binary

3. **Emulator binary** (`beam` or `beam.smp`)
   - Entry point: `erl_main.c` (Unix) or `erl_main.c` (Windows) - **C CODE**
   - `main()` function calls `erl_start(argc, argv)`

4. **`erl_start()` function** (`erts/emulator/beam/erl_init.c`)
   - Calls `early_init()` for early initialization
   - Parses command-line arguments
   - Calls `erl_init()` for main initialization
   - Starts the emulator loop

5. **`erl_init()` function** (`erts/emulator/beam/erl_init.c`)
   - Initializes all runtime components in sequence:
     - Global literals
     - Process management
     - Scheduling
     - BIF dispatcher
     - Emulator loop
     - Runtime utilities

#### Proposed Pure Rust Launch Path

The target pure Rust launch sequence:

1. **User invokes `erl` script** (`erts/etc/unix/erl.src`)
   - Shell script that calls Rust binary directly (no `erlexec`)

2. **Rust emulator binary** (`beam` or `beam.smp`) - **PURE RUST**
   - Entry point: `src/main.rs` in `frameworks_emulator_init` crate
   - `main()` function that:
     - Initializes signal stack
     - Parses command-line arguments (replaces `erlexec` functionality)
     - Sets up environment variables
     - Manages epmd daemon startup
     - Calls `erl_start()` directly (no separate binary launch)

3. **`erl_start()` function** (Rust implementation)
   - Calls `early_init()` for early initialization
   - Parses command-line arguments
   - Calls `erl_init()` for main initialization
   - Starts the emulator loop

4. **`erl_init()` function** (Rust implementation)
   - Initializes all runtime components in sequence

### Responsibilities at Each Step

#### `erlexec` Responsibilities (To Be Replaced in Rust)

**Current C Implementation:** `erts/etc/common/erlexec.c`

**Core Functions:**
- **Argument Processing:** Parses `-boot`, `-config`, `-sname`, `-name`, distribution flags, `-smp`, `-emu_type`, `-emu_flavor`, `-extra`, `-args_file`, etc.
- **Environment Setup:** Sets up `ROOTDIR`, `BINDIR`, `PROGNAME` environment variables
- **PATH Management:** Modifies PATH to include bindir and rootdir/bin, removes duplicates
- **epmd Management:** Starts epmd daemon if needed for distribution (calls `epmd -daemon`)
- **Emulator Selection:** Determines which emulator binary to launch (beam, beam.smp, etc.) based on `-smp` flags
- **Argument Construction:** Builds final argument array with `-root`, `-bindir`, `-progname` flags
- **Process Launch:** Uses `execv()`/`execvp()` to replace current process with emulator binary
- **Special Modes:** Handles `-emu_args_exit`, `-emu_name_exit`, `-emu_qouted_cmd_exit` for tooling compatibility
- **Detached Mode:** Handles detached emulator startup on Windows

**Rust Replacement Strategy (CRITICAL - Zero C Code):**
- **Integrate ALL `erlexec` functionality directly into Rust `main()` function**
- **No process replacement** - call `erl_start()` directly instead of using `execv()`/`execvp()`
- Use Rust argument parsing (e.g., `clap` crate) instead of manual C string parsing
- Use Rust environment variable handling (`std::env`) instead of C getenv/setenv
- Use Rust process spawning (`std::process::Command`) for epmd daemon instead of `system()` call
- Implement all argument processing logic in Rust (boot/config, distribution flags, etc.)
- Implement PATH manipulation in Rust (add bindir, remove duplicates)
- Handle all special modes (`-emu_args_exit`, `-emu_name_exit`, `-emu_qouted_cmd_exit`) in Rust
- Implement detached mode handling in Rust (Windows-specific)

#### `erl_main.c` Responsibilities
- **Signal Stack Initialization:** Sets up signal stack before scheduler threads are created
- **Entry Point:** Provides `main()` function that calls `erl_start()`
- **Minimal Logic:** Very thin wrapper around `erl_start()`

#### `erl_start()` Responsibilities
- **Early Initialization:** Calls `early_init()` for:
  - Command-line argument parsing
  - Memory allocator initialization
  - Thread progress setup
  - CPU topology detection
- **Configuration:** Parses command-line arguments for runtime configuration
- **Main Initialization:** Calls `erl_init()` to initialize all components
- **Startup Coordination:** Coordinates the complete startup sequence

#### `erl_init()` Responsibilities
- **Component Initialization:** Initializes all runtime components in correct order:
  - Global literals
  - Process management (`erts_init_process`)
  - Scheduling (`erts_init_scheduling`)
  - BIF dispatcher (`erts_init_bif`)
  - Emulator loop (`init_emulator`)
  - Runtime utilities

### Rust Implementation Status

#### Existing Rust Code

**Location:** `rust-conversion/rust/frameworks/frameworks_emulator_init/`

**Structure:**
- `lib.rs`: Library root, exports modules
- `early_init.rs`: Implements `early_init()` function
- `main_init.rs`: Implements `erl_start()` and `erl_init()` functions
- `initialization.rs`: Initialization state management

**Current Implementation:**

1. **`early_init()`** (`early_init.rs`)
   - ✅ CPU detection using `std::thread::available_parallelism()`
   - ✅ Scheduler calculation
   - ✅ Calls `infrastructure_runtime_utils::erts_init_utils()`
   - ✅ Calls `infrastructure_runtime_utils::erts_init_utils_mem()`
   - ⚠️ Missing: Full command-line argument parsing
   - ⚠️ Missing: Term system initialization (`erts_term_init()`)
   - ⚠️ Missing: Emulator argument saving (`erts_save_emu_args()`)

2. **`erl_start()`** (`main_init.rs`)
   - ✅ Calls `early_init()`
   - ✅ Builds `InitConfig` from early init results
   - ✅ Calls `erl_init()`
   - ⚠️ Missing: Command-line argument parsing for configuration overrides
   - ⚠️ Missing: Environment variable reading (e.g., `ERL_MAX_ETS_TABLES`)
   - ⚠️ Missing: Full argument processing logic from C implementation

3. **`erl_init()`** (`main_init.rs`)
   - ✅ Calls `infrastructure_runtime_utils::erts_init_utils()`
   - ✅ Calls `infrastructure_runtime_utils::erts_utils_sched_spec_data_init()`
   - ⚠️ Missing: Global literals initialization
   - ⚠️ Missing: Process management initialization (`erts_init_process`)
   - ⚠️ Missing: Scheduling initialization (`erts_init_scheduling`)
   - ⚠️ Missing: BIF dispatcher initialization (`erts_init_bif`)
   - ⚠️ Missing: Emulator loop initialization (`init_emulator`)

#### Missing Components

1. **No Rust Binary Entry Point**
   - ❌ No `main.rs` file in `frameworks_emulator_init` crate
   - ❌ No `[[bin]]` section in `Cargo.toml`
   - ❌ Current implementation is library-only

2. **No `erlexec` Functionality in Rust (CRITICAL GAP)**
   - ❌ No command-line argument parsing (must replace `erlexec` argument processing)
   - ❌ No environment variable setup (ROOTDIR, BINDIR, PROGNAME, PATH manipulation)
   - ❌ No epmd daemon management (must start epmd before emulator initialization)
   - ❌ No boot/config file path resolution
   - ❌ No emulator selection logic (beam vs beam.smp)
   - ❌ No special modes handling (`-emu_args_exit`, `-emu_name_exit`, `-emu_qouted_cmd_exit`)
   - ❌ No detached mode handling
   - ❌ No PATH manipulation (add bindir, remove duplicates)

3. **No Signal Stack Initialization**
   - ❌ `sys_init_signal_stack()` equivalent not implemented in Rust
   - ⚠️ Critical for scheduler thread safety

4. **Incomplete Initialization Sequence**
   - Many placeholder comments indicating missing implementations
   - Process management, scheduling, BIF dispatcher, emulator loop not initialized

### Build System Analysis

#### Current Cargo.toml Structure

**Workspace:** `rust-conversion/rust/Cargo.toml`
- ✅ Proper workspace configuration
- ✅ All crates listed as members
- ✅ Release profile configured with LTO and optimizations

**frameworks_emulator_init Cargo.toml:**
- ✅ Proper dependencies on entities, infrastructure, and usecases layers
- ❌ No `[[bin]]` section for binary target
- ✅ Library crate configuration is correct

#### Binary Configuration Gap

The `frameworks_emulator_init` crate is currently configured as a library only. To create a pure Rust binary entry point that **replaces both `erlexec` and `erl_main.c`**:

1. **Add `[[bin]]` section to `Cargo.toml`:**
   ```toml
   [[bin]]
   name = "beam"
   path = "src/main.rs"
   ```

2. **Create `src/main.rs`** with **ALL `erlexec` functionality integrated**:
   - **Command-line argument parsing** (complete replacement of `erlexec` argument processing)
     - Parse all flags: `-boot`, `-config`, `-sname`, `-name`, `-smp`, `-extra`, `-args_file`, etc.
     - Handle special modes: `-emu_args_exit`, `-emu_name_exit`, `-emu_qouted_cmd_exit`
     - Validate argument combinations
   - **Environment variable setup** (ROOTDIR, BINDIR, PROGNAME, PATH manipulation)
     - Determine rootdir and bindir from binary location or environment
     - Set ROOTDIR, BINDIR, PROGNAME environment variables
     - Manipulate PATH: add bindir to front, remove duplicate entries
   - **epmd daemon management** (start epmd if needed for distribution)
     - Detect if distribution is enabled (`-sname` or `-name` flags)
     - Start epmd daemon using `std::process::Command` before emulator initialization
     - Handle `-epmd` flag to specify epmd program path
     - Handle `-no_epmd` flag (requires `-proto_dist`)
   - **Boot/config file path resolution**
     - Resolve boot script path from `-boot` flag or default
     - Resolve config file path from `-config` flag
   - **Emulator selection logic** (determine if SMP is enabled)
   - **Detached mode handling** (Windows-specific)
   - Signal stack initialization (Rust equivalent of `sys_init_signal_stack()`)
   - `main()` function that calls `erl_start()` directly (no process replacement, no `execv()`)
   - Proper error handling and exit codes

**Key Difference from C Approach:**
- C approach: `erlexec` (C) → `execv()` → separate emulator binary (C `erl_main.c`)
- Rust approach: Single Rust binary handles **everything** - launcher + emulator in one process
- No process replacement needed - just call `erl_start()` directly
- **Zero C code in the launch path**

### Error Handling Analysis

#### Current Rust Error Handling

**Pattern Used:** `Result<T, String>` return types

**Examples:**
- `erl_start()` → `Result<(), String>`
- `erl_init()` → `Result<(), String>`
- `early_init()` → `Result<EarlyInitResult, String>`

**Error Propagation:**
- ✅ Uses `.map_err()` for error transformation
- ✅ Uses `?` operator for error propagation
- ⚠️ Uses `String` as error type (not idiomatic Rust)

#### Recommendations

1. **Use Custom Error Types:**
   - Create `InitError` enum with variants for different error types
   - Implement `std::error::Error` trait
   - Use `thiserror` or `anyhow` crate for error handling

2. **Error Context:**
   - Add context to errors using `.context()` or `.with_context()`
   - Provide actionable error messages

3. **Exit Codes:**
   - Map errors to appropriate exit codes
   - Follow Unix conventions (0 = success, non-zero = failure)

### Erlang/OTP Tooling Compatibility

#### epmd Integration

**Current State:**
- `erlexec` (C) starts epmd daemon before launching emulator
- epmd is started via `start_epmd_daemon()` function using `system()` call
- epmd program path can be specified via `-epmd` flag

**Rust Replacement Requirements:**
- ⚠️ Rust `main()` must start epmd daemon before calling `erl_start()`
- ⚠️ Use `std::process::Command` to spawn epmd daemon process
- ⚠️ Handle `-epmd` flag to specify epmd program path
- ⚠️ Handle `-no_epmd` flag (requires `-proto_dist` flag validation)
- ⚠️ Detect distribution mode from `-sname` or `-name` flags
- ✅ Distribution protocol compatibility maintained if argument interface matches

#### Distribution Protocol Compatibility

**Current State:**
- Distribution setup handled by `erlexec` (C) before emulator launch
- Emulator receives `-proto_dist` flag if alternative protocol is used
- Default `inet_tcp` protocol requires epmd

**Rust Replacement Requirements:**
- ⚠️ Rust `main()` must handle distribution setup before calling `erl_start()`
- ⚠️ Must parse `-sname`, `-name`, `-proto_dist`, `-no_epmd` flags
- ⚠️ Must validate `-no_epmd` requires `-proto_dist`
- ⚠️ Must start epmd daemon if distribution enabled and `-no_epmd` not set
- ✅ Distribution initialization happens after `erl_start()` returns (unchanged)
- ✅ Must accept same command-line arguments as C implementation

#### Other OTP Tooling

**Tools that depend on launch mechanism:**
- `start_erl`: Calls `erlexec` with boot/config arguments
- `run_erl`: Wraps `start_erl` for embedded systems
- `erl_call`: Spawns Erlang nodes using `erl` command
- `ct_slave`: Uses `erl` command to start test nodes

**Compatibility Requirements:**
- ⚠️ `erl` script may need modification to call Rust binary instead of `erlexec`
- ⚠️ OR: Rust binary can be named `erlexec` and placed in same location (drop-in replacement)
- ✅ All tools call `erl` script or `erlexec` directly - Rust binary must match interface
- ✅ Binary name and argument interface must match exactly
- ✅ Must support all special modes (`-emu_args_exit`, `-emu_name_exit`, `-emu_qouted_cmd_exit`) for tooling compatibility

---

## Recommendations

### Category 1: Binary Entry Point (Replaces `erl_main.c`)

#### Recommendation 1.1: Create Rust Binary Target

**Priority:** CRITICAL  
**Effort:** Low  
**Impact:** High

**Action:**
1. Add `[[bin]]` section to `frameworks_emulator_init/Cargo.toml`
2. Create `src/main.rs` with basic structure (detailed `erlexec` replacement in 1.3)

**Benefits:**
- Eliminates need for C `erl_main.c`
- Pure Rust entry point
- Better error handling and type safety

**Considerations:**
- Signal stack initialization needs Rust implementation
- Command-line arguments available via `std::env::args()`
- Exit code mapping

### Category 1A: Replace `erlexec` Functionality (CRITICAL - Zero C Code)

#### Recommendation 1.2: Implement `erlexec` Argument Parsing in Rust

**Priority:** CRITICAL  
**Effort:** High  
**Impact:** CRITICAL

**Action:**
1. Use `clap` crate for command-line argument parsing
2. Define all Erlang command-line flags:
   - Distribution: `-sname`, `-name`, `-proto_dist`, `-no_epmd`, `-epmd`
   - Boot/Config: `-boot`, `-config`, `-args_file`
   - SMP: `-smp`, `-smpenable`, `-smpdisable`, `-smpauto`
   - Emulator: `-emu_type`, `-emu_flavor`
   - Special modes: `-emu_args_exit`, `-emu_name_exit`, `-emu_qouted_cmd_exit`
   - Other: `-extra`, `-detached`, `-env`, etc.
3. Implement argument validation logic
4. Handle special modes (print and exit)

**Benefits:**
- Eliminates need for C `erlexec`
- Type-safe argument parsing
- Better error messages
- Zero C code dependency

**Dependencies:** None  
**Estimated Effort:** 3-5 days

#### Recommendation 1.3: Implement Environment Variable Setup

**Priority:** CRITICAL  
**Effort:** Medium  
**Impact:** High

**Action:**
1. Determine `rootdir` and `bindir` from:
   - Binary location (using `std::env::current_exe()`)
   - Environment variables (if set)
   - Default paths
2. Set environment variables: `ROOTDIR`, `BINDIR`, `PROGNAME`
3. Manipulate `PATH`:
   - Add `bindir` to front of PATH
   - Add `rootdir/bin` to PATH
   - Remove duplicate entries (especially `bindir`)

**Benefits:**
- Maintains compatibility with existing code that reads these variables
- Ensures correct binary/library paths

**Dependencies:** Recommendation 1.2  
**Estimated Effort:** 1-2 days

#### Recommendation 1.4: Implement epmd Daemon Management

**Priority:** CRITICAL  
**Effort:** Medium  
**Impact:** High

**Action:**
1. Detect if distribution is enabled (`-sname` or `-name` flags present)
2. Check `-no_epmd` flag (must have `-proto_dist` if set)
3. If distribution enabled and `-no_epmd` not set:
   - Determine epmd program path (from `-epmd` flag or default `bindir/epmd`)
   - Spawn epmd daemon using `std::process::Command`
   - Use `-daemon` flag for epmd
   - Handle errors (epmd may already be running)
4. Wait briefly for epmd to start (optional, may not be necessary)

**Benefits:**
- Eliminates C `system()` call dependency
- Better error handling
- Cross-platform process spawning

**Dependencies:** Recommendation 1.2  
**Estimated Effort:** 2-3 days

#### Recommendation 1.5: Implement Boot/Config Path Resolution

**Priority:** HIGH  
**Effort:** Low  
**Impact:** Medium

**Action:**
1. Parse `-boot` flag to get boot script path
2. Parse `-config` flag(s) to get config file path(s)
3. Resolve relative paths relative to `rootdir` or current directory
4. Validate paths exist (optional, may be handled later)
5. Pass resolved paths to `erl_start()` via arguments or environment

**Benefits:**
- Maintains compatibility with existing boot/config mechanism
- Proper path resolution

**Dependencies:** Recommendation 1.2  
**Estimated Effort:** 1 day

#### Recommendation 1.6: Implement Signal Stack Initialization

**Priority:** CRITICAL  
**Effort:** Medium  
**Impact:** High

**Action:**
- Research Rust equivalent of `sys_init_signal_stack()`
- May require FFI to platform-specific functions
- Consider using `libc` crate for signal handling
- Ensure thread safety before scheduler threads are created
- Must be called before any threads are spawned

**Location:**
- Create `frameworks_system_integration` module for platform-specific code
- Or use existing `frameworks_system_integration_unix` and `frameworks_system_integration_win32` crates

**Dependencies:** None  
**Estimated Effort:** 2-3 days

### Category 2: Error Handling

#### Recommendation 2.1: Implement Custom Error Types

**Priority:** HIGH  
**Effort:** Medium  
**Impact:** Medium

**Action:**
1. Create `InitError` enum in `frameworks_emulator_init`:
   ```rust
   #[derive(Debug, thiserror::Error)]
   pub enum InitError {
       #[error("Early initialization failed: {0}")]
       EarlyInit(String),
       #[error("Main initialization failed: {0}")]
       MainInit(String),
       // ... more variants
   }
   ```

2. Replace `Result<T, String>` with `Result<T, InitError>`
3. Use `?` operator for error propagation
4. Map errors to exit codes in `main()`

**Benefits:**
- Type-safe error handling
- Better error messages
- Easier debugging
- Follows Rust best practices

#### Recommendation 2.2: Add Error Context

**Priority:** MEDIUM  
**Effort:** Low  
**Impact:** Medium

**Action:**
- Use `anyhow::Context` or `thiserror` for error context
- Add context at each error propagation point
- Include relevant state information in error messages

### Category 3: Build System Integration

#### Recommendation 3.1: Binary Crate Configuration

**Priority:** CRITICAL  
**Effort:** Low  
**Impact:** High

**Action:**
1. Update `frameworks_emulator_init/Cargo.toml`:
   ```toml
   [[bin]]
   name = "beam"
   path = "src/main.rs"
   ```

2. Ensure binary name matches expected emulator name
3. Configure build to produce binary in correct location

**Considerations:**
- Binary name may need to be configurable (beam, beam.smp, etc.)
- May need separate binaries for different build configurations
- Consider using `cargo` features for different binary variants

#### Recommendation 3.2: Build Script Integration

**Priority:** LOW  
**Effort:** Medium  
**Impact:** Low

**Action:**
- Evaluate if `build.rs` is needed for:
  - Platform-specific compilation
  - Feature detection
  - Version information embedding

**Current Status:**
- No `build.rs` files found in Rust codebase
- May not be necessary for initial implementation

### Category 4: Initialization Completeness

#### Recommendation 4.1: Complete `erl_init()` Implementation

**Priority:** HIGH  
**Effort:** High  
**Impact:** High

**Action:**
Replace placeholder comments with actual initialization calls:

1. **Global Literals:**
   - Find or implement `init_global_literals()` equivalent

2. **Process Management:**
   - Call `usecases_process_management::erts_init_process()`
   - Pass `ncpu`, `proc_tab_sz`, `legacy_proc_tab` parameters

3. **Scheduling:**
   - Call `usecases_scheduling::erts_init_scheduling()`
   - Pass scheduler configuration

4. **BIF Dispatcher:**
   - Call `infrastructure_bif_dispatcher::erts_init_bif()`

5. **Emulator Loop:**
   - Call `infrastructure_emulator_loop::init_emulator()`

**Dependencies:**
- Requires completion of usecases and infrastructure layer implementations
- May need to coordinate with other crate development

#### Recommendation 4.2: Complete `early_init()` Implementation

**Priority:** HIGH  
**Effort:** Medium  
**Impact:** Medium

**Action:**
1. **Command-Line Parsing:**
   - Implement full argument parsing
   - Handle all Erlang command-line flags
   - May use `clap` or similar crate

2. **Term System:**
   - Call term system initialization
   - Find `erts_term_init()` equivalent in entities layer

3. **Emulator Arguments:**
   - Implement `erts_save_emu_args()` equivalent
   - Store arguments for later retrieval

### Category 5: Compatibility

#### Recommendation 5.1: Maintain Command-Line Interface

**Priority:** CRITICAL  
**Effort:** Low  
**Impact:** High

**Action:**
- Ensure Rust binary accepts same command-line arguments as C binary
- Maintain argument order and semantics
- Support all flags that `erlexec` passes to emulator

**Testing:**
- Test with existing OTP tooling (`start_erl`, `run_erl`, `erl_call`)
- Verify distribution setup works correctly
- Test embedded system startup

#### Recommendation 5.2: Verify epmd Compatibility

**Priority:** HIGH  
**Effort:** Medium  
**Impact:** High

**Action:**
- Test that Rust binary successfully starts epmd daemon
- Verify epmd startup sequence works correctly
- Test distribution with Rust emulator
- Verify `-no_epmd` flag works with `-proto_dist`
- Test epmd path resolution (`-epmd` flag)

**Note:**
- epmd is now started by Rust `main()`, not by separate C program
- Must ensure epmd is started before emulator initialization

---

## Migration Plan

### Phase 1: Replace `erlexec` and Create Binary Entry Point (Critical Path - Zero C Code)

**Goal:** Create Rust binary that replaces both `erlexec` (C) and `erl_main.c` (C) with zero C dependencies

**Steps:**
1. ✅ Analyze current `erlexec.c` and `erl_main.c` implementations
2. ⬜ Add `[[bin]]` section to `Cargo.toml`
3. ⬜ Create `src/main.rs` with basic structure
4. ⬜ Implement command-line argument parsing (replaces `erlexec` argument processing)
   - Use `clap` crate
   - Define all Erlang flags
   - Handle special modes (`-emu_args_exit`, etc.)
5. ⬜ Implement environment variable setup (ROOTDIR, BINDIR, PROGNAME, PATH)
6. ⬜ Implement epmd daemon management (start epmd if needed)
7. ⬜ Implement boot/config path resolution
8. ⬜ Implement signal stack initialization
9. ⬜ Integrate all functionality into `main()` function
10. ⬜ Add error handling and exit code mapping
11. ⬜ Test binary can be launched directly (bypassing `erlexec`)
12. ⬜ Test with `erl` script (may need script modification)

**Dependencies:** None  
**Estimated Effort:** 8-12 days (significant effort to replace all `erlexec` functionality)  
**Risk:** High (complex argument parsing, epmd management, environment setup)

### Phase 2: Error Handling Improvements

**Goal:** Implement idiomatic Rust error handling

**Steps:**
1. ⬜ Create `InitError` enum with `thiserror`
2. ⬜ Replace `Result<T, String>` with `Result<T, InitError>`
3. ⬜ Add error context at propagation points
4. ⬜ Map errors to exit codes in `main()`
5. ⬜ Test error paths

**Dependencies:** Phase 1  
**Estimated Effort:** 1-2 days  
**Risk:** Low

### Phase 3: Complete Initialization Sequence

**Goal:** Replace placeholder comments with actual initialization calls

**Steps:**
1. ⬜ Identify all missing initialization functions
2. ⬜ Verify usecases and infrastructure crates provide required functions
3. ⬜ Complete `erl_init()` implementation
4. ⬜ Complete `early_init()` implementation
5. ⬜ Test initialization sequence

**Dependencies:** Other crate development  
**Estimated Effort:** 5-10 days (depends on other crate completion)  
**Risk:** High (depends on other components being ready)

### Phase 4: Testing and Validation

**Goal:** Verify Rust binary works with all OTP tooling

**Steps:**
1. ⬜ Test basic REPL launch
2. ⬜ Test with `start_erl` and `run_erl`
3. ⬜ Test distribution setup
4. ⬜ Test with `erl_call`
5. ⬜ Test with `ct_slave`
6. ⬜ Performance comparison with C binary

**Dependencies:** Phases 1-3  
**Estimated Effort:** 3-5 days  
**Risk:** Medium

---

## Code Examples (Recommendations Only)

### Example 1: Binary Entry Point with `erlexec` Functionality Integrated

```rust
// src/main.rs
use std::env;
use std::process;
use std::path::{Path, PathBuf};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "beam")]
struct Args {
    // Distribution flags
    #[arg(long)]
    sname: Option<String>,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    proto_dist: Option<String>,
    #[arg(long)]
    no_epmd: bool,
    #[arg(long)]
    epmd: Option<String>,
    
    // Boot/config
    #[arg(long)]
    boot: Option<String>,
    #[arg(long)]
    config: Vec<String>,
    #[arg(long)]
    args_file: Option<String>,
    
    // SMP flags
    #[arg(long)]
    smp: Option<String>,
    
    // Special modes
    #[arg(long)]
    emu_args_exit: bool,
    #[arg(long)]
    emu_name_exit: bool,
    #[arg(long)]
    emu_qouted_cmd_exit: bool,
    
    // Other flags
    #[arg(long)]
    extra: bool,
    #[arg(long)]
    detached: bool,
    
    // Remaining arguments
    #[arg(trailing_var_arg = true)]
    remaining: Vec<String>,
}

fn main() {
    // Initialize signal stack before any threads are created
    unsafe {
        sys_init_signal_stack();
    }
    
    // Parse command-line arguments (replaces erlexec argument processing)
    let args = Args::parse();
    
    // Handle special modes (must exit early)
    if args.emu_name_exit {
        println!("beam");
        process::exit(0);
    }
    
    if args.emu_args_exit {
        // Print all arguments (simplified)
        for arg in env::args().skip(1) {
            println!("{}", arg);
        }
        process::exit(0);
    }
    
    if args.emu_qouted_cmd_exit {
        // Print quoted command line
        print!("\"beam\" ");
        for arg in env::args().skip(1) {
            print!("\"{}\" ", arg);
        }
        println!();
        process::exit(0);
    }
    
    // Determine rootdir and bindir
    let (rootdir, bindir) = determine_paths();
    
    // Set environment variables (replaces erlexec environment setup)
    env::set_var("ROOTDIR", &rootdir);
    env::set_var("BINDIR", &bindir);
    env::set_var("PROGNAME", "beam");
    manipulate_path(&bindir, &rootdir);
    
    // Start epmd daemon if needed (replaces erlexec epmd management)
    if should_start_epmd(&args) {
        start_epmd_daemon(&bindir, args.epmd.as_deref())
            .expect("Failed to start epmd daemon");
    }
    
    // Build arguments for erl_start (replaces erlexec argument construction)
    let mut emulator_args = build_emulator_args(&args, &rootdir, &bindir);
    
    // Call Rust erl_start directly (no execv, no process replacement)
    match frameworks_emulator_init::main_init::erl_start(&mut emulator_args.len(), &mut emulator_args) {
        Ok(()) => {
            // Start emulator loop (would be called after erl_init completes)
            // This is where the emulator would enter its main loop
            process::exit(0);
        }
        Err(e) => {
            eprintln!("Failed to start emulator: {}", e);
            process::exit(1);
        }
    }
}

fn determine_paths() -> (String, String) {
    // Determine rootdir and bindir from binary location or environment
    // Implementation details...
    let rootdir = env::var("ROOTDIR").unwrap_or_else(|_| "/usr/local/otp".to_string());
    let bindir = env::var("BINDIR").unwrap_or_else(|_| format!("{}/erts-*/bin", rootdir));
    (rootdir, bindir)
}

fn manipulate_path(bindir: &str, rootdir: &str) {
    // Add bindir to front of PATH, remove duplicates
    // Implementation details...
}

fn should_start_epmd(args: &Args) -> bool {
    // Check if distribution is enabled and epmd should be started
    (args.sname.is_some() || args.name.is_some()) && !args.no_epmd
}

fn start_epmd_daemon(bindir: &str, epmd_path: Option<&str>) -> Result<(), String> {
    let epmd = epmd_path.unwrap_or(&format!("{}/epmd", bindir));
    process::Command::new(epmd)
        .arg("-daemon")
        .spawn()
        .map_err(|e| format!("Failed to spawn epmd: {}", e))?;
    Ok(())
}

fn build_emulator_args(args: &Args, rootdir: &str, bindir: &str) -> Vec<String> {
    // Build argument vector for erl_start
    // Add -root, -bindir, -progname, boot, config, etc.
    let mut emulator_args = vec!["beam".to_string()];
    emulator_args.push("-root".to_string());
    emulator_args.push(rootdir.to_string());
    emulator_args.push("-bindir".to_string());
    emulator_args.push(bindir.to_string());
    // ... add other arguments
    emulator_args
}

// Platform-specific signal stack initialization
#[cfg(unix)]
unsafe fn sys_init_signal_stack() {
    // Use libc or platform-specific FFI
    // Equivalent to sys_init_signal_stack() from sys.c
}

#[cfg(windows)]
unsafe fn sys_init_signal_stack() {
    // Windows-specific implementation
}
```

### Example 2: Error Type Definition

```rust
// src/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InitError {
    #[error("Early initialization failed: {0}")]
    EarlyInit(String),
    
    #[error("Main initialization failed: {0}")]
    MainInit(String),
    
    #[error("Process initialization failed: {0}")]
    ProcessInit(String),
    
    #[error("Scheduling initialization failed: {0}")]
    SchedulingInit(String),
    
    #[error("BIF dispatcher initialization failed: {0}")]
    BifInit(String),
    
    #[error("Emulator loop initialization failed: {0}")]
    EmulatorLoopInit(String),
}

impl InitError {
    pub fn exit_code(&self) -> i32 {
        match self {
            InitError::EarlyInit(_) => 1,
            InitError::MainInit(_) => 2,
            // ... map to appropriate exit codes
            _ => 1,
        }
    }
}
```

### Example 3: Updated erl_start with Error Types

```rust
// src/main_init.rs
use crate::error::InitError;

pub fn erl_start(argc: &mut usize, argv: &mut Vec<String>) -> Result<(), InitError> {
    let early_result = early_init::early_init(argc, argv)
        .map_err(InitError::EarlyInit)?;
    
    let config = InitConfig {
        ncpu: early_result.ncpu,
        // ... build config
        ..Default::default()
    };
    
    erl_init(config)
        .map_err(InitError::MainInit)?;
    
    Ok(())
}
```

### Example 4: Cargo.toml Binary Configuration

```toml
# frameworks_emulator_init/Cargo.toml

[package]
name = "frameworks_emulator_init"
# ... existing configuration

[[bin]]
name = "beam"
path = "src/main.rs"

[dependencies]
# ... existing dependencies
thiserror = "1.0"
libc = "0.2"  # For signal stack initialization
clap = { version = "4.0", features = ["derive"] }  # For argument parsing
```

### Example 5: epmd Daemon Startup (Rust Replacement)

```rust
use std::process::Command;
use std::path::PathBuf;

fn start_epmd_daemon(bindir: &str, epmd_path: Option<&str>) -> Result<(), InitError> {
    let epmd_program = epmd_path
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(bindir).join("epmd"));
    
    // Spawn epmd daemon (replaces C system() call)
    let mut child = Command::new(&epmd_program)
        .arg("-daemon")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| InitError::EpmdStart(format!("Failed to spawn epmd: {}", e)))?;
    
    // Don't wait for epmd - it's a daemon
    // epmd may already be running, which is fine
    
    Ok(())
}
```

---

## Checklist

### Critical Requirements (Zero C Code)
- [ ] Create Rust binary entry point (`main.rs`)
- [ ] Add `[[bin]]` section to `Cargo.toml`
- [ ] **Implement `erlexec` argument parsing in Rust** (replaces C `erlexec`)
- [ ] **Implement environment variable setup** (ROOTDIR, BINDIR, PROGNAME, PATH)
- [ ] **Implement epmd daemon management** (start epmd if needed)
- [ ] **Implement boot/config path resolution**
- [ ] Implement signal stack initialization
- [ ] Integrate all `erlexec` functionality into `main()`
- [ ] Map errors to exit codes
- [ ] Test binary can be launched directly (bypassing `erlexec`)
- [ ] Test with `erl` script (may need script modification)

### Error Handling
- [ ] Create custom error types (`InitError`)
- [ ] Replace `Result<T, String>` with `Result<T, InitError>`
- [ ] Add error context at propagation points
- [ ] Test error paths

### Initialization Completeness
- [ ] Complete `erl_init()` implementation
- [ ] Complete `early_init()` implementation
- [ ] Initialize global literals
- [ ] Initialize process management
- [ ] Initialize scheduling
- [ ] Initialize BIF dispatcher
- [ ] Initialize emulator loop

### Compatibility (Zero C Code)
- [ ] Test with `start_erl` (may need to call Rust binary instead of `erlexec`)
- [ ] Test with `run_erl`
- [ ] Test with `erl_call`
- [ ] Test distribution setup
- [ ] Test epmd daemon startup from Rust
- [ ] Test `-no_epmd` with `-proto_dist`
- [ ] Test special modes (`-emu_args_exit`, `-emu_name_exit`, `-emu_qouted_cmd_exit`)
- [ ] Verify all command-line flags work
- [ ] Test PATH manipulation
- [ ] Test environment variable setup

### Build System (Zero C Code)
- [ ] Binary builds correctly
- [ ] Binary name matches expected emulator name (`beam` or `beam.smp`)
- [ ] Binary can be placed in same location as C `erlexec` (for drop-in replacement)
- [ ] OR: `erl` script modified to call Rust binary directly
- [ ] Release build optimizations work

### Documentation
- [ ] Document binary entry point
- [ ] Document error handling approach
- [ ] Document initialization sequence
- [ ] Document compatibility considerations

---

## Conclusion

The current Rust implementation provides a solid foundation with `erl_start()` and `erl_init()` functions, but lacks a binary entry point and all `erlexec` functionality. The critical path to a **pure Rust REPL launch with zero C code** is:

1. **Replace `erlexec` functionality in Rust** - This is the highest priority and most complex task
   - Implement complete argument parsing (all Erlang flags)
   - Implement environment variable setup (ROOTDIR, BINDIR, PROGNAME, PATH)
   - Implement epmd daemon management
   - Implement boot/config path resolution
   - Handle all special modes
2. **Create Rust binary entry point** - Integrate all `erlexec` functionality into `main()`
3. **Implement signal stack initialization** - Required for thread safety
4. **Complete initialization sequence** - Replace placeholders with actual calls
5. **Improve error handling** - Use idiomatic Rust error types
6. **Test compatibility** - Verify with all OTP tooling

**Key Insight:** The Rust binary must **replace `erlexec` entirely**, not just be launched by it. The launch path becomes: `erl` script → Rust binary (which does everything). This requires implementing all `erlexec` functionality in Rust, but eliminates the need for any C code in the launch path.

**Alternative Approach:** The Rust binary can be named `erlexec` and placed in the same location as the C `erlexec`, acting as a drop-in replacement. This allows existing scripts and tooling to work without modification, while still achieving zero C code in the actual launch path.

---

*Created using AALang and Gab*
